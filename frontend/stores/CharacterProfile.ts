import { defineStore } from "pinia"
import {
    AdvProgress,
    AdvProgressGrid,
    BoolGrid,
    createInputColumn,
    createStatusGrid,
    InputColumn,
    InputType,
    KeyedUpgradeInput,
    makeDefaultBoolGrid,
    makeDefaultNumGrid,
    NumGrid,
    StateBundle,
    StateGrid,
    StatusGrid,
} from "@/Utils/Interfaces"
import { createWorkerBundle } from "@/WasmInterface/worker_setup"
import { ADV_COLS, JUICE_LABELS, MATS_LABELS, NORMAL_COLS, NUM_PIECES, STORAGE_KEY } from "@/Utils/Constants"

export const useProfilesStore = defineStore("profiles", {
    state: () => DEFAULT_PROFILES_STATE,
    getters: {
        getActiveProfile: (state) => state.profiles[state.activeProfileId],
    },
    actions: {
        switchProfile(id: string) {
            this.activeProfileId = id
        },
        addProfile(profile: CharProfile) {
            this.profiles.push(profile)
        },
        setProfiles(profiles: CharProfile[]) {
            this.profiles = profiles
        },
    },
})
export function loadCharProfiles(): { profiles: CharProfile[]; activeProfileId: number } {
    const raw = localStorage.getItem(STORAGE_KEY + "_char_profiles")
    if (!raw) return DEFAULT_PROFILES_STATE

    const parsed = JSON.parse(raw)

    return parsed
}
const DEFAULT_PROFILES_STATE = {
    profiles: [],
    activeProfileId: null,
}

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

    KeyedUpgradeInput: KeyedUpgradeInput

    special_budget: number

    bound_budgets: InputColumn
    leftover_price: InputColumn

    special_state: number[]

    tier: number
    min_resolution: number
    num_threads: number
    metric_type: number
    cumulative_graph: boolean
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

    state_bundle: null,
    normal_grid: createStatusGrid(NUM_PIECES, NORMAL_COLS),
    adv_grid: createStatusGrid(NUM_PIECES, ADV_COLS),

    KeyedUpgradeInput: {},

    special_budget: 0,

    bound_budgets: createInputColumn(InputType.Int),
    leftover_price: createInputColumn(InputType.Float), // implicit 0 leftover here

    special_state: [],
    tier: 0,
    min_resolution: 1,
    num_threads: 1,
    metric_type: 1,

    cumulative_graph: true,
}
