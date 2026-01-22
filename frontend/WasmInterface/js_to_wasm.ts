import init, {
    cost_to_chance_wrapper,
    // cost_to_chance_arr_wrapper,
    // parser_wrapper_unified,
    // average_cost_wrapper,
    // monte_carlo_wrapper,
} from "@/../crates/wasm/pkg/hf_wasm.js"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver"]

// async function MonteCarloWasm(payload: any) {
//     await init()
//     return (monte_carlo_wrapper as any)(payload)
// }

async function CostToChanceWasm(payload: any) {
    await init()
    return (cost_to_chance_wrapper as any)(payload)
}

// async function CostToChanceArrWasm(payload: any) {
//     await init()
//     return (cost_to_chance_arr_wrapper as any)(payload)
// }

// async function ParserWasmUnified(payload: any) {
//     await init()
//     return (parser_wrapper_unified as any)(payload)
// }

// async function AverageCostWasm(payload: any) {
//     await init()
//     return (average_cost_wrapper as any)(payload)
// }

self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()

    const { id, payload, which_one } = msg

    if (
        !(
            (which_one == "CostToChance") // ||
            // which_one == "CostToChanceArr" ||
            // // which_one == "CostToChanceOptimized" ||
            // which_one == "ParserUnified" ||
            // which_one == "AverageCost" ||
            // which_one == "MonteCarlo"
            // which_one == "ChanceToCostOptimized"
        )
    ) {
        throw "Invalid js_to_wasm operation type: " + which_one
    }
    // console.log(which_one, "Began")

    let result
    // if (which_one == "MonteCarlo") {
    //     let out = await MonteCarloWasm(payload)

    //     // console.log("monte carlo out", out)
    //     result = { cost_data: out }
    // } else
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
            special_strings: out.special_strings || [],
            juice_strings_armor: out.juice_strings_armor || [],
            juice_strings_weapon: out.juice_strings_weapon || [],
            budgets_red_remaining: out.budgets_red_remaining,
            budgets_blue_remaining: out.budgets_blue_remaining,
            hundred_gold_costs: out.hundred_gold_costs,
            chance_if_buy: (out.chance_if_buy * 100).toFixed(2),
            typical_costs: out.typical_costs,
        }

        // console.log(result.typical_costs)
    }
    // else if (which_one == "CostToChanceArr") {
    //     let out = await CostToChanceArrWasm(payload)

    //     // Convert final chances to percentages
    //     const no_buy_chance_arr_percent = out.no_buy_chance_arr.map((chance: number) => (chance * 100).toFixed(2))

    //     const buy_chances_percent = out.buy_chance_arr.map((chance: number) => (chance * 100).toFixed(2))

    //     result = {
    //         no_buy_chance_arr: no_buy_chance_arr_percent,
    //         failure_rates_arr: out.typed_fail_counters,
    //         budgets_red_remaining: out.budgets_red_remaining,
    //         budgets_blue_remaining: out.budgets_blue_remaining,
    //         buy_chance_arr: buy_chances_percent,
    //         // buy_gold_costs: out.buy_gold_costs,
    //     }
    // } else if (which_one == "ParserUnified") {
    //     let out = await ParserWasmUnified(payload)

    //     result = {
    //         upgradeArr: out[0],
    //         unlocks: out[1],
    //         other_strategy_prob_dists: out[2],
    //     }
    // } else if (which_one == "AverageCost") {
    //     let out = await AverageCostWasm(payload)
    //     result = {
    //         average_costs: out,
    //     }
    // }

    result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)
    console.log(which_one + " finished after " + String(result.run_time))
    self.postMessage({ type: "result", id, result: result })
})
