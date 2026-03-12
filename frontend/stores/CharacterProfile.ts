import { defineStore } from "pinia"
import { AdvProgress, AdvProgressGrid, BoolGrid, createInputColumn, createStatusGrid, InputColumn, InputType, makeDefaultBoolGrid, makeDefaultNumGrid, NumGrid, StateBundle, StateGrid, StatusGrid } from "@/Utils/Interfaces"
import { createWorkerBundle } from "@/WasmInterface/worker_setup"
import { ADV_COLS, JUICE_LABELS, MATS_LABELS, NORMAL_COLS, NUM_PIECES } from "@/Utils/Constants"

export const useProfilesStore = defineStore("profiles", {
    state: () => ({
        profiles: [],
        activeProfileId: null,
    }),
    getters: {
        activeProfile: (state) => state.profiles.find((p) => p.id === state.activeProfileId) ?? null,
    },
    actions: {
        switchProfile(id: string) {
            this.activeProfileId = id
        },
        addProfile(profile: CharProfile) {
            this.profiles.push(profile)
        },
    },
})

export enum TreatmentPlan {
    TreatRosterAsTradable, // rat alt
    TreatRosterAsBound, // alt
    TreatTradableAsBound, // main
}
export interface CharProfile {
    treatment_plan: TreatmentPlan

    express_event: boolean
    char_name: string

    auto_start_optimizer: boolean
    has_run_optimizer: boolean
    optimizer_worker_bundle: any
    evaluate_worker_bundle: any
    histogram_worker_bundle: any

    state_bundle?: StateBundle
    normal_grid: StatusGrid
    adv_grid: StatusGrid

    special_owned: InputColumn
    bound_mats_owned: InputColumn
    mats_leftover: InputColumn

    bound_weap_juice_owned: InputColumn
    weap_juice_leftover: InputColumn

    bound_armor_juice_owned: InputColumn
    armor_juice_leftover: InputColumn

    normal_progress_grid: NumGrid
    normal_state_grid: StateGrid
    special_state: number[]
    normal_unlock_grid: BoolGrid
    succeeded_grid: BoolGrid
    adv_progress_grid: AdvProgressGrid

    tier: number
    min_resolution: number
    num_threads: number
    metric_type: number
}

export const DEFAULT_CHAR_PROFILE: CharProfile = {
     treatment_plan: TreatmentPlan.TreatRosterAsBound,

    express_event: false,
    char_name: "YourChar",

    auto_start_optimizer: false,
    has_run_optimizer: false,
    optimizer_worker_bundle: createWorkerBundle(),
    evaluate_worker_bundle: createWorkerBundle(),
    histogram_worker_bundle: createWorkerBundle(),

    state_bundle : null,
    normal_grid: createStatusGrid(NUM_PIECES, NORMAL_COLS),
    adv_grid:createStatusGrid(NUM_PIECES, ADV_COLS),

    special_owned: createInputColumn([MATS_LABELS[7]],InputType.Int,null, Object.fromEntries([[MATS_LABELS[7], 33333]])),
    bound_mats_owned: createInputColumn(MATS_LABELS.slice(0,7),InputType.Int),
    mats_leftover: createInputColumn(MATS_LABELS.slice(0,7),InputType.Float), // implicit 0 leftover here

    bound_weap_juice_owned: createInputColumn(JUICE_LABELS.map((x) => x[0]),InputType.Int),
    weap_juice_leftover:  createInputColumn(JUICE_LABELS.map((x) => x[0]),InputType.Float),

    bound_armor_juice_owned:createInputColumn(JUICE_LABELS.map((x) => x[1]),InputType.Int),
    armor_juice_leftover:  createInputColumn(JUICE_LABELS.map((x) => x[1]),InputType.Float),

    normal_progress_grid: makeDefaultNumGrid(NUM_PIECES,NORMAL_COLS),
    normal_state_grid:makeDefaultNumGrid(NUM_PIECES,NORMAL_COLS),
    special_state: number[]
    normal_unlock_grid: BoolGrid
    succeeded_grid: BoolGrid
    adv_progress_grid: AdvProgressGrid

    tier: number
    min_resolution: number
    num_threads: number
    metric_type: number

}
