<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router"
import { get_icon_path } from "./Utils/Helpers"
import { debounced_write_char_profiles, useProfilesStore } from "./Stores/CharacterProfile"
import { debounced_write_roster_config, useRosterStore } from "./Stores/RosterConfig"
import { useMediaIsNarrow } from "./Utils/WindowSize"
import { fetch_callback, useTimedFetch } from "./Utils/MarketDataFetcher"
import { computed, watchEffect } from "vue"
import { ALL_LABELS, SYNCED_LABELS } from "./Utils/Constants"
import { input_column_to_num } from "./Utils/InputColumn"

const roster_store = useRosterStore()
roster_store.init()
const profile_store = useProfilesStore()
profile_store.init()

const { start_fetch } = useTimedFetch(fetch_callback)
start_fetch(roster_store.roster_config.region)

profile_store.$subscribe((_mutation, state) => {
    debounced_write_char_profiles(state) // TODO make this ignore worker bundle changes, idk how to do that tho
})
roster_store.$subscribe((_mutation, state) => {
    debounced_write_roster_config(state)
})
const { isNarrow: is500Narrow } = useMediaIsNarrow(500)
const { isNarrow: is600Narrow } = useMediaIsNarrow(600)
const { isNarrow: is800Narrow } = useMediaIsNarrow()

const route = useRoute()
const router = useRouter()

const active_char_name = computed(() => profile_store.profiles.find((p) => route.path === "/" + p.char_name)?.char_name ?? "")

function onCharSelect(e: Event) {
    const name = (e.target as HTMLSelectElement).value
    if (name) router.push({ name: "char", params: { characterName: name } })
}
watchEffect(() => {
    let t4_price = input_column_to_num(roster_store.roster_config.mats_prices[0])
    let serca_price = input_column_to_num(roster_store.roster_config.mats_prices[1])
    roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map((_, index) => Math.min(t4_price[index] * 5, serca_price[index]))
})
watchEffect(() => {
    // one way sync from T4 to Serca, the ui modifies the T4 copy
    for (let serca_index = 0; serca_index < ALL_LABELS[1].length; serca_index++) {
        if (SYNCED_LABELS.includes(ALL_LABELS[1][serca_index])) {
            let T4_index = ALL_LABELS[0].findIndex((x) => x == ALL_LABELS[1][serca_index].replace("Serca ", ""))
            roster_store.roster_config.mats_prices[1].data[serca_index] = roster_store.roster_config.mats_prices[0].data[T4_index]
            roster_store.roster_config.tradable_mats_owned[1].data[serca_index] = roster_store.roster_config.tradable_mats_owned[0].data[T4_index]
            roster_store.roster_config.roster_mats_owned[1].data[serca_index] = roster_store.roster_config.roster_mats_owned[0].data[T4_index]
        }
    }
})
</script>

<template>
    <div class="hf-app-shell">
        <header>
            <!-- <div v-if="!is500Narrow" class="hf-page-header-row" style="width: 100%">
                <span>!!! The site is known to crash with the </span> <span style="color: aquamarine">&nbsp;React Developer Tools extension&nbsp;</span> on
                Chromium. Disable the extension, close the site and re-open! (Reloading is insufficient) See the bug report&nbsp;
                <a style="text-decoration: underline" href="https://github.com/facebook/react/issues/36162">here</a>.
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
                    <RouterLink
                        v-if="!is600Narrow || profile_store.profiles.length <= 1"
                        v-for="(profile, index) in profile_store.profiles"
                        :key="index"
                        :to="{ name: 'char', params: { characterName: profile.char_name } }"
                        class="hf-header-button"
                        :class="{ selected: route.path == '/' + profile.char_name }"
                    >
                        {{ profile.char_name }}
                    </RouterLink>

                    <!-- Mobile: dropdown -->
                    <select v-else class="hf-char-select" :value="active_char_name" @change="onCharSelect">
                        <option value="" disabled>Character</option>
                        <option v-for="(profile, index) in profile_store.profiles" :key="index" :value="profile.char_name">
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
