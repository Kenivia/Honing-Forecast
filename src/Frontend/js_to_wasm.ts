// import init from "../../pkg/honing_forecast_bg.js?init"
import init, {
    chance_to_cost_wrapper,
    cost_to_chance_wrapper,
    cost_to_chance_arr_wrapper,
    parser_wrapper_unified,
    average_cost_wrapper,
} from "../../pkg/honing_forecast.js" // or "../pkg/honing_wasm"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver"]

async function ChanceToCostWasm(payload: any) {
    await init()
    // Now returns an object { best_budget, best_chance, hist_counts, hist_mins, hist_maxs }
    const out = (chance_to_cost_wrapper as any)(payload)
    return out
}

async function CostToChanceWasm(payload: any) {
    try {
        await init() // MUST await initialization
        // debug: call the wrapper in a try/catch to capture thrown exceptions
        try {
            // Returns { chance, reason, hist_counts, hist_mins, hist_maxs }
            return cost_to_chance_wrapper(payload)
        } catch (e) {
            console.error("call threw:", e)
        }
    } catch (initErr) {
        console.error("init failed:", initErr)
    }
}

async function CostToChanceArrWasm(payload: any) {
    try {
        await init() // MUST await initialization
        try {
            // Returns { final_chances, typed_fail_counters, budgets_red_remaining, budgets_blue_remaining }
            return cost_to_chance_arr_wrapper(payload)
        } catch (e) {
            console.error("cost_to_chance_arr call threw:", e)
        }
    } catch (initErr) {
        console.error("cost_to_chance_arr init failed:", initErr)
    }
}

async function ParserWasmUnified(payload: any) {
    try {
        await init() // MUST await initialization
        try {
            // Returns array of Upgrade objects using unified payload format
            return parser_wrapper_unified(payload)
        } catch (e) {
            console.error("parser unified call threw:", e)
        }
    } catch (initErr) {
        console.error("parser unified init failed:", initErr)
    }
}

async function AverageCostWasm(payload: any) {
    try {
        await init() // MUST await initialization
        try {
            // Returns array of 7 f64 values representing average costs
            return average_cost_wrapper(payload)
        } catch (e) {
            console.error("average cost call threw:", e)
        }
    } catch (initErr) {
        console.error("average cost init failed:", initErr)
    }
}

self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()

    const { id, payload, which_one } = msg

    if (
        !(
            which_one == "CostToChance" ||
            which_one == "CostToChanceArr" ||
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
    } else if (which_one == "ChanceToCost") {
        // const this_labels = LABELS.concat(["Red juice", "Blue juice"])
        let out = await ChanceToCostWasm(payload)
        // console.log(out)
        result = {
            hundred_budgets: out.hundred_budgets,
            hundred_chances: out.hundred_chances,
            hist_counts: out.hist_counts,
            hist_mins: out.hist_mins,
            hist_maxs: out.hist_maxs,
        }
    } else if (which_one == "ParserUnified") {
        let out = await ParserWasmUnified(payload)
        // console.log(out)
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
    // console.log(result)
    self.postMessage({ type: "result", id, result: result })
})
