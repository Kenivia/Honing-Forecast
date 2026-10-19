<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import { get_icon_path } from "@/Utils/Helpers";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useMediaIsNarrow } from "@/Utils/WindowSize";
import { computed, watch } from "vue";
import { storeToRefs } from "pinia";
import { FETCHABLE_REGIONS, start_fetch } from "./Utils/MarketDataFetcher";
import { debounced_write_roster_config } from "./Stores/ConfigStorage";
import { useRuntimeStore } from "./Stores/RuntimeState";
import ScanSwitchDialog from "./Components/Character/InventoryScanner/ScanSwitchDialog.vue";

const roster_store = useRosterStore();
roster_store.init();

const { all_profiles, roster_ids } = storeToRefs(roster_store);

roster_store.$subscribe((_mutation, state) => {
  debounced_write_roster_config(state);
});

// The only place runtime state is kept in step with the character list, so adds and
// deletes all land here. Keyed by id, so a rename keeps its workers.
const runtime = useRuntimeStore();
watch(
  () => all_profiles.value.map((p) => p.id),
  (ids) => runtime.sync_profiles(ids),
  { immediate: true },
);

const is900Narrow = useMediaIsNarrow(900);

const route = useRoute();
const router = useRouter();

function on_char_select(e: Event) {
  const select = e.target as HTMLSelectElement;
  const name = select.value;
  // back to the shown character if the switch was held up
  if (name)
    router
      .push({ name: "char", params: { characterName: name } })
      .then(() => (select.value = char_name_from_route.value));
}
const char_name_from_route = computed(
  () => route.path.split("/")[route.path.split("/").length - 2],
);
FETCHABLE_REGIONS.forEach((region) => start_fetch(region, false, true));
</script>

<template>
  <div class="flex min-h-lvh max-w-full flex-col">
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

      <nav
        class="relative z-50 flex scrollbar-thin flex-row items-center justify-start overflow-x-auto border-b border-(--border-muted) bg-(--bg-muted) whitespace-nowrap"
      >
        <div
          v-if="!is900Narrow"
          class="flex w-50 min-w-50 flex-row items-center text-lg"
        >
          <router-link to="/">
            <div class="grid h-12 w-12 place-items-center">
              <img
                :src="get_icon_path('Forecast Icon')"
                alt="Forecast icon"
                class="h-9 w-9"
              /></div
          ></router-link>
          <div v-if="!is900Narrow">
            <h1
              class="color-(--text-main) flex flex-row items-center font-semibold"
            >
              Honing Forecast
            </h1>
          </div>
        </div>
        <router-link to="/roster-setup">
          <div
            class="header-button header-font"
            :class="{ selected: route.path == '/roster-setup' }"
          >
            Roster setup
          </div>
        </router-link>
        <router-link to="/market-mats">
          <div
            class="header-button header-font"
            :class="{ selected: route.path == '/market-mats' }"
          >
            Market & Mats
          </div>
        </router-link>
        <router-link to="/snapshots">
          <div
            class="header-button header-font"
            :class="{ selected: route.path == '/snapshots' }"
          >
            Snapshots
          </div>
        </router-link>

        <div class="header-spacer" />
        <div class="contents" v-if="!is900Narrow || all_profiles.length <= 1">
          <div
            v-for="roster_id in roster_ids"
            :key="`roster-${roster_id}`"
            class="flex flex-row"
          >
            <div
              v-for="profile in all_profiles.filter(
                (p) => p.roster_id === roster_id,
              )"
              :key="profile.id"
            >
              <RouterLink
                :to="{
                  name: 'char',
                  params: { characterName: profile.char_name },
                }"
                class="header-button header-font"
                :class="{
                  selected: char_name_from_route === profile.char_name,
                }"
              >
                {{ profile.char_name }}
              </RouterLink>
            </div>
            <span style="width: 16px"></span>
          </div>
        </div>

        <!-- Mobile: dropdown -->
        <select
          v-else
          class="header-button header-font"
          :value="char_name_from_route"
          @change="on_char_select"
        >
          <option value="" disabled key="empty selection">Character</option>
          <option
            v-for="profile in all_profiles"
            :key="profile.id"
            :value="profile.char_name"
          >
            {{ profile.char_name }}
          </option>
        </select>
      </nav>
    </header>

    <RouterView />
    <ScanSwitchDialog />
  </div>
</template>
