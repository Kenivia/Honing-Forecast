<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router"
import router from "./router"
import { iconPath } from "./Utils/Helpers"
import { useProfilesStore } from "./stores/CharacterProfile"
import { uesRosterStore } from "./stores/RosterConfig"
import { CharProfile } from "./stores/CharacterProfile"
import { storeToRefs } from "pinia"
import { toRaw } from "vue"

const profile_store = useProfilesStore()
profile_store.init()
const { all_profiles } = storeToRefs(profile_store)

const roster_store = uesRosterStore()
roster_store.init()
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
        <router-link to="/"> </router-link>
        <div v-for="(profile, index) in all_profiles">
            <router-link :to="'/' + profile.char_name" @click="profile_store.active_profile_index = index"> {{ profile.char_name }} </router-link>
        </div>
    </div>
    <main class="bg-white">
        <RouterView />
    </main>
</template>
