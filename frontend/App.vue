<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router"
import router from "./router"
import { iconPath } from "./Utils/Helpers"
import { debounced_write_char_profiles, useProfilesStore } from "./stores/CharacterProfile"
import { debounced_write_roster_config, uesRosterStore } from "./stores/RosterConfig"
import { CharProfile } from "./stores/CharacterProfile"
import { storeToRefs } from "pinia"
import { toRaw, watchEffect } from "vue"

const roster_store = uesRosterStore()
roster_store.init()
const profile_store = useProfilesStore()
profile_store.init()
const { all_profiles } = storeToRefs(profile_store)

profile_store.$subscribe((_mutation, state) => {
    debounced_write_char_profiles(state)
})
roster_store.$subscribe((_mutation, state) => {
    debounced_write_roster_config(state)
})
</script>

<template>
    <header>
        <div class="wrapper">
            <nav class="flex justify-between mb-10 pt-10">
                <div class="hf-brand">
                    <div class="hf-brand-icon">
                        <img :src="iconPath('Forecast Icon')" alt="Forecast icon" style="width: 34px; height: 34px" />
                    </div>
                    <div>
                        <h1 class="hf-title">Honing Forecast</h1>
                        <div class="hf-subtitle">Lost Ark Upgrade Planner</div>
                    </div>
                </div>
                <div class="hf-header-links">
                    <a href="https://github.com/Kenivia/Honing-Forecast">GitHub</a>
                </div>
            </nav>
        </div>
    </header>
    <div>
        <router-link to="/"> Roster setup </router-link>
        <div v-for="(profile, index) in all_profiles">
            <router-link :to="'/' + profile.char_name" @click="profile_store.active_profile_index = index"> {{ profile.char_name }} </router-link>
        </div>
    </div>
    <main>
        <RouterView />
    </main>
</template>
