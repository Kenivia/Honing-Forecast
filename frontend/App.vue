<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import { get_icon_path } from "./Utils/Helpers";
import { CharProfile } from "./Stores/CharacterProfile";
import { debounced_write_roster_config, useRosterStore } from "./Stores/RosterConfig";
import { useMediaIsNarrow } from "./Utils/WindowSize";
import { fetch_callback, useTimedFetch } from "./Utils/MarketDataFetcher";
import { computed } from "vue";
import { storeToRefs } from "pinia";

const roster_store = useRosterStore();
roster_store.init();

const { all_profiles, roster_ids } = storeToRefs(roster_store);

const { start_fetch } = useTimedFetch(fetch_callback);
start_fetch(roster_store.roster_config.region);

roster_store.$subscribe((_mutation, state) => {
    debounced_write_roster_config(state);
});
const { isNarrow: is500Narrow } = useMediaIsNarrow(500);
const { isNarrow: is600Narrow } = useMediaIsNarrow(600);
const { isNarrow: is800Narrow } = useMediaIsNarrow();

const route = useRoute();
const router = useRouter();

const active_char_name = computed(
    () => all_profiles.value.find((p) => route.path === "/" + p.char_name)?.char_name ?? "",
);

function onCharSelect(e: Event) {
    const name = (e.target as HTMLSelectElement).value;
    if (name) router.push({ name: "char", params: { characterName: name } });
}
const char_name_from_route = computed(() => route.path.split("/")[route.path.split("/").length - 2]);
</script>

<template>
    <div class="app-shell flex-col">
        <header>
            <!-- <div v-if="!is500Narrow" class="page-header-row" style="width: 100%; justify-content: space-around">
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
            <nav class="flex-row justify-start align-center bottom-border-subtle gap-8 top-nav">
                <div v-if="!is500Narrow" class="flex-row">
                    <router-link to="/">
                        <div class="brand-icon">
                            <img
                                :src="get_icon_path('Forecast Icon')"
                                alt="Forecast icon"
                                style="width: 34px; height: 34px"
                            /></div
                    ></router-link>
                    <div v-if="!is800Narrow">
                        <h1 class="title">Honing Forecast</h1>
                    </div>
                </div>
                <router-link to="/roster-setup">
                    <div class="header-button header-font" :class="{ selected: route.path == '/roster-setup' }">
                        Roster setup
                    </div>
                </router-link>
                <router-link to="/market-mats">
                    <div class="header-button header-font" :class="{ selected: route.path == '/market-mats' }">
                        Market & Mats
                    </div>
                </router-link>

                <div class="header-spacer" />
                <div style="display: contents" v-if="!is600Narrow || all_profiles.length <= 1">
                    <div
                        v-for="roster_id in roster_ids"
                        style="display: flex; flex-direction: row"
                        :key="`roster-${roster_id}`"
                    >
                        <div
                            v-for="[profile, profile_index] in all_profiles
                                .map((x, index): [CharProfile, number] => [x, index])
                                .filter((y) => y[0].roster_id === roster_id)"
                            :key="`profile-${profile_index}`"
                        >
                            <RouterLink
                                :to="{ name: 'char', params: { characterName: profile.char_name } }"
                                class="header-button header-font"
                                :class="{ selected: char_name_from_route === profile.char_name }"
                            >
                                {{ profile.char_name }}
                            </RouterLink>
                        </div>
                        <span style="width: 16px"></span>
                    </div>
                </div>

                <!-- Mobile: dropdown -->
                <select v-else class="char-select" :value="active_char_name" @change="onCharSelect">
                    <option value="" disabled>Character</option>
                    <option v-for="(profile, index) in all_profiles" :key="index" :value="profile.char_name">
                        {{ profile.char_name }}
                    </option>
                </select>
            </nav>
        </header>

        <RouterView />
    </div>
</template>
<style>
.header-button {
    color: var(--text-main);
    border-radius: 4px;
    border: 1px solid var(--border-subtle);
    background-color: var(--bg-medium);
    margin: 0 2px;
    text-wrap-mode: nowrap;
    padding: 12px;
    cursor: pointer;
}
</style>
<style scoped>
.app-shell {
    min-height: 100vh;
}

.top-nav {
    position: relative;
    z-index: 50;
    background: var(--bg-dark);
    white-space: nowrap;
    scrollbar-width: thin;
}

.header-button:hover {
    background-color: var(--bg-very-bright);
}

.header-button.selected {
    color: var(--gold);
}

.header-spacer {
    width: 10px;
    min-width: 10px;
}

.brand {
    display: flex;
    align-items: center;
    width: 200px;
}

.brand-icon {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
}

.title {
    color: var(--text-main);
    font-size: 20px;
    line-height: 1;
    width: min-content;
}
</style>
