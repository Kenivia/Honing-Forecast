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
    unlock_costs: number[]
    costs: number[]

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
export function parse_input(input_column: InputColumn, index: number, input: string, pretend_enabled?: boolean): number {
    if (!input_column.enabled[index] && !pretend_enabled) {
        return 999999999
    }
    let out = input_column.type === InputType.Int ? parseInt(input.replace(/,/g, "")) : parseFloat(input.replace(/,/g, ""))
    // console.log(input_column.upper_bound)
    return isFinite(out) ? Math.min(input_column.upper_bound[index], out) : 0
}
export function validate_input_column(old: InputColumn, default_example: InputColumn) {
    for (let row = 0; row < old.data.length; row++) {
        let correct_len = default_example.data.length
        if (
            old.data.length !== correct_len ||
            old.keys.length !== correct_len ||
            old.upper_bound.length !== correct_len ||
            old.enabled.length !== correct_len
        ) {
            old = structuredClone(default_example)
        } else {
            old.data[row] = parse_input(old, row, old.data[row], true).toLocaleString()
        }
    }
}
export function validate_input_column_array(old: InputColumn[], example: InputColumn[]) {
    for (let index = 0; index < old.length; index++) {
        validate_input_column(old[index], example[index])
    }
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

export function is_enum<T extends object>(targetEnum: T, inp: unknown): inp is T[keyof T] {
    return Object.values(targetEnum)
        .filter((v) => typeof v === "number")
        .includes(inp as number)
}

// returning instead of mutating in place because javascript
export function get_valid_status_grid(status_grid: StatusGrid, example: StatusGrid) {
    const isValid =
        status_grid.length === example.length &&
        example.every((row, i) => row.length === status_grid[i].length && status_grid[i].every((cell) => is_enum(UpgradeStatus, cell)))
    console.log(isValid, status_grid, example)
    return isValid ? status_grid : example.slice()
}
// ========================================================================================
// These are to interface between UI and rust

//                    'bound','tradable', leftover(bound), tradable sell price, market price
export type OneMaterial = [number, number, number, number, number] // an array of this is passed into rust

//                        piece type, upgrade index, is_normal, normal_progress, state, unlock, succeeded, adv_progress
export type OneUpgrade = [number, number, boolean, number | null, OneState[], boolean, boolean, AdvProgress | null]
// an array of this is passed into rust

export type OneUpgradeKey = `${number},${number},${"true" | "false"},${number}`
export type KeyedUpgrades = Record<OneUpgradeKey, OneUpgrade> // This is modified by UI

export function to_upgrade_key(piece_type: number, upgrade_index: number, is_normal: boolean, tier: number): OneUpgradeKey {
    return `${piece_type},${upgrade_index},${is_normal},${tier}`
}

export function grids_to_keyed(normal_grid: StatusGrid, adv_grid: StatusGrid, all_keyed: KeyedUpgrades, tier: number) {
    let new_keyed: KeyedUpgrades = {}
    for (const [piece_type, row] of status_to_bool_grid(normal_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, true, tier)
            if (cell) {
                if (key in all_keyed && isOneUpgrade(all_keyed[key])) {
                    new_keyed[key] = all_keyed[key]
                } else {
                    new_keyed[key] = [piece_type, upgrade_index, true, 0, null, false, false, null]
                }
            }
        }
    }
    for (const [piece_type, row] of status_to_bool_grid(adv_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, false, tier)
            if (cell) {
                if (key in all_keyed && isOneUpgrade(all_keyed[key])) {
                    new_keyed[key] = all_keyed[key]
                } else {
                    new_keyed[key] = [piece_type, upgrade_index, false, null, null, false, false, [0, 0, false, false]]
                }
            }
        }
    }
    // console.log(new_keyed)
    return new_keyed
}

function isOneUpgrade(foo: unknown): foo is OneUpgrade {
    if (!Array.isArray(foo) || foo.length !== 8) return false

    const [f0, f1, f2, f3, f4, f5, f6, f7] = foo

    return (
        typeof f0 === "number" &&
        typeof f1 === "number" &&
        typeof f2 === "boolean" &&
        (f3 === null || typeof f3 === "number") &&
        Array.isArray(f4) &&
        f4.every((x) => x.length == 2 && typeof x[0] === "boolean" && typeof x[1] === "number") &&
        typeof f5 === "boolean" &&
        typeof f6 === "boolean" &&
        (f7 === null || (typeof f7[0] === "number" && typeof f7[1] === "number" && typeof f7[2] === "boolean" && typeof f7[3] === "boolean"))
    )
}
