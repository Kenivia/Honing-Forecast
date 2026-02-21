import { PIECE_NAMES } from "./Constants.ts"
const ordinalRules = new Intl.PluralRules("en", { type: "ordinal" })

const SCALE_NAMES = {
    3: "thousand",
    6: "million",
    9: "billion",
    12: "trillion",
    15: "quadrillion",
    18: "quintillion",
    21: "sextillion",
    24: "septillion",
    27: "octillion",
    30: "nonillion",
    33: "decillion",
    36: "undecillion",
    39: "duodecillion",
}

export function powerOfTenToWords(exponent) {
    if (exponent < 5) {
        // handle small ones explicitly
        if (exponent === 0) return "1"
        if (exponent === 1) return "10"
        if (exponent === 2) return "100"
        if (exponent === 3) return (1000).toLocaleString("en-US", { maximumFractionDigits: 0, minimumFractionDigits: 0 })
        if (exponent === 4) return (10000).toLocaleString("en-US", { maximumFractionDigits: 0, minimumFractionDigits: 0 })
        return `10^${exponent}`
    }

    const remainder = exponent % 3
    const baseExponent = exponent - remainder
    const scaleName = SCALE_NAMES[baseExponent]

    if (!scaleName) {
        return `10^${exponent}` // fallback if scale not defined
    }

    let prefix = "1"
    if (remainder === 1) prefix = "10"
    if (remainder === 2) prefix = "100"

    return `${prefix} ${scaleName}`
}

export function taxRound(label: string, numericMax: number) {
    const factor = label == "Silver" ? 0.0001 : label == "Shards" ? 0.001 : label == "Red" || label == "Blue" ? 0.01 : 1

    return Math.max(Math.floor((numericMax * 0.05) / factor) * factor, factor)
}
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
export function piece_display_name(upgrade: any): string {
    return (
        PIECE_NAMES[upgrade.piece_type] +
        (upgrade.is_normal_honing ? " +" : " AH") +
        "" +
        (upgrade.is_normal_honing ? "" : " " + String(upgrade.upgrade_index * 10 + 1) + "-") +
        String((upgrade.upgrade_index + 1) * (upgrade.is_normal_honing ? 1 : 10))
    )
}
export function formatSig(x: number, place: number = 3): string {
    if (!isFinite(x)) return ""
    if (x > 1) {
        place = Math.max(1, Math.min(100, place))

        const abs = Math.abs(x)
        let suffix = ""
        let divisor = 1

        if (abs >= 1_000_000_000) {
            suffix = "B"
            divisor = 1_000_000_000
        } else if (abs >= 1_000_000) {
            suffix = "M"
            divisor = 1_000_000
        } else if (abs >= 1_000) {
            suffix = "K"
            divisor = 1_000
        }

        const scaled = x / divisor

        // keep `place` significant figures, but trim trailing zeros
        let s = parseFloat(Number(scaled.toFixed(place)).toPrecision(place)).toLocaleString("en-US", {
            minimumFractionDigits: 1, // show decimals for small K/M/B
            maximumFractionDigits: place,
        })

        return s + suffix
    } else {
        return x.toPrecision(3)
    }
}

// TypeScript interfaces
export interface Upgrade {
    is_normal_honing: boolean
    prob_dist: number[]
    original_prob_dist: number[]
    base_chance: number
    costs: number[]
    one_juice_cost: number
    adv_juice_cost: number[]
    special_cost: number
    values: number[]
    prob_dist_len: number
    is_weapon: boolean
    artisan_rate: number
    tap_offset: number
    upgrade_index: number
    special_value: number
    equipment_type?: string // Added for equipment type
    is_finished?: boolean // Track if upgrade is completed
    completion_order?: number // Track order of completion
    current_artisan?: number // Track current artisan for this upgrade
    taps_so_far?: number // Number of taps attempted so far
    juice_taps_so_far?: number // Number of taps with juice so far
    free_taps_so_far?: number // Number of free taps so far
    use_juice?: boolean // Whether juice is currently enabled for this upgrade
    cumulative_chance?: number // Cumulative chance of success for normal honing
    other_prob_dist?: number[] // Probability distribution for the other strategy (for advanced honing)
}

export function sortedUpgrades(upgradeArr: Upgrade[]) {
    let out = [...upgradeArr]
    out.sort((a, b) => {
        // Unfinished normal honing upgrades first
        if (a.is_finished < b.is_finished) {
            return -1
        }
        if (a.is_finished > b.is_finished) {
            return 1
        }
        if (a.is_normal_honing < b.is_normal_honing) {
            return 1
        }
        if (a.is_normal_honing > b.is_normal_honing) {
            return -1
        }

        // Then finished upgrades by completion order
        if (a.is_finished && b.is_finished) {
            return (a.completion_order || 0) - (b.completion_order || 0)
        }
        if (!a.is_finished && !b.is_finished) {
            if (a.upgrade_index < b.upgrade_index) {
                return -1
            }
            if (a.upgrade_index > b.upgrade_index) {
                return 1
            }
            return PIECE_NAMES.findIndex((value, _) => a.equipment_type == value) - PIECE_NAMES.findIndex((value, _) => b.equipment_type == value)
        }
        return 0
    })
    return out
}

// Helper function to get the next unfinished upgrade index
export function getNextUnfinishedIndex(upgradeArr: Upgrade[], excludeIndex?: number): number {
    let first_try = upgradeArr.findIndex((z) => z == sortedUpgrades(upgradeArr).find((upg, i) => !upg.is_finished && i > excludeIndex))
    // console.log(first_try)
    if (first_try < 0) {
        return upgradeArr.findIndex((z) => z == sortedUpgrades(upgradeArr).find((upg, i) => !upg.is_finished && i !== excludeIndex))
    }
    return first_try
}

// Helper function to calculate tap record costs
export function calculateTapRecordCosts(upgrade: Upgrade) {
    const costs = new Array(10).fill(0)
    const taps = upgrade.taps_so_far ?? 0
    const juiceTaps = upgrade.juice_taps_so_far ?? 0
    const freeTaps = upgrade.free_taps_so_far ?? 0

    // Regular costs multiplied by taps
    for (let i = 0; i < 7; i++) {
        costs[i] = upgrade.costs[i] * taps
    }

    // Juice costs
    if (juiceTaps > 0) {
        const juiceCost = upgrade.one_juice_cost * juiceTaps
        if (upgrade.is_weapon) {
            costs[8] = juiceCost // Weapons add to 9th slot (index 8)
        } else {
            costs[7] = juiceCost // Armors add to 8th slot (index 7)
        }
    }

    // Free tap costs

    costs[9] = upgrade.special_cost * freeTaps

    return costs
}

export function calculateCurrentChance(upgrade: Upgrade) {
    if (!upgrade.is_normal_honing) return 0
    const baseChance = upgrade.base_chance
    const minCount = Math.min(upgrade.taps_so_far, 10)
    const currentChance = baseChance + (baseChance / 10) * minCount
    return Math.max(0, Math.min(1, upgrade.current_artisan >= 1 ? 1 : upgrade.use_juice ? currentChance + upgrade.base_chance : currentChance))
}

export function updateCumulativeChance(upgrade: Upgrade, attemptChance: number) {
    if (!upgrade.is_normal_honing) return

    // Initialize cumulative chance if it doesn't exist
    if (upgrade.cumulative_chance === undefined) {
        upgrade.cumulative_chance = 0
    }

    // Update cumulative chance: add the probability of succeeding on this attempt
    // given that all previous attempts failed
    const previousFailureProbability = 1 - upgrade.cumulative_chance
    upgrade.cumulative_chance += attemptChance * previousFailureProbability

    // Ensure it doesn't exceed 1
    upgrade.cumulative_chance = Math.min(1, upgrade.cumulative_chance)
}

export function getTapCountRange(upgrade: Upgrade) {
    if (upgrade.is_normal_honing) return null

    // Use other strategy's probability distribution if juice is ticked
    const probDistToUse = upgrade.use_juice && upgrade.other_prob_dist ? upgrade.other_prob_dist : upgrade.prob_dist

    const range = `${upgrade.tap_offset} - ${upgrade.tap_offset + probDistToUse.length}`
    const isUsingOtherStrategy = upgrade.use_juice && upgrade.other_prob_dist

    return { range, isUsingOtherStrategy }
}

/**
 * Convert ticks (boolean grid) to counts (integer array)
 * Mirrors the Rust implementation in helpers.rs
 *
 * @param ticks - 6xN boolean array where:
 *   - rows 0-4 represent armor pieces (Helmet, Shoulder, Chest, Pants, Glove)
 *   - row 5 represents weapon
 * @returns 2xN array where:
 *   - [0][i] = sum of ticks[0..4][i] (armor count for level i)
 *   - [1][i] = ticks[5][i] ? 1 : 0 (weapon count for level i)
 */
export function ticksToCounts(ticks: boolean[][]): number[][] {
    if (ticks.length === 0 || ticks[0].length === 0) {
        return [[], []]
    }

    const cols = ticks[0].length
    const out: number[][] = [[], []]

    for (let i = 0; i < cols; i++) {
        // Sum ticks[0..4][i] for armor count
        let armorCount = 0
        for (let row = 0; row < 5; row++) {
            if (ticks[row] && ticks[row][i]) {
                armorCount++
            }
        }
        out[0][i] = armorCount

        // ticks[5][i] as 0/1 for weapon count
        out[1][i] = ticks[5] && ticks[5][i] ? 1 : 0
    }

    return out
}

/**
 * Convert counts (integer array) to ticks (boolean grid)
 * Inverse of ticksToCounts - fills checkboxes from top
 *
 * @param counts - 2xN array where:
 *   - [0][i] = armor count for level i (max 5)
 *   - [1][i] = weapon count for level i (max 1)
 * @returns 6xN boolean array
 */
export function countsToTicks(counts: number[][]): boolean[][] {
    if (counts.length < 2 || counts[0].length === 0) {
        return Array.from({ length: 6 }, () => [])
    }

    const cols = counts[0].length
    const ticks: boolean[][] = Array.from({ length: 6 }, () => Array(cols).fill(false))

    for (let i = 0; i < cols; i++) {
        const armorCount = Math.min(5, Math.max(0, counts[0][i] || 0))
        const weaponCount = Math.min(1, Math.max(0, counts[1][i] || 0))

        // Fill armor checkboxes from top
        for (let row = 0; row < armorCount; row++) {
            ticks[row][i] = true
        }

        // Fill weapon checkbox if needed
        if (weaponCount > 0) {
            ticks[5][i] = true
        }
    }

    return ticks
}
