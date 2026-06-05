<script setup lang="ts">
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import { get_icon_path } from "@/Utils/Helpers";
import { CharProfile } from "@/Stores/CharacterProfile";
import {
  debounced_write_roster_config,
  useRosterStore,
} from "@/Stores/RosterConfig";
import { useMediaIsNarrow } from "@/Utils/WindowSize";
import { computed } from "vue";
import { storeToRefs } from "pinia";

const roster_store = useRosterStore();
roster_store.init();

const { all_profiles, roster_ids } = storeToRefs(roster_store);

roster_store.$subscribe((_mutation, state) => {
  debounced_write_roster_config(state);
});

const is900Narrow = useMediaIsNarrow(900);

const route = useRoute();
const router = useRouter();

function on_char_select(e: Event) {
  const name = (e.target as HTMLSelectElement).value;
  if (name) router.push({ name: "char", params: { characterName: name } });
}
const char_name_from_route = computed(
  () => route.path.split("/")[route.path.split("/").length - 2],
);
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

        <div class="header-spacer" />
        <div class="contents" v-if="!is900Narrow || all_profiles.length <= 1">
          <div
            v-for="roster_id in roster_ids"
            :key="`roster-${roster_id}`"
            class="flex flex-row"
          >
            <div
              v-for="[profile, profile_index] in all_profiles
                .map((x, index): [CharProfile, number] => [x, index])
                .filter((y) => y[0].roster_id === roster_id)"
              :key="`profile-${profile_index}`"
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
            :key="profile.char_name"
            :value="profile.char_name"
          >
            {{ profile.char_name }}
          </option>
        </select>
      </nav>
    </header>

    <RouterView />
  </div>
</template>
