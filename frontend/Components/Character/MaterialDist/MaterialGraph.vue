<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue"

// Input data: [cost, probability] pairs — same shape as before.
// After this change the axes are swapped:
//   X axis = probability (always 0–100)
//   Y axis = cost       (0 – max cost in data)
// Annotation values (average, secondaryAnnotation) are COST values;
// their horizontal lines run from the left Y-axis to the curve.
type DataPoint = [number, number]

type Point = {
    cost: number
    prob: number
}

const props = withDefaults(
    defineProps<{
        data: DataPoint[] | null | undefined
        /** Cost value to mark with the "Average" annotation line */
        average?: number | null
        /** Cost value to mark with the "You have" annotation line */
        secondaryAnnotation?: number | null
        colorVar: string
        height?: number
    }>(),
    {
        average: null,
        secondaryAnnotation: null,
        height: 48,
    },
)

const rootRef = ref<HTMLElement | null>(null)
const width = ref(0)
let observer: ResizeObserver | null = null

const margin = {
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
}

const plotWidth = computed(() => Math.max(100, width.value - margin.left - margin.right))
const plotHeight = computed(() => Math.max(40, props.height - margin.top - margin.bottom))

// Parse & normalise — input pairs are [cost, prob]
const points = computed<Point[]>(() => {
    if (!Array.isArray(props.data) || props.data.length === 0) return []

    return props.data
        .map((pair) => ({
            cost: Number(pair?.[0] ?? 0),
            prob: Number(pair?.[1] ?? 0),
        }))
        .filter((p) => Number.isFinite(p.cost) && Number.isFinite(p.prob))
})

// X axis spans 0–100 (probability). Always fixed.
const PROB_MAX = 1

// Y axis spans 0–maxCost (with a small headroom multiplier).
const maxCost = computed(() => {
    if (!points.value.length) return 1
    return Math.max(1e-9, ...points.value.map((p) => p.cost))
})

const isEmpty = computed(() => points.value.length === 0)

/** Map a probability value (0–100) to an SVG x coordinate. */
function scaleX(prob: number): number {
    return (prob / PROB_MAX) * plotWidth.value
}

/** Map a cost value to an SVG y coordinate (higher cost = higher on screen). */
function scaleY(cost: number): number {
    const top = maxCost.value * 1.1
    if (top <= 0) return plotHeight.value
    const normalised = cost / top
    return Math.max(0, Math.min(plotHeight.value, plotHeight.value - normalised * plotHeight.value))
}

/**
 * Given a target cost value, find the corresponding probability on the curve
 * by interpolating between the data points sorted by cost.
 * Returns an SVG x coordinate.
 */
function interpolateX(targetCost: number): number {
    const list = [...points.value].sort((a, b) => a.cost - b.cost)
    if (!list.length) return 0

    if (targetCost <= list[0].cost) return scaleX(list[0].prob)

    for (let i = 1; i < list.length; i++) {
        const prev = list[i - 1]
        const cur = list[i]
        if (targetCost <= cur.cost) {
            const dc = cur.cost - prev.cost
            const t = dc === 0 ? 0 : (targetCost - prev.cost) / dc
            const prob = prev.prob + (cur.prob - prev.prob) * t
            return scaleX(prob)
        }
    }

    return scaleX(list[list.length - 1].prob)
}

// SVG paths — prob on X, cost on Y
const linePath = computed(() => {
    if (!points.value.length) return ""
    return points.value.map((p, i) => `${i === 0 ? "M" : "L"}${scaleX(p.prob)} ${scaleY(p.cost)}`).join(" ")
})

const areaPath = computed(() => {
    if (!points.value.length) return ""
    const first = points.value[0]
    const last = points.value[points.value.length - 1]
    return `${linePath.value} L ${scaleX(last.prob)} ${plotHeight.value} L ${scaleX(first.prob)} ${plotHeight.value} Z`
})

// ── Annotations ───────────────────────────────────────────────────────────────
// Each annotation is a COST value. Horizontal dashed line from x=0 to the curve.

const averageCurveX = computed(() => (props.average !== null && props.average !== undefined ? interpolateX(props.average) : null))

const secondaryCurveX = computed(() =>
    props.secondaryAnnotation !== null && props.secondaryAnnotation !== undefined ? interpolateX(props.secondaryAnnotation) : null,
)

const averageSvgY = computed(() => (props.average !== null && props.average !== undefined ? scaleY(props.average) : null))

const secondarySvgY = computed(() => (props.secondaryAnnotation !== null && props.secondaryAnnotation !== undefined ? scaleY(props.secondaryAnnotation) : null))

// ── Utilities ─────────────────────────────────────────────────────────────────
function cssVar(name: string, fallback: string) {
    if (typeof window === "undefined") return fallback
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
    return value || fallback
}

const resolvedColor = computed(() => cssVar(props.colorVar, props.colorVar))

function formatWhole(value: number) {
    return Math.round(value).toLocaleString("en-US")
}

function clamp(value: number, min: number, max: number) {
    return Math.max(min, Math.min(max, value))
}

// Badges sit just to the left of the curve dot
const BADGE_W = 112
const BADGE_H = 18

const averageBadge = computed(() => {
    if (props.average === null || props.average === undefined || averageSvgY.value === null) return null
    const centerY = clamp(averageSvgY.value, BADGE_H / 2 + 2, plotHeight.value - BADGE_H / 2 - 2)
    return { text: `Avg: ${formatWhole(props.average)}`, centerY }
})

const ownedBadge = computed(() => {
    if (props.secondaryAnnotation === null || props.secondaryAnnotation === undefined || secondarySvgY.value === null) return null
    const centerY = clamp(secondarySvgY.value, BADGE_H / 2 + 2, plotHeight.value - BADGE_H / 2 - 2)
    return { text: `You: ${formatWhole(props.secondaryAnnotation)}`, centerY }
})

const axisColor = computed(() => cssVar("--text-very-muted", "#9aa3b2"))
const surfaceTextColor = computed(() => cssVar("--hf-text-main", "#c9d0da"))
const badgeFill = computed(() => cssVar("--hf-bg-void", "#08090b"))

// ── Lifecycle ─────────────────────────────────────────────────────────────────
onMounted(() => {
    const updateWidth = () => {
        if (!rootRef.value) return
        width.value = Math.max(0, Math.floor(rootRef.value.getBoundingClientRect().width))
    }
    updateWidth()
    observer = new ResizeObserver(updateWidth)
    if (rootRef.value) observer.observe(rootRef.value)
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
                <!-- Bottom axis line -->
                <line x1="0" :x2="plotWidth" :y1="plotHeight" :y2="plotHeight" :stroke="axisColor" stroke-width="1" />

                <!-- Filled area & curve -->
                <path v-if="!isEmpty" :d="areaPath" :fill="`url(#hf-graph-fill-${colorVar.replace(/[^a-z0-9]/gi, '')})`" stroke="none" />
                <path v-if="!isEmpty" :d="linePath" fill="none" :stroke="resolvedColor" stroke-width="2.1" />

                <!-- Average annotation: horizontal line from left edge (x=0) to curve -->
                <g v-if="average !== null && average !== undefined && averageSvgY !== null && averageCurveX !== null && averageBadge">
                    <!-- <line x1="0" :x2="averageCurveX" :y1="averageSvgY" :y2="averageSvgY" stroke="#ffffff" stroke-width="1" stroke-dasharray="4 4" /> -->
                    <circle :cx="averageCurveX" :cy="averageSvgY" r="3.5" fill="#ffffff" />
                    <!-- Badge sits just left of the dot -->
                    <rect
                        :x="averageCurveX - averageBadge.text.length * 7"
                        :y="averageBadge.centerY - BADGE_H / 2"
                        :width="averageBadge.text.length * 7"
                        :height="BADGE_H"
                        rx="4"
                        :fill="badgeFill"
                        fill-opacity="0.5"
                    />
                    <text
                        :x="averageCurveX - (averageBadge.text.length * 7) / 2"
                        :y="averageBadge.centerY + 4"
                        text-anchor="middle"
                        font-size="12"
                        font-weight="600"
                        fill="white"
                    >
                        {{ averageBadge.text }}
                    </text>
                </g>

                <!-- Secondary annotation: horizontal line from left edge (x=0) to curve -->
                <g
                    v-if="
                        secondaryAnnotation !== null &&
                        secondaryAnnotation !== undefined &&
                        secondarySvgY !== null &&
                        secondaryCurveX !== null &&
                        !isEmpty &&
                        ownedBadge
                    "
                >
                    <!-- <line
                        x1="0"
                        :x2="secondaryCurveX"
                        :y1="secondarySvgY"
                        :y2="secondarySvgY"
                        :stroke="surfaceTextColor"
                        stroke-width="1"
                        stroke-dasharray="2 2"
                    /> -->
                    <circle :cx="secondaryCurveX" :cy="secondarySvgY" r="3.5" :fill="surfaceTextColor" :stroke="resolvedColor" stroke-width="1" />
                    <rect
                        :x="secondaryCurveX - ownedBadge.text.length * 7"
                        :y="ownedBadge.centerY - (ownedBadge.text.length * 7) / 2"
                        :width="BADGE_W"
                        :height="BADGE_H"
                        rx="4"
                        :fill="badgeFill"
                        fill-opacity="0.5"
                    />
                    <text :x="secondaryCurveX - BADGE_W / 2 - 6" :y="ownedBadge.centerY + 4" text-anchor="middle" font-size="12" :fill="surfaceTextColor">
                        {{ ownedBadge.text }}
                    </text>
                </g>

                <!-- Empty state -->
                <text v-if="isEmpty" x="0" :y="Math.max(16, plotHeight * 0.6)" font-size="12" :fill="surfaceTextColor" opacity="0.85">
                    No distribution data for this material yet.
                </text>
            </g>
        </svg>
    </div>
</template>

<style scoped>
.hf-material-graph {
    height: 42px;
}
</style>
