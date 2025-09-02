import { parser } from "./InputParser.js"
import { MonteCarlosData } from "./MonteCarlos.js"
import { countFailuresGAS, add_cost } from "./Helper.js"


export async function ChanceToCost(
    hone_counts : number[][],
     chances : number[],
 weap_costs: number[][],
 armor_costs: number[][],
 weap_unlock: number[][],
 armor_unlock: number[][],
 p : number,
 adv_counts: number[][],
 adv_costs: number[][],
 adv_unlock: number[][],
 adv_data_10_20_juice: number[][],
 adv_data_30_40_juice: number[][],
 adv_data_10_20: number[][],
 adv_data_30_40: number[][],
 adv_hone_strategy:string) {
    // p = success rate

    // the idea is to generate a bunch of random budgets via MC_data and also generate
    // a bunch of random costs, see which budgets pass with about the same prob as p, - then take 
    // the closest num_average 'th ones and take the average
    const cost_size = 50000;
    const budget_size = 1000;
    let [prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs, tags] = parser(hone_counts, chances, weap_costs, armor_costs, adv_counts, adv_costs, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy)

    let [cost_data, budget_data] = await MonteCarlosData(cost_size, budget_size, prob_dist_arr, hone_costs, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags)
    let failure_counts = countFailuresGAS(cost_data, budget_data)

    const N = cost_data.length;
    const k = Math.floor((1 - p) * N);
    const n = cost_data[0].length;
    const diffs = failure_counts.map(ci => Math.abs(ci - k));
    const sorted_indices = diffs
        .map((_, i) => i)
        .sort((a, b) => {
            if (diffs[a] === diffs[b]) return a - b; // stable tie-breaker
            return diffs[a] - diffs[b];
        })
    const best_budget = budget_data[sorted_indices[0]]
    let potential_budgets = [best_budget]

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
    let new_failures = countFailuresGAS(cost_data, potential_budgets)
    const new_diffs = new_failures.map(ci => Math.abs(ci - k));
    const new_sorted = new_diffs
        .map((_, i) => i)
        .sort((a, b) => {
            if (new_diffs[a] === new_diffs[b]) return a - b; // stable tie-breaker
            return new_diffs[a] - new_diffs[b];
        })




    let y = Array(n + 1).fill(0);
    for (let j = 0; j < n; j++) {
        y[j] += potential_budgets[new_sorted[0]][j]
    }

    // }
    y[n] = ((1 - new_failures[new_sorted[0]] / cost_data.length) * 100).toFixed(4)
    // y[n+1] = sorted_indices[0]/budget_data.length
    return y
}