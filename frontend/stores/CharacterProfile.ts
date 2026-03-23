import { defineStore, storeToRefs } from "pinia"
import {
    create_input_column,
    createStatusGrid,
    validate_input_column,
    InputColumn,
    InputType,
    KeyedUpgrades,
    StatusGrid,
    get_valid_status_grid,
    grids_to_keyed,
    validate_input_column_array,
    is_enum,
} from "@/Utils/Interfaces"
import { createWorkerBundle } from "@/WasmInterface/worker_setup"
import { ADV_COLS, ALL_LABELS, NORMAL_COLS, NUM_PIECES, SPECIAL_LEAP_LABEL, STORAGE_KEY } from "@/Utils/Constants"
import { WasmOp } from "@/WasmInterface/js_to_wasm"
import { debounce, formatCharName } from "@/Utils/Helpers"
import { build_payload } from "@/WasmInterface/payload"
import { useRosterStore } from "./RosterConfig"
import { parse } from "path"
import { toRaw } from "vue"

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
        },

        updateActiveProfile(updates: Partial<CharProfile>) {
            Object.assign(this.profiles[this.active_profile_index], updates)
        },

        resetActiveProfile() {
            const { roster_config } = storeToRefs(useRosterStore())
            this.profiles[this.active_profile_index] = create_default_char_profile()
            this.profiles[this.active_profile_index].char_name = formatCharName(
                this.profiles[this.active_profile_index].char_name,
                this.active_profile_index,
                this.profiles,
            )
            this.profiles[this.active_profile_index].optimizer_worker_bundle.start(
                WasmOp.Parser,
                build_payload(WasmOp.Parser, this.profiles[this.active_profile_index], roster_config.value),
            )
        },
    },
})

export interface CharProfile {
    optimizer_treatment_plan: TreatmentPlan
    histogram_treatment_plan: TreatmentPlan
    express_event: boolean
    char_name: string

    evaluation_worker_bundle: any
    optimizer_worker_bundle: any
    histogram_worker_bundle: any
    normal_grid: StatusGrid
    adv_grid: StatusGrid

    keyed_upgrades: KeyedUpgrades // see Interface for the definition of these

    special_budget: InputColumn // just a 1 cell column

    bound_budgets: InputColumn[] // bound_budgets[tier].data[row] = "123" (data is string even though we don't directly modify it, should change this to number at some point ig)
    leftover_price: InputColumn[] // The tier distinction is because there's different number of mats (rows) for each tier

    auto_start_optimizer: boolean
    tier: number
    min_resolution: number // currently not used (always 1)
    num_threads: number // currently not used (always 1)
    metric_type: number // currently not used (always 1)
    material_re_render_trigger: boolean // This is here to trigger an update in the special cell in MaterialDist from the change in the confirmation popup in InstructionRow
}

export function load_char_profiles(): { profiles: CharProfile[]; active_profile_index: number } {
    const raw = localStorage.getItem(STORAGE_KEY + "_char_profiles")
    if (!raw) return DEFAULT_PROFILES_STATE

    const parsed = JSON.parse(raw)
    const default_profile = create_default_char_profile()
    for (let i = 0; i < parsed.profiles.length; i++) {
        let this_parsed: CharProfile = { ...create_default_char_profile(), ...parsed.profiles[i] }

        this_parsed.char_name = formatCharName(this_parsed.char_name, i, parsed.profiles.slice(0, i))
        validate_input_column_array(this_parsed.bound_budgets, default_profile.bound_budgets)
        validate_input_column_array(this_parsed.leftover_price, default_profile.leftover_price)
        validate_input_column(this_parsed.special_budget, default_profile.special_budget)

        this_parsed.normal_grid = get_valid_status_grid(this_parsed.normal_grid, default_profile.normal_grid)
        this_parsed.adv_grid = get_valid_status_grid(this_parsed.adv_grid, default_profile.adv_grid)

        this_parsed.keyed_upgrades = grids_to_keyed(this_parsed.normal_grid, this_parsed.adv_grid, this_parsed.keyed_upgrades, this_parsed.tier)

        this_parsed.tier = this_parsed.tier === 0 || this_parsed.tier === 1 ? this_parsed.tier : 0
        this_parsed.material_re_render_trigger = true
        this_parsed.min_resolution = default_profile.min_resolution
        this_parsed.num_threads = default_profile.num_threads
        this_parsed.metric_type = default_profile.metric_type

        let this_profile = recreate_char_profile(this_parsed)
        parsed.profiles[i] = {
            ...parsed.profiles[i],
            ...this_profile,
        }
        // console.log(parsed.profiles[i], parsed.profiles[i].tier)
    }

    return { ...DEFAULT_PROFILES_STATE, ...parsed }
}

export const debounced_write_char_profiles = debounce(write_char_profiles, 500)

export function write_char_profiles(state) {
    let copy = JSON.parse(JSON.stringify(state))
    for (let i = 0; i < copy.profiles.length; i++) {
        delete copy.profiles[i].evaluation_worker_bundle
        delete copy.profiles[i].optimizer_worker_bundle
        delete copy.profiles[i].histogram_worker_bundle
    }
    // console.log("writing ", copy)
    localStorage.setItem(STORAGE_KEY + "_char_profiles", JSON.stringify(copy))
}
const DEFAULT_PROFILES_STATE = {
    profiles: [create_default_char_profile()],
    active_profile_index: 0,
}

export enum TreatmentPlan {
    TreatRosterAsTradable, // rat alt, treat roster as if we could've sold them
    TreatRosterAsBound, // alt, treat char & roster bound as 0 if there's any leftover, taxed market price if any tradable leftover
    TreatTradableAsBound, // main, treat everything as 0 if any leftover
}

export function create_default_char_profile(): CharProfile {
    return {
        optimizer_treatment_plan: TreatmentPlan.TreatRosterAsBound,
        histogram_treatment_plan: TreatmentPlan.TreatRosterAsTradable,
        express_event: false,
        char_name: "NewChar",

        auto_start_optimizer: true,
        evaluation_worker_bundle: createWorkerBundle(),
        optimizer_worker_bundle: createWorkerBundle(),
        histogram_worker_bundle: createWorkerBundle(),

        normal_grid: createStatusGrid(NUM_PIECES, NORMAL_COLS),
        adv_grid: createStatusGrid(NUM_PIECES, ADV_COLS),

        keyed_upgrades: {},
        special_budget: create_input_column(InputType.Int, [SPECIAL_LEAP_LABEL], ["0"], [33333]),

        bound_budgets: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)),
        leftover_price: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)), // implicit 0 leftover here, currently UI does not allow changing this

        tier: 0,
        min_resolution: 1,
        num_threads: 1,
        metric_type: 1,
        material_re_render_trigger: true,
    }
}

// Worker bundles are not writable to string(and prolly shouldnt anyway), we re-make them on load
export function recreate_char_profile(parsed): CharProfile {
    return {
        ...parsed,
        evaluation_worker_bundle: createWorkerBundle(),
        optimizer_worker_bundle: createWorkerBundle(),
        histogram_worker_bundle: createWorkerBundle(),
    }
}
