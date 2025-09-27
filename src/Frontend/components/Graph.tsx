import React, { useMemo, useState, useCallback, useRef } from 'react'
import { XYChart, AnimatedAxis, AnimatedGrid, AnimatedLineSeries, Tooltip, darkTheme } from '@visx/xychart'
import { localPoint } from '@visx/event'
import { remapCountsToLockedXAxis } from '../features/honing_forecast/HistogramUtils.ts'
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
    additionalBudgets?: number[] | null // Additional set of budget points for second POI
    hasSelection?: boolean
    isLoading?: boolean
    cumulative?: boolean
    // Lock x-axis props
    lockXAxis?: boolean
    lockedMins?: number[] | null
    lockedMaxs?: number[] | null
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
        minimumFractionDigits: 1, // show decimals for small K/M/B
        maximumFractionDigits: place
    })

    return s + suffix
}

/**
 * Calculates the optimal number of decimal places for rounding based on cumulative percentage and data size.
 * The function finds the smallest integer n where |1-cumpct| < 1/10^n, 
 * but caps n such that 10^(n+1) <= data_size.
 * 
 * @param cumpct - The cumulative percentage (0-100)
 * @param data_size - The total number of data points
 * @returns The number of decimal places to use for rounding
 */
function calculateDecimalPlaces(cumpct: number, data_size: number): number {
    // Convert cumpct from percentage to decimal (0-1)
    const cumPctDecimal = cumpct / 100;

    // Calculate the difference from 1
    const diff = Math.abs(1 - cumPctDecimal);
    if (diff == 0) { return 0 }

    // Find the smallest n where diff < 1/10^n
    // This means n > -log10(diff), so n = Math.ceil(-log10(diff))
    let n = Math.ceil(-Math.log10(diff));

    // Cap n such that 10^(n+1) <= data_size
    // This means n+1 <= log10(data_size), so n <= log10(data_size) - 1
    const maxN = Math.floor(Math.log10(data_size)) - 2;

    // Ensure n is at least 0 and at most maxN
    n = Math.max(0, Math.min(n, maxN));

    return n;
}

function to_step(arr: number[]): number[] {
    const n = arr.length;
    if (n === 0) return [];

    // find last non-zero index
    let lastNonZero = -1;
    for (let i = 0; i < n; i++) {
        if (arr[i] !== 0) lastNonZero = i;
    }

    const out: number[] = [];
    let cur = 0;
    let extraUsed = true; // whether we've used the "one extra repeat" after lastNonZero

    if (lastNonZero === -1) {
        // array is all zeros — return zeros
        return new Array(n).fill(0);
    }

    for (let i = 0; i < n; i++) {
        if (arr[i] !== 0) {
            // normal non-zero — update current value
            cur = arr[i];
            // if this is the last non-zero, reset the extraUsed flag so we can allow one extra repeat
            if (i === lastNonZero) extraUsed = false;
        } else {
            // arr[i] is zero
            if (i > lastNonZero) {
                // we're past the last non-zero entry in the input
                if (!extraUsed) {
                    // allow one extra repeat of the last non-zero value
                    cur = arr[lastNonZero];
                    extraUsed = true;
                } else {
                    // after the one extra repeat, force zero
                    cur = 0;
                }
            }
            // else: we're between non-zero values earlier in the array -> keep cur (carry forward)
        }

        out.push(cur);
    }

    return out;
}
function to_step_points(points: Point[]): Point[] {
    const ys = points.map(p => p.y);      // extract y values
    const stepped = to_step(ys);          // reuse our to_step function
    // console.log("stepped", ys, stepped)
    return points.map((p, i) => ({ x: p.x, y: stepped[i] }));
}

function Graph({ title, labels, counts, mins, maxs, width = 640, height = 320, budgets = null, additionalBudgets = null, hasSelection = false, isLoading = false, cumulative = true, lockXAxis = false, lockedMins = null, lockedMaxs = null }: GraphProps) {
    const [visible, setVisible] = useState<boolean[]>(() => [true, true, false, false, false, true, false])
    const [hoverSeries, setHoverSeries] = useState<number | null>(null)
    const [hoverBucket, setHoverBucket] = useState<number | null>(null)
    const chartRef = useRef<HTMLDivElement | null>(null)

    // Create stable pointer handler
    const handleSeriesPointerMove = useCallback((idx: number) => () => {
        setHoverSeries(idx)
    }, [setHoverSeries])

    // Compute "effective" inputs: if lockXAxis is on we use the locked snapshots for axis range
    // and remap incoming counts to those locked ranges when lockedMax > incoming newMax.
    const effectiveCounts: number[][] | null = useMemo(() => {
        if (!counts) return null;
        if (!lockXAxis || !lockedMaxs) return counts;
        // remap using lockedMaxs vs incoming maxs
        const remapped = remapCountsToLockedXAxis(counts, maxs, lockedMaxs);
        // console.log('Lock x-axis enabled:', { lockXAxis, lockedMaxs, maxs, originalCounts: counts, remappedCounts: remapped });
        return remapped;
    }, [counts, lockXAxis, lockedMaxs, maxs]);

    const effectiveMins: number[] | null = useMemo(() => {
        if (!lockXAxis || !lockedMins) return mins ?? null;
        // console.log('Using locked mins:', { lockXAxis, lockedMins, originalMins: mins });
        return lockedMins;
    }, [lockXAxis, lockedMins, mins]);

    const effectiveMaxs: number[] | null = useMemo(() => {
        if (!lockXAxis || !lockedMaxs) return maxs ?? null;
        // console.log('Using locked maxs:', { lockXAxis, lockedMaxs, originalMaxs: maxs });
        return lockedMaxs;
    }, [lockXAxis, lockedMaxs, maxs]);

    const bucketLen = effectiveCounts?.[0]?.length || counts?.[0]?.length || 1000
    const data_size = Math.max(...Array.from({ length: 7 }, (_, i) =>
        (effectiveCounts ?? counts)?.[i]?.reduce((partialSum, a) => partialSum + a, 0))) || 1;

    // Drop any series where all frequency falls in a single bucket (<=1 positive bin)
    const keepMask: boolean[] = useMemo(() => {
        const srcCounts = effectiveCounts ?? counts;
        if (!srcCounts) return new Array(7).fill(false)
        return srcCounts.map(series => {
            let positiveBins = 0
            for (let i = 0; i < series.length && positiveBins <= 1; i++) if (series[i] > 0) positiveBins++
            return positiveBins > 1
        })
    }, [effectiveCounts, counts])

    const cdfSeries: number[][] | null = useMemo(() => {
        const src = effectiveCounts ?? counts;
        if (!src) return null
        return src.map(series => {
            const total = series.reduce((a, b) => a + b, 0) || 1
            let acc = 0
            const out = new Array(series.length)
            for (let i = 0; i < series.length; i++) {
                acc += series[i]
                out[i] = acc / total
            }
            return out
        })
    }, [effectiveCounts, counts])

    const normalizedCounts: number[][] | null = useMemo(() => {
        const src = effectiveCounts ?? counts;
        if (!src) return null
        const denom = data_size || 1
        return src.map(series => series.map(v => v / denom))
    }, [effectiveCounts, counts, data_size])

    const dataSeries: Point[][] = useMemo(() => {
        // console.log("Effective counts", effectiveCounts)
        const srcCounts = effectiveCounts ?? counts;
        if (!srcCounts) return [] as Point[][]
        const source = cumulative && cdfSeries ? cdfSeries : (normalizedCounts || srcCounts)
        let out: Point[][] = Array.from({ length: source.length }, () => []);
        for (let i = 0; i < source.length; i++) {
            let first = true;
            let prev: number | null = null;
            for (let b = 0; b < source[i].length; b++) {

                out[i].push({ x: b, y: source[i][b] })
                first = false;
            }
            if (first) {
                out[i] = [{ x: 0, y: 0 }]
            }
            else {
                out[i].push({ x: source[i].length, y: (prev ?? source[i][source[i].length - 1]) })
            }
        }
        return out
    }, [effectiveCounts, counts, cumulative, cdfSeries, normalizedCounts])

    const yMax: number = useMemo(() => {

        if (!dataSeries) return 0
        if (cumulative) return 1
        // const denom = data_size || 1
        let m = 0

        for (let i = 0; i < dataSeries.length; i++) {
            if (!visible[i] || !keepMask[i]) continue
            let this_series = to_step_points(dataSeries[i])
            for (let j = 0; j < dataSeries[i].length; j++) { m = Math.max(m, this_series[j].y); }
        }
        // console.log(m, data_size, dataSeries, ind)
        return m
    }, [dataSeries, visible, keepMask, cumulative])

    // --- New: detect whether incoming counts contain data outside (to the right of) the effective x-axis ---
    // i.e. when incoming newMax > effectiveMax AND counts beyond that cutoff are non-zero
    const hasUnplottedPoints: boolean = useMemo(() => {
        // Only relevant if we have incoming counts and effective (locked) maxs
        if (!counts || !maxs || !effectiveMaxs) return false;
        const nSeries = counts.length;
        for (let i = 0; i < nSeries; i++) {
            const seriesCounts = counts[i];
            const incomingMax = maxs[i];
            const effectiveMax = effectiveMaxs[i];
            if (incomingMax == null || effectiveMax == null) continue;
            // If effectiveMax already >= incomingMax, nothing is truncated
            if (effectiveMax >= incomingMax) continue;

            // compute incoming bucket width
            const nBuckets = seriesCounts?.length || bucketLen || 1;
            // protect against division by zero
            if (!isFinite(incomingMax) || incomingMax <= 0) {
                // can't reason about ranges; skip this series
                continue;
            }
            const incomingBin = incomingMax / nBuckets;
            // first bucket index whose start >= effectiveMax:
            // bucket j start = j * incomingBin. We want j s.t. j * incomingBin >= effectiveMax
            // so j >= effectiveMax / incomingBin. Use Math.ceil to get the first index that lies fully >= effectiveMax.
            const cutoffIndex = Math.max(0, Math.ceil(effectiveMax / incomingBin));

            if (cutoffIndex < seriesCounts.length) {
                // check if any count beyond cutoffIndex is positive
                for (let b = cutoffIndex; b < seriesCounts.length; b++) {
                    if ((seriesCounts[b] ?? 0) > 0) {
                        return true;
                    }
                }
            }
        }
        return false;
    }, [counts, maxs, effectiveMaxs, bucketLen]);

    // const onLegendToggle = (i: number) => setVisible(v => v.map((b, idx) => idx === i ? !b : b))

    const sortedVisibleIndices: number[] = useMemo(() => {
        const base = labels.map((_, i) => i).filter(i => visible[i] && keepMask[i])
        if (hoverSeries == null) return base
        return base.filter(i => i !== hoverSeries).concat([hoverSeries])
    }, [labels, visible, hoverSeries, keepMask])

    const xAccessor = (d: Point) => { return d.x }
    const yAccessor = (d: Point) => d.y

    const handleMouseMove = useCallback((ev: React.MouseEvent<HTMLDivElement>) => {
        const srcCounts = effectiveCounts ?? counts;
        if (!srcCounts) return
        const p = localPoint(ev)
        const innerW = Math.max(1, width - plotLeft - plotRight)
        const innerH = height - plotTop - plotBottom
        // console.log(p.y)
        // if (p.y < plotTop) { handleMouseLeave(); return }


        const x = Math.min(Math.max((p?.x ?? 0) - plotLeft, 0), innerW)
        const bucket = Math.min(bucketLen - 1, Math.round((x / innerW) * (bucketLen)))
        // choose series with closest y to cursor vertically if possible; otherwise pick the highest y
        const src = cumulative && cdfSeries ? cdfSeries : (normalizedCounts || srcCounts)
        const denomY = Math.max(1e-9, yMax)
        const ys = src.map((s, i) => ({ i, y: to_step(s)[bucket] / denomY * innerH, vis: visible[i] && keepMask[i] }))
        const visibleYs = ys.filter(o => o.vis)
        if (visibleYs.length === 0) { setHoverSeries(null); return }
        // use highest y bucket as proxy for nearest vertically
        const actual_y = innerH + plotBottom - p.y
        const best = visibleYs.reduce((a, b) => Math.abs(a.y - actual_y) < Math.abs(b.y - actual_y) ? a : b)
        setHoverSeries(best.i)
        setHoverBucket(bucket)
    }, [effectiveCounts, counts, visible, keepMask, bucketLen, width, height, yMax, cdfSeries, normalizedCounts, cumulative])

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
        if (fallbackSeries == null || !effectiveMins || !effectiveMaxs) return formatSig3(val)

        const min = effectiveMins[fallbackSeries]
        const max = effectiveMaxs[fallbackSeries]
        const bucketIdx = typeof val === 'number' ? val : Number(val)
        const width = (max - min) / bucketLen
        const mid = min + (bucketIdx) * width
        return formatSig3(mid)
    }, [fallbackSeries, effectiveMins, effectiveMaxs, bucketLen])


    const anyVisible = useMemo(() => labels.some((_, i) => visible[i] && keepMask[i]), [labels, visible, keepMask])

    // Memoize the AnimatedLineSeries nodes
    const lineSeriesNodes = useMemo(() => {
        return sortedVisibleIndices.map((seriesIdx) => (
            <AnimatedLineSeries
                key={labels[seriesIdx]}
                dataKey={labels[seriesIdx]}
                data={to_step_points(dataSeries[seriesIdx]) || []}
                xAccessor={xAccessor}
                yAccessor={yAccessor}
                stroke={SERIES_COLORS_VARS[seriesIdx]}
                strokeWidth={hoverSeries === seriesIdx ? 4 : 1.5}
                opacity={1}
                onPointerMove={handleSeriesPointerMove(seriesIdx)}
            />
        ));
    }, [sortedVisibleIndices, dataSeries, hoverSeries, labels, handleSeriesPointerMove]);

    // Memoize Points Of Interest (POI)
    const poiNodes = useMemo(() => {
        const srcCounts = effectiveCounts ?? counts;
        if (!srcCounts || !anyVisible) return null;
        const innerW = width - plotLeft - plotRight;
        const innerH = height - plotTop - plotBottom;
        const elems: React.ReactElement[] = [];

        const renderFor = (budgetData: number[] | null, keyPrefix: string, circleRadius = 7, strokeColor = "#000") => {
            if (!budgetData) return;
            labels.forEach((_, i) => {
                if (!visible[i] || !keepMask[i]) return;
                const range = Math.max(1e-9, ((effectiveMaxs![i] - effectiveMins![i])));
                const frac = (budgetData[i] - effectiveMins![i]) / range;
                let bucket_idx = Math.max(0, Math.min(bucketLen, Math.floor(frac * (bucketLen))));
                const cx = plotLeft + (bucket_idx / Math.max(1, bucketLen)) * innerW;
                bucket_idx = Math.min(bucket_idx, bucketLen - 1)
                const seriesVals = cumulative && cdfSeries ? cdfSeries[i] : ((normalizedCounts && normalizedCounts[i]) || srcCounts[i]);
                const denomY2 = Math.max(1e-9, yMax);
                // console.log(seriesVals)
                const cy = plotTop + innerH - to_step(seriesVals)[bucket_idx] / denomY2 * innerH;
                const labelText = formatSig3(budgetData[i]);
                const boxW = Math.max(16, labelText.length * 8);
                const boxH = 18;

                elems.push(
                    <g key={`${keyPrefix}-${i}`}>
                        <circle cx={cx} cy={cy} r={circleRadius} fill={SERIES_COLORS_VARS[i]} stroke={strokeColor} strokeWidth={2} />
                        <rect x={cx + 6} y={cy - boxH - 4} width={boxW} height={boxH} fill="rgba(0,0,0,0.5)" rx={3} ry={3} />
                        <text x={cx + 10} y={cy - 8} fill={SERIES_COLORS_VARS[i]} fontSize={12}>{labelText}</text>
                    </g>
                );
            });
        };

        renderFor(budgets, "poi-primary", 5, "#000");
        renderFor(additionalBudgets, "poi-additional", 7, "var(--bright-green)");

        return elems.length ? <g>{elems}</g> : null;
    }, [budgets, additionalBudgets, effectiveCounts, counts, anyVisible, visible, keepMask, width, height, bucketLen, effectiveMins, effectiveMaxs, cdfSeries, normalizedCounts, cumulative, yMax, labels]);

    // Memoize hover marker computation
    const hoverMarker = useMemo(() => {
        const srcCounts = effectiveCounts ?? counts;
        if (fallbackSeries == null || hoverBucket == null || !srcCounts || !visible[fallbackSeries] || !keepMask[fallbackSeries]) return null;
        const innerW = width - plotLeft - plotRight;
        const innerH = height - plotTop - plotBottom;
        const cx = plotLeft + (hoverBucket / Math.max(1, bucketLen)) * innerW;
        let bucket_idx = Math.min(hoverBucket, bucketLen - 1)
        const hoverSeriesVals = cumulative && cdfSeries ? cdfSeries[fallbackSeries] : ((normalizedCounts && normalizedCounts[fallbackSeries]) || srcCounts[fallbackSeries]);
        const denomY3 = Math.max(1e-9, yMax);
        const cy = plotTop + innerH - (to_step(hoverSeriesVals)[bucket_idx] / denomY3) * innerH;
        return { cx, cy, series: fallbackSeries };
    }, [fallbackSeries, hoverBucket, effectiveCounts, counts, visible, keepMask, cumulative, cdfSeries, normalizedCounts, bucketLen, width, height, yMax]);

    return (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8, padding: 16, borderRadius: 16, backgroundColor: 'var(--bg-tertiary)' }}>
            {title ? (
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <div style={{ color: 'var(--text-primary)', fontSize: "var(--font-size-lg)", fontWeight: 600 }}>{title}</div>
                    {lockXAxis && effectiveMaxs && fallbackSeries != null ? (
                        <div
                            role="status"
                            aria-live="polite"
                            title="Graph may look a bit wonky in some cases, unlock x-axis to get a cleaner graph"
                            style={{
                                display: 'inline-flex',
                                alignItems: 'center',
                                gap: 6,
                                background: 'var(--warning-bg)',
                                color: 'white',
                                padding: '4px 8px',
                                borderRadius: 8,
                                fontSize: 12,
                                border: '1px solid var(--warning-border)'
                            }}
                        >
                            x-axis locked at {formatSig3(effectiveMaxs[fallbackSeries])}
                            <span style={{ color: SERIES_COLORS_VARS[fallbackSeries] }}>{labels[fallbackSeries].padEnd(Math.max(...labels.slice(0, 7).map(l => l.length)), ' ')}</span>
                            {hasUnplottedPoints && (
                                <span>- Some trials used more and were not plotted</span>
                            )}
                        </div>
                    ) : null}
                </div>
            ) : null}
            <div ref={chartRef} onMouseMove={handleMouseMove} onMouseLeave={handleMouseLeave}>
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
                    {lineSeriesNodes}

                    {/* Primary & additional POI */}
                    {poiNodes}

                    {/* Hover point snapped to selected series */}
                    {hoverMarker && (
                        <g>
                            <circle cx={hoverMarker.cx} cy={hoverMarker.cy} r={4.5} fill={SERIES_COLORS_VARS[hoverMarker.series]} stroke="#fff" strokeWidth={2} />
                        </g>
                    )}

                    <Tooltip
                        snapTooltipToDatumX
                        snapTooltipToDatumY={true}
                        showSeriesGlyphs={false}
                        renderTooltip={() => {
                            if (fallbackSeries == null || hoverBucket == null || !effectiveMins || !effectiveMaxs) return null
                            const min = effectiveMins[fallbackSeries]
                            const max = effectiveMaxs[fallbackSeries]
                            const widthRange = Math.max(1e-9, max - min);
                            const mid = min + (hoverBucket + 0.5) * (widthRange / bucketLen)
                            const srcCounts = effectiveCounts ?? counts;
                            const cumPctRaw = srcCounts[fallbackSeries].slice(0, hoverBucket + 1).reduce((partialSum, a) => partialSum + a, 0) / data_size * 100
                            const decimalPlaces = calculateDecimalPlaces(cumPctRaw, data_size)
                            const cumPct = cumPctRaw.toFixed(decimalPlaces)
                            return (
                                <div style={{ color: 'var(--text-primary)' }}>
                                    <div style={{ color: SERIES_COLORS_VARS[fallbackSeries], fontWeight: 600 }}>{labels[fallbackSeries]}</div>
                                    <div>In a room of 100 people,</div>
                                    <div>{cumPct} used less than {formatSig3(mid, 3)} {labels[fallbackSeries]}</div>
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
                <div style={{ marginTop: -250, height: 300 }}>
                    <div style={{ color: 'var(--text-secondary)', fontSize: 50, alignSelf: 'center', justifySelf: 'center' }}>
                        {hasSelection ? isLoading ? "Loading..." : 'Nothing to plot, couple possibilities:' : 'Nothing to plot, tick an upgrade!'}
                    </div>
                    <div style={{ fontSize: 16 }}>{isLoading && hasSelection ? "Please allow up to ~5s, if it still doesnt load then gg" : ""}</div>
                    <div style={{ fontSize: 16 }}>{isLoading && hasSelection ? "Also the first run is slower because it has to spin up WebAssembly" : ""}</div>
                    <div style={{ fontSize: 16 }}>{!isLoading && hasSelection ? "1. All your ticks have 100% success rate(+1 to +3)" : ""}</div>
                    <div style={{ fontSize: 16 }}>{!isLoading && hasSelection ? "2. The x-axis was locked at too high a value, so everything fell within the first pixel/tick.(Both situations are due to every point landing on the same x value)" : ""}</div>
                </div>
            )}
        </div>
    )
}

// Export memoized component for performance
export default React.memo(Graph)
