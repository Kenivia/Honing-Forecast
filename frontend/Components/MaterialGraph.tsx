import React, { useMemo, useCallback } from "react"
import { AreaClosed, LinePath, Bar } from "@visx/shape"
import { curveMonotoneX } from "@visx/curve"
import { scaleLinear } from "@visx/scale"
import { AxisBottom } from "@visx/axis"
import { localPoint } from "@visx/event"
import { useTooltip, useTooltipInPortal, defaultStyles } from "@visx/tooltip"
import { ParentSize } from "@visx/responsive"
import { bisector } from "d3-array"
import { IconMap } from "@/Utils/Constants.ts"
import { powerOfTenToWords } from "@/Utils/Helpers.ts"
function getCssVar(name: string) {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}
// --- Types ---

// Matches the JS output of Vec<(f64, f64)>
type DataPoint = [number, number]

interface MaterialGraphProps {
    /** The serialized data: Vec<(f64, f64)> -> Array of [x, y] */
    data: DataPoint[] | null
    /** The average x-value */
    average: number | null
    /** A secondary annotation x-value */
    secondaryAnnotation?: number | null
    /** Primary color for the line and fill */
    color: string
    /** Toggle between CDF (true) and PDF (false) */
    cumulative: boolean
    /** Height of the individual graph component */
    height?: number
    label: string
}

// Accessors for our data tuples
const getX = (d: DataPoint) => d[0]
const getY = (d: DataPoint) => d[1]

// D3 bisector to find the closest index based on X
const bisectDate = bisector<DataPoint, number>((d) => d[0]).left

const GraphContent = ({ width, height, data, average, secondaryAnnotation, color, cumulative, label }: MaterialGraphProps & { width: number }) => {
    // 1. Data Processing
    // If data is null, we just render an empty container later
    // Compute the data to display (CDF or PDF)
    const displayData = useMemo(() => {
        if (cumulative) return data

        // Calculate PDF: slope between points (dy/dx)
        // Assuming uniform x-steps usually, but this formula handles variable steps
        let prev_slope = 0
        return data.map((point, i) => {
            if (i === 0) return [point[0], point[1] / (data[1][0] - data[0][0])] as DataPoint
            const prev = data[i - 1]

            const dx = point[0] - prev[0]
            const dy = point[1] - prev[1]
            // Avoid division by zero
            const slope = dx === 0 ? 0 : dy / dx
            if (slope == 0) {
                let out = [point[0], prev_slope] as DataPoint
                prev_slope = 0
                return out
            }
            prev_slope = slope
            return [point[0], slope] as DataPoint
        })
    }, [data, cumulative])

    // 2. Scales
    const margin = useMemo(() => {
        return { top: 20, right: 20, bottom: 25, left: 10 }
    }, [])
    const xMax = width - margin.left - margin.right
    const yMax = height - margin.top - margin.bottom

    const min_cost = Math.min(...displayData.map(getX).filter((x) => x > 0))
    const max_cost = Math.max(...displayData.map(getX))
    const xScale = useMemo(
        () =>
            scaleLinear({
                range: [0, xMax],
                domain: [0, max_cost],
            }),
        [xMax, displayData],
    )

    const yScale = useMemo(
        () =>
            scaleLinear({
                range: [yMax, 0],
                domain: [0, Math.max(...displayData.map(getY)) * 1.1], // Add 10% headroom
                nice: true,
            }),
        [yMax, displayData],
    )

    // 3. Tooltip Logic
    const { tooltipOpen, tooltipLeft, tooltipTop, tooltipData, hideTooltip, showTooltip } = useTooltip<DataPoint>()
    const { containerRef, TooltipInPortal } = useTooltipInPortal({
        scroll: true,
    })

    const handleTooltip = useCallback(
        (event: React.TouchEvent<SVGRectElement> | React.MouseEvent<SVGRectElement>) => {
            const { x } = localPoint(event) || { x: 0 }
            const x0 = xScale.invert(x - margin.left)

            // Find closest point in data
            const index = bisectDate(displayData, x0, 1)
            const d0 = displayData[index - 1]
            const d1 = displayData[index]
            let d = d0
            if (d1 && d0) {
                d = x0 - getX(d0) > getX(d1) - x0 ? d1 : d0
            }

            // REQUIREMENT 6: Tooltip must show CUMULATIVE value regardless of mode.
            // We find the original CDF value corresponding to this X.
            // Since arrays are parallel, we can use the same index (or re-search if strictly safe).
            // Here we assume displayData maps 1:1 to props.data
            const originalIndex = displayData.indexOf(d)
            const originalDatum = data[originalIndex]

            showTooltip({
                tooltipData: originalDatum, // Pass the CDF datum to the tooltip
                tooltipLeft: xScale(getX(d)) + margin.left,
                tooltipTop: yScale(getY(d)) + margin.top, // Snap annotation to the visible curve
            })
        },
        [showTooltip, xScale, yScale, displayData, data, margin],
    )

    // 4. Annotations Helpers
    // Helper to find Y position for specific X on the *visible* graph
    const getYForX = (targetX: number) => {
        // Find closest data point to targetX
        const idx = bisectDate(displayData, targetX, 1)
        const p0 = displayData[idx - 1]
        const p1 = displayData[idx]

        // Simple interpolation for smoother placement
        if (!p0) return 0
        if (!p1) return yScale(getY(p0))

        // Linear interpolate Y
        const rangeX = getX(p1) - getX(p0)
        const rangeY = getY(p1) - getY(p0)
        const dist = targetX - getX(p0)
        const interpolatedY = getY(p0) + (dist / rangeX) * rangeY

        return yScale(interpolatedY)
    }
    const resolvedColor = getCssVar(color)

    const inputColor = getCssVar("--grid-cell-bg")
    const hoverColor = getCssVar("--bright-green")
    const axisColor = getCssVar("--text-very-muted")
    // if (!data || data.length === 0 || (data.map((x) => x[0]).reduce((prev, next) => Math.max(prev, next))) == 0) return null
    const isEmpty = data.map((x) => x[0]).reduce((prev, next) => Math.max(prev, next)) == 0
    // console.log(data.length, label)
    // console.log(secondaryAnnotation, xScale(secondaryAnnotation))

    let y_value = tooltipData ? getY(tooltipData) : null
    let place = Math.max(Math.ceil(y_value < 0.5 ? Math.abs(Math.log10(y_value)) : Math.abs(Math.log10(1 - y_value))), 2)

    const cutoff = y_value < 0.5 ? 65 : 15 // f64 precision shinanigans, near 0 should be able to go lower? but gonna stop there cos that's where english runs out of names
    const too_big = place > cutoff
    if (too_big) {
        y_value = y_value < 0.5 ? 0 : 1
    }
    place = Math.min(place, cutoff)
    const base = Math.pow(10, place)
    const rounded = tooltipData ? (too_big ? (y_value < 0.5 ? 0 : base) : Math.round(getY(tooltipData) * base)) : null

    const is_edge = tooltipData && ((getX(tooltipData) < min_cost && y_value <= 0) || getX(tooltipData) == max_cost)

    const numberFormat = { maximumFractionDigits: 0, minimumFractionDigits: 0 }
    const formattedX = tooltipData !== undefined ? getX(tooltipData).toLocaleString("en-US", numberFormat) : null
    const formattedRounded = rounded !== null ? rounded.toLocaleString("en-US", numberFormat) : null
    const formattedFailCount = (base - rounded).toLocaleString("en-US", numberFormat)

    const isSmallProb = y_value < 0.5 || place < 4
    // console.log(tooltipData, displayData)
    return (
        // ref for the tooltip portal
        <div ref={containerRef} style={{ position: "relative" }}>
            <svg width={width} height={height}>
                <defs>
                    <linearGradient id={`gradient-${resolvedColor}`} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={resolvedColor} stopOpacity={0.4} />
                        <stop offset="100%" stopColor={resolvedColor} stopOpacity={0.1} />
                    </linearGradient>
                </defs>

                <g transform={`translate(${margin.left},${margin.top})`}>
                    {/* Main Area (Shaded) */}
                    <AreaClosed<DataPoint>
                        data={displayData}
                        x={(d) => xScale(getX(d))}
                        y={(d) => yScale(getY(d))}
                        yScale={yScale}
                        fill={`url(#gradient-${resolvedColor})`}
                        curve={curveMonotoneX}
                    />

                    {/* Main Line */}
                    <LinePath<DataPoint>
                        data={displayData}
                        x={(d) => xScale(getX(d))}
                        y={(d) => yScale(getY(d))}
                        stroke={resolvedColor}
                        strokeWidth={2}
                        curve={curveMonotoneX}
                    />

                    {/* Annotation 1: Average */}
                    {average !== null && (
                        <g>
                            <line
                                x1={xScale(average)}
                                x2={xScale(average)}
                                y1={yMax}
                                y2={getYForX(average)}
                                stroke="white"
                                strokeWidth={1}
                                strokeDasharray="4,4"
                            />
                            <circle cx={xScale(average)} cy={getYForX(average)} r={4} fill="white" />
                            <image href={IconMap[label]}></image>
                            <text
                                x={Math.max(Math.min(xScale(average), xMax - 40), 40)}
                                y={isEmpty ? -5 : 0}
                                textAnchor="middle"
                                fontSize={14}
                                fill={resolvedColor}
                            >
                                Average = {average.toLocaleString("en-US", { maximumFractionDigits: 0, minimumFractionDigits: 0 })}
                            </text>
                        </g>
                    )}

                    {/* Annotation 2: Secondary */}
                    {secondaryAnnotation != null && !isEmpty && (
                        <g>
                            <line
                                x1={xScale(secondaryAnnotation)}
                                x2={xScale(secondaryAnnotation)}
                                y1={yMax}
                                y2={getYForX(secondaryAnnotation)}
                                stroke={inputColor}
                                strokeWidth={1}
                                strokeDasharray="2,2"
                            />
                            <circle
                                cx={xScale(secondaryAnnotation)}
                                cy={getYForX(secondaryAnnotation)}
                                r={4}
                                fill={inputColor}
                                stroke={color}
                                strokeWidth={2}
                            />
                            <text
                                x={Math.max(Math.min(xScale(secondaryAnnotation), xMax - 40), 40)}
                                y={yMax - 3}
                                textAnchor="middle"
                                fontSize={14}
                                fill={inputColor}
                            >
                                You have: {secondaryAnnotation.toLocaleString("en-US", { maximumFractionDigits: 0, minimumFractionDigits: 0 })}
                            </text>
                        </g>
                    )}

                    {tooltipOpen && tooltipData != undefined && !isEmpty && (
                        <g>
                            <line
                                x1={xScale(getX(tooltipData))}
                                x2={xScale(getX(tooltipData))}
                                y1={yMax}
                                y2={0}
                                stroke={hoverColor}
                                strokeWidth={1}
                                strokeDasharray="2,2"
                            />
                            <circle cx={xScale(getX(tooltipData))} cy={getYForX(getX(tooltipData))} r={4} fill={hoverColor} stroke={color} strokeWidth={2} />
                        </g>
                    )}

                    {/* Transparent Overlay for Hover Events */}
                    <Bar
                        x={0}
                        y={0}
                        width={xMax}
                        height={yMax}
                        fill="transparent"
                        rx={14}
                        onTouchStart={handleTooltip}
                        onTouchMove={handleTooltip}
                        onMouseMove={handleTooltip}
                        onMouseLeave={() => hideTooltip()}
                    />

                    {/* Axis Bottom - 0.1 steps logic */}
                    <AxisBottom
                        scale={xScale}
                        top={yMax}
                        // Force ticks to specific interval logic if needed,
                        // or let Visx handle "approx 0.1" via numTicks
                        numTicks={width > 500 ? 10 : 5}
                        stroke={axisColor}
                        tickStroke={axisColor}
                        tickLabelProps={() => ({
                            fill: axisColor,
                            fontSize: 10,
                            textAnchor: "middle",
                        })}
                    />
                </g>
            </svg>

            {/* Tooltip Content */}
            {tooltipOpen && tooltipData && (
                <TooltipInPortal
                    top={tooltipTop}
                    left={Math.min(tooltipLeft, xMax * (is_edge ? 0.85 : 0.8))}
                    style={{ ...defaultStyles, backgroundColor: "rgba(0, 0, 0, 0.41)", color: "white" }}
                >
                    <div style={{ fontSize: 11 }}>
                        {is_edge ? (
                            <>
                                <strong>{(isSmallProb ? "Impossible to succeed within " : "Impossible to fail after ") + formattedX + " "}</strong>
                                <span>{label}</span>
                            </>
                        ) : (
                            <>
                                {isSmallProb ? (
                                    <>
                                        <span style={{ fontWeight: "bold", color: hoverColor }}>
                                            {too_big && "~"}
                                            {(y_value * 100).toPrecision(3)}%
                                        </span>{" "}
                                        chance to succeed within{" "}
                                    </>
                                ) : (
                                    <>
                                        {}
                                        <span style={{ fontWeight: "bold", color: hoverColor }}>
                                            {too_big && "~"}
                                            {((1 - y_value) * 100).toPrecision(too_big ? 1 : 3)}%
                                        </span>{" "}
                                        chance to fail after{" "}
                                    </>
                                )}
                                {formattedX} <span>{label}</span>
                                <br />
                                {/* Room description */}
                                {place > 3 && (
                                    <>
                                        In a room of <strong>{powerOfTenToWords(place)} </strong>people,
                                        <strong>
                                            {isSmallProb ? (
                                                ` ${formattedRounded}`
                                            ) : (
                                                <>
                                                    {formattedFailCount !== "0" && " Only "}
                                                    {" " + formattedFailCount + " "}
                                                </>
                                            )}
                                        </strong>
                                        {isSmallProb ? " will succeed" : " will fail"}{" "}
                                    </>
                                )}
                            </>
                        )}
                    </div>
                </TooltipInPortal>
            )}
        </div>
    )
}

// --- Main Wrapper Component ---

export const MaterialGraph: React.FC<MaterialGraphProps> = (props) => {
    const height = props.height || 150 // Default height

    // 8. Null State: Show empty graph container (height preserved)
    if (!props.data) {
        return (
            <div style={{ width: "100%", height, backgroundColor: "#f5f5f5", border: "1px dashed #ccc" }}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "#999", fontSize: "12px" }}>
                    No Data Available
                </div>
            </div>
        )
    }

    return (
        <div style={{ width: "100%", height }}>
            <ParentSize>{({ width }) => <GraphContent width={width} height={height} {...props} />}</ParentSize>
        </div>
    )
}
