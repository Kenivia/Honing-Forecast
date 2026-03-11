import { InputsColumn } from "@/Utils/Interfaces"
import { defineStore } from "pinia"

export interface RosterConfig {
    mats_prices: InputsColumn
    weap_juice_prices: InputsColumn
    armor_juice_prices: InputsColumn

    roster_mats_owned: InputsColumn
    roster_weap_juice_owned: InputsColumn
    roster_armor_juice_owned: InputsColumn
    // roster bound leftover will inherit the character's bound leftover values

    tradable_mats_owned: InputsColumn
    tradable_weap_juice_owned: InputsColumn
    tradable_armor_juice_owned: InputsColumn

    // these are just price with tax applied
    tradable_mats_leftover: InputsColumn
    tradable_weap_juice_leftover: InputsColumn
    tradable_armor_juice_leftover: InputsColumn
}
export const uesRosterStore = defineStore("roster", {
    state: () => ({
        roster_config: null,
    }),
})
