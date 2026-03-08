import { JUICE_LABELS, MATS_LABELS } from "@/Utils/Constants.ts"

export type InputsValues = {
    mats: {
        owned: Record<string, string>
        prices: Record<string, string>
        leftover: Record<string, string>
    }
    juice: {
        weapon: {
            owned: Record<string, string>
            prices: Record<string, string>
            leftover: Record<string, string>
        }
        armor: {
            owned: Record<string, string>
            prices: Record<string, string>
            leftover: Record<string, string>
        }
    }
}

type BuildPayloadOptions = {
    topGrid: boolean[][]
    bottomGrid: boolean[][]
    advHoneStrategy: string
    expressEvent: boolean
    bucketCount: string
    dataSize: string
    inputs: InputsValues
    progressGrid: number[][]
    unlockGrid: boolean[][]
    succeededGrid: boolean[][]
    stateBundleGrid: any[][][]
    specialState: number[]
    minResolution: number
    metricType: number
}

function parseNum(input: string) {
    const out = Number.parseFloat(input)
    return Number.isFinite(out) ? out : 0
}

export function buildPayload({
    topGrid,
    bottomGrid,
    advHoneStrategy,
    expressEvent,
    bucketCount,
    dataSize,
    inputs,
    progressGrid,
    unlockGrid,
    succeededGrid,
    stateBundleGrid,
    specialState,
    minResolution,
    metricType,
}: BuildPayloadOptions) {
    const { mats, juice } = inputs

    return {
        mats_budget: MATS_LABELS.slice(0, 8).map((label) => parseNum(mats.owned[label] || "0")),
        adv_hone_strategy: advHoneStrategy,
        express_event: expressEvent,
        bucket_count: Math.max(2, Math.min(1000, Math.floor(Number(bucketCount) || 2))),
        user_price_arr: MATS_LABELS.slice(0, 7).map((label) => parseNum(mats.prices[label] || "0")),
        data_size: Math.max(1000, Math.floor(Number(dataSize) || 0)),
        inp_leftover_values: MATS_LABELS.slice(0, 7).map((label) => parseNum(mats.leftover[label] || "0")),
        juice_books_budget: JUICE_LABELS.map((labels) => [parseNum(juice.weapon.owned[labels[0]]), parseNum(juice.armor.owned[labels[1]])]),
        juice_prices: JUICE_LABELS.map((labels) => [parseNum(juice.weapon.prices[labels[0]]), parseNum(juice.armor.prices[labels[1]])]),
        inp_leftover_juice_values: JUICE_LABELS.map((labels) => [parseNum(juice.weapon.leftover[labels[0]]), parseNum(juice.armor.leftover[labels[1]])]),
        progress_grid: progressGrid,
        unlocked_grid: unlockGrid,
        succeeded_grid: succeededGrid,
        state_grid: stateBundleGrid,
        special_state: specialState,
        min_resolution: Math.max(1, Math.min(219, Math.floor(minResolution || 1))),
        num_threads: navigator.hardwareConcurrency,
        metric_type: metricType,
        normal_hone_ticks: topGrid,
        adv_hone_ticks: bottomGrid,
        tier: 0,
    }
}
