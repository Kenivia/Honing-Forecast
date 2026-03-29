import { CharProfile } from "@/Stores/CharacterProfile"
import { RosterConfig } from "@/Stores/RosterConfig"
import { DEFAULT_ARTISAN_MULTIPLIER, JOINED_ADV_JUICE } from "@/Utils/Constants"
import { input_column_to_num } from "@/Utils/InputColumn"
import { Upgrade } from "@/Utils/Interfaces"

// --- Req 1: Scrollable Instructions Logic ---
export interface NormalStreak {
    juice: boolean
    book: boolean
    count: number
}
export interface AdvStreak {
    juice: boolean
    scroll: boolean
    grace: boolean
    count: number
}

export function aggregate_streaks(upgrade: Upgrade, juice_info: any): NormalStreak[] | AdvStreak[] {
    if (upgrade.state.length === 0) return []

    if (upgrade.is_normal_honing) {
        const streaks: NormalStreak[] = []
        let current: NormalStreak | null = null
        let index = 0
        for (const [juice, book] of upgrade.state.slice(0, upgrade.normal_dist.length - 1)) {
            if (index == upgrade.normal_dist.length - 2 && artisan_function(upgrade, index, juice_info) === "100.00") {
                // this corresponds to not showing the pity tap
                // Rust side does not enforce that the pity tap is unjuiced (it just ignores the state after that index)
                // so we need to hide it from the user
                // however for upgrades that naturally has a 100% success rate (below like +5) we don't want to skip
                // just a weird edge case
                continue
            }
            const hasBook = book > 0
            if (current && current.juice === juice && current.book === hasBook) {
                current.count++
            } else {
                current = { juice, book: hasBook, count: 1 }
                streaks.push(current)
            }
            index += 1
        }
        return streaks
    } else {
        const streaks: AdvStreak[] = []
        let [juice_grace, juice_non_grace] = JOINED_ADV_JUICE[upgrade.state[0][1]]
        let [scroll_grace, scroll_non_grace] = JOINED_ADV_JUICE[upgrade.state[1][1]]
        // These 4 numbers correspond to how many taps to perform on the respective conditions
        // They range from 0 to 255, with 255 considered infinite, see rust advanced_honing/utils for what numbers they can actually take

        let both_grace = Math.min(juice_grace, scroll_grace)
        if (both_grace > 0) streaks.push({ juice: true, scroll: true, grace: true, count: both_grace })

        let one_grace = Math.max(juice_grace, scroll_grace) == 255 ? 255 : Math.max(juice_grace, scroll_grace) - both_grace
        if (one_grace > 0) streaks.push({ juice: juice_grace > scroll_grace, scroll: scroll_grace > juice_grace, grace: true, count: one_grace })

        let both_non_grace = Math.min(juice_non_grace, scroll_non_grace)
        if (both_non_grace > 0) streaks.push({ juice: true, scroll: true, grace: false, count: both_non_grace })

        let one_non_grace = Math.max(juice_non_grace, scroll_non_grace) == 255 ? 255 : Math.max(juice_non_grace, scroll_non_grace) - both_non_grace
        if (one_non_grace > 0)
            streaks.push({ juice: juice_non_grace > scroll_non_grace, scroll: scroll_non_grace > juice_non_grace, grace: false, count: one_non_grace })

        if (streaks.length == 0) {
            streaks.push({ juice: false, scroll: false, grace: true, count: 255 })
        }
        // console.log(juice_grace, juice_non_grace, scroll_grace, scroll_non_grace, upgrade.state)
        return streaks
    }
}

export function streaks_to_text(upgrade: Upgrade, streaks: NormalStreak[] | AdvStreak[], juice_info: any) {
    let out = []
    let taps = 0
    for (let index = 0; index < streaks.length; index++) {
        let streak: any = streaks[index]

        let isNormal = upgrade.is_normal_honing
        let topIconActive = streak.juice
        let bottomIconActive = isNormal ? streak.book : streak.scroll
        let name_line =
            (streak.juice ? "Juice" : "") +
            ((streak.juice && streak.book) || (streak.juice && streak.scroll) ? " & " : "") +
            (streak.book ? "Book" : streak.scroll ? "Scroll" : "") +
            (!streak.juice && !streak.juice && !streak.book && !streak.scroll ? "Raw tap" : "")
        let line1 = ""
        let line2 = ""

        if (isNormal) {
            line1 = `x${streak.count}`
            taps += streak.count
            line2 = `until ${artisan_function(upgrade, taps, juice_info)}%<br>artisan`
        } else {
            let graceText = streak.grace ? "Grace" : "non-Grace"
            if (!streak.juice && !streak.scroll) {
                line1 = "Nothing"
                line2 = `on ${graceText}`
            } else {
                // console.log(upgrade.adv_dists)
                line1 = streak.count < 255 ? `First ${streak.count}` : streaks.length == 1 ? "All" : "All"
                line2 = graceText
            }
        }

        out.push({ topIconActive, bottomIconActive, line1, line2, name_line })
    }
    return out
}

export function artisan_function(upgrade: Upgrade, total_count: number, juice_info: any): string {
    let extra_arr = upgrade.state.slice(0, total_count).map(([juice, id]) => {
        let chance = 0.0
        // console.log(juice, id, active_profile.optimizer_worker_bundle.result.prep_output.juice_info.all_juices[0].data.get(String(upgrade.upgrade_index)))
        if (juice) {
            chance += juice_info.all_juices[0].data.get(String(upgrade.upgrade_index)).normal_chance
        }
        if (id > 0) {
            chance += juice_info.all_juices[id].data.get(String(upgrade.upgrade_index)).normal_chance
        }
        return chance
    })
    let artisan = 0
    // console.log(upgrade.normal_dist, extra_arr, upgrade.state)

    for (let count = 0; count < total_count; count++) {
        let min_count = Math.min(count, 10)

        let current_chance = Math.min(1, upgrade.base_chance + upgrade.extra_chance + min_count * upgrade.base_chance * 0.1 + extra_arr[count])
        if (artisan >= 1.0) {
            break
        }

        artisan += DEFAULT_ARTISAN_MULTIPLIER * current_chance * upgrade.artisan_rate
        if (current_chance == 1.0) {
            break // for upgrades that have 100% passrate immediately or upgrades that have above 100% success rate (juicing last few taps of like +4 or something)
        }
    }

    return (Math.min(artisan, 1) * 100).toFixed(2)
}

export function cumulative_chance(upgrade: Upgrade, total_count: number, juice_info: any): string {
    let extra_arr = upgrade.state.slice(0, total_count).map(([juice, id]) => {
        let chance = 0.0
        // console.log(juice, id, active_profile.optimizer_worker_bundle.result.prep_output.juice_info.all_juices[0].data.get(String(upgrade.upgrade_index)))
        if (juice) {
            chance += juice_info.all_juices[0].data.get(String(upgrade.upgrade_index)).normal_chance
        }
        if (id > 0) {
            chance += juice_info.all_juices[id].data.get(String(upgrade.upgrade_index)).normal_chance
        }
        return chance
    })
    let artisan = 0
    let cum_chance = 1.0
    // console.log(upgrade.normal_dist, extra_arr, upgrade.state)

    for (let count = 0; count < total_count; count++) {
        let min_count = Math.min(count, 10)

        let current_chance = Math.min(1, upgrade.base_chance + upgrade.extra_chance + min_count * upgrade.base_chance * 0.1 + extra_arr[count])
        if (artisan >= 1.0) {
            return (100.0).toFixed(2)
        }
        cum_chance *= 1 - current_chance
        artisan += DEFAULT_ARTISAN_MULTIPLIER * current_chance * upgrade.artisan_rate
        if (current_chance == 1.0) {
            return (100.0).toFixed(2)
        }
    }

    return (Math.max(1 - cum_chance, 0) * 100).toFixed(2)
}

export function compute_used_materials(upgrade: Upgrade, taps_so_far: number, juice_info: any, adv_juice_used: number, adv_scroll_used: number): number[] {
    if (!upgrade.cost_dist) return []
    let out = new Array(upgrade.cost_dist.length).fill(0)

    for (let cost_type = 0; cost_type < 7; cost_type++) {
        out[cost_type] = upgrade.unlock_costs[cost_type] + upgrade.costs[cost_type] * taps_so_far
    }

    let relevant_id_map = upgrade.is_normal_honing ? juice_info.normal_uindex_to_id : juice_info.adv_uindex_to_id
    // console.log(relevant_id_map[upgrade.upgrade_index])
    for (const id of relevant_id_map[upgrade.upgrade_index]) {
        let juice_cost = 0

        let juice_type = juice_info.all_juices[id].data.get(String(upgrade.upgrade_index))
        let amt = upgrade.is_normal_honing ? juice_type.normal_amt_used : juice_type.adv_amt_used

        if (upgrade.is_normal_honing) {
            for (let index = 0; index < Math.min(taps_so_far, upgrade.normal_dist.length - 1); index++) {
                if (!upgrade.is_normal_honing) {
                    juice_cost += amt
                } else if ((upgrade.state[index][0] === true && id == 0) || (upgrade.state[index][1] === id && id !== 0)) {
                    juice_cost += amt
                }
                // console.log(juice_cost)
            }
        } else {
            if (id === 0) {
                juice_cost = adv_juice_used * amt
            } else {
                juice_cost = adv_scroll_used * amt
            }
        }

        out[7 + id + (upgrade.is_weapon ? 0 : juice_info.num_juice_avail)] = juice_cost
    }
    return out
}

export function compute_remaininig_materials(used_materials: number[], active_profile: CharProfile, roster_config: RosterConfig) {
    const bound_budgets: number[] = []
    const roster_mats: number[] = []
    const tradable_mats: number[] = []
    used_materials.forEach((cost, index) => {
        if (cost <= 0) {
            bound_budgets.push(input_column_to_num(active_profile.bound_budgets[active_profile.tier])[index])
            roster_mats.push(input_column_to_num(roster_config.roster_mats_owned[active_profile.tier])[index])
            tradable_mats.push(input_column_to_num(roster_config.tradable_mats_owned[active_profile.tier])[index])
            return
        }
        let remaining_cost = cost
        // 1. Bound
        let bound_owned = input_column_to_num(active_profile.bound_budgets[active_profile.tier])[index]
        let deduct_bound = Math.min(bound_owned, remaining_cost)
        bound_budgets.push(Math.max(0, bound_owned - deduct_bound))
        remaining_cost -= deduct_bound
        // 2. Roster
        let roster_owned = input_column_to_num(roster_config.roster_mats_owned[active_profile.tier])[index]
        if (remaining_cost > 0 && roster_config.roster_mats_owned[index] !== undefined) {
            let deduct_roster = Math.min(roster_owned, remaining_cost)
            roster_mats.push(Math.max(0, roster_owned - deduct_roster))
            remaining_cost -= deduct_roster
        } else {
            roster_mats.push(roster_owned)
        }
        // 3. Tradable
        let tradable_owned = input_column_to_num(roster_config.tradable_mats_owned[active_profile.tier])[index]
        if (remaining_cost > 0 && roster_config.tradable_mats_owned[index] !== undefined) {
            let deduct_tradable = Math.min(tradable_owned, remaining_cost)
            tradable_mats.push(Math.max(0, tradable_owned - deduct_tradable))
        } else {
            tradable_mats.push(tradable_owned)
        }
    })
    // console.log("computed")
    return { bound_budgets, roster_mats, tradable_mats }
}
