
import { weighted_random, Unlock } from "./Helper.js"


export function MC_data(ind_chances, costs, _time_limit, count_limit, counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, rigged = false, rigged_taps = []) {

    let trials = 1
    let cost_data = []

    while (trials <= count_limit) {
        let cur_costs = Array(costs.length + 2).fill(0)
        for (const piece of [...Array(ind_chances.length).keys()]) {
            let taps = [...Array(ind_chances[piece].length).keys()].map((x) => x + 1)
            let tap;
            if (rigged) {
                if (rigged_taps.length > 0) {
                    tap = rigged_taps[piece]
                }
                else {
                    tap = Math.ceil(trials / count_limit * taps.length)
                }
            }
            else {
                tap = weighted_random(taps, ind_chances[piece])
            }
            for (const cost_type of [...Array(costs.length).keys()]) {
                cur_costs[cost_type] += costs[cost_type][piece] * tap
            }
        }
        if (Math.max(...adv_counts[0]) + Math.max(...adv_counts[1]) > 0) {
            for (const piece of [...Array(adv_hone_chances.length).keys()]) {
                const offset = adv_hone_costs[0][2][0]/(adv_hone_costs[0][2][1]- adv_hone_costs[0][2][0])
                let taps = [...Array(adv_hone_chances[piece].length).keys()].map((x) =>
                     x + offset )
                let tap;
                if (rigged) {
                    if (rigged_taps.length > 0) {
                        tap = rigged_taps[piece]
                    }
                    else {
                        tap = Math.ceil(trials / count_limit * taps.length)-1
                    }
                }
                else {
                    tap = weighted_random(taps, adv_hone_chances[piece])-offset
                }
                for (const cost_type of [...Array(adv_hone_costs[0].length).keys()]) {
                    cur_costs[cost_type] += adv_hone_costs[piece][cost_type][tap]
                }
            }
        }


        cost_data.push(cur_costs)
        trials++
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

    let cost_data = []
    let budget_data = []

    // build all keys we expect in cache (use chunk index i so each chunk has its own key)


    // compute fresh
    cost_data = MC_data(ind_chances, hone_costs, time_limit, cost_size, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock,);
    budget_data = MC_data(ind_chances, hone_costs, time_limit, budget_size, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, true)


    return [cost_data, budget_data]
}

