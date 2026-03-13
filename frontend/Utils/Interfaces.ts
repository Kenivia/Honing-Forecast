import { assert } from "console"
import { ALL_LABELS } from "./Constants"
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
    type: InputType
    upper_bound: number[]
    enabled: boolean[]
    toNum(): number[]
}
export enum InputType {
    Int,
    Float,
}
export function createInputColumn(type: InputType, data?: string[], upper_bound?: number[], enabled?: boolean[]): InputColumn {
    return {
        type,
        data: data ?? ALL_LABELS.map((_) => "0"),
        upper_bound: upper_bound ?? ALL_LABELS.map((_) => Infinity),
        enabled: enabled ?? ALL_LABELS.map((_) => false),
        toNum(): number[] {
            return this.data.map((x: string, index: number) => parseInput(this, index, x))
        },
    }
}
export function modifyInputColumn(input_column: InputColumn, index: number, event: Event) {
    input_column[index] = parseInput(input_column, index, (event.target as HTMLInputElement).value)
}
function parseInput(input_column: InputColumn, index: number, input: string): number {
    if (!input_column.enabled[index]) {
        return 0
    }
    let out = input_column.type == InputType.Int ? parseInt(input) : parseFloat(input)
    return isFinite(out) ? Math.min(input_column.upper_bound[index], out) : 0
}
export interface StatusGrid {
    data: UpgradeStatus[][]
    toBool(): BoolGrid
}

export function createStatusGrid(rows: number, cols: number): StatusGrid {
    return {
        data: Array.from({ length: rows }, () => Array(cols).fill(UpgradeStatus.NotYet)),

        toBool(): BoolGrid {
            return this.data.map((row: UpgradeStatus[]) => row.map((cell) => cell == UpgradeStatus.Want))
        },
    }
}
export type OneMaterial = [number, number, number, number, number]
export type MaterialInput = OneMaterial[]

//                 piece type, upgrade index, is_adv, normal_progress, state, unlock, succeeded, adv_progress
export type OneUpgrade = [number, number, boolean, number | null, State[], boolean, boolean, AdvProgress | null]
export type UpgradeInput = OneUpgrade[]
export const DEFAULT_ONE_UPGRADE = [0, [], false, false, [0, 0, false, false]] // excluding the first 3

export type KeyedUpgradeInput = Record<OneUpgradeKey, OneUpgrade>
type OneUpgradeKey = `${number},${number},${"true" | "false"}`
export function to_upgrade_key(piece_type: number, upgrade_index: number, is_adv: boolean): OneUpgradeKey {
    return `${piece_type},${upgrade_index},${is_adv}`
}
export function keyed_to_array(KeyedUpgradeInput: KeyedUpgradeInput): UpgradeInput {
    return Object.entries(KeyedUpgradeInput).map(([_, one_upgrade]) => one_upgrade)
}
export function grids_to_keyed(normal_grid: StatusGrid, adv_grid: StatusGrid) {
    let out: KeyedUpgradeInput = {}
    for (const [piece_type, row] of normal_grid.toBool().entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            if (cell) {
                let key = to_upgrade_key(piece_type, upgrade_index, false)
                out[key] = [piece_type, upgrade_index, false, 0, [], false, false, null]
            }
        }
    }
    for (const [piece_type, row] of adv_grid.toBool().entries()) {
        for (const [upgrade_index, cell] of row.entries()) {
            if (cell) {
                let key = to_upgrade_key(piece_type, upgrade_index, true)

                out[key] = [piece_type, upgrade_index, true, null, [], false, false, [0, 0, false, false]]
            }
        }
    }
    return out
}

export type HistogramPair = [number, number]
export interface HistogramOutputs {
    cum_percentiles: HistogramPair[][]
    average: number[]
}
