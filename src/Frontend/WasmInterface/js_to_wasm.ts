import init, {
    chance_to_cost_wrapper,
    cost_to_chance_wrapper,
    cost_to_chance_arr_wrapper,
    parser_wrapper_unified,
    average_cost_wrapper,
} from "@/../pkg/honing_forecast.js"

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
            (
                which_one == "CostToChance" ||
                which_one == "CostToChanceArr" ||
                // which_one == "CostToChanceOptimized" ||
                which_one == "ChanceToCost" ||
                which_one == "ParserUnified" ||
                which_one == "AverageCost"
            )
            // which_one == "ChanceToCostOptimized"
        )
    ) {
        throw "Invalid js_to_wasm operation type: " + which_one
    }

    let result
    if (which_one == "CostToChance") {
        // always run optimized
        let out = await CostToChanceWasm(payload)

        // Convert f64 failure rates to formatted strings
        const reasons = out.reasons.map((rate: number, index: number) => {
            const percentage = (rate * 100).toFixed(2)
            return `${LABELS[index]}: ${percentage}% chance to have enough ${LABELS[index]}`
        })

        // const optimized_reasons = out.optimized_reasons.map((rate: number, index: number) => {
        //     const percentage = (rate * 100).toFixed(2)
        //     return `${percentage}% chance to have enough ${LABELS[index]}`
        // })

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
            hundred_gold_costs: out.hundred_gold_costs,
            chance_if_buy: (out.chance_if_buy * 100).toFixed(2),
        }
    } else if (which_one == "CostToChanceArr") {
        let out = await CostToChanceArrWasm(payload)

        // Convert final chances to percentages
        const final_chances_percent = out.final_chances.map((chance: number) => (chance * 100).toFixed(2))

        const buy_chances_percent = out.buy_chances.map((chance: number) => (chance * 100).toFixed(2))

        result = {
            final_chances: final_chances_percent,
            failure_rates_arr: out.typed_fail_counters,
            budgets_red_remaining: out.budgets_red_remaining,
            budgets_blue_remaining: out.budgets_blue_remaining,
            buy_chances: buy_chances_percent,
            // buy_gold_costs: out.buy_gold_costs,
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
            upgradeArr: out[0],
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
    console.log(which_one + " finished after" + String(result.run_time))
    self.postMessage({ type: "result", id, result: result })
})
