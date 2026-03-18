<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router"
import router from "./router"
import { iconPath } from "./Utils/Helpers"
import { debounced_write_char_profiles, useProfilesStore } from "./stores/CharacterProfile"
import { debounced_write_roster_config, useRosterStore } from "./stores/RosterConfig"
import { CharProfile } from "./stores/CharacterProfile"
import { storeToRefs } from "pinia"
import { computed, onMounted, onUnmounted, toRaw, watchEffect } from "vue"
import { ref } from "vue"
import { useMediaIsNarrow } from "./Utils/WindowSize"

const roster_store = useRosterStore()
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

const { isNarrow } = useMediaIsNarrow()
</script>

<template>
    <header>
        <div class="wrapper">
            <nav class="flex justify-between mb-10 pt-10">
                <div class="hf-brand">
                    <div class="hf-brand-icon">
                        <img :src="iconPath('Forecast Icon')" alt="Forecast icon" style="width: 34px; height: 34px" />
                    </div>
                    <div v-if="!isNarrow">
                        <h1 class="hf-title">Honing Forecast</h1>
                    </div>
                </div>
                <div class="hf-page-header-row">
                    <router-link to="/">
                        <div class="hf-header-button">Roster & market setup</div>
                    </router-link>
                    <div v-for="(profile, index) in all_profiles">
                        <router-link :to="'/' + profile.char_name" @click="profile_store.active_profile_index = index">
                            <div class="hf-header-button">{{ profile.char_name }}</div>
                        </router-link>
                    </div>
                </div>
                <div class="hf-header-links">
                    <a href="https://github.com/Kenivia/Honing-Forecast">GitHub</a>
                </div>
            </nav>
        </div>
    </header>

    <main>
        <RouterView />
    </main>
</template>

<style>
.hf-header-button {
    margin: 12px;
    color: var(--hf-header-button);
    border-radius: 6px;
}
.hf-header-button:hover {
    color: var(--hf-header-button-hovered);
}
.hf-header-button {
    padding: 12px;
}
.hf-page-header-row {
    display: flex;
    flex-direction: row;
    width: 100%;
    gap: 0px;
    padding: 4px;
    align-items: center;
}
.hf-brand {
    display: flex;
    align-items: center;
    gap: 0px;
    /* border: 1px solid var(--hf-gold-dim); */
    /* border-radius: var(--hf-radius); */
    /* background: linear-gradient(135deg, var(--hf-bg-card), var(--hf-bg-panel));
    box-shadow: none; */
    padding: 4px;
    padding-right: 8px;
}

.hf-brand-icon {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
}

.hf-title {
    margin: 0;
    font-family: var(--hf-font-display);
    color: var(--hf-text-bright);
    letter-spacing: 0.03em;
    font-size: 20px;
    line-height: 1;
    font-weight: 700;
    width: min-content;
}
</style>
