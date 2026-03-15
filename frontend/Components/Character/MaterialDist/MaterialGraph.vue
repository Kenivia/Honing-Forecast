<script setup lang="ts">
import { GRAPH_FONT_SIZE, GRAPH_HEIGHT } from "@/Utils/Constants"
import { cssVar } from "@/Utils/Helpers"
import { computed, onBeforeUnmount, onMounted, ref } from "vue"

type DataPoint = [number, number]

type Point = {
    x: number
    y: number
}

const props = withDefaults(
    defineProps<{
        data: DataPoint[] | null | undefined
        average?: number | null
        secondaryAnnotation?: number | null
        colorVar: string
        cumulative: boolean
        height?: number
    }>(),
    {
        average: null,
        secondaryAnnotation: null,
        height: 40,
    },
)

const rootRef = ref<HTMLElement | null>(null)
const width = ref(0)
let observer: ResizeObserver | null = null

const plotWidth = computed(() => Math.max(100, width.value))

const points = computed<Point[]>(() => {
    if (!Array.isArray(props.data) || props.data.length === 0) {
        return []
    }

    const normalized = props.data
        .map((pair) => [Number(pair?.[0] ?? 0), Number(pair?.[1] ?? 0)] as DataPoint)
        .filter((pair) => Number.isFinite(pair[0]) && Number.isFinite(pair[1]))

    if (normalized.length < 2) {
        return normalized.map(([x, y]) => ({ x, y }))
    }

    if (props.cumulative) {
        return normalized.map(([x, y]) => ({ x, y }))
    }

    let prevSlope = 0
    return normalized.map(([x, y], index) => {
        if (index === 0) {
            const nextX = normalized[1][0]
            const dx = nextX - x
            const slope = dx === 0 ? 0 : y / dx
            prevSlope = slope
            return { x, y: slope }
        }
        const [prevX, prevY] = normalized[index - 1]
        const dx = x - prevX
        const slope = dx === 0 ? 0 : (y - prevY) / dx
        if (slope === 0) {
            const out = { x, y: prevSlope }
            prevSlope = 0
            return out
        }
        prevSlope = slope
        return { x, y: slope }
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

    if (targetX <= list[0].x) {
        return scaleY(list[0].y)
    }

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
    const commands = points.value.map((point, index) => `${index === 0 ? "M" : "L"}${scaleX(point.x)} ${scaleY(point.y)}`)
    return commands.join(" ")
})

const areaPath = computed(() => {
    if (!points.value.length) return ""
    const firstX = scaleX(points.value[0].x)
    const lastX = scaleX(points.value[points.value.length - 1].x)
    return `${linePath.value} L ${lastX} ${GRAPH_HEIGHT} L ${firstX} ${GRAPH_HEIGHT} Z`
})

const averageY = computed(() => {
    if (props.average === null || props.average === undefined) return null
    return interpolateY(props.average)
})

const secondaryY = computed(() => {
    if (props.secondaryAnnotation === null || props.secondaryAnnotation === undefined) return null
    return interpolateY(props.secondaryAnnotation)
})

const xTicks = computed(() => {
    if (maxX.value <= 0) return []
    const count = 6
    return Array.from({ length: count + 1 }, (_, index) => {
        const value = (maxX.value * index) / count
        return {
            value,
            x: scaleX(value),
        }
    })
})

const resolvedColor = computed(() => {
    return cssVar(props.colorVar, props.colorVar)
})

const resolvedBoundColor = computed(() => {
    return cssVar("--hf-graph-bound-color", "--input-bg")
})
const resolvedAvgColor = computed(() => {
    return cssVar("--hf-graph-average-color", "white")
})

const resolvedTradableColor = computed(() => {
    return cssVar("--hf-graph-tradable-color", "--input-bg")
})

function formatWhole(value: number) {
    return Math.round(value).toLocaleString("en-US")
}
function clamp(value: number, min: number, max: number) {
    return Math.max(min, Math.min(max, value))
}
function badgeWidth(text: string) {
    return Math.min(220, text.length * 7)
}

const averageLabel = computed(() => {
    if (props.average === null || props.average === undefined) return null
    return `Avg`
})

const averageBadge = computed(() => {
    if (!averageLabel.value || props.average === null || props.average === undefined || averageY.value === null) return null
    const widthPx = badgeWidth(averageLabel.value)
    const endX = clamp(scaleX(props.average), widthPx, plotWidth.value - widthPx)
    return {
        text: averageLabel.value,
        width: widthPx,
        endX: endX + widthPx / 2,
        textY: GRAPH_FONT_SIZE,
    }
})

const ownedLabel = computed(() => {
    if (props.secondaryAnnotation === null || props.secondaryAnnotation === undefined) return null
    return `Bound`
})

const ownedBadge = computed(() => {
    if (!ownedLabel.value || props.secondaryAnnotation === null || props.secondaryAnnotation === undefined || secondaryY.value === null) return null
    const widthPx = badgeWidth(ownedLabel.value)
    const endX = clamp(scaleX(props.secondaryAnnotation), widthPx, plotWidth.value - widthPx)
    const textY = clamp(secondaryY.value, 18, GRAPH_HEIGHT - GRAPH_FONT_SIZE - 2)
    return {
        text: ownedLabel.value,
        width: widthPx,
        endX,
        textY,
    }
})

const axisColor = computed(() => cssVar("--text-very-muted", "#9aa3b2"))
const surfaceTextColor = computed(() => cssVar("--hf-text-main", "#c9d0da"))
const badgeFill = computed(() => cssVar("--hf-bg-void", "#08090b"))

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
    <div ref="rootRef" class="hf-material-graph">
        <svg :width="Math.max(width, 320)" :height="height" role="img" aria-label="Material graph">
            <defs>
                <linearGradient :id="`hf-graph-fill-${colorVar.replace(/[^a-z0-9]/gi, '')}`" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" :stop-color="resolvedColor" stop-opacity="0.38" />
                    <stop offset="100%" :stop-color="resolvedColor" stop-opacity="0.08" />
                </linearGradient>
            </defs>

            <g>
                <line x1="0" :x2="plotWidth" :y1="GRAPH_HEIGHT" :y2="GRAPH_HEIGHT" :stroke="axisColor" stroke-width="1" />

                <path v-if="!isEmpty" :d="areaPath" :fill="`url(#hf-graph-fill-${colorVar.replace(/[^a-z0-9]/gi, '')})`" stroke="none" />
                <path v-if="!isEmpty" :d="linePath" fill="none" :stroke="resolvedColor" stroke-width="2.1" />

                <g v-if="average !== null && average !== undefined && averageY !== null && averageBadge">
                    <line
                        :x1="scaleX(average)"
                        :x2="scaleX(average)"
                        :y1="GRAPH_HEIGHT"
                        :y2="averageY"
                        :stroke="resolvedAvgColor"
                        stroke-width="1"
                        stroke-dasharray="4 4"
                    />
                    <circle :cx="scaleX(average)" :cy="averageY" r="3.5" :fill="resolvedAvgColor" />

                    <text :x="averageBadge.endX" :y="averageBadge.textY" text-anchor="end" font-size="GRAPH_FONT_SIZE" :fill="resolvedAvgColor">
                        {{ averageBadge.text }}
                    </text>
                </g>

                <!-- <g v-if="secondaryAnnotation !== null && secondaryAnnotation !== undefined && secondaryY !== null && !isEmpty && ownedBadge">
                    <line
                        :x1="scaleX(secondaryAnnotation)"
                        :x2="scaleX(secondaryAnnotation)"
                        :y1="GRAPH_HEIGHT"
                        :y2="secondaryY"
                        :stroke="resolvedBoundColor"
                        stroke-width="1"
                        stroke-dasharray="2 2"
                    />
                    <circle :cx="scaleX(secondaryAnnotation)" :cy="secondaryY" r="3.5" :fill="surfaceTextColor" :stroke="resolvedBoundColor" stroke-width="1" />

                    <text
                        :x="ownedBadge.endX"
                        :y="ownedBadge.textY + GRAPH_FONT_SIZE + 2"
                        text-anchor="start"
                        font-size="GRAPH_FONT_SIZE"
                        :fill="resolvedBoundColor"
                    >
                        {{ ownedBadge.text }}
                    </text>
                </g> -->

                <text v-if="isEmpty" x="4" :y="Math.max(16, GRAPH_HEIGHT * 0.55)" font-size="12" :fill="surfaceTextColor" opacity="0.85">
                    No distribution data for this material yet.
                </text>

                <g v-for="tick in xTicks" :key="`tick-${tick.value}`">
                    <line :x1="tick.x" :x2="tick.x" :y1="GRAPH_HEIGHT" :y2="GRAPH_HEIGHT + 4" :stroke="axisColor" stroke-width="1" />
                    <text :x="tick.x" :y="GRAPH_HEIGHT + 16" text-anchor="middle" font-size="10" :fill="axisColor">
                        {{ formatWhole(tick.value) }}
                    </text>
                </g>
            </g>
        </svg>
    </div>
</template>

<style scoped>
.hf-material-graph {
    width: 100%;
}
</style>
