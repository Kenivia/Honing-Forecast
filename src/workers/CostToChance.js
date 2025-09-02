import { parser } from "./InputParser.js"
import { MonteCarlosData } from "./MonteCarlos.js"
import { myformat } from "./Helper.js"


export async function CostToChance(hone_counts, chances, weap_costs, armor_costs, weap_unlock, armor_unlock, actual_budgets, labels, time_limit, adv_counts, adv_costs, adv_unlock, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy) {
    let [prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs,tags] = parser(hone_counts, chances, weap_costs, armor_costs, adv_counts, adv_costs, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy)


    let [cost_data, _] = await MonteCarlosData(100000, 0, prob_dist_arr, hone_costs, time_limit, hone_counts, weap_unlock, armor_unlock, adv_counts, adv_hone_chances, adv_hone_costs, adv_unlock, tags)

    let cumulative_pie = Array(actual_budgets[0].length).fill(0)
    let fail_counter = []
    for (let i of [...Array(actual_budgets[0].length).keys()]) {
        fail_counter[i] = Array(actual_budgets.length).fill(0)
    }

    for (let data of cost_data) {
        for (let [cost_type, _] of actual_budgets[0].entries()) { // only compare top 7 entries, ignore juice
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
            if (spread >= 0.1 || !displayed) {
                this_failed.push(labels[z] + "(" + spread.toString() + "%)")
            }
            displayed = true

        }
        if (Math.max(...i) == 0) {
            failed_labels.push("None")
        }
        else { failed_labels.push(this_failed.join(", ")) }

    }
    return [cumulative_pie, failed_labels]
}
