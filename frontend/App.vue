<script setup lang="ts">
import { RouterLink, RouterView, useRoute } from "vue-router";
import { get_icon_path } from "@/Utils/Helpers";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useElementSize, useMediaQuery } from "@vueuse/core";
import { computed, reactive, ref, useTemplateRef, watch, watchEffect } from "vue";
import { storeToRefs } from "pinia";
import { FETCHABLE_REGIONS, start_fetch } from "./Utils/MarketDataFetcher";
import {
  debounced_write_roster_config,
  save_watch_source,
  write_state,
} from "./Stores/ConfigStorage";
import { useRuntimeStore } from "./Stores/RuntimeState";
import ScanSwitchDialog from "./Components/Character/InventoryScanner/ScanSwitchDialog.vue";
import { upload_snapshots } from "./Utils/Analytics";
import { bible_auth, sign_out, signed_in } from "./Utils/BibleAuth";
import { auto_refresh } from "./Utils/BibleSync";
import BibleSyncPopup from "./Components/Common/Uwuowo/BibleSyncPopup.vue";

const roster_store = useRosterStore();
roster_store.init();

const { all_profiles, roster_ids } = storeToRefs(roster_store);

watch(
  () => save_watch_source(roster_store.roster_config),
  () => debounced_write_roster_config(roster_store.$state),
  { deep: true },
);
// the debounced write may not have landed when the tab closes
window.addEventListener("pagehide", () => write_state(roster_store.$state));

// The only place runtime state is kept in step with the character list, so adds and
// deletes all land here. Keyed by id, so a rename keeps its workers.
const runtime = useRuntimeStore();
watch(
  () => all_profiles.value.map((p) => p.id),
  (ids) => runtime.sync_profiles(ids),
  { immediate: true },
);

// on load, then as snapshots are taken or sharing is turned back on
watch(
  () => [
    roster_store.roster_config.snapshots.length,
    roster_store.roster_config.analytics,
  ],
  () => upload_snapshots(roster_store.roster_config),
  { immediate: true },
);

auto_refresh();

const is900Narrow = useMediaQuery("(max-width: 900px)");

const route = useRoute();

const char_name_from_route = computed(
  () => (route.params.characterName as string | undefined) ?? "",
);
const routed_profile = computed(() =>
  all_profiles.value.find((p) => p.char_name === char_name_from_route.value),
);

// session only: a roster button goes back to the roster's last opened character
const last_opened: Record<string, string> = reactive({});
watch(
  routed_profile,
  (p) => {
    if (p) last_opened[p.roster_id] = p.id;
  },
  { immediate: true },
);
// session only: the Info tab goes back to the last opened info page
const last_info = ref("/info");
watch(
  () => route.path,
  (p) => {
    if (p.startsWith("/info/")) last_info.value = route.fullPath;
  },
  { immediate: true },
);
function roster_target(roster_id: string) {
  const chars = all_profiles.value.filter((p) => p.roster_id === roster_id);
  return chars.find((p) => p.id === last_opened[roster_id]) ?? chars[0];
}

// the sidebar sticks under the header
const top_row = useTemplateRef<HTMLElement>("top_row");
const { height: top_row_h } = useElementSize(top_row, undefined, {
  box: "border-box",
});
watchEffect(() =>
  document.documentElement.style.setProperty(
    "--top-row-h",
    `${top_row_h.value}px`,
  ),
);
FETCHABLE_REGIONS.forEach((region) => start_fetch(region));
</script>

<template>
  <div class="flex min-h-lvh max-w-full flex-col">
    <header ref="top_row" class="sticky top-0 z-50">
      <nav
        class="flex min-h-12 scrollbar-thin flex-row items-center justify-start overflow-x-auto border-b border-(--border-muted) bg-(--bg-muted) whitespace-nowrap"
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
          <h1
            class="color-(--text-main) flex flex-row items-center font-semibold"
          >
            Honing Forecast
          </h1>
        </div>
        <RouterLink
          :to="last_info"
          class="nav-tab"
          :class="{ selected: route.path.startsWith('/info') }"
        >
          Info
        </RouterLink>
        <RouterLink
          to="/roster-setup"
          class="nav-tab"
          :class="{ selected: route.path == '/roster-setup' }"
        >
          Roster setup
        </RouterLink>
        <RouterLink
          to="/market-mats"
          class="nav-tab"
          :class="{ selected: route.path == '/market-mats' }"
        >
          Market & Mats
        </RouterLink>

        <div class="header-spacer" />
        <RouterLink
          v-for="(roster_id, i) in roster_ids"
          :key="roster_id"
          :to="{
            name: 'char',
            params: { characterName: roster_target(roster_id).char_name },
          }"
          class="nav-tab"
          :class="{ selected: routed_profile?.roster_id === roster_id }"
        >
          {{ roster_ids.length === 1 ? "Character" : `Roster ${i + 1}` }}
        </RouterLink>
        <div
          v-if="signed_in"
          class="ml-auto flex flex-row items-center gap-2 px-3 text-sm text-(--text-muted)"
        >
          <img
            v-if="bible_auth.avatar"
            :src="bible_auth.avatar"
            :alt="bible_auth.name"
            :title="bible_auth.name"
            class="h-8 w-8 rounded-full"
          />
          <span v-else>{{ bible_auth.name }}</span>
          <button class="cursor-pointer hover:underline" @click="sign_out">
            Sign out
          </button>
        </div>
      </nav>
    </header>

    <RouterView />
    <ScanSwitchDialog />
    <BibleSyncPopup />
  </div>
</template>
