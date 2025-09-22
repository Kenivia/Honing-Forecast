import React, { useMemo, useState, useCallback, useRef } from 'react'
import { XYChart, AnimatedAxis, AnimatedGrid, AnimatedLineSeries, Tooltip, darkTheme } from '@visx/xychart'
import { localPoint } from '@visx/event'
const plotLeft = 50, plotRight = 50, plotTop = 50, plotBottom = 50
const GRID_COUNT = 10
type GraphProps = {
    title?: string
    labels: string[] // length 7
    counts?: number[][] | null // 7 x 1000 (or undefined while loading)
    mins?: number[] | null // length 7
    maxs?: number[] | null // length 7
    width?: number
    height?: number
    // pointOfInterestBucket?: number | null // 0..999 or null
    // displayMode?: 'probability' | 'cost'
    budgets?: number[] | null
    hasSelection?: boolean
    isLoading?: boolean
    cumulative?: boolean
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

function formatSig3(n: number, place: number = 3): string {
    if (!isFinite(n)) return ''

    const abs = Math.abs(n)
    let suffix = ''
    let divisor = 1

    if (abs >= 1_000_000_000) {
        suffix = 'B'
        divisor = 1_000_000_000
    } else if (abs >= 1_000_000) {
        suffix = 'M'
        divisor = 1_000_000
    } else if (abs >= 1_000) {
        suffix = 'K'
        divisor = 1_000
    }

    const scaled = n / divisor

    // keep `place` significant figures, but trim trailing zeros
    let s = parseFloat(
        Number(scaled.toFixed(place)).toPrecision(place)
    ).toLocaleString('en-US', {
        minimumFractionDigits: scaled < 10 && suffix ? 1 : 0, // show decimals for small K/M/B
        maximumFractionDigits: place
    })

    return s + suffix
}


function to_step(arr: number[]) {
    let cur = 0;
    let out = [];
    for (let i = 0; i < arr.length; i++) {
        if (arr[i] != 0) {
            cur = arr[i]
        }
        out.push(cur)
    }
    return out
}
// function findNearestNonZeroIndex(arr, startIndex) {
//     if (!Array.isArray(arr) || startIndex < 0 || startIndex >= arr.length) {
//         console.error("Invalid input: Please provide a valid array and starting index.");
//         return -1; // Or throw an error
//     }

//     // Check if the element at startIndex is already non-zero
//     if (arr[startIndex] !== 0) {
//         return startIndex;
//     }

//     let leftIndex = startIndex - 1;
//     let rightIndex = startIndex + 1;
//     let first_zero_arr = Array.from({ length: arr.length }, () => false)
//     let first = true
//     for (let i = 0; i < arr.length; i++) {
//         if (arr[i] != 0 && first) {
//             first = false
//         }
//         first_zero_arr[i] = first

//     }

//     while (leftIndex >= 0 || rightIndex < arr.length) {
//         // Check left side
//         if (leftIndex >= 0 && (arr[leftIndex] !== 0 || first_zero_arr[leftIndex])) {
//             return leftIndex;
//         }

//         // Check right side
//         if (rightIndex < arr.length && (arr[rightIndex] !== 0 || first_zero_arr[rightIndex])) {
//             return rightIndex;
//         }

//         leftIndex--;
//         rightIndex++;
//     }

//     return startIndex; // No non-zero element found in the array
// }
export default function Graph({ title, labels, counts, mins, maxs, width = 640, height = 320, budgets = null, hasSelection = false, isLoading = false, cumulative = true }: GraphProps) {
    const [visible, setVisible] = useState<boolean[]>(() => [true, true, false, false, false, true, false])
    const [hoverSeries, setHoverSeries] = useState<number | null>(null)
    const [hoverBucket, setHoverBucket] = useState<number | null>(null)
    const chartRef = useRef<HTMLDivElement | null>(null)

    const bucketLen = counts?.[0]?.length || 1000
    const data_size = counts?.[0].reduce((partialSum, a) => partialSum + a, 0) || 1;

    // Drop any series where all frequency falls in a single bucket (<=1 positive bin)
    const keepMask: boolean[] = useMemo(() => {
        if (!counts) return new Array(7).fill(false)
        return counts.map(series => {
            let positiveBins = 0
            for (let i = 0; i < series.length && positiveBins <= 1; i++) if (series[i] > 0) positiveBins++
            return positiveBins > 1
        })
    }, [counts])

    const cdfSeries: number[][] | null = useMemo(() => {
        if (!counts) return null
        return counts.map(series => {
            const total = series.reduce((a, b) => a + b, 0) || 1
            let acc = 0
            const out = new Array(series.length)
            for (let i = 0; i < series.length; i++) {
                acc += series[i]
                out[i] = acc / total
            }
            return out
        })
    }, [counts])

    const normalizedCounts: number[][] | null = useMemo(() => {
        if (!counts) return null
        const denom = data_size || 1
        return counts.map(series => series.map(v => v / denom))
    }, [counts, data_size])

    const dataSeries: Point[][] = useMemo(() => {
        if (!counts) return [] as Point[][]
        const source = cumulative && cdfSeries ? cdfSeries : (normalizedCounts || counts)
        let out: Point[][] = Array.from({ length: source.length }, () => []);
        for (let i = 0; i < source.length; i++) {
            let first = true;
            let prev: number | null = null;
            for (let b = 0; b < source[i].length; b++) {
                if (first) {
                    out[i].push({ x: b, y: 0 })
                }
                const y = source[i][b]
                if (y > 0) {
                    out[i].push({ x: b, y })
                    first = false;
                    prev = y
                } else if (prev != null) {
                    out[i].push({ x: b, y: prev })
                }
            }
            if (first) {
                out[i] = [{ x: 0, y: 0 }]
            }
            else {
                out[i].push({ x: source[i].length, y: (prev ?? source[i][source[i].length - 1]) })
            }
        }
        return out
    }, [counts, cumulative, cdfSeries, normalizedCounts])

    const yMax: number = useMemo(() => {
        if (!counts) return 0
        if (cumulative) return 1
        const denom = data_size || 1
        let m = 0
        for (let i = 0; i < counts.length; i++) {
            if (!visible[i] || !keepMask[i]) continue
            for (let j = 0; j < counts[i].length; j++) m = Math.max(m, counts[i][j] / denom)
        }
        return m
    }, [counts, visible, keepMask, cumulative, data_size])

    // const onLegendToggle = (i: number) => setVisible(v => v.map((b, idx) => idx === i ? !b : b))

    const sortedVisibleIndices: number[] = useMemo(() => {
        const base = labels.map((_, i) => i).filter(i => visible[i] && keepMask[i])
        if (hoverSeries == null) return base
        return base.filter(i => i !== hoverSeries).concat([hoverSeries])
    }, [labels, visible, hoverSeries, keepMask])

    const xAccessor = (d: Point) => { return d.x }
    const yAccessor = (d: Point) => d.y

    const handleMouseMove = useCallback((ev: React.MouseEvent<HTMLDivElement>) => {
        if (!counts) return
        const p = localPoint(ev)
        const innerW = Math.max(1, width - plotLeft - plotRight)
        const innerH = height - plotTop - plotBottom
        const x = Math.min(Math.max((p?.x ?? 0) - plotLeft, 0), innerW)
        const bucket = Math.round((x / innerW) * (bucketLen - 1))
        // choose series with closest y to cursor vertically if possible; otherwise pick the highest y
        const src = cumulative && cdfSeries ? cdfSeries : (normalizedCounts || counts)
        const denomY = Math.max(1e-9, yMax)
        const ys = src.map((s, i) => ({ i, y: s[bucket] / denomY * innerH, vis: visible[i] && keepMask[i] }))
        const visibleYs = ys.filter(o => o.vis)
        if (visibleYs.length === 0) { setHoverSeries(null); return }
        // use highest y bucket as proxy for nearest vertically
        const actual_y = innerH + plotBottom - p.y
        const best = visibleYs.reduce((a, b) => Math.abs(a.y - actual_y) < Math.abs(b.y - actual_y) ? a : b)
        setHoverSeries(best.i)
        setHoverBucket(bucket)
    }, [counts, visible, keepMask, bucketLen, width, height, yMax, cdfSeries, normalizedCounts, cumulative])

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
        // const last = bucketLen - 1
        // if (maxs) {
        let out = Array.from({ length: GRID_COUNT + 1 }, (_, i) => Math.min(bucketLen, Math.round(bucketLen * i / (GRID_COUNT))))
        return out
        // }
        // else {
        //     return Array(GRID_COUNT)
        // }

    }, [bucketLen])
    const bottomTickFormat = useCallback((val: any) => {
        if (fallbackSeries == null || !mins || !maxs) return formatSig3(val)

        const min = mins[fallbackSeries]
        const max = maxs[fallbackSeries]
        // return maxs[fallbackSeries]
        const bucketIdx = typeof val === 'number' ? val : Number(val)
        const width = (max - min) / bucketLen
        const mid = min + (bucketIdx) * width
        return formatSig3(mid)
    }, [fallbackSeries, mins, maxs, bucketLen])


    const anyVisible = useMemo(() => labels.some((_, i) => visible[i] && keepMask[i]), [labels, visible, keepMask])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, padding: 16, borderRadius: 16, backgroundColor: 'var(--bg-tertiary)' }} onMouseMove={handleMouseMove} onMouseLeave={handleMouseLeave}>
            {title ? <div style={{ color: 'var(--text-primary)', fontWeight: 600 }}>{title}</div> : null}
            <div ref={chartRef}>
                <XYChart
                    height={height}
                    width={width}
                    xScale={{ type: 'linear', domain: [0, Math.max(0, bucketLen)] }}
                    yScale={{ type: 'linear', domain: cumulative ? [0, 1] : [0, Math.max(1e-6, yMax)] }}
                    theme={darkTheme}
                >
                    {/* Remove horizontal grid and y-axis; add vertical grid lines */}
                    <AnimatedGrid columns numTicks={GRID_COUNT} rows={false} />
                    <AnimatedAxis orientation="bottom"
                        label={'Normalized mats cost(0 to pity)'}
                        tickValues={tickVals}
                        tickFormat={bottomTickFormat as any} tickLabelProps={() => ({ fill: hoverColor, fontSize: 11, angle: -0 })} />
                    {/* Y axis with custom ticks */}
                    <AnimatedAxis
                        orientation="left"
                        label={cumulative ? 'Cumulative Probability of Success' : 'Probability distribution'}
                        tickValues={(() => {
                            if (cumulative) {
                                // 0..1 inclusive at 0.1 steps
                                const vals: number[] = []
                                for (let i = 0; i <= 10; i++) vals.push(i / 10)
                                return vals
                            } else {
                                const vals: number[] = []
                                // let sum = counts[0].reduce((partialSum, a) => partialSum + a, 0);
                                for (let i = 0; i <= 10; i++) vals.push(yMax * i / 10)
                                return vals
                            }
                        })()}
                        tickFormat={(val: any) => {
                            // if (cumulative) {
                            const n = typeof val === 'number' ? val : Number(val)
                            return (n * 100).toFixed(0) + "%"
                            // }
                            // return formatSig3(typeof val === 'number' ? val * 100 : Number(val * 100)) + "%"
                        }}
                        tickLabelProps={() => ({ fill: hoverColor, fontSize: 11 })}
                    />
                    {sortedVisibleIndices.map((seriesIdx) => (
                        <AnimatedLineSeries
                            key={labels[seriesIdx]}
                            dataKey={labels[seriesIdx]}
                            data={dataSeries[seriesIdx] || []}
                            xAccessor={xAccessor}
                            yAccessor={yAccessor}
                            stroke={SERIES_COLORS_VARS[seriesIdx]}
                            strokeWidth={hoverSeries === seriesIdx ? 4 : 1.5}
                            opacity={1}
                            onPointerMove={() => setHoverSeries(seriesIdx)}
                        />
                    ))}

                    {/* Point of interest - draw a dot for each visible series at poi bucket */}
                    {counts && anyVisible && budgets && (
                        <g>
                            {labels.map((_, i) => {
                                if (!visible[i] || !keepMask[i]) return null

                                const innerW = width - plotLeft - plotRight
                                const innerH = height - plotTop - plotBottom
                                let cx, cy, label;

                                // if (displayMode === 'cost') {
                                const range = Math.max(1e-9, (maxs[i] - mins[i]))
                                const frac = (budgets[i] - mins[i]) / range
                                const bucket_idx = Math.max(0, Math.min(bucketLen - 1, Math.round(frac * (bucketLen - 1))))
                                // const cumPct = Math.round((counts[i].slice(0, bucket_idx).reduce((partialSum, a) => partialSum + a, 0) / data_size * 100));
                                cx = plotLeft + (bucket_idx / Math.max(1, bucketLen - 1)) * innerW;
                                const seriesVals = cumulative && cdfSeries ? cdfSeries[i] : ((normalizedCounts && normalizedCounts[i]) || counts[i])
                                const denomY2 = Math.max(1e-9, yMax)
                                cy = plotTop + innerH - seriesVals[bucket_idx] / denomY2 * innerH;
                                label = formatSig3(budgets[i])


                                const boxW = Math.max(16, label.length * 8)
                                const boxH = 18
                                return (
                                    <g key={`poi - ${i} `}>
                                        <circle cx={cx} cy={cy} r={7} fill={SERIES_COLORS_VARS[i]} stroke="#000" />
                                        <rect x={cx + 6} y={cy - boxH - 4} width={boxW} height={boxH} fill="rgba(0,0,0,0.5)" rx={3} ry={3} />
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
                                // const bucket_idx = findNearestNonZeroIndex(counts[fallbackSeries], Math.max(Math.min(Math.round((budgets[fallbackSeries] * bucketLen / (maxs[fallbackSeries] - mins[fallbackSeries]))), bucketLen - 1), 0))
                                const innerW = width - plotLeft - plotRight
                                const innerH = height - plotTop - plotBottom
                                const cx = plotLeft + (hoverBucket / Math.max(1, bucketLen)) * innerW
                                const hoverSeriesVals = cumulative && cdfSeries ? cdfSeries[fallbackSeries] : ((normalizedCounts && normalizedCounts[fallbackSeries]) || counts[fallbackSeries])
                                const denomY3 = Math.max(1e-9, yMax)
                                const cy = plotTop + innerH - (to_step(hoverSeriesVals)[hoverBucket] / denomY3) * innerH

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
                        snapTooltipToDatumY={true}
                        showSeriesGlyphs={false}
                        renderTooltip={() => {
                            if (fallbackSeries == null || hoverBucket == null || !mins || !maxs) return null
                            const min = mins[fallbackSeries]
                            const max = maxs[fallbackSeries]
                            // const innerW = Math.max(1, width - plotLeft - plotRight)
                            const mid = min + (hoverBucket + 0.5) * max / bucketLen
                            const cumPct = (counts[fallbackSeries].slice(0, hoverBucket + 1).reduce((partialSum, a) => partialSum + a, 0) / data_size * 100).toFixed(1)
                            return (
                                <div style={{ color: 'var(--text-primary)' }}>
                                    <div style={{ color: SERIES_COLORS_VARS[fallbackSeries], fontWeight: 600 }}>{labels[fallbackSeries]}</div>
                                    <div>{cumPct}% of trials used less</div>
                                    <div>than {formatSig3(mid)} {labels[fallbackSeries]}</div>
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
                            if (i === hoverSeries && visible[i]) {
                                setHoverSeries(null)
                            }
                            else {
                                setHoverSeries(i)
                            }

                        }}
                        style={{ display: 'flex', alignItems: 'center', gap: 6, background: 'transparent', border: '1px solid var(--border-secondary)', padding: '2px 6px', cursor: 'pointer', opacity: visible[i] ? 1 : 0.4 }}>
                        <span style={{ width: 10, height: 10, background: SERIES_COLORS_VARS[i], display: 'inline-block' }} />
                        <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{lab}</span>
                    </button>
                ) : null)}
            </div>

            {!anyVisible && (
                <div>
                    <div style={{ color: 'var(--text-secondary)', fontSize: 50, alignSelf: 'center', justifySelf: 'center', marginTop: -200 }}>
                        {hasSelection ? isLoading ? "Loading..." : 'Everything have 100% success rate, nothing to plot.' : 'Nothing to plot, tick an upgrade!'}
                    </div>
                    <div style={{ fontSize: 12 }}>{isLoading ? "Please allow up to ~5s, if it still doesnt load then something went probably wrong" : ""}</div>
                    <div style={{ fontSize: 12 }}>{isLoading ? "Also the first run is slower because it has to spin up WebAssembly" : ""}</div>
                </div>
            )}
        </div>
    )
}


