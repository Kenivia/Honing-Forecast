import { defineStore } from "pinia"
import { InputsColumn, StateBundle, StatusGrid } from "@/Utils/Interfaces"

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
    optimizer_worker: any
    evaluate_worker: any

    has_run_optimizer: boolean
    char_name: string

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
