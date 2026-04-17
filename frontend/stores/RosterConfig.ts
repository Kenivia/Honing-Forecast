import { ALL_LABELS, DEFAULT_TIER, STORAGE_KEY } from "@/Utils/Constants"
import { debounce, format_char_name } from "@/Utils/Helpers"
import { create_input_column, validate_input_column, validate_input_column_array } from "@/Utils/InputColumn"
import { InputColumn, InputType, WasmOp } from "@/Utils/Interfaces"
import { defineStore } from "pinia"
import { CharProfile, create_default_char_profile, load_char_profiles, recreate_char_profile } from "./CharacterProfile"
import { build_payload } from "@/WasmInterface/PayloadBuilder"
import { get_valid_status_grid } from "@/Utils/StatusGrid"
import { grids_to_keyed } from "@/Utils/KeyedUpgrades"

export interface RosterConfig {
    mats_prices: InputColumn[][] // mats_prices[tier].data[row] = "123"
    roster_mats_owned: InputColumn[][] // Same as in char profile, the tier distinction is because there's different number of mats (rows) for each tier
    tradable_mats_owned: InputColumn[][]
    active_roster: number

    tier: number
    cumulative_graph: boolean
    selected_shard_bag_size: number
    region: string
    effective_serca_price: number[] // This is the one that's actually used (instead of mats_prices) for serca mats in build_material_info
    latest_market_data: Record<string, [number, any]> // [timestamp, raw_response_data]

    profiles: CharProfile[][]
    active_profile_indices: number[]
}
export const useRosterStore = defineStore("roster", {
    state: () => ({
        roster_config: DEFAULT_ROSTER_CONFIG,
    }),
    getters: {
        active_mats_prices: (state): InputColumn[] => state.roster_config.mats_prices[state.roster_config.active_roster],
        active_roster_mats_owned: (state): InputColumn[] => state.roster_config.roster_mats_owned[state.roster_config.active_roster],
        active_tradable_mats_owned: (state): InputColumn[] => state.roster_config.tradable_mats_owned[state.roster_config.active_roster],

        this_roster_profiles: (state): CharProfile[] => {
            return state.roster_config.profiles[state.roster_config.active_profile_indices[state.roster_config.active_roster]]
        },
        active_profile: (state): CharProfile => {
            return state.roster_config.profiles[state.roster_config.active_profile_indices[state.roster_config.active_roster]][
                state.roster_config.active_profile_indices[state.roster_config.active_roster]
            ]
        },
        active_profile_index: (state): number => {
            return state.roster_config.active_profile_indices[state.roster_config.active_roster]
        },
    },
    actions: {
        init() {
            this.roster_config = load_roster_config()
        },
        switchProfile(id: number) {
            this.active_profile_indices[this.roster_config.active_roster] = id
        },
        addProfile(profile: CharProfile) {
            this.this_roster_profiles.push(profile)
        },
        updateActiveProfile(updates: Partial<CharProfile>) {
            Object.assign(this.profiles[this.active_profile_index], updates)
        },

        resetActiveProfile() {
            this.profiles[this.active_profile_index] = create_default_char_profile()
            this.profiles[this.active_profile_index].char_name = format_char_name(
                this.profiles[this.active_profile_index].char_name,
                this.active_profile_index,
                this.profiles,
            )
            this.profiles[this.active_profile_index].optimizer_worker_bundle.start(WasmOp.Parser, build_payload(WasmOp.Parser))
        },
    },
})

export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
    mats_prices: [ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels))], // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
    roster_mats_owned: [ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels))],
    tradable_mats_owned: [ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels))],
    active_roster: 0,
    tier: DEFAULT_TIER,
    cumulative_graph: true,
    selected_shard_bag_size: 3000,
    region: "NAE",
    effective_serca_price: ALL_LABELS[1].map(() => 0),
    latest_market_data: {},

    profiles: [[create_default_char_profile()]],
    active_profile_indices: [0],
}

export function load_roster_config(): RosterConfig {
    const raw = localStorage.getItem(STORAGE_KEY + "_roster")

    let out = (() => {
        try {
            return JSON.parse(raw) ?? DEFAULT_ROSTER_CONFIG
        } catch {
            return DEFAULT_ROSTER_CONFIG
        }
    })()

    if (!Array.isArray(out.mats_prices[0])) {
        out.mats_prices = [out.mats_prices]
        out.roster_mats_owned = [out.roster_mats_owned]
        out.tradable_mats_owned = [out.tradable_mats_owned]
    }

    if (!Array.isArray(out.profiles[0])) {
        out.profiles = [out.profiles]
    }
    for (let i = 0; i < out.mats_prices.length; i++) {
        validate_input_column_array(out.mats_prices[i], DEFAULT_ROSTER_CONFIG.mats_prices[0])
        validate_input_column_array(out.roster_mats_owned[i], DEFAULT_ROSTER_CONFIG.roster_mats_owned[0])
        validate_input_column_array(out.tradable_mats_owned[i], DEFAULT_ROSTER_CONFIG.tradable_mats_owned[0])

        let default_profile = create_default_char_profile()
        for (let j = 0; j < out.profiles[i].length; j++) {
            let this_profile = out.profiles[i][j]
            let this_parsed: CharProfile = { ...create_default_char_profile(), ...this_profile }

            this_parsed.char_name = format_char_name(this_parsed.char_name, i, this_profile.slice(0, i))
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

            out.profiles[i][j] = recreate_char_profile(this_parsed)
            // console.log(parsed.profiles[i], parsed.profiles[i].tier)
        }
    }

    return { ...DEFAULT_ROSTER_CONFIG, ...out }
}
export function write_roster_config(state) {
    localStorage.setItem(STORAGE_KEY + "_roster", JSON.stringify(state.roster_config))
}

export const debounced_write_roster_config = debounce(write_roster_config, 500)
