import { InputColumn } from "@/Utils/Interfaces"
import { defineStore } from "pinia"

export interface RosterConfig {
    mats_prices: InputColumn
    weap_juice_prices: InputColumn
    armor_juice_prices: InputColumn

    roster_mats_owned: InputColumn
    roster_weap_juice_owned: InputColumn
    roster_armor_juice_owned: InputColumn
    // roster bound leftover will inherit the character's bound leftover values

    tradable_mats_owned: InputColumn
    tradable_weap_juice_owned: InputColumn
    tradable_armor_juice_owned: InputColumn

    // these are just price with tax applied
    tradable_mats_price: InputColumn
    tradable_weap_juice_price: InputColumn
    tradable_armor_juice_price: InputColumn
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
