<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router"
import { iconPath } from "./Utils/Helpers"
import { debounced_write_char_profiles, useProfilesStore } from "./Stores/CharacterProfile"
import { debounced_write_roster_config, useRosterStore } from "./Stores/RosterConfig"

import { useMediaIsNarrow } from "./Utils/WindowSize"
import { fetch_callback, useTimedFetch } from "./Utils/MarketDataFetcher"

const roster_store = useRosterStore()
roster_store.init()
const profile_store = useProfilesStore()
profile_store.init()

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
    <div class="hf-app-shell">
        <header>
            <nav class="hf-top-nav">
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
                    <div class="hf-header-spacer" />
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
        <main class="hf-main-slot">
            <RouterView />
        </main>
        <footer class="hf-footer-bar">
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
            <span class="hf-footer-note">Made with love by Kenivia with help from many awesome people.</span>
        </footer>
    </div>
</template>

<style>
.hf-app-shell {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
}

.hf-top-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
    padding: 0;
    margin: 0;
    gap: 8px;
}

.hf-main-slot {
    flex: 1;
    min-width: 0;
}

.hf-header-button {
    display: inline-flex;
    align-items: center;
    color: var(--hf-header-button);
    border-radius: 6px;
    border: 1px solid var(--separator-color);
    background-color: var(--hf-bg-header);
    margin: 0 2px;
    text-wrap-mode: nowrap;
    padding: 12px;
}

.hf-header-button:hover {
    color: var(--accent-hover);
    background-color: var(--hf-bg-hover);
}

.hf-header-button.selected {
    color: var(--accent-hover);
    background-color: var(--hf-bg-hover);
}

.hf-header-spacer {
    width: 10px;
    min-width: 10px;
}

.hf-page-header-row {
    display: flex;
    flex-direction: row;
    width: 100%;
    gap: 0;
    align-items: center;
    overflow-x: auto;
    overflow-y: hidden;
    white-space: nowrap;
    scrollbar-width: thin;
}

.hf-brand {
    display: flex;
    align-items: center;
    gap: 0;
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

.hf-footer-bar {
    display: flex;
    flex-direction: row-reverse;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding: 8px 10px;
}

.hf-footer-note {
    color: var(--text-very-muted);
    font-size: 12px;
    text-align: center;
}

@media (max-width: 900px) {
    .hf-top-nav {
        flex-direction: column;
        align-items: stretch;
        padding-bottom: 8px;
    }

    .hf-brand {
        padding-right: 0;
        padding-left: 8px;
    }

    .hf-page-header-row {
        padding: 0 8px;
        gap: 4px;
    }

    .hf-header-button {
        padding: 10px 9px;
        font-size: 13px;
    }

    .hf-footer-bar {
        justify-content: center;
    }

    .hf-footer-note {
        width: 100%;
    }
}
</style>
