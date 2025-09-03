import { parser } from "./InputParser.js"
import { MonteCarlosData } from "./MonteCarlos.js"
import { myformat } from "./Helper.js"

// This actually came before ChanceToCost on my spreadsheet implementation, and it was designed to handle a bunch of budgets(across different weeks)
// Uses MonetCarlosData and a special failure counting loop that records the reason why it failed
export async function CostToChance(
    hone_counts: number[][],
    chances: number[],
    weap_costs: number[][],
    armor_costs: number[][],
    weap_unlock: number[][],
    armor_unlock: number[][],
    actual_budgets: number[][],
    labels: string[],
    adv_counts: number[][],
    adv_costs: number[][],
    adv_unlock: number[][],
    adv_data_10_20_juice: number[][],
    adv_data_30_40_juice: number[][],
    adv_data_10_20: number[][],
    adv_data_30_40: number[][],
    adv_hone_strategy: string
): Promise<[number[], string[]]> {
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

    let [cost_data, _] = await MonteCarlosData(
        100000,
        0,
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

    let cumulative_pie = Array(actual_budgets[0].length).fill(0)
    let fail_counter = []
    for (let i of [...Array(actual_budgets[0].length).keys()]) {
        fail_counter[i] = Array(actual_budgets.length).fill(0)
    }

    for (let data of cost_data) {
        for (let [cost_type, _] of actual_budgets[0].entries()) {
            // only compare top 7 entries
            let failed = false
            for (let i of [...Array(actual_budgets.length).keys()]) {
                if (actual_budgets[i][cost_type] < data[i]) {
                    failed = true
                    fail_counter[cost_type][i]++
                }
            }
            if (!failed) {
                cumulative_pie[cost_type] += 1 / cost_data.length
            }
        }
    }

    let failed_labels = []
    for (let [_, i] of fail_counter.entries()) {
        let failed_indices = [...Array(i.length).keys()].sort((a, b) => i[b] - i[a])
        let this_failed = []
        let displayed = false
        for (let z of failed_indices) {
            let spread = myformat(i[z] / cost_data.length)
            if (Number(spread) >= 0.1 || !displayed) {
                this_failed.push(labels[z] + "(" + spread.toString() + "%)")
            }
            displayed = true
        }
        if (Math.max(...i) == 0) {
            failed_labels.push("None")
        } else {
            failed_labels.push(this_failed.join("\n"))
        }
    }
    return [cumulative_pie, failed_labels] // TODO make it output the failed labels as an object so that it can be expanded more nicely in the UI
}
