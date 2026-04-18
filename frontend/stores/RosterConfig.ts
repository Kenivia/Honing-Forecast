import { ALL_LABELS, DEFAULT_TIER, STORAGE_KEY } from "@/Utils/Constants"
import { debounce, format_char_name } from "@/Utils/Helpers"
import { create_input_column, validate_input_column, validate_input_column_array } from "@/Utils/InputColumn"
import { InputColumn, InputType } from "@/Utils/Interfaces"
import { defineStore } from "pinia"
import { CharProfile, create_default_char_profile, recreate_char_profile } from "./CharacterProfile"
import { get_valid_status_grid } from "@/Utils/StatusGrid"
import { grids_to_keyed } from "@/Utils/KeyedUpgrades"

export interface RosterConfig {
    mats_prices: InputColumn[] // mats_prices[tier].data[row] = "123"
    roster_mats_owned: Record<number, InputColumn[]> // Same as in char profile, the tier distinction is because there's different number of mats (rows) for each tier
    tradable_mats_owned: Record<number, InputColumn[]>

    tier: number
    cumulative_graph: boolean
    selected_shard_bag_size: number
    region: string
    effective_serca_price: number[] // This is the one that's actually used (instead of mats_prices) for serca mats in build_material_info
    latest_market_data: Record<string, [number, any]> // [timestamp, raw_response_data]

    profiles: CharProfile[]
    active_profile_index: number
}
export const useRosterStore = defineStore("roster", {
    state: () => ({
        roster_config: DEFAULT_ROSTER_CONFIG,
    }),
    getters: {
        active_roster_mats_owned: (state): InputColumn[] => {
            let active_profile = state.roster_config.profiles[state.roster_config.active_profile_index]
            return state.roster_config.roster_mats_owned[active_profile.roster_id]
        },
        active_tradable_mats_owned: (state): InputColumn[] => {
            let active_profile = state.roster_config.profiles[state.roster_config.active_profile_index]
            // console.log(state.roster_config.active_profile_index, active_profile, state.roster_config.tradable_mats_owned)
            return state.roster_config.tradable_mats_owned[active_profile.roster_id]
        },

        active_profile: (state): CharProfile => {
            // console.log(state.roster_config.profiles)
            return state.roster_config.profiles[state.roster_config.active_profile_index]
        },

        all_profiles: (state): CharProfile[] => {
            return state.roster_config.profiles
        },
        roster_ids: (state): number[] => {
            return [...new Set(state.roster_config.profiles.map((x) => x.roster_id))].sort((a, b) => a - b)
        },
    },
    actions: {
        init() {
            this.roster_config = load_roster_config()
        },
        switchProfile(id: number) {
            this.roster_config.active_profile_index = id
        },
        addProfile(profile: CharProfile) {
            this.roster_config.profiles.push(profile)
        },

        resetActiveProfile() {
            this.roster_config.profiles[this.active_profile_index] = create_default_char_profile()
            this.roster_config.profiles[this.active_profile_index].char_name = format_char_name(
                this.roster_config.profiles[this.active_profile_index].char_name,
                this.roster_config.active_profile_index,
            )
        },

        get_this_roster_profile(roster_index): CharProfile[] {
            return this.roster_config.profiles.filter((x) => x.roster_number == roster_index)
        },
    },
})

export function create_default_owned_input_column(): InputColumn[] {
    return ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels))
}
export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
    mats_prices: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)), // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
    roster_mats_owned: { 0: create_default_owned_input_column() },
    tradable_mats_owned: { 0: create_default_owned_input_column() },
    tier: DEFAULT_TIER,
    cumulative_graph: true,
    selected_shard_bag_size: 3000,
    region: "NAE",
    effective_serca_price: ALL_LABELS[1].map(() => 0),
    latest_market_data: {},

    profiles: [create_default_char_profile()],
    active_profile_index: 0,
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

    const old_char_profiles = localStorage.getItem("HF_UI_STATE_V3_char_profiles")
    if (old_char_profiles !== null) {
        try {
            let parsed = JSON.parse(old_char_profiles)
            out.profiles = parsed.profiles
        } catch {
            out.profiles = [create_default_char_profile()]
        }

        localStorage.removeItem("HF_UI_STATE_V3_char_profiles")
    }
    const old_roster = localStorage.getItem("HF_UI_STATE_V3_roster")
    if (old_roster !== null) {
        try {
            let parsed = JSON.parse(old_roster)
            out.roster_mats_owned = { 0: parsed.roster_mats_owned }
            out.tradable_mats_owned = { 0: parsed.tradable_mats_owned }
        } catch {
            out.roster_mats_owned = { 0: create_default_owned_input_column() }
            out.tradable_mats_owned = { 0: create_default_owned_input_column() }
        }

        localStorage.removeItem("HF_UI_STATE_V3_roster")
    }
    // console.log(out.roster_mats_owned)
    validate_input_column_array(out.mats_prices, DEFAULT_ROSTER_CONFIG.mats_prices)
    for (const key in out.roster_mats_owned) {
        validate_input_column_array(out.roster_mats_owned[key], DEFAULT_ROSTER_CONFIG.roster_mats_owned[0])
        validate_input_column_array(out.tradable_mats_owned[key], DEFAULT_ROSTER_CONFIG.tradable_mats_owned[0])
    }
    let default_profile = create_default_char_profile()

    for (let i = 0; i < out.profiles.length; i++) {
        let this_profile = out.profiles[i]
        let this_parsed: CharProfile = { ...create_default_char_profile(), ...this_profile }

        this_parsed.char_name = format_char_name(this_parsed.char_name, i, out.profiles.slice(0, i))
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
        // console.log(this_parsed.roster_id, out.roster_mats_owned)
        if (this_parsed.roster_id === null || this_parsed.roster_id === undefined || !out.roster_mats_owned.hasOwnProperty(this_parsed.roster_id)) {
            this_parsed.roster_id = out.roster_mats_owned.keys()[0]
        }

        out.profiles[i] = recreate_char_profile(this_parsed)
        // console.log(parsed.profiles[i], parsed.profiles[i].tier)
    }
    out.active_profile_index = !out.active_profile_index ? 0 : Math.max(0, Math.min(out.profiles.length - 1, out.active_profile_index))
    return { ...DEFAULT_ROSTER_CONFIG, ...out }
}
export function write_roster_config(state) {
    localStorage.setItem(STORAGE_KEY + "_roster", JSON.stringify(state.roster_config))
}

export const debounced_write_roster_config = debounce(write_roster_config, 500)
