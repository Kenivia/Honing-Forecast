// import init from "../../pkg/honing_forecast_bg.js?init"
import init, { chance_to_cost_wrapper, cost_to_chance_wrapper, parser_wrapper_unified, average_cost_wrapper } from "../../pkg/honing_forecast.js" // or "../pkg/honing_wasm"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)"]

async function ChanceToCostWasm(payload: any) {
    await init()
    // Now returns an object { best_budget, actual_prob, hist_counts, hist_mins, hist_maxs }
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

    if (!(which_one == "CostToChance" || which_one == "ChanceToCost" || which_one == "ParserUnified" || which_one == "AverageCost")) {
        throw "Invalid operation type" + which_one
    }

    let result
    if (which_one == "CostToChance") {
        let out = await CostToChanceWasm(payload)
        result = {
            chance: (out.chance * 100).toFixed(2),
            reasons: out.reasons,
            hist_counts: out.hist_counts,
            hist_mins: out.hist_mins,
            hist_maxs: out.hist_maxs,
            upgrade_strings: out.upgrade_strings || [],
            juice_order_armor: out.juice_order_armor || [],
            juice_order_weapon: out.juice_order_weapon || [],
            budgets_red_remaining: out.budgets_red_remaining,
            budgets_blue_remaining: out.budgets_blue_remaining,
        }
    } else if (which_one == "ChanceToCost") {
        const this_labels = LABELS.concat(["Red juice", "Blue juice"])
        let out = await ChanceToCostWasm(payload)
        const bestBudget = out.best_budget
        result = Object.fromEntries(this_labels.map((l, ind) => [l, bestBudget[ind]]))
        result.actual_prob = out.actual_prob
        result.hist_counts = out.hist_counts
        result.hist_mins = out.hist_mins
        result.hist_maxs = out.hist_maxs
    } else if (which_one == "ParserUnified") {
        let out = await ParserWasmUnified(payload)
        console.log(out)
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
