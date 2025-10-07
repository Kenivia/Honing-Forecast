import init, {
    chance_to_cost_wrapper,
    cost_to_chance_wrapper,
    cost_to_chance_arr_wrapper,
    cost_to_chance_optimized_wrapper,
    parser_wrapper_unified,
    average_cost_wrapper,
} from "../../pkg/honing_forecast.js"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver"]

async function ChanceToCostWasm(payload: any) {
    await init()
    return (chance_to_cost_wrapper as any)(payload)
}

async function CostToChanceWasm(payload: any) {
    await init()
    return (cost_to_chance_wrapper as any)(payload)
}

async function CostToChanceArrWasm(payload: any) {
    await init()
    return (cost_to_chance_arr_wrapper as any)(payload)
}
async function CostToChanceOptimizedWasm(payload: any) {
    await init()
    return (cost_to_chance_optimized_wrapper as any)(payload)
}

async function ParserWasmUnified(payload: any) {
    await init()
    return (parser_wrapper_unified as any)(payload)
}

async function AverageCostWasm(payload: any) {
    await init()
    return (average_cost_wrapper as any)(payload)
}

self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()

    const { id, payload, which_one } = msg

    if (
        !(
            which_one == "CostToChance" ||
            which_one == "CostToChanceArr" ||
            which_one == "CostToChanceOptimized" ||
            which_one == "ChanceToCost" ||
            which_one == "ParserUnified" ||
            which_one == "AverageCost"
        )
    ) {
        throw "Invalid operation type" + which_one
    }

    let result
    if (which_one == "CostToChance") {
        let out = await CostToChanceWasm(payload)

        // Convert f64 failure rates to formatted strings
        const reasons = out.reasons.map((rate: number, index: number) => {
            const percentage = (rate * 100).toFixed(2)
            return `${percentage}% chance to have enough ${LABELS[index]}`
        })

        result = {
            chance: (out.chance * 100).toFixed(2),
            reasons: reasons,
            hist_counts: out.hist_counts,
            hist_mins: out.hist_mins,
            hist_maxs: out.hist_maxs,
            upgrade_strings: out.upgrade_strings || [],
            juice_strings_armor: out.juice_strings_armor || [],
            juice_strings_weapon: out.juice_strings_weapon || [],
            budgets_red_remaining: out.budgets_red_remaining,
            budgets_blue_remaining: out.budgets_blue_remaining,
        }
    } else if (which_one == "CostToChanceArr") {
        let out = await CostToChanceArrWasm(payload)

        // Convert final chances to percentages
        const final_chances_percent = out.final_chances.map((chance: number) => (chance * 100).toFixed(2))

        // Convert typed fail counters to failure rates for each budget
        const failure_rates_arr = out.typed_fail_counters.map((typed_fail_counter: number[]) => {
            return typed_fail_counter.map((fail_count: number, _index: number) => {
                const failure_rate = fail_count / (out.final_chances.length > 0 ? 100000 : 1) // Assuming data_size
                const percentage = failure_rate
                return percentage
            })
        })

        result = {
            final_chances: final_chances_percent,
            failure_rates_arr: failure_rates_arr,
            budgets_red_remaining: out.budgets_red_remaining,
            budgets_blue_remaining: out.budgets_blue_remaining,
        }
    } else if (which_one == "CostToChanceOptimized") {
        let out = await CostToChanceOptimizedWasm(payload)
        // Convert f64 failure rates to formatted strings
        const reasons = out.reasons.map((rate: number, index: number) => {
            const percentage = (rate * 100).toFixed(2)
            return `${percentage}% chance to have enough ${LABELS[index]}`
        })

        result = {
            chance: (out.chance * 100).toFixed(2),
            reasons: reasons,
            hist_counts: out.hist_counts,
            hist_mins: out.hist_mins,
            hist_maxs: out.hist_maxs,
            upgrade_strings: out.upgrade_strings || [],
            juice_strings_armor: out.juice_strings_armor || [],
            juice_strings_weapon: out.juice_strings_weapon || [],
            budgets_red_remaining: out.budgets_red_remaining,
            budgets_blue_remaining: out.budgets_blue_remaining,
        }
    } else if (which_one == "ChanceToCost") {
        let out = await ChanceToCostWasm(payload)

        result = {
            hundred_budgets: out.hundred_budgets,
            hundred_chances: out.hundred_chances,
            hist_counts: out.hist_counts,
            hist_mins: out.hist_mins,
            hist_maxs: out.hist_maxs,
        }
    } else if (which_one == "ParserUnified") {
        let out = await ParserWasmUnified(payload)

        result = {
            upgrades: out[0],
            unlocks: out[1],
            other_strategy_prob_dists: out[2],
        }
    } else if (which_one == "AverageCost") {
        let out = await AverageCostWasm(payload)
        result = {
            average_costs: out,
        }
    }

    result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)
    self.postMessage({ type: "result", id, result: result })
})
