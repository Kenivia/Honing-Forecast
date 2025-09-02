import { assert, ticks_to_counts, Unlock, average, pity } from "./Helper.js"
import { CostToChance } from "./CostToChance.js"
import { parser } from "./InputParser.js"
import { ChanceToCost } from "./ChanceToCost.js";

const LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver(WIP)"]


export function MaxrollAverage(hone_counts, chances, weap_costs, armor_costs, weap_unlock, armor_unlock, adv_counts, adv_costs, adv_unlock, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy) {
    let [ind_chances, hone_costs, adv_hone_chances, adv_hone_costs] = parser(hone_counts, chances, weap_costs, armor_costs, adv_counts, adv_costs, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy)

    let unlock = Unlock(hone_counts, weap_unlock, armor_unlock, adv_counts, adv_unlock);
    return average(ind_chances, hone_costs, unlock, adv_hone_chances, adv_hone_costs)
}

export function MaxrollPity(hone_counts, chances, weap_costs, armor_costs, weap_unlock, armor_unlock, adv_counts, adv_costs, adv_unlock, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy) {
    let [ind_chances, hone_costs, adv_hone_chances, adv_hone_costs] = parser(hone_counts, chances, weap_costs, armor_costs, adv_counts, adv_costs, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy)

    let unlock = Unlock(hone_counts, weap_unlock, armor_unlock, adv_counts, adv_unlock);
    return pity(ind_chances, hone_costs, unlock, adv_hone_chances, adv_hone_costs)
}

async function CostToChanceWrapper(payload) {
    const normal_hone_ticks = payload.normal_hone_ticks
    const adv_hone_ticks = payload.adv_hone_ticks
    const budget = payload.budget


    // do some pretend heavy work (sync here for demo)
    // For large numeric arrays prefer to get an ArrayBuffer and use typed arrays


    const resp = await fetch("/Honing-forecast/data.json")
    const text = await resp.text();
    const { normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        adv_hone_cost,
        adv_hone_unlock,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40


    } = JSON.parse(text)
    // const result = normal_hone_chances
    
    const [chances, reasons] = CostToChance(
        ticks_to_counts(normal_hone_ticks),
        normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        Array.from(LABELS, x => [budget[x]]),
        LABELS,
        10,
        ticks_to_counts(adv_hone_ticks),
        adv_hone_cost,
        adv_hone_unlock, adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40, "No juice", true
    )
    let formatted_chance = (chances[0] * 100).toFixed(2)
    let formatted_reason = reasons[0]
    return { chance: formatted_chance, reason: formatted_reason, }

}

async function ChanceToCostWrapper(payload) {
    const normal_hone_ticks = payload.normal_hone_ticks
    const adv_hone_ticks = payload.adv_hone_ticks
    const desired_chance = payload.desired_chance
    const adv_hone_strategy = payload.adv_hone_strategy


    // do some pretend heavy work (sync here for demo)
    // For large numeric arrays prefer to get an ArrayBuffer and use typed arrays


    const resp = await  fetch("/Honing-forecast/data.json")
    const text = await resp.text();
    const { normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        adv_hone_cost,
        adv_hone_unlock,
        adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40


    } = JSON.parse(text)
    // const result = normal_hone_chances
    const out = ChanceToCost(
        ticks_to_counts(normal_hone_ticks),
        normal_hone_chances,
        normal_hone_weapon_cost,
        normal_hone_armor_cost,
        normal_hone_weapon_unlock,
        normal_hone_armor_unlock,
        10,
        desired_chance / 100,
        ticks_to_counts(adv_hone_ticks),
        adv_hone_cost,
        adv_hone_unlock, adv_data_10_20_juice,
        adv_data_30_40_juice,
        adv_data_10_20,
        adv_data_30_40,
        adv_hone_strategy,
        true
    )
    const this_labels = LABELS.concat(["Red juice", "Blue juice", "Est. Probability"])


    return Object.fromEntries(this_labels.map((l,ind) => [l, out[ind]]))

}
self.addEventListener('message', async (ev) => {
    const msg = ev.data;
    // Example: worker expects { type: , id, payload }
    let start_time = Date.now()
    const { id, payload, which_one } = msg;
    assert(which_one == "CostToChance" || which_one == "ChanceToCost")
    let result;
    if (which_one == "CostToChance") {
        result = await CostToChanceWrapper(payload)
    }
    else if (which_one == "ChanceToCost") {
        // result = (payload)
        result = await ChanceToCostWrapper(payload)

    }
    result.run_time = ((Date.now() - start_time) / 1000).toFixed(2)

    // reply with the same id so caller knows which request this matches
    self.postMessage({ type: 'result', id, result: result });

});