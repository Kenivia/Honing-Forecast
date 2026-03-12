import { STORAGE_KEY } from "@/Utils/Constants"
import { createInputColumn, InputColumn, InputType } from "@/Utils/Interfaces"
import { defineStore } from "pinia"

export interface RosterConfig {
    mats_prices: InputColumn
    roster_mats_owned: InputColumn // roster bound leftover will inherit the character's bound leftover values
    tradable_mats_owned: InputColumn
    tradable_mats_price: InputColumn // these are just price with tax applied
}
export const uesRosterStore = defineStore("roster", {
    state: () => ({
        roster_config: null,
    }),
    getters: {
        getRoster: (state) => state.roster_config,
    },
    actions: {
        initRoster(roster: RosterConfig) {
            this.roster_config = roster
        },
    },
})

export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
    mats_prices: createInputColumn(InputType.Float),
    roster_mats_owned: createInputColumn(InputType.Int),
    tradable_mats_owned: createInputColumn(InputType.Int),
    tradable_mats_price: createInputColumn(InputType.Float),
}

export function loadRosterConfig(): RosterConfig {
    const raw = localStorage.getItem(STORAGE_KEY + "_roster")
    if (!raw) return DEFAULT_ROSTER_CONFIG

    const parsed = JSON.parse(raw)

    return {
        ...DEFAULT_ROSTER_CONFIG, // can't be bother to do actual checking, just gonna hope that parsed has a correct enough form
        ...parsed,
    }
}
