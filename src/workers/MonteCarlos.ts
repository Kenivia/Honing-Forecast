import { shuffle, Unlock } from "./Helper.js"
import { flatten2DUint32, reconstruct1DTo2D, saveTypedArray, loadTypedArray } from "./Cache.js"

// Um heres the part where its a scam
// it's not actually Monte Carlos - its just weighted draw * total_draw(with randomness introduced at decimal values)
// like if the prob dist is [0.5, 0.5] and total draw is 1001 it'll just put 500 in each bucket and 50/50 the last one
// but it seems to give less variance than actual monte carlos and seems to be faster
// IDK i mean its easy to implement actual monte carlos
// TODO test how much better this is than monte carlos and the performance tradeoff
function normal_hone_data(prob_dist_arr: number[][], costs: number[][], count_limit: number, rigged: boolean, shuffle_or_not: boolean, piece: number) {
    let out = Array.from({ length: count_limit }, () => new Uint32Array(7).fill(0))
    let cum_weights = prob_dist_arr[piece].map(
        (
            (sum) => (value: number) =>
                (sum += value)
        )(0)
    )
    cum_weights[cum_weights.length - 1] = 1
    let tap_map = Array(count_limit).fill(0)
    let cur_samples = 0
    let j = 0
    for (let i = 0; i < cum_weights.length; i++) {
        cur_samples = Math.max(cur_samples, cum_weights[i] * count_limit)
        cur_samples = cur_samples - Math.floor(cur_samples) > Math.random() ? Math.floor(cur_samples) + 1 : Math.floor(cur_samples)
        for (; j < cur_samples; j++) {
            tap_map[j] = i + 1
        }
    }
    for (let cost_type = 0; cost_type < 7; cost_type++) {
        for (let trials = 1; trials <= count_limit; trials++) {
            if (rigged) {
                out[trials - 1][cost_type] = costs[cost_type][piece] * Math.ceil((trials / count_limit) * prob_dist_arr[piece].length)
            } else {
                out[trials - 1][cost_type] = costs[cost_type][piece] * Math.ceil(tap_map[trials])
            }
        }
    }
    if (shuffle_or_not) {
        shuffle(out)
    }
    return out
}

// same thing as above, i just made a different function cos its slightly different how the costs are calculated & off by 1 or whatever
// couldnt be bothered
function adv_hone_data(
    count_limit: number,
    adv_hone_chances: number[][],
    adv_hone_costs: number[][][],
    rigged: boolean,
    shuffle_or_not: boolean,
    piece: number
) {
    let out = Array.from({ length: count_limit }, () => new Uint32Array(9).fill(0))

    let cum_weights = adv_hone_chances[piece].map(
        (
            (sum) => (value: number) =>
                (sum += value)
        )(0)
    )

    cum_weights[cum_weights.length - 1] = 1
    let tap_map = Array(count_limit).fill(0)
    let cur_samples = 0
    let j = 0
    for (let i = 0; i < cum_weights.length; i++) {
        cur_samples = Math.max(cur_samples, cum_weights[i] * count_limit)
        cur_samples = cur_samples - Math.floor(cur_samples) > Math.random() ? Math.floor(cur_samples) + 1 : Math.floor(cur_samples)
        for (; j < cur_samples; j++) {
            tap_map[j] = i
        }
    }
    for (let cost_type = 0; cost_type < 9; cost_type++) {
        for (let trials = 0; trials < count_limit; trials++) {
            if (rigged) {
                out[trials][cost_type] = adv_hone_costs[piece][cost_type][Math.ceil(((trials + 1) / count_limit) * adv_hone_costs[piece][cost_type].length) - 1]
            } else {
                out[trials][cost_type] = adv_hone_costs[piece][cost_type][tap_map[trials]]
            }
        }
    }
    if (shuffle_or_not) {
        shuffle(out)
    }
    return out
}

// wrapper
// this is where the cacheing happens
async function load_or_compute_mc_data(
    hone_type: string,
    prob_dist_arr: number[][],
    costs: number[][],
    count_limit: number,
    adv_hone_chances: number[][],
    adv_hone_costs: number[][][],
    tags: { [x: string]: any },
    rigged: boolean,
    piece: number
) {
    const key = hone_type + tags[piece + (hone_type == "Adv" ? prob_dist_arr.length : 0)] + rigged.toString() + count_limit.toString()
    let loaded = null
    if (typeof indexedDB !== "undefined") {
        loaded = await loadTypedArray(key)
    }
    if (loaded == null) {
        let out: Uint32Array<ArrayBufferLike>[]
        if (hone_type == "Normal") {
            out = normal_hone_data(prob_dist_arr, costs, count_limit, rigged, !rigged, piece)
        } else if (hone_type == "Adv") {
            out = adv_hone_data(count_limit, adv_hone_chances, adv_hone_costs, rigged, !rigged, piece)
        } else {
            throw new Error("Invalid hone type " + hone_type)
        }
        if (typeof indexedDB !== "undefined") {
            saveTypedArray(key, flatten2DUint32(out))
        }
        return out
    } else {
        return reconstruct1DTo2D(loaded.arr, count_limit)
    }
}

// wrapper on wrapper
// we take advantage of the fact that we cache the pieces individually, so if we change a tick we only need to run 1 new piece
export async function sum_mc_data(
    prob_dist_arr: number[][],
    costs: number[][],
    count_limit: number,
    counts: number[][],
    weap_unlock: number[][],
    armor_unlock: number[][],
    adv_counts: number[][],
    adv_hone_chances: number[][],
    adv_hone_costs: number[][][],
    adv_unlock: number[][],
    tags: string[],
    rigged = false
) {
    let cost_data = Array.from({ length: count_limit }, () => new Uint32Array(9).fill(0))
    for (let piece = 0; piece < prob_dist_arr.length; piece++) {
        let this_cost = await load_or_compute_mc_data("Normal", prob_dist_arr, costs, count_limit, adv_hone_chances, adv_hone_costs, tags, rigged, piece)
        for (let i = 0; i < this_cost.length; i++) {
            for (let j = 0; j < this_cost[0].length; j++) {
                cost_data[i][j] += this_cost[i][j]
            }
        }
    }

    for (let piece = 0; piece < adv_hone_chances.length; piece++) {
        let this_cost = await load_or_compute_mc_data("Adv", prob_dist_arr, costs, count_limit, adv_hone_chances, adv_hone_costs, tags, rigged, piece)
        for (let i = 0; i < this_cost.length; i++) {
            for (let j = 0; j < this_cost[0].length; j++) {
                cost_data[i][j] += this_cost[i][j]
            }
        }
    }
    let unlock = Unlock(counts, weap_unlock, armor_unlock, adv_counts, adv_unlock)
    // apply unlock adjustments
    for (let i = 0; i < cost_data.length; i++) {
        cost_data[i][3] += unlock[0]
        cost_data[i][6] += unlock[1]
    }
    return cost_data
}

// wrapper on wrapper on wrapper on wrapper on wrapper on wrapper on wrapper on wrapper on wrapper on wrapper
export async function MonteCarlosData(
    cost_size: number,
    budget_size: number,
    prob_dist_arr: number[][],
    hone_costs: number[][],
    hone_counts: number[][],
    weap_unlock: number[][],
    armor_unlock: number[][],
    adv_counts: number[][],
    adv_hone_chances: number[][],
    adv_hone_costs: number[][][],
    adv_unlock: number[][],
    tags: string[]
): Promise<[Uint32Array<ArrayBuffer>[], Uint32Array<ArrayBuffer>[]]> {
    const cost_data = await sum_mc_data(
        prob_dist_arr,
        hone_costs,
        cost_size,
        hone_counts,
        weap_unlock,
        armor_unlock,
        adv_counts,
        adv_hone_chances,
        adv_hone_costs,
        adv_unlock,
        tags
    )
    if (budget_size > 0) {
        const budget_data = await sum_mc_data(
            prob_dist_arr,
            hone_costs,
            budget_size,
            hone_counts,
            weap_unlock,
            armor_unlock,
            adv_counts,
            adv_hone_chances,
            adv_hone_costs,
            adv_unlock,
            tags,
            true
        )
        return [cost_data, budget_data]
    } else {
        return [cost_data, []]
    }
}
