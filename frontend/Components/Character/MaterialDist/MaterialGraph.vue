<script setup lang="ts">
import { BUCKET_COUNT, FLOAT_TOL, GRAPH_FONT_SIZE, GRAPH_HEIGHT } from "@/Utils/Constants"
import { cssVar } from "@/Utils/Helpers"
import { computed, onBeforeUnmount, onMounted, ref } from "vue"

type DataPoint = [number, number]

type Point = {
    x: number
    y: number
    cumulativeY: number // Always stores the raw cumulative Y regardless of slope calculation
}

const props = withDefaults(
    defineProps<{
        data: DataPoint[] | null | undefined
        graphColor?: string
        cumulative: boolean
        height?: number
        materialLabel: string
        // Generalized Annotations
        annotations?: number[]
        annotationColors?: string[]
        annotationPositions?: ("top" | "middle" | "bottom" | "graph")[]
        annotationLabels?: string[]

        // Tooltip Function
        tooltipTextFn?: (x: number, y: number, cumulativeY: number, material: string, color: string) => string
    }>(),
    {
        graphColor: "--hf-graph-default-color",
        height: 40,
        annotations: () => [],
        annotationColors: () => [],
        annotationPositions: () => [],
        annotationLabels: () => [],
        // Default tooltip just displays values
        tooltipTextFn: (x, y, cy, material, color) => `<b>X:</b> ${x} <br/> <b>Y:</b> ${cy}`,
    },
)

const rootRef = ref<HTMLElement | null>(null)
const width = ref(0)
const mouseX = ref<number | null>(null)
let observer: ResizeObserver | null = null

const plotWidth = computed(() => Math.max(100, width.value))

const points = computed<Point[]>(() => {
    if (!Array.isArray(props.data) || props.data.length === 0) {
        return []
    }
    // console.log(props.data)
    const normalized = props.data
        .map((pair) => [Number(pair?.[0] ?? 0), Number(pair?.[1] ?? 0)] as DataPoint)
        .filter((pair) => Number.isFinite(pair[0]) && Number.isFinite(pair[1]))

    if (normalized.length < 2) {
        return normalized.map(([x, y]) => ({ x, y, cumulativeY: y }))
    }

    if (props.cumulative) {
        return normalized.map(([x, y]) => ({ x, y, cumulativeY: y }))
    }
    let gap_size = (normalized[normalized.length - 1][0] - normalized[0][0]) / BUCKET_COUNT
    let prevSlope = 0
    return normalized.map(([x, y], index) => {
        if (index === 0) {
            const nextX = normalized[1][0]
            const dx = nextX - x
            const slope = dx === 0 ? 0 : y / dx
            prevSlope = slope
            return { x, y: slope, cumulativeY: y }
        }

        const [prevX, prevY] = normalized[index - 1]
        let this_gap = x - prevX
        if (Math.abs(this_gap - gap_size) > FLOAT_TOL) {
            return { x, y: prevSlope, cumulativeY: y }
        }
        const dx = x - prevX
        const slope = dx === 0 ? 0 : (y - prevY) / dx
        if (slope < FLOAT_TOL) {
            const out = { x, y: prevSlope, cumulativeY: y }
            return out
        }
        prevSlope = slope
        return { x, y: slope, cumulativeY: y }
    })
})

const maxX = computed(() => {
    if (!points.value.length) return 0
    return Math.max(0, ...points.value.map((point) => point.x))
})

const maxY = computed(() => {
    if (!points.value.length) return 1
    return Math.max(1e-9, ...points.value.map((point) => point.y))
})

const isEmpty = computed(() => points.value.length === 0 || maxX.value <= 0)

function scaleX(x: number) {
    if (maxX.value <= 0) return 0
    return (x / maxX.value) * plotWidth.value
}

function scaleY(y: number) {
    if (maxY.value <= 0) return GRAPH_HEIGHT
    const normalized = y / (maxY.value * 1.1)
    return Math.max(0, Math.min(GRAPH_HEIGHT, GRAPH_HEIGHT - normalized * GRAPH_HEIGHT))
}

function interpolateY(targetX: number) {
    const list = points.value
    if (!list.length) return GRAPH_HEIGHT

    if (targetX <= list[0].x) return scaleY(list[0].y)

    for (let index = 1; index < list.length; index++) {
        const prev = list[index - 1]
        const cur = list[index]
        if (targetX <= cur.x) {
            const dx = cur.x - prev.x
            const t = dx === 0 ? 0 : (targetX - prev.x) / dx
            const y = prev.y + (cur.y - prev.y) * t
            return scaleY(y)
        }
    }

    return scaleY(list[list.length - 1].y)
}

const linePath = computed(() => {
    if (!points.value.length) return ""
    return points.value.map((point, index) => `${index === 0 ? "M" : "L"}${scaleX(point.x)} ${scaleY(point.y)}`).join(" ")
})

const areaPath = computed(() => {
    if (!points.value.length) return ""
    const firstX = scaleX(points.value[0].x)
    const lastX = scaleX(points.value[points.value.length - 1].x)
    return `${linePath.value} L ${lastX} ${GRAPH_HEIGHT} L ${firstX} ${GRAPH_HEIGHT} Z`
})

// Mouse and Hover Logic
function onMouseMove(event: MouseEvent) {
    if (!rootRef.value) return
    const rect = rootRef.value.getBoundingClientRect()
    mouseX.value = event.clientX - rect.left
}

function onMouseLeave() {
    mouseX.value = null
}

const hoveredPoint = computed(() => {
    if (mouseX.value === null || !points.value.length) return null

    // Snap to the closest point along the X axis
    let closest = points.value[0]
    let minDistance = Math.abs(scaleX(closest.x) - mouseX.value)

    for (const point of points.value) {
        const dist = Math.abs(scaleX(point.x) - mouseX.value)
        if (dist < minDistance) {
            minDistance = dist
            closest = point
        }
    }
    return closest
})

// Generalize Annotations Output
function clamp(value: number, min: number, max: number) {
    return Math.max(min, Math.min(max, value))
}

function badgeWidth(text: string) {
    return Math.min(220, (text.length * GRAPH_FONT_SIZE) / 2) // why / 2 ? idk
}

const processedAnnotations = computed(() => {
    if (isEmpty.value) return []

    return props.annotations
        .filter((xVal) => xVal !== null)
        .map((xVal, i) => {
            const color = cssVar(props.annotationColors[i] || "white", "white")
            const positionStr = props.annotationPositions[i] || "top"
            const label = props.annotationLabels[i] || ""

            const svgX = scaleX(xVal)
            const svgY = interpolateY(xVal)
            const textW = badgeWidth(label)

            // Clamp text X so it never falls off the chart left/right
            // console.log(label, xVal, svgX, textW, textW / 2)
            const clampedX = clamp(svgX, textW / 2, plotWidth.value - textW / 2)

            let textY = GRAPH_FONT_SIZE + 4
            if (positionStr === "bottom") textY = GRAPH_HEIGHT
            if (positionStr === "middle") textY = (GRAPH_HEIGHT + GRAPH_FONT_SIZE + 4) / 2
            if (positionStr === "graph") textY = clamp(svgY - GRAPH_FONT_SIZE, GRAPH_FONT_SIZE + 4, GRAPH_HEIGHT - GRAPH_FONT_SIZE)
            return {
                xVal,
                svgX,
                svgY,
                color,
                label,
                textY,
                clampedX,
            }
        })
})

const resolvedColor = computed(() => cssVar(props.graphColor, props.graphColor))
const axisColor = computed(() => cssVar("--text-very-muted", "#9aa3b2"))
const surfaceTextColor = computed(() => cssVar("--hf-text-main", "#c9d0da"))

onMounted(() => {
    const updateWidth = () => {
        if (!rootRef.value) return
        width.value = Math.max(320, Math.floor(rootRef.value.getBoundingClientRect().width))
    }

    updateWidth()
    observer = new ResizeObserver(updateWidth)
    if (rootRef.value) {
        observer.observe(rootRef.value)
    }
})

onBeforeUnmount(() => {
    observer?.disconnect()
    observer = null
})
</script>

<template>
    <div ref="rootRef" class="hf-material-graph" @mousemove="onMouseMove" @mouseleave="onMouseLeave">
        <svg :width="Math.max(width, 320)" :height="height" role="img" aria-label="Material graph">
            <defs>
                <linearGradient :id="`hf-graph-fill-${graphColor.replace(/[^a-z0-9]/gi, '')}`" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" :stop-color="resolvedColor" stop-opacity="0.38" />
                    <stop offset="100%" :stop-color="resolvedColor" stop-opacity="0.08" />
                </linearGradient>
            </defs>

            <g>
                <line x1="0" :x2="plotWidth" :y1="GRAPH_HEIGHT" :y2="GRAPH_HEIGHT" :stroke="axisColor" stroke-width="1" />

                <path v-if="!isEmpty" :d="areaPath" :fill="`url(#hf-graph-fill-${graphColor.replace(/[^a-z0-9]/gi, '')})`" stroke="none" />
                <path v-if="!isEmpty" :d="linePath" fill="none" :stroke="resolvedColor" stroke-width="2.1" />

                <g v-for="(annotation, index) in processedAnnotations" :key="index">
                    <line
                        :x1="annotation.svgX"
                        :x2="annotation.svgX"
                        :y1="GRAPH_HEIGHT"
                        :y2="annotation.svgY"
                        :stroke="annotation.color"
                        stroke-width="1"
                        stroke-dasharray="4 4"
                    />
                    <circle :cx="annotation.svgX" :cy="annotation.svgY" r="3.5" :fill="annotation.color" />

                    <text :x="annotation.clampedX" :y="annotation.textY" text-anchor="middle" :font-size="GRAPH_FONT_SIZE" :fill="annotation.color">
                        {{ annotation.label }}
                    </text>
                </g>

                <g v-if="hoveredPoint">
                    <line
                        :x1="scaleX(hoveredPoint.x)"
                        :x2="scaleX(hoveredPoint.x)"
                        :y1="GRAPH_HEIGHT"
                        :y2="scaleY(hoveredPoint.y)"
                        stroke="var(--hf-graph-hover-color, #ffffff)"
                        stroke-width="1"
                    />
                    <circle :cx="scaleX(hoveredPoint.x)" :cy="scaleY(hoveredPoint.y)" r="4" fill="var(--hf-graph-hover-color, #ffffff)" />
                </g>

                <text v-if="isEmpty" x="4" :y="Math.max(16, GRAPH_HEIGHT * 0.55)" font-size="12" :fill="surfaceTextColor" opacity="0.85">
                    This material is never used.
                </text>
            </g>
        </svg>

        <div
            v-if="hoveredPoint"
            class="hf-graph-tooltip"
            :style="{ left: scaleX(hoveredPoint.x) + 'px' }"
            v-html="props.tooltipTextFn!(hoveredPoint.x, hoveredPoint.y, hoveredPoint.cumulativeY, materialLabel, resolvedColor)"
        ></div>
    </div>
</template>

<style>
.hf-material-graph {
    width: 100%;
    position: relative; /* Essential for tooltip positioning */
    cursor: crosshair; /* Helps signal it's interactive */
}

/* Tooltip container styling */
.hf-graph-tooltip {
    position: absolute;
    transform: translateX(-110%) translateY(-100%);
    background: var(--tooltip-bg);
    color: var(--hf-text-main);
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    pointer-events: none; /* Prevents tooltip from interfering with mousemove events */
    white-space: nowrap;
    border: 1px solid var(--text-very-muted);
    box-shadow: 0px 4px 12px rgba(0, 0, 0, 0.2);
    z-index: 100;
}
</style>
