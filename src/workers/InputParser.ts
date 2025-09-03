import { assert } from "./Helper.js"

// For example if a piece has 10% base rate, this will output:
// 0.1, 0.11, 0.12, 0.13 .... 0.2, 0.2 , ... , 0.2, 1(pity)
// This matches how honing works in Lost Ark but isn't what we need just yet
// The 46.51 matches maxroll, it seems that's also what they use, but idk what the true value is in LA
// Was written to support manual input of aritsan multipliers and juice rates/counts, might need it in the future
function raw_chance(base: number, artisan_rate = 1, extra = 0, extra_num = 0): number[] {
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

// This simplifies honing into a simple weighted draw
// as in, this tells you the probability to land on each tap individually(before u begin)
function probability_distribution(raw: number[]): number[] {
    let chances = new Array(raw.length)
    let cum_chance = 1
    for (const [index, element] of raw.entries()) {
        chances[index] = cum_chance * element
        cum_chance *= 1 - element
    }
    return chances
}

// giga cursed parser, honestly i shouldn't place so much restraints on the counts(0<=x<=5) because it might be useful
// for when raising multilpe characters
// This simplifies normal honing into their probability distribution & cost of 1 tap
// but adv honing also has juice cost, which isn't linear with tap count,
// so this turns advanced hoining into their probability distribution(computed via the python script) & total cost at that tap(hence the 3 dimensional array)
// Also gives a tag to identify what piece it is for cacheing reasons.
export function parser(
    normal_counts: number[][],
    normal_chances: number[],
    weap_costs: number[][],
    armor_costs: number[][],
    adv_counts: number[][],
    adv_costs: number[][],
    adv_data_10_20_juice: number[][],
    adv_data_30_40_juice: number[][],
    adv_data_10_20: number[][],
    adv_data_30_40: number[][],
    adv_hone_strategy: string
): [number[][], number[][], number[][], number[][][], string[]] {
    assert(normal_counts.length == 2)
    assert(normal_counts[0].length == normal_counts[1].length)

    assert(Math.max(...normal_counts[0]) <= 5)
    assert(Math.min(...normal_counts[0]) >= 0)
    assert(Math.max(...normal_counts[1]) <= 1)
    assert(Math.min(...normal_counts[1]) >= 0)
    for (const i of normal_counts[0]) {
        assert(Number.isInteger(i))
    }
    for (const i of normal_counts[1]) {
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

    let base_rates = normal_chances

    for (const i of base_rates) {
        assert(0 < i && i <= 1)
    }

    assert(adv_hone_strategy == "Juice on grace" || adv_hone_strategy == "No juice")

    let tags = []
    let prob_dist_arr = []
    let hone_costs = Array.from({ length: weap_costs.length }, () => new Array())
    for (let piece_type = 0; piece_type < normal_counts.length; piece_type++) {
        let cur_cost = piece_type == 0 ? armor_costs : weap_costs
        let current_counter = 0

        for (let i = 0; i < normal_counts[piece_type].length; ) {
            if (current_counter >= normal_counts[piece_type][i]) {
                i++
                current_counter = 0
                continue
            }
            tags.push("Normal" + (piece_type == 0 ? " Armor " : " Weapon ") + " +" + i.toString() + "#" + current_counter.toString())
            let base = base_rates[i]
            // let artisan_rate = artisan_rates[i]
            // let extra = extra_rates[i]
            // let extra_num = extra_counts[i]
            let raw = raw_chance(base)
            let prob_dist = probability_distribution(raw)

            for (const cost_type of [...Array(weap_costs.length).keys()]) {
                hone_costs[cost_type].push(cur_cost[cost_type][i])
            }

            prob_dist_arr.push(prob_dist)
            current_counter++
        }
    }
    let adv_hone_costs = []
    let adv_hone_chances = []
    for (let wep_or_arm = 0; wep_or_arm < adv_counts.length; wep_or_arm++) {
        let current_counter = 0
        for (let i = 0; i < adv_counts[wep_or_arm].length; ) {
            if (current_counter >= adv_counts[wep_or_arm][i]) {
                i++
                current_counter = 0
                continue
            }
            tags.push("Adv" + (wep_or_arm == 0 ? " Armor " : " Weapon ") + " +" + (i * 10).toString() + adv_hone_strategy + "#" + current_counter.toString())
            let relevant_data: number[][]
            if (adv_hone_strategy == "Juice on grace") {
                relevant_data = i <= 1 ? adv_data_10_20_juice : adv_data_30_40_juice
            } else {
                relevant_data = i <= 1 ? adv_data_10_20 : adv_data_30_40
            }
            let this_chances = Array(relevant_data.length).fill(0)
            let sum_taps = relevant_data.map((row) => row[2]).reduce((acc, x) => acc + x, 0)
            let this_cost = Array.from({ length: 9 }, () => new Array(this_chances.length).fill(0))
            for (let row = 0; row < this_chances.length; row++) {
                this_chances[row] = relevant_data[row][2] / sum_taps
                for (let cost_type = 0; cost_type < 7; cost_type++) {
                    this_cost[cost_type][row] = adv_costs[cost_type][2 * i + (1 - wep_or_arm)] * relevant_data[row][0]
                }
                for (let cost_type = 7; cost_type < 9; cost_type++) {
                    this_cost[cost_type][row] =
                        adv_costs[cost_type][2 * i + (1 - wep_or_arm)] * relevant_data[row][1] * (adv_hone_strategy == "Juice on grace" ? 1 : 0)
                }
            }
            adv_hone_chances.push(this_chances)
            adv_hone_costs.push(this_cost)
            current_counter++
        }
    }
    if (adv_hone_chances.length == 0) {
        adv_hone_chances = []
    }
    if (adv_hone_costs.length == 0) {
        adv_hone_costs = []
    }
    return [prob_dist_arr, hone_costs, adv_hone_chances, adv_hone_costs, tags]
}
