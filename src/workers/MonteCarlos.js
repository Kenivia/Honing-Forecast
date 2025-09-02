
import { shuffle, Unlock } from "./Helper.js"



function normal_hone_blob(ind_chances, costs, _time_limit, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, rigged, shuffle_or_not, piece) {
    let out = Array.from({ length: count_limit }, () => new Uint32Array(7).fill(0))

    let cum_weights = ind_chances[piece].map(
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
                out[trials - 1][cost_type] = costs[cost_type][piece] * Math.ceil(trials / count_limit * ind_chances[piece].length)
            }
            else {
                out[trials - 1][cost_type] = costs[cost_type][piece] * Math.ceil(tap_map[trials])
            }

        }
    }
    if (shuffle_or_not) { shuffle(out) }
    return out
}


function adv_hone_blob(ind_chances, costs, _time_limit, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, rigged, shuffle_or_not, piece) {

    let out = Array.from({ length: count_limit }, () => new Uint32Array(9).fill(0))

    let cum_weights = adv_hone_chances[piece].map(
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


export function MC_data(ind_chances, costs, _time_limit, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, rigged = false) {

    let cost_data = Array.from({ length: count_limit }, () => new Uint32Array(9).fill(0))
    for (let piece = 0; piece < ind_chances.length; piece++) {
        let this_cost = normal_hone_blob(ind_chances, costs, _time_limit, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, rigged, !rigged, piece)
        for (let i = 0; i < this_cost.length; i++) {
            for (let j = 0; j < this_cost[0].length; j++) {
                cost_data[i][j] += this_cost[i][j]
            }
        }
    }

    for (let piece = 0; piece < adv_hone_chances.length; piece++) {
        let this_cost = adv_hone_blob(ind_chances, costs, _time_limit, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, rigged, !rigged, piece)
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



// vibe coded
export function LoadorComputeData(cost_size, budget_size, chunkSize, ind_chances, hone_costs, time_limit, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock) {


    // build all keys we expect in cache (use chunk index i so each chunk has its own key)


    // compute fresh
    const cost_data = MC_data(ind_chances, hone_costs, time_limit, cost_size, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock,);
    if (budget_size > 0) {
        const budget_data = MC_data(ind_chances, hone_costs, time_limit, budget_size, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, true)
        return [cost_data, budget_data]
    }
    else { return [cost_data, []] }


}

