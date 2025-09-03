import { assert, ticks_to_counts, LABELS } from "./Helper.js"
import { CostToChance } from "./CostToChance.js"
import { ChanceToCost } from "./ChanceToCost.js"

async function CostToChanceWrapper(payload: {
    normal_hone_ticks: boolean[][]
    adv_hone_ticks: boolean[][]
    budget: string[]
}): Promise<{ [key: string]: number | string }> {
    const normal_hone_ticks = payload.normal_hone_ticks
    const adv_hone_ticks = payload.adv_hone_ticks
    const budget = payload.budget

    const resp = await fetch("/Honing-Forecast/data.json")
    const text = await resp.text()
    const {
        normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        adv_hone_cost,
        adv_hone_unlock,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40,
    } = JSON.parse(text)

    const [chances, reasons] = await CostToChance(
        ticks_to_counts(normal_hone_ticks),
        normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        Array.from(LABELS, (x) => [Number(budget[x])]),
        LABELS,
        ticks_to_counts(adv_hone_ticks),
        adv_hone_cost,
        adv_hone_unlock,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40,
        "No juice" // Currently just not using juice even when provided, but will need to figure this out eventually
    )
    let formatted_chance = (chances[0] * 100).toFixed(2)
    let formatted_reason = reasons[0]
    return { chance: formatted_chance, reason: formatted_reason }
}

async function ChanceToCostWrapper(payload: {
    normal_hone_ticks: boolean[][]
    adv_hone_ticks: boolean[][]
    desired_chance: string
    adv_hone_strategy: string
}): Promise<{ [key: string]: number | string }> {
    const normal_hone_ticks = payload.normal_hone_ticks
    const adv_hone_ticks = payload.adv_hone_ticks
    const desired_chance = payload.desired_chance
    const adv_hone_strategy = payload.adv_hone_strategy

    const resp = await fetch("/Honing-Forecast/data.json")
    const text = await resp.text()
    const {
        normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        adv_hone_cost,
        adv_hone_unlock,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40,
    } = JSON.parse(text)

    const out = await ChanceToCost(
        ticks_to_counts(normal_hone_ticks),
        normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        Number(desired_chance) / 100,
        ticks_to_counts(adv_hone_ticks),
        adv_hone_cost,
        adv_hone_unlock,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40,
        adv_hone_strategy
    )
    const this_labels = LABELS.concat(["Red juice", "Blue juice", "Est. Probability"])

    return Object.fromEntries(this_labels.map((l, ind) => [l, out[ind]]))
}
self.addEventListener("message", async (ev) => {
    const msg = ev.data
    let start_time = Date.now()
    const { id, payload, which_one } = msg
    assert(which_one == "CostToChance" || which_one == "ChanceToCost")
    let result
    if (which_one == "CostToChance") {
        result = await CostToChanceWrapper(payload)
    } else if (which_one == "ChanceToCost") {
        result = await ChanceToCostWrapper(payload)
    }
    result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)
    self.postMessage({ type: "result", id, result: result })
})
