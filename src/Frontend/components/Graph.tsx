import React, { useMemo, useState, useCallback, useRef } from 'react'
import { XYChart, AnimatedAxis, AnimatedGrid, AnimatedLineSeries, Tooltip, darkTheme } from '@visx/xychart'
import { localPoint } from '@visx/event'

const GRID_COUNT = 10
type GraphProps = {
    title?: string
    labels: string[] // length 7
    counts?: number[][] | null // 7 x 1000 (or undefined while loading)
    mins?: number[] | null // length 7
    maxs?: number[] | null // length 7
    width?: number
    height?: number
    pointOfInterestBucket?: number | null // 0..999 or null
    displayMode?: 'probability' | 'cost'
    budgets?: number[] | null
    hasSelection?: boolean
}

type Point = { x: number, y: number }

const SERIES_COLORS_VARS: string[] = [
    'var(--series-red)',
    'var(--series-blue)',
    'var(--series-leaps)',
    'var(--series-shards)',
    'var(--series-oreha)',
    'var(--series-gold)',
    'var(--series-silver)',
]

function formatSig3(n: number): string {
    if (!isFinite(n)) return ''
    // toPrecision(3), then strip trailing zeros and optional dot
    const s = parseFloat(Number(n).toPrecision(3)).toLocaleString('en-US', {
        minimumFractionDigits: 0,
        maximumFractionDigits: 0
    });
    return s.replace(/\.0+$/, '').replace(/(\.[0-9]*?)0+$/, '$1')
}


export default function Graph({ title, labels, counts, mins, maxs, width = 640, height = 320, pointOfInterestBucket = null, displayMode = 'probability', budgets = null, hasSelection = false }: GraphProps) {
    const [visible, setVisible] = useState<boolean[]>(() => new Array(7).fill(true))
    const [hoverSeries, setHoverSeries] = useState<number | null>(null)
    const [hoverBucket, setHoverBucket] = useState<number | null>(null)
    const chartRef = useRef<HTMLDivElement | null>(null)

    const bucketLen = counts?.[0]?.length || 1000

    // Drop any series where all frequency falls in a single bucket (<=1 positive bin)
    const keepMask: boolean[] = useMemo(() => {
        if (!counts) return new Array(7).fill(false)
        return counts.map(series => {
            let positiveBins = 0
            for (let i = 0; i < series.length && positiveBins <= 1; i++) if (series[i] > 0) positiveBins++
            return positiveBins > 1
        })
    }, [counts])

    const dataSeries: Point[][] = useMemo(() => {
        if (!counts) return [] as Point[][]
        return counts.map(series => series.map((y, x) => ({ x, y })).filter(p => p.y > 0))
    }, [counts])

    const yMax: number = useMemo(() => {
        if (!counts) return 0
        let m = 0
        for (let i = 0; i < counts.length; i++) {
            if (!visible[i] || !keepMask[i]) continue
            for (let j = 0; j < counts[i].length; j++) m = Math.max(m, counts[i][j])
        }
        return m
    }, [counts, visible, keepMask])

    // const onLegendToggle = (i: number) => setVisible(v => v.map((b, idx) => idx === i ? !b : b))

    const sortedVisibleIndices: number[] = useMemo(() => {
        const base = labels.map((_, i) => i).filter(i => visible[i] && keepMask[i])
        if (hoverSeries == null) return base
        return base.filter(i => i !== hoverSeries).concat([hoverSeries])
    }, [labels, visible, hoverSeries, keepMask])

    const xAccessor = (d: Point) => d.x
    const yAccessor = (d: Point) => d.y

    const handleMouseMove = useCallback((ev: React.MouseEvent<HTMLDivElement>) => {
        if (!counts) return
        const p = localPoint(ev)
        const plotLeft = 50, plotRight = 50
        const innerW = Math.max(1, width - plotLeft - plotRight)
        const x = Math.min(Math.max((p?.x ?? 0) - plotLeft, 0), innerW)
        const bucket = Math.round((x / innerW) * (bucketLen - 1))
        // choose series with closest y to cursor vertically if possible; otherwise pick the highest y
        const ys = counts.map((s, i) => ({ i, y: s[bucket], vis: visible[i] && keepMask[i] }))
        const visibleYs = ys.filter(o => o.vis)
        if (visibleYs.length === 0) { setHoverSeries(null); return }
        // use highest y bucket as proxy for nearest vertically
        const best = visibleYs.reduce((a, b) => Math.abs(a.y - p.y) < Math.abs(b.y - p.y) ? a : b)
        setHoverSeries(best.i)
        // if (displayMode === 'cost' && budgets && mins && maxs) {
        //     const budget = budgets[best.i] ?? 0
        //     const min = mins[best.i]
        //     const max = maxs[best.i]
        //     const denom = Math.max(1, max - min)
        //     const b = Math.round(((budget - min) / denom) * (bucketLen - 1))
        //     setHoverBucket(Math.max(0, Math.min(bucketLen - 1, b)))
        // } else {
        setHoverBucket(bucket)
        // }
    }, [counts, visible, keepMask, bucketLen, width])

    const handleMouseLeave = () => { setHoverSeries(null); setHoverBucket(null) }

    const fallbackSeries = useMemo(() => {
        if (hoverSeries != null && keepMask[hoverSeries]) return hoverSeries
        const goldIdx = labels.indexOf('Gold')
        if (goldIdx >= 0 && keepMask[goldIdx] && visible[goldIdx]) return goldIdx
        const first = labels.map((_, i) => i).find(i => keepMask[i] && visible[i])
        return first ?? null
    }, [hoverSeries, labels, keepMask, visible])

    const hoverColor = fallbackSeries != null ? SERIES_COLORS_VARS[fallbackSeries] : 'var(--text-secondary)'

    // bottom axis tick values and formatter based on hovered series
    const tickVals = useMemo(() => {
        const last = bucketLen - 1
        return Array.from({ length: GRID_COUNT }, (_, i) => (i != GRID_COUNT - 1) ? Math.round(last * i / GRID_COUNT) : last)

    }, [bucketLen])
    const bottomTickFormat = useCallback((val: any) => {
        if (fallbackSeries == null || !mins || !maxs) return formatSig3(val)
        const min = mins[fallbackSeries]
        const max = maxs[fallbackSeries]
        const bucketIdx = typeof val === 'number' ? val : Number(val)
        const width = (max - min) / bucketLen
        const mid = min + (bucketIdx + 0.5) * width
        return formatSig3(mid)
    }, [fallbackSeries, mins, maxs, bucketLen])

    const poiBucketClamped = useMemo(() => {
        return pointOfInterestBucket == null ? null : Math.max(0, Math.min(bucketLen - 1, pointOfInterestBucket))
    }, [pointOfInterestBucket, bucketLen])

    const anyVisible = useMemo(() => labels.some((_, i) => visible[i] && keepMask[i]), [labels, visible, keepMask])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }} onMouseMove={handleMouseMove} onMouseLeave={handleMouseLeave}>
            {title ? <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{title}</div> : null}
            <div ref={chartRef}>
                <XYChart
                    height={height}
                    width={width}
                    xScale={{ type: 'linear', domain: [0, Math.max(0, bucketLen - 1)] }}
                    yScale={{ type: 'linear', domain: [0, Math.max(1, yMax)] }}
                    theme={darkTheme}
                >
                    {/* Remove horizontal grid and y-axis; add vertical grid lines */}
                    <AnimatedGrid columns numTicks={tickVals.length} rows={false} />
                    <AnimatedAxis orientation="bottom" tickValues={tickVals} tickFormat={bottomTickFormat as any} tickLabelProps={() => ({ fill: hoverColor, fontSize: 11 })} />
                    {/* hide y-axis */}
                    {/* <AnimatedAxis orientation="left" numTicks={0} /> */}
                    {sortedVisibleIndices.map((seriesIdx) => (
                        <AnimatedLineSeries
                            key={labels[seriesIdx]}
                            dataKey={labels[seriesIdx]}
                            data={dataSeries[seriesIdx] || []}
                            xAccessor={xAccessor}
                            yAccessor={yAccessor}
                            stroke={SERIES_COLORS_VARS[seriesIdx]}
                            strokeWidth={hoverSeries === seriesIdx ? 3 : 1.5}
                            opacity={1}
                            onPointerMove={() => setHoverSeries(seriesIdx)}
                        />
                    ))}

                    {/* Point of interest - draw a dot for each visible series at poi bucket */}
                    {poiBucketClamped != null && counts && anyVisible && (
                        <g>
                            {labels.map((_, i) => {
                                if (!visible[i] || !keepMask[i]) return null
                                const plotLeft = 50, plotRight = 50, plotTop = 50, plotBottom = 50
                                const innerW = width - plotLeft - plotRight
                                const innerH = height - plotTop - plotBottom
                                let cx, cy, label;

                                if (displayMode === 'cost') {
                                    cx = plotLeft + budgets[i] / (maxs[i] - mins[i]) * innerW;
                                    cy = plotTop + innerH - counts[i][(Number((budgets[i] * bucketLen / (maxs[i] - mins[i])))).toFixed(0)] / Math.max(1, yMax) * innerH;
                                    label = budgets[i].toString()
                                }
                                else {
                                    cx = plotLeft + (poiBucketClamped / Math.max(1, bucketLen - 1)) * innerW
                                    cy = plotTop + innerH - (counts[i][poiBucketClamped] / Math.max(1, yMax)) * innerH

                                    const costMid = mins ? (mins[i] + (maxs![i] - mins[i]) * ((poiBucketClamped) / bucketLen)) : 0
                                    const prob = (100 * (poiBucketClamped) / bucketLen).toFixed(0)
                                    label = displayMode === 'probability' ? `${prob}%` : `${formatSig3(costMid)}`
                                }

                                const boxW = Math.max(34, label.length * 8 + 8)
                                const boxH = 18
                                return (
                                    <g key={`poi - ${i} `}>
                                        <circle cx={cx} cy={cy} r={4.5} fill={SERIES_COLORS_VARS[i]} stroke="#000" />
                                        <rect x={cx + 6} y={cy - boxH - 4} width={boxW} height={boxH} fill="rgba(0,0,0,0.3)" rx={3} ry={3} />
                                        <text x={cx + 10} y={cy - 8} fill={SERIES_COLORS_VARS[i]} fontSize={12}>{label}</text>
                                    </g>
                                )
                            })}
                        </g>
                    )}

                    {/* Hover point snapped to selected series */}
                    {fallbackSeries != null && hoverBucket != null && counts && visible[fallbackSeries] && keepMask[fallbackSeries] && (
                        <g>
                            {(() => {
                                const plotLeft = 50, plotRight = 50, plotTop = 50, plotBottom = 50
                                const innerW = width - plotLeft - plotRight
                                const innerH = height - plotTop - plotBottom
                                const cx = plotLeft + (hoverBucket / Math.max(1, bucketLen - 1)) * innerW
                                const cy = plotTop + innerH - (counts[fallbackSeries][hoverBucket] / Math.max(1, yMax)) * innerH

                                return (
                                    <g>
                                        <circle cx={cx} cy={cy} r={4.5} fill={SERIES_COLORS_VARS[fallbackSeries]} stroke="#000" />
                                    </g>
                                )
                            })()}
                        </g>
                    )}

                    <Tooltip
                        snapTooltipToDatumX
                        snapTooltipToDatumY={false}
                        showSeriesGlyphs={false}
                        renderTooltip={() => {
                            if (fallbackSeries == null || hoverBucket == null || !mins || !maxs) return null
                            const min = mins[fallbackSeries]
                            const max = maxs[fallbackSeries]
                            const width = (max - min) / bucketLen
                            const mid = min + (hoverBucket + 0.5) * width
                            const cumPct = (100 * (hoverBucket + 1) / bucketLen).toFixed(1)
                            return (
                                <div style={{ color: 'var(--text-primary)' }}>
                                    <div style={{ color: SERIES_COLORS_VARS[fallbackSeries], fontWeight: 600 }}>{labels[fallbackSeries]}</div>
                                    <div>Cost: {formatSig3(mid)}</div>
                                    <div>Cumulative: {cumPct}%</div>
                                </div>
                            )
                        }}
                    />
                </XYChart>
            </div>
            {/* Legend */}
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 10 }}>
                {labels.map((lab, i) => keepMask[i] ? (
                    <button
                        key={lab}
                        onClick={() => {
                            // toggle visibility and bring to front by moving last in visible order
                            setVisible(v => v.map((b, idx) => idx === i ? !b : b))
                            setHoverSeries(i)
                        }}
                        style={{ display: 'flex', alignItems: 'center', gap: 6, background: 'transparent', border: '1px solid var(--border-secondary)', padding: '2px 6px', cursor: 'pointer', opacity: visible[i] ? 1 : 0.4 }}>
                        <span style={{ width: 10, height: 10, background: SERIES_COLORS_VARS[i], display: 'inline-block' }} />
                        <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{lab}</span>
                    </button>
                ) : null)}
            </div>

            {!anyVisible && (
                <div style={{ color: 'var(--text-secondary)', fontSize: 30, marginTop: -200 }}>
                    {hasSelection ? 'Everything have 100% success rate, nothing to plot.' : 'Nothing to plot, tick an upgrade!'}
                </div>
            )}
        </div>
    )
}


