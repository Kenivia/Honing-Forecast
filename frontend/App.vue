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
import { fetch_callback, useTimedFetch } from "./Utils/MarketDataFetcher"

const roster_store = useRosterStore()
roster_store.init()
const profile_store = useProfilesStore()
profile_store.init()
// const { all_profiles } = storeToRefs(profile_store)
const { start_fetch } = useTimedFetch(fetch_callback)
start_fetch(roster_store.roster_config.region)

profile_store.$subscribe((_mutation, state) => {
    debounced_write_char_profiles(state)
})
roster_store.$subscribe((_mutation, state) => {
    debounced_write_roster_config(state)
})

const { isNarrow } = useMediaIsNarrow()
</script>

<template>
    <div style="display: flex; flex-direction: column; min-height: 100vh; justify-items: flex-end">
        <header>
            <nav
                style="
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    border-bottom: 1px solid var(--separator-color);
                    padding: 0px;
                    margin: 0px;
                "
            >
                <div class="hf-brand">
                    <router-link to="/">
                        <div class="hf-brand-icon">
                            <img :src="iconPath('Forecast Icon')" alt="Forecast icon" style="width: 34px; height: 34px" /></div
                    ></router-link>
                    <div v-if="!isNarrow">
                        <h1 class="hf-title">Honing Forecast</h1>
                    </div>
                </div>
                <div class="hf-page-header-row">
                    <router-link to="/roster-setup">
                        <div class="hf-header-button">Roster setup</div>
                    </router-link>
                    <router-link to="/market-mats">
                        <div class="hf-header-button">Market & Mats</div>
                    </router-link>
                    <div style="width: 10px"></div>
                    <RouterLink
                        v-for="(profile, index) in profile_store.profiles"
                        :key="index"
                        :to="{ name: 'char', params: { characterName: profile.char_name } }"
                        class="hf-header-button"
                        :class="{ selected: index == profile_store.active_profile_index }"
                    >
                        {{ profile.char_name }}
                    </RouterLink>
                </div>
            </nav>
        </header>
        <main style="flex: 1">
            <RouterView />
        </main>
        <footer style="display: flex; flex-direction: row-reverse; align-items: center">
            <a v-if="!isNarrow" href="https://ko-fi.com/kenivia" class="hf-header-links">
                <img src="/Icons/kofi.png" alt="Ko-fi" />
                <span>Donate</span>
            </a>
            <a v-if="!isNarrow" href="https://discord.gg/KWDpQyvgzc" class="hf-header-links">
                <img src="/Icons/Discord.png" alt="Discord" />
                <span>Discord</span>
            </a>
            <a v-if="!isNarrow" href="https://github.com/Kenivia/Honing-Forecast" class="hf-header-links">
                <img src="/Icons/GitHub.png" alt="GitHub" />
                <span>GitHub</span>
            </a>
            <span style="color: var(--text-very-muted)">Made with love by Kenivia with help from many awesome people.</span>
        </footer>
    </div>
</template>

<style>
.hf-header-button {
    color: var(--hf-header-button);
    border-radius: 6px;
    border: 1px solid var(--separator-color);
    background-color: var(--hf-bg-header);
    margin: 0px 2px 0px 2px;
    text-wrap-mode: nowrap;
}
.hf-header-button:hover {
    color: var(--accent-hover);
    background-color: var(--hf-bg-hover);
}
.hf-header-button.selected {
    color: var(--accent-hover);
    background-color: var(--hf-bg-hover);
}
.hf-header-button {
    padding: 12px;
}
.hf-page-header-row {
    display: flex;
    flex-direction: row;
    width: 100%;
    gap: 0px;
    /* padding: 4px; */
    align-items: center;
    overflow-x: auto;
    overflow-y: hidden;
}
.hf-brand {
    display: flex;
    align-items: center;
    gap: 0px;
    /* border: 1px solid var(--hf-gold-dim); */
    /* border-radius: var(--hf-radius); */
    /* background: linear-gradient(135deg, var(--hf-bg-card), var(--hf-bg-panel));
    box-shadow: none; */
    /* padding: 4px; */
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
