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
}

export type BoolGrid = boolean[][]
export type NumGrid = number[][]
export type StateGrid = [boolean, number][][][]
// ^ these types go directly to wasm and should not become an interface with multiple fields

export enum UpgradeStatus {
    Done,
    Want,
    NotYet,
}

export interface InputsColumn {
    data: Record<string, string>
    keys: string[]
    type: "int" | "float"
    upper_bound: number
    enabled: Record<string, boolean>
    toNum(): number[]
}
export function createInputColumn(
    data: Record<string, string>,
    keys: string[],
    type: "int" | "float",
    upper_bound: number,
    enabled: Record<string, boolean>,
): InputsColumn {
    return {
        data,
        keys,
        type,
        upper_bound,
        enabled,
        toNum() {
            return this.keys.map((k: string) => (!enabled[k] ? 0 : this.type === "int" ? parseInt(this.data[k]) : parseFloat(this.data[k])))
        },
    }
}
export function modifyInputColumn() {
    // WIP
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
