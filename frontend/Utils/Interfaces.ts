import { ALL_LABELS, TIER_LABELS } from "./Constants"

// THESE BELOW DIRECTLY CORRESPOND TO A RUST STRUCT

export type HistogramPair = [number, number]
export interface HistogramOutputs {
    cum_percentiles: HistogramPair[][]
    average: number[]
    state_bundle: StateBundle
    bound_chance: number[]
    tradable_chance: number[]
}

export interface Upgrade {
    piece_type: number
    upgrade_index: number
    is_normal_honing?: boolean
    is_weapon?: boolean
    normal_dist?: number[]
    adv_dists?: number[][]
    state?: OneState[]
    succeeded?: boolean
    unlocked?: boolean
    alr_failed?: number
    adv_config: AdvConfig
    cost_dist: Support[]
    artisan_rate: number
    base_chance: number
    extra_chance: number

    // added for UI purpose, not in rust
    this_special_chance?: number
}
export interface Support {
    support: number[]
}
export interface AdvConfig {
    start_xp: number
    start_balls: number
    next_free: boolean
    next_big: boolean
    double_balls: boolean
    is_30_40: boolean

    grace_juice_target: number
    non_grace_juice_target: number
    grace_scroll_target: number
    non_grace_scroll_target: number
}
export interface StateBundle {
    upgrade_arr: Upgrade[]
    special_state: number[]
    special_invalid_index?: number
    latest_special_probs?: number[]
    min_resolution: number
    gold_breakdown?: number[]
    prep_output: any
    special_cache: any
    adv_cache: any
    metric?: number
}

export type OneState = [boolean, number] // juice, bookid
export type AdvProgress = [number, number, boolean, boolean] // current xp(0 to 100 or 99 ig), current balls ( 0 to 6), next_free, next_big

// ========================================================================================

// This section is for the InputColumn

export interface InputColumn {
    data: string[]
    keys: string[]
    type: InputType
    upper_bound: number[]
    enabled: boolean[]
}
export enum InputType {
    Int,
    Float,
}
export function create_input_column(type: InputType, keys: string[], data?: string[], upper_bound?: number[], enabled?: boolean[]): InputColumn {
    return {
        type,
        keys,
        data: data ?? keys.map((_) => "0"),
        upper_bound: upper_bound ?? keys.map((_) => 999999999),
        enabled: enabled ?? keys.map((_) => true),
    }
}

export function input_column_to_num(input_column: InputColumn): number[] {
    return input_column.data.map((x: string, index: number) => parse_input(input_column, index, x))
}

export function get_modified_cell(input_column: InputColumn, index: number, event: Event) {
    if (!input_column.enabled[index]) {
        return input_column.data[index]
    }
    return parse_input(input_column, index, (event.target as HTMLInputElement).value.replace(/[^\d,]/g, "")).toLocaleString()
}
export function parse_input(input_column: InputColumn, index: number, input: string): number {
    if (!input_column.enabled[index]) {
        return 999999999
    }
    let out = input_column.type === InputType.Int ? parseInt(input.replace(/,/g, "")) : parseFloat(input.replace(/,/g, ""))
    // console.log(input_column.upper_bound)
    return isFinite(out) ? Math.min(input_column.upper_bound[index], out) : 0
}
export function fill_new_tiers_with_default(old: InputColumn[]) {
    while (old.length < TIER_LABELS.length) {
        old.push(create_input_column(InputType.Int, ALL_LABELS[old.length]))
    }
}

// ========================================================================================
// Status grid (TickboxGrid) stuff
export enum UpgradeStatus {
    Done,
    Want,
    NotYet,
}

export type BoolGrid = boolean[][]
export type StatusGrid = UpgradeStatus[][]

export function status_to_bool_grid(status_grid: StatusGrid): BoolGrid {
    return status_grid.map((row: UpgradeStatus[]) => row.map((cell) => cell == UpgradeStatus.Want))
}
export function createStatusGrid(
    rows: number,
    cols: number,
    data: UpgradeStatus[][] = Array.from({ length: rows }, () => Array(cols).fill(UpgradeStatus.NotYet)),
): StatusGrid {
    if (!data) {
        data = Array.from({ length: rows }, () => Array(cols).fill(UpgradeStatus.NotYet))
    }
    return data
}

// ========================================================================================
// These are to interface between UI and rust

//                    'bound','tradable', leftover(bound), tradable sell price, market price
export type OneMaterial = [number, number, number, number, number] // an array of this is passed into rust

//                        piece type, upgrade index, is_normal, normal_progress, state, unlock, succeeded, adv_progress
export type OneUpgrade = [number, number, boolean, number | null, OneState[], boolean, boolean, AdvProgress | null]
// an array of this is passed into rust

type OneUpgradeKey = `${number},${number},${"true" | "false"},${number}`
export type KeyedUpgrades = Record<OneUpgradeKey, OneUpgrade> // This is modified by UI

export function to_upgrade_key(piece_type: number, upgrade_index: number, is_normal: boolean, tier: number): OneUpgradeKey {
    return `${piece_type},${upgrade_index},${is_normal},${tier}`
}

export function grids_to_keyed(normal_grid: StatusGrid, adv_grid: StatusGrid, all_keyed: KeyedUpgrades, tier: number) {
    let new_keyed: KeyedUpgrades = {}
    for (const [piece_type, row] of status_to_bool_grid(normal_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, false, tier)
            if (cell) {
                if (key in all_keyed) {
                    new_keyed[key] = all_keyed[key]
                } else {
                    new_keyed[key] = [piece_type, upgrade_index, true, 0, null, false, false, null]
                }
            }
        }
    }
    for (const [piece_type, row] of status_to_bool_grid(adv_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, true, tier)
            if (cell) {
                if (key in all_keyed) {
                    new_keyed[key] = all_keyed[key]
                } else {
                    new_keyed[key] = [piece_type, upgrade_index, false, null, null, false, false, [0, 0, false, false]]
                }
            }
        }
    }
    return new_keyed
}
