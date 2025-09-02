
import { shuffle, Unlock } from "./Helper.js"
import { flatten2DUint32, reconstruct1DTo2D, saveTypedArray, loadTypedArray } from "./Cache.js"



function normal_hone_data(prob_dist_arr, costs, count_limit, rigged, shuffle_or_not, piece) {
    let out = Array.from({ length: count_limit }, () => new Uint32Array(7).fill(0))
    let cum_weights = prob_dist_arr[piece].map(
        (
            (sum) => (value) =>
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
                out[trials - 1][cost_type] = costs[cost_type][piece] * Math.ceil(trials / count_limit * prob_dist_arr[piece].length)
            }
            else {
                out[trials - 1][cost_type] = costs[cost_type][piece] * Math.ceil(tap_map[trials])
            }

        }
    }
    if (shuffle_or_not) { shuffle(out) }
    return out
}


function adv_hone_data(count_limit, adv_hone_chances, adv_hone_costs, rigged, shuffle_or_not, piece) {

    let out = Array.from({ length: count_limit }, () => new Uint32Array(9).fill(0))

    let cum_weights = adv_hone_chances[piece].map(
        ((sum) => (value) => (sum += value))(0))

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
                out[trials][cost_type] = adv_hone_costs[piece][cost_type][Math.ceil((trials + 1) / count_limit * adv_hone_costs[piece][cost_type].length) - 1]
            }
            else {
                out[trials][cost_type] = adv_hone_costs[piece][cost_type][tap_map[trials]]
            }

        }
    }
    if (shuffle_or_not) { shuffle(out) }
    return out
}


async function data_wrapper(hone_type, prob_dist_arr, costs, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags, rigged, piece) {

    const key = hone_type + tags[piece + (hone_type == "Adv" ? prob_dist_arr.length : 0)] + rigged.toString() + count_limit.toString()

    const loaded = await loadTypedArray(key)
    if (loaded == null) {
        let out;
        if (hone_type == "Normal") {
            out = normal_hone_data(prob_dist_arr, costs, count_limit, rigged, !rigged, piece)
        }
        else if (hone_type == "Adv") {
            out = adv_hone_data(count_limit, adv_hone_chances, adv_hone_costs, rigged, !rigged, piece)
        }
        else { throw new Error("Invalid hone type " + hone_type) }
        saveTypedArray(key, flatten2DUint32(out))
        return out
    }
    else {
        return reconstruct1DTo2D(loaded.arr, count_limit)
    }

}


export async function _mc_data(prob_dist_arr, costs, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags, rigged = false) {

    let cost_data = Array.from({ length: count_limit }, () => new Uint32Array(9).fill(0))
    for (let piece = 0; piece < prob_dist_arr.length; piece++) {
        let this_cost = await data_wrapper("Normal", prob_dist_arr, costs, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags, rigged, piece)
        for (let i = 0; i < this_cost.length; i++) {
            for (let j = 0; j < this_cost[0].length; j++) {
                cost_data[i][j] += this_cost[i][j]
            }
        }
    }

    for (let piece = 0; piece < adv_hone_chances.length; piece++) {
        let this_cost = await data_wrapper("Adv", prob_dist_arr, costs, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags, rigged, piece)
        for (let i = 0; i < this_cost.length; i++) {
            for (let j = 0; j < this_cost[0].length; j++) {
                cost_data[i][j] += this_cost[i][j]
            }
        }
    }
    let unlock = Unlock(counts, weap_unlock, armor_unlock, adv_counts, adv_unlock);
    // apply unlock adjustments
    for (let i = 0; i < cost_data.length; i++) {
        cost_data[i][3] += unlock[0];
        cost_data[i][6] += unlock[1];
    }

    return cost_data
}



export async function MonteCarlosData(cost_size, budget_size, prob_dist_arr, hone_costs, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags) {


    const cost_data = await _mc_data(prob_dist_arr, hone_costs, cost_size, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags);
    if (budget_size > 0) {
        const budget_data = await _mc_data(prob_dist_arr, hone_costs, budget_size, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags, true)
        return [cost_data, budget_data]
    }
    else { return [cost_data, []] }


}

