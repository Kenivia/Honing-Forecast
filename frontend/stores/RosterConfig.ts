import { ALL_LABELS, DEFAULT_TIER, STORAGE_KEY, TIER_LABELS } from "@/Utils/Constants"
import { debounce } from "@/Utils/Helpers"
import { create_input_column, fill_new_tiers_with_default, InputColumn, InputType } from "@/Utils/Interfaces"
import { defineStore } from "pinia"

export interface RosterConfig {
    mats_prices: InputColumn[]
    roster_mats_owned: InputColumn[] // roster bound leftover will inherit the character's bound leftover values
    tradable_mats_owned: InputColumn[]
    tier: number
    cumulative_graph: boolean
    // tradable_mats_price: InputColumn // these are just price with tax applied
}
export const useRosterStore = defineStore("roster", {
    state: () => ({
        data: null as RosterConfig | null,
    }),
    getters: {
        roster_config: (state): RosterConfig => state.data,
    },
    actions: {
        init() {
            this.data = load_roster_config() // we have to use a data field instead of just doing this = load_roster_config() cos that's how vue works ig
        },
    },
})

export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
    mats_prices: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)), // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
    roster_mats_owned: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)),
    tradable_mats_owned: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)),
    tier: DEFAULT_TIER,
    cumulative_graph: true,
    // tradable_mats_price: createInputColumn(InputType.Float),
}

export function load_roster_config(): RosterConfig {
    const raw = localStorage.getItem(STORAGE_KEY + "_roster")
    if (!raw) return DEFAULT_ROSTER_CONFIG
    let out = JSON.parse(raw)
    fill_new_tiers_with_default(out.mats_prices)
    fill_new_tiers_with_default(out.roster_mats_owned)
    fill_new_tiers_with_default(out.tradable_mats_owned)

    return { ...DEFAULT_ROSTER_CONFIG, ...out }
}
export function write_roster_config(state) {
    localStorage.setItem(STORAGE_KEY + "_roster", JSON.stringify(state.data))
}

export const debounced_write_roster_config = debounce(write_roster_config, 500)
