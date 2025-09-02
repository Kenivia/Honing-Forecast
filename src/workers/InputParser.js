
import { assert } from "./Helper.js"




function raw_chance(base, artisan_rate = 1, extra = 0, extra_num = 0) {
    let chances = []
    let artisan = 0
    let current_chance = base
    let count = 0
    while (true) {
        if (extra_num <= 0) {
            extra = 0
        }
        current_chance = base + (Math.min(count, 10) * base) / 10 + extra
        extra_num -= 1

        if (artisan >= 1) {
            current_chance = 1
            chances[count] = current_chance
            break
        }
        chances[count] = current_chance
        count += 1
        artisan += (46.51 / 100) * current_chance * artisan_rate
    }
    return chances
}
function individual_chance(raw) {
    let chances = new Array(raw.length)
    let cum_chance = 1
    for (const [index, element] of raw.entries()) {
        chances[index] = cum_chance * element
        cum_chance *= 1 - element
    }
    return chances
}

export function parser(counts, chances, weap_costs, armor_costs, adv_counts, adv_costs, adv_data_10_20_juice, adv_data_30_40_juice, adv_data_10_20, adv_data_30_40, adv_hone_strategy) {
    assert(counts.length == 2)
    assert(counts[0].length == counts[1].length)

    assert(Math.max(...counts[0]) <= 5)
    assert(Math.min(...counts[0]) >= 0)
    assert(Math.max(...counts[1]) <= 1)
    assert(Math.min(...counts[1]) >= 0)
    for (const i of counts[0]) {
        assert(Number.isInteger(i))
    }
    for (const i of counts[1]) {
        assert(Number.isInteger(i))
    }

    assert(Math.max(...adv_counts[0]) <= 5)
    assert(Math.min(...adv_counts[0]) >= 0)
    assert(Math.max(...adv_counts[1]) <= 1)
    assert(Math.min(...adv_counts[1]) >= 0)
    for (const i of adv_counts[0]) {
        assert(Number.isInteger(i))
    }
    for (const i of adv_counts[1]) {
        assert(Number.isInteger(i))
    }

    // let global_width = counts[0].length

    // assert(chances.length == 4)
    // for (const i of chances) {
    //     assert(i.length == global_width)
    // }
    let base_rates = chances
    // let artisan_rates = chances[1]
    // let extra_rates = chances[2]
    // let extra_counts = chances[3]

    for (const i of base_rates) {
        assert(0 < i <= 1)
    }
    // for (const i of artisan_rates) {
    //     assert(0 < i)
    // }
    // for (const i of extra_rates) {
    //     assert(0 <= i)
    // }
    // for (const i of extra_counts) {
    //     assert(Number.isInteger(i))
    // }
    // for (const i of extra_counts) {
    //     assert(0 <= i)
    // }

    assert(adv_hone_strategy == "Juice on grace" || adv_hone_strategy == "No juice")
    // won't check the costs because they shouldn't be touched

    let tags = []

    let ind_chances = []
    let hone_costs = Array.from({ length: weap_costs.length }, () => new Array())

    for (let piece_type = 0; piece_type < counts.length; piece_type++) {
        let cur_cost = piece_type == 0 ? armor_costs : weap_costs
        let current_counter = 0
        
        for (let i = 0; i < counts[piece_type].length;) {
            if (current_counter >= counts[piece_type][i]) {
                i++
                current_counter = 0
                continue
            }
            tags.push("Normal" + (piece_type == 0 ? " Armor " : " Weapon ") + " +"+ i.toString())
            let base = base_rates[i]
            // let artisan_rate = artisan_rates[i]
            // let extra = extra_rates[i]
            // let extra_num = extra_counts[i]
            let raw = raw_chance(base)
            let ind_chance = individual_chance(raw)

            for (const cost_type of [...Array(weap_costs.length).keys()]) {
                hone_costs[cost_type].push(cur_cost[cost_type][i])
            }

            ind_chances.push(ind_chance)
            current_counter++
        }
    }
    let adv_hone_costs = []
    let adv_hone_chances = []
    // for (let i = 0; i < adv_costs.length; i++) {
    //     adv_hone_costs[i] = []
    // }

    for (let wep_or_arm = 0; wep_or_arm < adv_counts.length; wep_or_arm++) {
        let current_counter = 0
        for (let i = 0; i < adv_counts[wep_or_arm].length;) {
            if (current_counter >= adv_counts[wep_or_arm][i]) {
                i++
                current_counter = 0
                continue
            }
            tags.push("Adv" + (wep_or_arm == 0 ? " Armor " : " Weapon ") + " +"+ (i*10).toString())
            let relevant_data;
            if (adv_hone_strategy == "Juice on grace") {
                relevant_data = i <= 1 ? adv_data_10_20_juice : adv_data_30_40_juice
            }
            else {
                relevant_data = i <= 1 ? adv_data_10_20 : adv_data_30_40
            }
            let this_chances = Array(relevant_data.length).fill(0)
            let sum_taps = (relevant_data.map(row => row[2])).reduce((acc, x) => acc + x, 0)
            let this_cost = Array.from({ length: 9 }, () => new Array(this_chances.length).fill(0));
            for (let row = 0; row < this_chances.length; row++) {
                this_chances[row] = relevant_data[row][2] / sum_taps
                for (let cost_type = 0; cost_type < 7; cost_type++) {
                    this_cost[cost_type][row] = adv_costs[cost_type][2 * i + (1 - wep_or_arm)] * relevant_data[row][0]
                }
                for (let cost_type = 7; cost_type < 9; cost_type++) {
                    this_cost[cost_type][row] = adv_costs[cost_type][2 * i + (1 - wep_or_arm)] * relevant_data[row][1] * (adv_hone_strategy == "Juice on grace" ? 1 : 0)
                }
            }
            adv_hone_chances.push(this_chances)
            adv_hone_costs.push(this_cost)
            current_counter++
        }
    }
    if (adv_hone_chances.length == 0) { adv_hone_chances = [[0]] }
    if (adv_hone_costs.length == 0) { adv_hone_costs = [[[0], [0], [0], [0], [0], [0], [0], [0], [0]]] }
    return [ind_chances, hone_costs, adv_hone_chances, adv_hone_costs, tags]
}
