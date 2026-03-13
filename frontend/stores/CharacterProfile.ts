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
import { Ref, ref } from "vue"
import { WasmOp } from "@/WasmInterface/js_to_wasm"

export const useProfilesStore = defineStore("profiles", {
    state: () => DEFAULT_PROFILES_STATE,
    getters: {
        active_profile: (state) => {
            return state.profiles[state.active_profile_index]
        },
        all_profiles: (state) => {
            return state.profiles
        },
    },
    actions: {
        switchProfile(id: number) {
            this.active_profile_index = id
        },
        addProfile(profile: CharProfile) {
            this.profiles.push(profile)
        },
        init() {
            const loaded = loadCharProfiles()
            this.profiles = loaded.profiles
            this.active_profile_index = loaded.active_profile_index

            for (let index = 0; index < this.profiles.length; index++) {
                if (this.profiles[index].state_bundle === null) {
                    this.profiles[index].optimizer_worker_bundle.start(WasmOp.Parser)
                }
            }
        },

        updateActiveProfile(updates: Partial<CharProfile>) {
            Object.assign(this.profiles[this.active_profile_index], updates)
        },

        resetActiveProfile() {
            this.profiles[this.active_profile_index] = createDefaultCharProfile()
            this.profiles[this.active_profile_index].optimizer_worker_bundle.start(WasmOp.Parser)
        },
    },
})
export function loadCharProfiles(): { profiles: CharProfile[]; active_profile_index: number } {
    const raw = localStorage.getItem(STORAGE_KEY + "_char_profiles")
    if (!raw) return DEFAULT_PROFILES_STATE

    const parsed = JSON.parse(raw)
    for (const profile of parsed.profiles) {
        profile.state_bundle = ref(parsed.state_bundle_raw)
        profile.optimizer_worker_bundle = createWorkerBundle(profile.state_bundle)
        profile.histogram_worker_bundle = createWorkerBundle(profile.state_bundle)
    }

    return parsed
}
export function writeCharProfiles() {
    //WIP
}
const DEFAULT_PROFILES_STATE = {
    profiles: new Array(createDefaultCharProfile()),
    active_profile_index: 0,
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
    histogram_worker_bundle: any
    state_bundle: Ref<null | StateBundle>

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

export function createDefaultCharProfile(): CharProfile {
    let state_bundle: Ref<null | StateBundle> = ref(null)
    return {
        treatment_plan: TreatmentPlan.TreatRosterAsBound,

        express_event: false,
        char_name: "YourChar",

        auto_start_optimizer: false,
        has_run_optimizer: false,
        optimizer_worker_bundle: createWorkerBundle(state_bundle),
        histogram_worker_bundle: createWorkerBundle(state_bundle),
        state_bundle,
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
}
