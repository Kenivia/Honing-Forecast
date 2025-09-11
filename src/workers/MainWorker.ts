import { assert, LABELS } from "./Helper.js"
// import init from "../../pkg/honing_forecast_bg.js?init"
import init, { chance_to_cost_wrapper, cost_to_chance_wrapper } from "../../pkg/honing_forecast.js" // or "../pkg/honing_wasm"

async function ChanceToCostWasm(payload: any) {
    await init()
    // chance_to_cost expects a JS object (JsValue); pass payload directly
    const out = (chance_to_cost_wrapper as any)(payload)
    // out will be a JS array/object depending on what Rust returned
    console.log(out)
    return out
}

async function CostToChanceWasm(payload: any) {
    await init()
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
    // await ensureWasm()
    // // chance_to_cost expects a JS object (JsValue); pass payload directly
    // console.log(payload)
    // const out = (cost_to_chance_wrapper as any)(payload)
    // // out will be a JS array/object depending on what Rust returned
    // console.log(out)
    // return out
}

self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()
    // const resp = await fetch("/Honing-Forecast/data.json")
    // const text = await resp.text()
    // const {
    //     normal_hone_chances,
    //     normal_hone_weapon_cost,
    //     normal_hone_armor_cost,
    //     normal_hone_weapon_unlock,
    //     normal_hone_armor_unlock,
    //     adv_hone_cost,
    //     adv_hone_unlock,
    //     adv_data_10_20_juice,
    //     adv_data_30_40_juice,
    //     adv_data_10_20,
    //     adv_data_30_40,
    // } = JSON.parse(text)
    const { id, payload, which_one } = msg
    // payload.normal_hone_chances = normal_hone_chances
    // payload.normal_hone_weapon_cost = normal_hone_weapon_cost
    // payload.normal_hone_armor_cost = normal_hone_armor_cost
    // payload.normal_hone_weapon_unlock = normal_hone_weapon_unlock
    // payload.normal_hone_armor_unlock = normal_hone_armor_unlock
    // payload.adv_hone_cost = adv_hone_cost
    // payload.adv_hone_unlock = adv_hone_unlock
    // payload.adv_data_10_20_juice = adv_data_10_20_juice
    // payload.adv_data_30_40_juice = adv_data_30_40_juice
    // payload.adv_data_10_20 = adv_data_10_20
    // payload.adv_data_30_40 = adv_data_30_40

    assert(which_one == "CostToChance" || which_one == "ChanceToCost")
    console.log(JSON.stringify(payload))
    const this_labels = LABELS.concat(["Red juice", "Blue juice", "Est. Probability"])
    let result
    if (which_one == "CostToChance") {
        let out = await CostToChanceWasm(payload)
        result = { chance: (out[0] * 100).toFixed(2), reason: out[1] }
    } else if (which_one == "ChanceToCost") {
        let out = await ChanceToCostWasm(payload)
        result = Object.fromEntries(this_labels.map((l, ind) => [l, out[ind]]))
    }

    console.log(result)

    result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)
    self.postMessage({ type: "result", id, result: result })
})
