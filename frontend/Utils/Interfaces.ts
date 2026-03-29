export enum WasmOp {
    EvaluateAverage,
    OptimizeAverage,
    Histogram,
    Parser,
}
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
// ========================================================================================
// Status grid (TickboxGrid) stuff
export enum UpgradeStatus {
    Done,
    Want,
    NotYet,
}

export type BoolGrid = boolean[][]
export type StatusGrid = UpgradeStatus[][]

// ========================================================================================
// These are to interface between UI and rust

//                    'bound','tradable', leftover(bound), tradable sell price, market price
export type OneMaterial = [number, number, number, number, number] // an array of this is passed into rust

//                        piece type, upgrade index, is_normal, normal_progress, state, unlock, succeeded, adv_progress
export type OneUpgrade = [number, number, boolean, number | null, OneState[] | null, boolean, boolean, AdvProgress | null]
// an array of this is passed into rust

export type OneUpgradeKey = `${number},${number},${"true" | "false"},${number}`
export type KeyedUpgrades = Record<OneUpgradeKey, OneUpgrade> // This is modified by UI
