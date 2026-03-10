import { defineStore } from "pinia"
import { InputsColumn, StateBundle, StatusGrid } from "@/Utils/Interfaces.ts"
import { ref } from "vue"

export const useProfilesStore = defineStore("profiles", {
    state: () => ({
        profiles: [],
        activeProfileId: null,
    }),
    getters: {
        activeProfile: (state) => state.profiles.find((p) => p.id === state.activeProfileId) ?? null,
    },
    actions: {
        switchProfile(id: string) {
            this.activeProfileId = id
        },
        addProfile(profile: CharProfile) {
            this.profiles.push(profile)
        },
    },
})

export interface CharProfile {
    state_bundle?: StateBundle
    normal_grid: StatusGrid
    adv_grid: StatusGrid

    bound_mats_owned: InputsColumn
    mats_leftover: InputsColumn

    bound_weap_juice_owned: InputsColumn
    weap_juice_leftover: InputsColumn

    bound_armor_juice_owned: InputsColumn
    armor_juice_leftover: InputsColumn
}
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
