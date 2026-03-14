import { defineStore } from "pinia"
import {
    AdvProgress,
    AdvProgressGrid,
    BoolGrid,
    create_input_column,
    createStatusGrid,
    HistogramOutputs,
    InputColumn,
    InputType,
    keyed_upgrades,
    makeDefaultBoolGrid,
    makeDefaultNumGrid,
    NumGrid,
    StateBundle,
    StateGrid,
    StatusGrid,
    UpgradeStatus,
} from "@/Utils/Interfaces"
import { createWorkerBundle } from "@/WasmInterface/worker_setup"
import { ADV_COLS, ALL_LABELS, JUICE_LABELS, MATS_LABELS, NORMAL_COLS, NUM_PIECES, SPECIAL_LEAP_LABEL, STORAGE_KEY } from "@/Utils/Constants"
import { Ref, ref } from "vue"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { debounce } from "@/Utils/Helpers"

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
            const loaded = load_char_profiles()
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
            this.profiles[this.active_profile_index] = create_default_char_profile()
            this.profiles[this.active_profile_index].optimizer_worker_bundle.start(WasmOp.Parser)
        },
    },
})

export const debounced_write_char_profiles = debounce(write_char_profiles, 500)
export function load_char_profiles(): { profiles: CharProfile[]; active_profile_index: number } {
    const raw = localStorage.getItem(STORAGE_KEY + "_char_profiles")
    if (!raw) return DEFAULT_PROFILES_STATE

    const parsed = JSON.parse(raw)
    for (let i = 0; i < parsed.profiles.length; i++) {
        let this_parsed = parsed.profiles[i]
        let this_profile = recreate_char_profile(this_parsed)
        parsed.profiles[i] = {
            ...parsed.profiles[i],
            ...this_profile,
        }
        console.log(parsed.profiles[i])
    }

    return parsed
}
export function write_char_profiles(profiles, active_profile_index: number) {
    const serializable = {
        active_profile_index,
        profiles: profiles,
    }

    localStorage.setItem(STORAGE_KEY + "_char_profiles", JSON.stringify(serializable))
}
const DEFAULT_PROFILES_STATE = {
    profiles: new Array(create_default_char_profile()),
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
    evaluation_worker_bundle: any
    optimizer_worker_bundle: any
    histogram_worker_bundle: any
    normal_grid: StatusGrid
    adv_grid: StatusGrid

    keyed_upgrades: keyed_upgrades

    special_budget: InputColumn

    bound_budgets: InputColumn
    leftover_price: InputColumn

    special_state: number[]

    tier: number
    min_resolution: number
    num_threads: number
    metric_type: number
}

export function create_default_char_profile(): CharProfile {
    return {
        treatment_plan: TreatmentPlan.TreatRosterAsBound,

        express_event: false,
        char_name: "YourChar",

        auto_start_optimizer: false,
        has_run_optimizer: false,
        evaluation_worker_bundle: createWorkerBundle(),
        optimizer_worker_bundle: createWorkerBundle(),
        histogram_worker_bundle: createWorkerBundle(),

        normal_grid: createStatusGrid(NUM_PIECES, NORMAL_COLS),
        adv_grid: createStatusGrid(NUM_PIECES, ADV_COLS),

        keyed_upgrades: {},

        special_budget: create_input_column(InputType.Int, [SPECIAL_LEAP_LABEL]),

        bound_budgets: create_input_column(InputType.Int, ALL_LABELS),
        leftover_price: create_input_column(InputType.Int, ALL_LABELS), // implicit 0 leftover here

        special_state: [],
        tier: 0,
        min_resolution: 1,
        num_threads: 1,
        metric_type: 1,
    }
}

export function recreate_char_profile(parsed): CharProfile {
    return {
        ...parsed,
        evaluation_worker_bundle: createWorkerBundle(),
        optimizer_worker_bundle: createWorkerBundle(),
        histogram_worker_bundle: createWorkerBundle(),
    }
}
