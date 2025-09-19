// import init from "../../pkg/honing_forecast_bg.js?init"
import init, { chance_to_cost_wrapper, cost_to_chance_wrapper } from "../../pkg/honing_forecast.js" // or "../pkg/honing_wasm"

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)"]

async function ChanceToCostWasm(payload: any) {
    await init()
    // chance_to_cost expects a JS object (JsValue); pass payload directly
    const out = (chance_to_cost_wrapper as any)(payload)
    // out will be a JS array/object depending on what Rust returned
    console.log(out)
    return out
}

async function CostToChanceWasm(payload: any) {
    try {
        await init() // MUST await initialization
        // debug: call the wrapper in a try/catch to capture thrown exceptions
        try {
            return cost_to_chance_wrapper(payload)
        } catch (e) {
            console.error("call threw:", e)
        }
    } catch (initErr) {
        console.error("init failed:", initErr)
    }
}

self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()

    const { id, payload, which_one } = msg

    if (!(which_one == "CostToChance" || which_one == "ChanceToCost")) {
        throw "Invalid operation type" + which_one
    }

    const this_labels = LABELS.concat(["Red juice", "Blue juice", "Special Leaps"])
    let result
    if (which_one == "CostToChance") {
        let out = await CostToChanceWasm(payload)
        result = { chance: (out[0] * 100).toFixed(2), reason: out[1] }
    } else if (which_one == "ChanceToCost") {
        let out = await ChanceToCostWasm(payload)
        result = Object.fromEntries(this_labels.map((l, ind) => [l, out[0][ind]]))
        result.actual_prob = out[1]
    }

    result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)
    console.log(result)
    self.postMessage({ type: "result", id, result: result })
})
