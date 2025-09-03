import { parser } from "./InputParser.js"
import { MonteCarlosData } from "./MonteCarlos.js"
import { add_cost } from "./Helper.js"

//vibe coded, counts how many of each budget fails.
//TODO maybe is possible to optimize? maybe sort by each cost type? will need to test
// or maybe some advanced compiled shit
export function countFailures(cost_data: Uint32Array<ArrayBuffer>[], budget_data: Uint32Array<ArrayBuffer>[]): number[] {
    const N = cost_data.length
    const M = budget_data.length
    if (N === 0 || M === 0) return new Array(M).fill(0)

    const count = new Uint32Array(M)
    // assume n === 7
    for (let m = 0; m < N; m++) {
        const c = cost_data[m]
        const c0 = c[0],
            c1 = c[1],
            c2 = c[2],
            c3 = c[3],
            c4 = c[4],
            c5 = c[5],
            c6 = c[6]

        for (let i = 0; i < M; i++) {
            const b = budget_data[i]
            // inline/unrolled comparison with a single boolean expression and no inner loop
            if (c0 > b[0] || c1 > b[1] || c2 > b[2] || c3 > b[3] || c4 > b[4] || c5 > b[5] || c6 > b[6]) {
                count[i]++
            }
        }
    }
    return Array.from(count)
}

// p = success rate
// the idea is to generate a bunch of random budgets via MonteCarlosData and also generate
// a bunch of random budgets, see which budgets pass with about the same prob as p, then adjust
// those budgets by +1 or -1 tap(need to do this for advanced honing)
export async function ChanceToCost(
    hone_counts: number[][],
    chances: number[],
    weap_costs: number[][],
    armor_costs: number[][],
    weap_unlock: number[][],
    armor_unlock: number[][],
    p: number,
    adv_counts: number[][],
    adv_costs: number[][],
    adv_unlock: number[][],
    adv_data_10_20_juice: number[][],
    adv_data_30_40_juice: number[][],
    adv_data_10_20: number[][],
    adv_data_30_40: number[][],
    adv_hone_strategy: string
) {
    const cost_size = 50000 // seems to be the limit right now, any higher and it takes too long
    const budget_size = 1000
    let [prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs, tags] = parser(
        hone_counts,
        chances,
        weap_costs,
        armor_costs,
        adv_counts,
        adv_costs,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40,
        adv_hone_strategy
    )

    let [cost_data, budget_data] = await MonteCarlosData(
        cost_size,
        budget_size,
        prob_dist_arr,
        hone_costs,
        hone_counts,
        weap_unlock,
        armor_unlock,
        adv_counts,
        adv_hone_chances,
        adv_hone_costs,
        adv_unlock,
        tags
    )
    let failure_counts = countFailures(cost_data, budget_data)

    const N = cost_data.length
    const k = Math.floor((1 - p) * N)
    const n = cost_data[0].length
    const diffs = failure_counts.map((ci) => Math.abs(ci - k))
    const sorted_indices = diffs
        .map((_, i) => i)
        .sort((a, b) => {
            if (diffs[a] === diffs[b]) return a - b // stable tie-breaker
            return diffs[a] - diffs[b]
        })
    const best_budget = budget_data[sorted_indices[0]]
    let potential_budgets = [best_budget]

    // Right now the tweaking generates piece^2 number of adjusted budgets, 1 for each starting budget
    // but that's a bit too big for when everything is ticked(because counting failure is O(n*m))
    // TODO make it so that it's capped at some value, and randomize the starting piece if it's too big
    for (let piece = 0; piece < prob_dist_arr.length; piece++) {
        let pos_budget = best_budget.slice()
        let neg_budget = best_budget.slice()
        let not_seen_taps = [Array(prob_dist_arr.length).keys()]
        not_seen_taps.splice(piece, 1)
        add_cost(pos_budget, hone_costs, piece)
        add_cost(neg_budget, hone_costs, piece, -1)
        potential_budgets.push(pos_budget.slice())
        potential_budgets.push(neg_budget.slice())
        for (let depth = 1; depth < prob_dist_arr.length - 1; depth++) {
            let random = Math.floor(Math.random() * not_seen_taps.length)
            add_cost(pos_budget, hone_costs, random)
            add_cost(neg_budget, hone_costs, random, -1)
            not_seen_taps.splice(random, 1)
            potential_budgets.push(pos_budget.slice())
            potential_budgets.push(neg_budget.slice())
        }
    }
    let new_failures = countFailures(cost_data, potential_budgets)
    const new_diffs = new_failures.map((ci) => Math.abs(ci - k))
    const new_sorted = new_diffs
        .map((_, i) => i)
        .sort((a, b) => {
            if (new_diffs[a] === new_diffs[b]) return a - b // stable tie-breaker
            return new_diffs[a] - new_diffs[b]
        })

    let y = Array(n + 1).fill(0)
    for (let j = 0; j < n; j++) {
        y[j] += potential_budgets[new_sorted[0]][j]
    }

    // What this one's pass rate was, useful for debug but kinda confusing for user i think, not displayed rn
    y[n] = ((1 - new_failures[new_sorted[0]] / cost_data.length) * 100).toFixed(4)
    return y
}
