<script setup lang="ts">
import { RouterLink, RouterView, useRoute } from "vue-router";
import { get_icon_path } from "@/Utils/Helpers";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useElementSize, useMediaQuery } from "@vueuse/core";
import {
  computed,
  reactive,
  ref,
  useTemplateRef,
  watch,
  watchEffect,
} from "vue";
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
import { flush_move_report } from "./Utils/MoveReport";
import { bible_auth, sign_in, sign_out, signed_in } from "./Utils/BibleAuth";
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
flush_move_report();

const is900Narrow = useMediaQuery("(max-width: 900px)");

const route = useRoute();

const routed_profile = computed(
  () =>
    all_profiles.value[
      roster_store.profile_index_by_route(
        route.params.region as string,
        route.params.characterName as string,
      )
    ],
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
    <header
      ref="top_row"
      @dragstart.prevent
      class="group sticky top-0 z-50 flex flex-row border-b border-(--border-muted) bg-(--bg-muted)"
    >
      <!-- Sidebar.vue's burger, shown while its page is narrow -->
      <div id="header-burger" class="flex shrink-0" />
      <nav
        class="scroll-x flex min-h-12 min-w-0 flex-1 flex-row items-center justify-start whitespace-nowrap"
      >
        <!-- hidden with the burger too, under 900px at least -->
        <div
          v-if="!is900Narrow"
          class="flex w-50 min-w-50 flex-row items-center text-lg group-has-[.burger]:hidden"
        >
          <router-link to="/">
            <div class="grid h-12 w-12 place-items-center">
              <img
                :src="get_icon_path('Forecast Icon')"
                alt="Forecast icon"
                class="h-9 w-9"
              /></div
          ></router-link>
          <router-link to="/">
            <h1
              class="color-(--text-main) flex flex-row items-center font-semibold"
            >
              Honing Forecast
            </h1></router-link
          >
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
          :class="{ selected: route.path.startsWith('/market-mats') }"
        >
          Market & Mats
        </RouterLink>

        <div class="header-spacer" />
        <RouterLink
          v-for="roster_id in roster_ids"
          :key="roster_id"
          :to="{
            name: 'char',
            params: roster_store.char_params(roster_target(roster_id)),
          }"
          class="nav-tab"
          :class="{ selected: routed_profile?.roster_id === roster_id }"
        >
          {{
            roster_ids.length === 1
              ? "Characters"
              : roster_store.roster_label(roster_id)
          }}
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
        <button
          v-else
          class="ml-auto flex cursor-pointer flex-row items-center gap-2 px-3 text-sm text-(--text-muted) hover:underline"
          title="Sign in with lostark.bible"
          @click="sign_in"
        >
          Sign in
          <img
            :src="get_icon_path('Artist Cry')"
            alt="Artist crying"
            class="h-8 w-8"
          />
        </button>
      </nav>
    </header>

    <RouterView />
    <ScanSwitchDialog />
    <BibleSyncPopup />
  </div>
</template>
