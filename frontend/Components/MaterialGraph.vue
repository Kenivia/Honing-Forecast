<script setup lang="ts">
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
        height: 118,
    },
)

const rootRef = ref<HTMLElement | null>(null)
const width = ref(0)
let observer: ResizeObserver | null = null

const margin = {
    top: 18,
    right: 16,
    bottom: 24,
    left: 4,
}

const plotWidth = computed(() => Math.max(100, width.value - margin.left - margin.right))
const plotHeight = computed(() => Math.max(40, props.height - margin.top - margin.bottom))

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
    if (maxY.value <= 0) return plotHeight.value
    const normalized = y / (maxY.value * 1.1)
    return Math.max(0, Math.min(plotHeight.value, plotHeight.value - normalized * plotHeight.value))
}

function interpolateY(targetX: number) {
    const list = points.value
    if (!list.length) return plotHeight.value

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
    return `${linePath.value} L ${lastX} ${plotHeight.value} L ${firstX} ${plotHeight.value} Z`
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

function cssVar(name: string, fallback: string) {
    if (typeof window === "undefined") return fallback
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
    return value || fallback
}

const resolvedColor = computed(() => {
    return cssVar(props.colorVar, props.colorVar)
})

function formatWhole(value: number) {
    return Math.round(value).toLocaleString("en-US")
}

function badgeWidth(text: string) {
    return Math.max(88, Math.min(220, text.length * 7 + 18))
}

function clamp(value: number, min: number, max: number) {
    return Math.max(min, Math.min(max, value))
}

const averageLabel = computed(() => {
    if (props.average === null || props.average === undefined) return null
    return `Average: ${formatWhole(props.average)}`
})

const averageBadge = computed(() => {
    if (!averageLabel.value || props.average === null || props.average === undefined || averageY.value === null) return null
    const widthPx = badgeWidth(averageLabel.value)
    const centerX = clamp(scaleX(props.average), widthPx / 2 + 4, plotWidth.value - widthPx / 2 - 4)
    return {
        text: averageLabel.value,
        width: widthPx,
        centerX,
        textY: 2,
    }
})

const ownedLabel = computed(() => {
    if (props.secondaryAnnotation === null || props.secondaryAnnotation === undefined) return null
    return `You have: ${formatWhole(props.secondaryAnnotation)}`
})

const ownedBadge = computed(() => {
    if (!ownedLabel.value || props.secondaryAnnotation === null || props.secondaryAnnotation === undefined || secondaryY.value === null) return null
    const widthPx = badgeWidth(ownedLabel.value)
    const centerX = clamp(scaleX(props.secondaryAnnotation), widthPx / 2 + 4, plotWidth.value - widthPx / 2 - 4)
    const textY = clamp(secondaryY.value - 10, 18, plotHeight.value - 10)
    return {
        text: ownedLabel.value,
        width: widthPx,
        centerX,
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

            <g :transform="`translate(${margin.left}, ${margin.top})`">
                <line x1="0" :x2="plotWidth" :y1="plotHeight" :y2="plotHeight" :stroke="axisColor" stroke-width="1" />

                <path
                    v-if="!isEmpty"
                    :d="areaPath"
                    :fill="`url(#hf-graph-fill-${colorVar.replace(/[^a-z0-9]/gi, '')})`"
                    stroke="none"
                />
                <path v-if="!isEmpty" :d="linePath" fill="none" :stroke="resolvedColor" stroke-width="2.1" />

                <g v-if="average !== null && average !== undefined && averageY !== null && averageBadge">
                    <line
                        :x1="scaleX(average)"
                        :x2="scaleX(average)"
                        :y1="plotHeight"
                        :y2="averageY"
                        stroke="#ffffff"
                        stroke-width="1"
                        stroke-dasharray="4 4"
                    />
                    <circle :cx="scaleX(average)" :cy="averageY" r="3.5" fill="#ffffff" />
                    <rect
                        :x="averageBadge.centerX - averageBadge.width / 2"
                        :y="averageBadge.textY - 13"
                        :width="averageBadge.width"
                        height="18"
                        rx="4"
                        :fill="badgeFill"
                        fill-opacity="0.84"
                        :stroke="resolvedColor"
                        stroke-opacity="0.5"
                        stroke-width="1"
                    />
                    <text
                        :x="averageBadge.centerX"
                        :y="averageBadge.textY"
                        text-anchor="middle"
                        font-size="12"
                        font-weight="600"
                        :fill="resolvedColor"
                    >
                        {{ averageBadge.text }}
                    </text>
                </g>

                <g v-if="secondaryAnnotation !== null && secondaryAnnotation !== undefined && secondaryY !== null && !isEmpty && ownedBadge">
                    <line
                        :x1="scaleX(secondaryAnnotation)"
                        :x2="scaleX(secondaryAnnotation)"
                        :y1="plotHeight"
                        :y2="secondaryY"
                        :stroke="surfaceTextColor"
                        stroke-width="1"
                        stroke-dasharray="2 2"
                    />
                    <circle :cx="scaleX(secondaryAnnotation)" :cy="secondaryY" r="3.5" :fill="surfaceTextColor" :stroke="resolvedColor" stroke-width="1" />
                    <rect
                        :x="ownedBadge.centerX - ownedBadge.width / 2"
                        :y="ownedBadge.textY - 13"
                        :width="ownedBadge.width"
                        height="18"
                        rx="4"
                        :fill="badgeFill"
                        fill-opacity="0.84"
                        :stroke="surfaceTextColor"
                        stroke-opacity="0.5"
                        stroke-width="1"
                    />
                    <text
                        :x="ownedBadge.centerX"
                        :y="ownedBadge.textY"
                        text-anchor="middle"
                        font-size="12"
                        :fill="surfaceTextColor"
                    >
                        {{ ownedBadge.text }}
                    </text>
                </g>

                <text
                    v-if="isEmpty"
                    x="4"
                    :y="Math.max(16, plotHeight * 0.55)"
                    font-size="12"
                    :fill="surfaceTextColor"
                    opacity="0.85"
                >
                    No distribution data for this material yet.
                </text>

                <g v-for="tick in xTicks" :key="`tick-${tick.value}`">
                    <line
                        :x1="tick.x"
                        :x2="tick.x"
                        :y1="plotHeight"
                        :y2="plotHeight + 4"
                        :stroke="axisColor"
                        stroke-width="1"
                    />
                    <text
                        :x="tick.x"
                        :y="plotHeight + 16"
                        text-anchor="middle"
                        font-size="10"
                        :fill="axisColor"
                    >
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
    min-height: 118px;
}
</style>
