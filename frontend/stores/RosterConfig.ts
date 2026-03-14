import { ALL_LABELS, STORAGE_KEY } from "@/Utils/Constants"
import { debounce } from "@/Utils/Helpers"
import { create_input_column, InputColumn, InputType } from "@/Utils/Interfaces"
import { defineStore } from "pinia"

export interface RosterConfig {
    mats_prices: InputColumn
    roster_mats_owned: InputColumn // roster bound leftover will inherit the character's bound leftover values
    tradable_mats_owned: InputColumn
    // tradable_mats_price: InputColumn // these are just price with tax applied
}
export const uesRosterStore = defineStore("roster", {
    state: () => ({
        data: null as RosterConfig | null,
    }),
    getters: {
        roster_config: (state): RosterConfig => state.data,
    },
    actions: {
        init() {
            this.data = load_roster_config()
        },
    },
})

export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
    mats_prices: create_input_column(InputType.Int, ALL_LABELS), // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
    roster_mats_owned: create_input_column(InputType.Int, ALL_LABELS),
    tradable_mats_owned: create_input_column(InputType.Int, ALL_LABELS),
    // tradable_mats_price: createInputColumn(InputType.Float),
}

export function load_roster_config(): RosterConfig {
    const raw = localStorage.getItem(STORAGE_KEY + "_roster")
    if (!raw) return DEFAULT_ROSTER_CONFIG

    return JSON.parse(raw)
}
export function write_roster_config(roster_config: RosterConfig) {
    localStorage.setItem(STORAGE_KEY + "_roster", JSON.stringify(roster_config))
}

export const debounced_write_roster_config = debounce(write_roster_config, 500)
