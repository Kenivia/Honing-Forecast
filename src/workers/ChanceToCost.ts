import { parser } from "./InputParser.js"
import { MonteCarlosData } from "./MonteCarlos.js"
import { add_cost, shuffle } from "./Helper.js"

/**
 * Checks if a single cost vector can be afforded by a single budget vector.
 * A cost "passes" if every one of its components is less than or equal to the corresponding budget component.
 * @param {Uint32Array} cost - The cost vector.
 * @param {Uint32Array} budget - The budget vector.
 * @returns {boolean} - True if the cost passes the budget, false otherwise.
 */
function costPassesBudget(cost: Uint32Array, budget: Uint32Array): boolean {
    // This assumes a fixed length of 7, as in the original function.
    // It's faster than a loop for small, fixed-size arrays.
    return (
        cost[0] <= budget[0] &&
        cost[1] <= budget[1] &&
        cost[2] <= budget[2] &&
        cost[3] <= budget[3] &&
        cost[4] <= budget[4] &&
        cost[5] <= budget[5] &&
        cost[6] <= budget[6]
    )
}

/**
 * Efficiently counts, for each budget, how many costs fail it.
 * This optimized version uses binary search, leveraging the fact that budget_data is sorted.
 * @param {Uint32Array[]} cost_data - An array of N cost vectors.
 * @param {Uint32Array[]} budget_data - An array of M budget vectors, sorted element-wise in ascending order.
 * @returns {number[]} An array of M numbers, where the element at index i is the total number of costs that fail the i-th budget.
 */
function countFailuresAscending(cost_data: Uint32Array[], budget_data: Uint32Array[]): number[] {
    const N = cost_data.length
    const M = budget_data.length
    if (N === 0 || M === 0) return new Array(M).fill(0)

    // Step 1: Create a "difference array". We'll use this to efficiently mark the ranges of failures.
    // We use Int32Array because it can hold negative values.
    const diffs = new Int32Array(M + 1)

    // Step 2: For each cost, find how many budgets it fails.
    for (let m = 0; m < N; m++) {
        const cost = cost_data[m]

        // Binary search to find the *first* budget that this cost passes.
        // Because budgets are sorted, all budgets before this index will be failures.
        let low = 0
        let high = M - 1
        let firstPassIndex = M // Default to M, meaning it fails ALL budgets.

        while (low <= high) {
            // Prevent potential overflow and use bitwise shift for performance
            const mid = low + ((high - low) >> 1)

            if (costPassesBudget(cost, budget_data[mid])) {
                // This budget is a pass. It could be the first one.
                // Store it and check the left half for an even earlier pass.
                firstPassIndex = mid
                high = mid - 1
            } else {
                // This budget is a failure. The first pass must be in the right half.
                low = mid + 1
            }
        }

        // `firstPassIndex` is the number of budgets this cost fails.
        // We need to increment the count for all budgets in the range [0, firstPassIndex - 1].
        // We do this in O(1) by marking the start and end points in our difference array.
        if (firstPassIndex > 0) {
            diffs[0]++
            diffs[firstPassIndex]-- // Mark the end of the range.
        }
    }

    // Step 3: Convert the difference array into the final counts using a prefix sum.
    // This reconstructs the results from our range markers in a single O(M) pass.
    const count = new Uint32Array(M)
    count[0] = diffs[0]
    for (let i = 1; i < M; i++) {
        count[i] = count[i - 1] + diffs[i]
    }

    return Array.from(count)
}

export function countFailures(cost_data: Uint32Array<ArrayBuffer>[], budget_data: Uint32Array<ArrayBuffer>[], asc = false): number[] {
    if (asc) {
        return countFailuresAscending(cost_data, budget_data)
    } else {
        return countFailuresNaive(cost_data, budget_data)
    }
}
//vibe coded, counts how many of each budget fails.
//TODO maybe is possible to optimize? maybe sort by each cost type? will need to test
// or maybe some advanced compiled shit
export function countFailuresNaive(cost_data: Uint32Array<ArrayBuffer>[], budget_data: Uint32Array<ArrayBuffer>[]): number[] {
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
    console.log("start")
    const start_time = Date.now()
    const cost_size = 50000 // seems to be the limit right now, any higher and it takes too long
    const budget_size = 10000
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
    console.log("parser done: " + ((Date.now() - start_time) / 1000).toString())

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
    console.log("Monte carlos done: " + ((Date.now() - start_time) / 1000).toString())
    let failure_counts = countFailures(cost_data, budget_data, true)
    console.log("Failure count 1 done: " + ((Date.now() - start_time) / 1000).toString())
    const N = cost_data.length
    const k = Math.floor((1 - p) * N)
    const n = cost_data[0].length
    const diffs = failure_counts.map((ci) => Math.abs(ci - k))
    const sorted_indices = diffs
        .map((_, i) => i)
        .sort((a, b) => {
            if (diffs[a] === diffs[b]) return a - b // s table tie-breaker
            return diffs[a] - diffs[b]
        })
    const best_budget = budget_data[sorted_indices[0]]

    let potential_budgets = [best_budget]
    let first_layer = Array.from(Array(prob_dist_arr.length + adv_hone_chances.length).keys())
    shuffle(first_layer)
    // first_layer = first_layer.slice(0, Math.floor(1000 / (prob_dist_arr.length + adv_hone_chances.length)))
    let piece = 0
    for (let p = 0; p < Math.max(1, Math.min(first_layer.length, Math.floor(1000 / (prob_dist_arr.length + adv_hone_chances.length)))); p++) {
        piece = first_layer[p]
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
    console.log("Failure 2 done: " + ((Date.now() - start_time) / 1000).toString())
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
    console.log("Finish: " + ((Date.now() - start_time) / 1000).toString())
    return y
}
