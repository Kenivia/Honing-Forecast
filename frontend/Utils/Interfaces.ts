import { assert } from "console"

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
    data: Record<string, string>
    keys: string[]
    type: InputType
    upper_bound: Record<string, number>
    enabled: Record<string, boolean>
    toNumArray(): number[]
    concat(_: InputColumn): InputColumn
    toNumObj(): Record<string, number>
}
export enum InputType {
    Int,
    Float,
}
export function createInputColumn(
    keys: string[],
    type: InputType,

    data?: Record<string, string>,
    upper_bound?: Record<string, number>,
    enabled?: Record<string, boolean>,
): InputColumn {
    return {
        keys,
        type,
        data: Object.fromEntries(keys.map((k: string) => [k, data[k] ?? "0"])),
        upper_bound: Object.fromEntries(keys.map((k: string) => [k, upper_bound[k] ?? Infinity])),
        enabled: Object.fromEntries(keys.map((k: string) => [k, enabled[k] ?? false])),
        toNumArray() {
            return this.keys.map((k: string) => parseInput(this, k, this.data[k]))
        },
        toNumObj() {
            return Object.fromEntries(this.keys.map((k: string) => [k, parseInput(this, k, this.data[k])]))
        },
        concat(other: InputColumn) {
            assert(this.keys.intersection(other.keys).length == 0)
            assert(this.type == other.type)
            this.data = { ...this.data, ...other.data }
            this.keys = this.keys.concat(other.keys)
            this.upper_bound = { ...this.upper_bound, ...other.upper_bound }
            this.enabled = { ...this.enabled, ...other.enabled }
            return this
        },
    }
}
export function modifyInputColumn(input_column: InputColumn, label: string, event: Event) {
    input_column[label] = parseInput(input_column, label, (event.target as HTMLInputElement).value)
}
function parseInput(input_column: InputColumn, label: string, input: string): number {
    if (!input_column.enabled[label]) {
        return 0
    }
    let out = input_column.type == InputType.Int ? parseInt(input) : parseFloat(input)
    return isFinite(out) ? Math.min(input_column.upper_bound[label], out) : 0
}
export interface StatusGrid {
    data: UpgradeStatus[][]
    toBool(): BoolGrid
}

export function createStatusGrid(rows: number, cols: number): StatusGrid {
    return {
        data: Array.from({ length: rows }, () => Array(cols).fill(UpgradeStatus.NotYet)),

        toBool(): BoolGrid {
            return this.data.map((row: UpgradeStatus[]) => row.map((cell) => cell == UpgradeStatus.NotYet))
        },
    }
}
