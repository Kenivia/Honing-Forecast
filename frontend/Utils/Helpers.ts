import { CharProfile } from "@/Stores/CharacterProfile"
import { ADV_COLS, IconMap, NUM_PIECES, PIECE_NAMES, PLUS_TIER_CONVERSION } from "./Constants"
import { InputColumn, InputType, Upgrade, UpgradeStatus } from "./Interfaces"
import { storeToRefs } from "pinia"
import { useRosterStore } from "@/Stores/RosterConfig"

export function format_char_name(raw: string, char_index: number, roster_index: number): string {
    const { all_profiles, roster_config } = storeToRefs(useRosterStore())
    let result = raw.replace(/ /g, "") //
    // 2. Remove non-alphanumeric (keep underscores)
    result = result.replace(/[^a-zA-Z0-9_]/g, "")
    // 3. Lowercase every letter after the first
    result = result.replace(/(?<=.)[A-Z]/g, (c) => c.toLowerCase()).slice(0, 16)
    // 4. If empty, or already taken by another profile, append index
    const otherNames = all_profiles.value.filter((_, i) => i !== char_index).map((x) => x.char_name)
    while (!result || otherNames.includes(result)) {
        result += String(char_index + 1)
        if (roster_index > 0) {
            result += roster_config.value.roster_names[roster_index]
        }
    }
    return result
}

export function check_adv_all_done() {
    const { active_profile } = storeToRefs(useRosterStore())
    for (let row = 0; row < NUM_PIECES; row++) {
        for (let col = 0; col < ADV_COLS; col++) {
            if (active_profile.value.adv_grid[row][col] != UpgradeStatus.Done) {
                return false
            }
        }
    }
    return true
}
export function check_all_plus_20(): number | boolean {
    const { active_profile } = storeToRefs(useRosterStore())
    for (let row = 0; row < NUM_PIECES; row++) {
        let highest_done = active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done) + 1

        if (!(highest_done >= 20)) {
            return false
        }
    }
    return true
}
export function check_revert_ilevel_ok(): number | boolean {
    const { active_profile } = storeToRefs(useRosterStore())
    if (active_profile.value.tier == 0) {
        return true
    }
    for (let row = 0; row < NUM_PIECES; row++) {
        let highest_done = active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done) + 1

        if (!PLUS_TIER_CONVERSION[active_profile.value.tier].hasOwnProperty(String(highest_done))) {
            return highest_done
        }
        let highest_want = active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done || value == UpgradeStatus.Want) + 1

        if (!PLUS_TIER_CONVERSION[active_profile.value.tier].hasOwnProperty(String(highest_want))) {
            return highest_want
        }
    }
    return true
}
// export function check_revert_eligibility(): boolean {
//     return check_adv_all_done() && check_revert_ilevel_ok() === true
// }
export function achieved_ilevel(profile: CharProfile): string {
    let out = profile.tier == 0 ? 1590 : 1635
    if (profile.tier == 0) {
        for (let row = 0; row < NUM_PIECES; row++) {
            let highest_plus = profile.adv_grid[row].findLastIndex((value) => value == UpgradeStatus.Done) + 1
            out += (highest_plus * 10) / 6
        }
    } else {
        out += 40
    }
    for (let row = 0; row < NUM_PIECES; row++) {
        let highest_plus = profile.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done) + 1
        // find last index = -1 if nothing is done
        if (highest_plus == 0) {
            return "?"
        } else {
            out += (highest_plus * 5) / 6
        }
    }
    return out.toFixed(2)
}

export function pending_ilevel(active_profile: CharProfile): string {
    let out = active_profile.tier == 0 ? 1590 : 1635
    if (active_profile.tier == 0) {
        for (let row = 0; row < NUM_PIECES; row++) {
            let highest_plus = active_profile.adv_grid[row].findLastIndex((value) => value == UpgradeStatus.Done || value == UpgradeStatus.Want) + 1
            out += (highest_plus * 10) / 6
        }
    } else {
        out += 40
    }
    for (let row = 0; row < NUM_PIECES; row++) {
        let highest_plus = active_profile.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done || value == UpgradeStatus.Want) + 1
        // find last index = -1 if nothing is done
        if (highest_plus == 0) {
            return "?"
        } else {
            out += (highest_plus * 5) / 6
        }
    }
    return out.toFixed(2)
}

export function cssVar(name: string, fallback: string) {
    if (typeof window === "undefined") return fallback
    const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
    return value || fallback
}

export function debounce<T extends (...args: any[]) => void>(fn: T, delay: number): T {
    let timer: ReturnType<typeof setTimeout>
    return ((...args: any[]) => {
        clearTimeout(timer)
        timer = setTimeout(() => fn(...args), delay)
    }) as T
}

export function metric_to_text(metric: number | null | undefined) {
    if (metric === null || metric === undefined || !Number.isFinite(metric)) return "N/A"
    return `${Math.round(metric == 0 ? metric : -metric).toLocaleString("en-US")}g`
}
export function get_icon_path(name: string) {
    return IconMap[name] ?? ""
}
export function get_piece_name(upgrade: Upgrade) {
    return PIECE_NAMES[upgrade.piece_type]
}

const ordinalRules = new Intl.PluralRules("en", { type: "ordinal" })
export function toOrdinal(n: number): string {
    const suffixes: Record<Intl.LDMLPluralRule, string> = {
        zero: "th",
        one: "st",
        two: "nd",
        few: "rd",
        many: "th",
        other: "th",
    }

    return `${n}${suffixes[ordinalRules.select(n)]}`
}
