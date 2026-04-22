<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router"
import { get_icon_path } from "./Utils/Helpers"
import { CharProfile } from "./Stores/CharacterProfile"
import { debounced_write_roster_config, useRosterStore } from "./Stores/RosterConfig"
import { useMediaIsNarrow } from "./Utils/WindowSize"
import { fetch_callback, useTimedFetch } from "./Utils/MarketDataFetcher"
import { computed } from "vue"
import { storeToRefs } from "pinia"

const roster_store = useRosterStore()
roster_store.init()

const { all_profiles, roster_ids } = storeToRefs(roster_store)

const { start_fetch } = useTimedFetch(fetch_callback)
start_fetch(roster_store.roster_config.region)

roster_store.$subscribe((_mutation, state) => {
    debounced_write_roster_config(state)
})
const { isNarrow: is500Narrow } = useMediaIsNarrow(500)
const { isNarrow: is600Narrow } = useMediaIsNarrow(600)
const { isNarrow: is800Narrow } = useMediaIsNarrow()

const route = useRoute()
const router = useRouter()

const active_char_name = computed(() => all_profiles.value.find((p) => route.path === "/" + p.char_name)?.char_name ?? "")

function onCharSelect(e: Event) {
    const name = (e.target as HTMLSelectElement).value
    if (name) router.push({ name: "char", params: { characterName: name } })
}
</script>

<template>
    <div class="hf-app-shell">
        <header>
            <!-- <div v-if="!is500Narrow" class="hf-page-header-row" style="width: 100%; justify-content: space-around">
                <span>
                    <span style="font-weight: 600">Happy Serca porg!&nbsp;</span>
                    <a
                        style="text-decoration: underline"
                        href="https://docs.google.com/spreadsheets/d/1UWJ5TCNZ2kIZxXQwR839c-P1rlivkwdW3fOCMDMXZZo/edit?usp=sharing"
                        >Data on this site</a
                    >
                    came from Korean API, if something is different in-game, plz let me know via
                    <a style="text-decoration: underline" href="https://github.com/Kenivia/Honing-Forecast/issues">a GitHub issue</a>
                    or
                    <a style="text-decoration: underline" href="https://discord.gg/KWDpQyvgzc">on Discord</a>!
                </span>
            </div> -->
            <nav class="hf-top-nav">
                <div class="hf-page-header-row" style="width: 100%">
                    <div v-if="!is500Narrow" class="hf-brand">
                        <router-link to="/">
                            <div class="hf-brand-icon">
                                <img :src="get_icon_path('Forecast Icon')" alt="Forecast icon" style="width: 34px; height: 34px" /></div
                        ></router-link>
                        <div v-if="!is800Narrow">
                            <h1 class="hf-title">Honing Forecast</h1>
                        </div>
                    </div>
                    <router-link to="/roster-setup">
                        <div class="hf-header-button" :class="{ selected: route.path == '/roster-setup' }">Roster setup</div>
                    </router-link>
                    <router-link to="/market-mats">
                        <div class="hf-header-button" :class="{ selected: route.path == '/market-mats' }">Market & Mats</div>
                    </router-link>

                    <div class="hf-header-spacer" />

                    <div
                        v-if="!is600Narrow || all_profiles.length <= 1"
                        v-for="roster_id in roster_ids"
                        class="hf-char-row"
                        style="display: flex; flex-direction: row"
                        :key="'roster-${roster_id}'"
                    >
                        <div
                            v-for="[profile, profile_index] in all_profiles
                                .map((x, index): [CharProfile, number] => [x, index])
                                .filter((y) => y[0].roster_id === roster_id)"
                            class="hf-char-row"
                            :key="'profile-${profile_index}'"
                        >
                            <!-- {{ console.log(roster_id, profile.roster_id) }} -->
                            <RouterLink
                                :to="{ name: 'char', params: { characterName: profile.char_name } }"
                                class="hf-header-button"
                                :class="{ selected: route.path == '/' + profile.char_name }"
                            >
                                {{ profile.char_name }}
                            </RouterLink>
                        </div>
                        <span style="width: 16px"></span>
                    </div>

                    <!-- Mobile: dropdown -->
                    <select v-else class="hf-char-select" :value="active_char_name" @change="onCharSelect">
                        <option value="" disabled>Character</option>
                        <option v-for="(profile, index) in all_profiles" :key="index" :value="profile.char_name">
                            {{ profile.char_name }}
                        </option>
                    </select>
                </div>
            </nav>
        </header>
        <main class="hf-main-slot">
            <RouterView />
        </main>
        <footer class="hf-footer-bar">
            <a href="https://ko-fi.com/kenivia" class="hf-header-links">
                <img src="/Icons/kofi.png" alt="Ko-fi" />
                <span>Donate</span>
            </a>
            <a href="https://discord.gg/KWDpQyvgzc" class="hf-header-links">
                <img src="/Icons/Discord.png" alt="Discord" />
                <span>Discord</span>
            </a>
            <a href="https://github.com/Kenivia/Honing-Forecast" class="hf-header-links">
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
    justify-content: flex-start;
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
    width: fit-content;
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
    .hf-footer-bar {
        justify-content: center;
    }

    .hf-footer-note {
        width: 100%;
    }
}
</style>
