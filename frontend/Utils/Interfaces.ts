import { assert } from "console"

import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"

export interface Upgrade {
    piece_type: number
    upgrade_index: number
    is_normal_honing?: boolean
    is_weapon?: boolean
    prob_dist?: number[]
    state?: [boolean, number][]
    succeeded?: boolean
    unlocked?: boolean
    alr_failed?: number
}

export interface StateBundle {
    upgrade_arr: Upgrade[]
    special_state: number[]
    special_invalid_index?: number
    latest_special_probs?: number[]
    min_resolution: number
    average_breakdown?: number[]
    prep_output: any
    special_cache: any
    adv_cache: any
    metric?: number
}

export type BoolGrid = boolean[][]
export type NumGrid = number[][]
export type State = [boolean, number] // juice, bookid
export type StateGrid = State[][][]
export type AdvProgress = [number, number, boolean, boolean] // current xp(0 to 100 or 99 ig), current balls ( 0 to 6), next_free, next_big
export type AdvProgressGrid = AdvProgress[][]
// ^ these types go directly to wasm and should not become an interface with multiple fields
export function makeDefaultBoolGrid(rows: number, cols: number): BoolGrid {
    return new Array(rows).fill(new Array(cols).fill(false))
}
export function makeDefaultNumGrid(rows: number, cols: number): NumGrid {
    return new Array(rows).fill(new Array(cols).fill(0))
}
export function makeDefaultStateGrid(rows: number, cols: number): StateGrid {
    return new Array(rows).fill(new Array(cols).fill([]))
}
export function makeDefaultAdvProgressGrid(rows: number, cols: number): AdvProgressGrid {
    return new Array(rows).fill(new Array(cols).fill([0, 0, false, false]))
}

export enum UpgradeStatus {
    Done,
    Want,
    NotYet,
}

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
export function forbid_non_numeric(input_column: InputColumn, index: number, event: Event) {
    const el = event.target as HTMLInputElement
    if (!input_column.enabled[index]) {
        el.value = ""
        return ""
    }
    const filtered = String(el.value).replace(/[^\d,]/g, "")
    el.value = filtered
    return filtered
}
export function get_modified_cell(input_column: InputColumn, index: number, event: Event) {
    return parse_input(input_column, index, (event.target as HTMLInputElement).value).toLocaleString()
}
function parse_input(input_column: InputColumn, index: number, input: string): number {
    if (!input_column.enabled[index]) {
        return 0
    }
    let out = input_column.type === InputType.Int ? parseInt(input.replace(/,/g, "")) : parseFloat(input.replace(/,/g, ""))
    return isFinite(out) ? Math.min(input_column.upper_bound[index], out) : 0
}

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
export type OneMaterial = [number, number, number, number, number]
export type MaterialInput = OneMaterial[]

//               enabled,  piece type, upgrade index, is_adv, normal_progress, state, unlock, succeeded, adv_progress
export type OneUpgrade = [number, number, boolean, number | null, State[], boolean, boolean, AdvProgress | null]
export type UpgradeInput = OneUpgrade[]
export const DEFAULT_ONE_UPGRADE = [0, [], false, false, [0, 0, false, false]] // excluding the first 3

export type keyed_upgrades = Record<OneUpgradeKey, [boolean, OneUpgrade]>
type OneUpgradeKey = `${number},${number},${"true" | "false"}`
export function to_upgrade_key(piece_type: number, upgrade_index: number, is_adv: boolean): OneUpgradeKey {
    return `${piece_type},${upgrade_index},${is_adv}`
}
export function keyed_to_array(keyed_upgrades: keyed_upgrades) {
    return Object.entries(keyed_upgrades)
        .map(([_, pair]) => pair)
        .filter((x) => x[0])
        .map((x) => x[1])
}
export function grids_to_keyed(normal_grid: StatusGrid, adv_grid: StatusGrid, all_keyed: keyed_upgrades) {
    // console.log(all_keyed)
    for (const [piece_type, row] of status_to_bool_grid(normal_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, false)
            if (cell) {
                if (key in all_keyed) {
                    all_keyed[key] = all_keyed[key]
                    all_keyed[key][0] = true
                } else {
                    all_keyed[key] = [true, [piece_type, upgrade_index, false, 0, [], false, false, null]]
                }
            } else {
                if (key in all_keyed) {
                    all_keyed[key] = all_keyed[key]
                    all_keyed[key][0] = false
                }
            }
        }
    }
    for (const [piece_type, row] of status_to_bool_grid(adv_grid).entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            let key = to_upgrade_key(piece_type, upgrade_index, false)
            if (cell) {
                if (key in all_keyed) {
                    all_keyed[key] = all_keyed[key]
                    all_keyed[key][0] = true
                } else {
                    all_keyed[key] = [true, [piece_type, upgrade_index, true, null, [], false, false, [0, 0, false, false]]]
                }
            } else {
                if (key in all_keyed) {
                    all_keyed[key] = all_keyed[key]
                    all_keyed[key][0] = false
                }
            }
        }
    }
    return all_keyed
}

export type HistogramPair = [number, number]
export interface HistogramOutputs {
    cum_percentiles: HistogramPair[][]
    average: number[]
}

export interface EvalAverageOutput {
    average_gold_per_treatement: number[]
}
