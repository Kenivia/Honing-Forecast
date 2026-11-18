<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { computed, onUnmounted, watch } from "vue";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import ControlPanel from "@/Components/Character/ControlPanel.vue";
import ScannerControlPanel from "@/Components/Character/InventoryScanner/ScannerControlPanel.vue";
import { scanner_shown } from "@/Components/Character/InventoryScanner/ScannerUIutils";
import Sidebar from "@/Components/Common/Sidebar.vue";
import ClassIcon from "@/Components/Common/ClassIcon.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const DEV = import.meta.env.DEV;
const runtime = useRuntimeStore();

const route = useRoute();
const router = useRouter();

const roster_store = useRosterStore();
const { active_profile } = storeToRefs(roster_store);

// The router guard has already checked the name; this only maps it to the active index.
const index = computed(() =>
  roster_store.profile_index_by_name(route.params.characterName as string),
);
watch(
  index,
  (i) => {
    if (i < 0 || i === roster_store.roster_config.active_profile_index) return;
    runtime.cancel_active();
    roster_store.switch_profile(i);
  },
  { immediate: true },
);
// a rename moves the route along, so a reload finds the character
watch(
  () => active_profile.value.char_name,
  (name) => router.replace({ params: { characterName: name } }),
);

const is_calc = computed(() => route.name === "calc");
const roster_chars = computed(() =>
  roster_store.all_profiles.filter(
    (p) => p.roster_id === active_profile.value.roster_id,
  ),
);

function add_char() {
  const added = roster_store.add_char(active_profile.value.roster_id);
  router.push({ params: { characterName: added.char_name } });
}

onUnmounted(() => {
  // kill workers when going to market / roster view
  runtime.cancel_active();
});
</script>
<template>
  <Sidebar :width="is_calc ? 1255 : 1201" :header="active_profile.char_name">
    <template #top>
      <nav
        aria-label="Characters"
        class="flex min-h-11 scrollbar-thin flex-row items-center overflow-x-auto border-b border-(--border-muted) bg-(--bg-muted) whitespace-nowrap"
      >
        <RouterLink
          v-for="profile in roster_chars"
          :key="profile.id"
          :to="{
            name: route.name,
            params: { characterName: profile.char_name },
          }"
          class="nav-tab"
          :class="{ selected: profile.id === active_profile.id }"
        >
          <ClassIcon :class_name="profile.class_name" />
          {{ profile.char_name }}
        </RouterLink>
        <button class="nav-tab" @click="add_char">
          <svg
            class="size-5 shrink-0"
            viewBox="0 0 20 20"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            aria-hidden="true"
          >
            <path d="M10 4v12M4 10h12" />
          </svg>
          Add Char
        </button>
      </nav>
    </template>
    <template #sidebar="{ close }">
      <div class="flex flex-col">
        <RouterLink to="calc" class="side-bar-link" @click="close">
          Calc
        </RouterLink>
        <RouterLink v-if="DEV" to="income" class="side-bar-link" @click="close">
          Income
        </RouterLink>
        <RouterLink to="choices" class="side-bar-link" @click="close">
          Choices
        </RouterLink>
        <RouterLink to="scanner" class="side-bar-link" @click="close">
          Scanner
          <!-- span class="new-tag ml-2">beta</span> -->
        </RouterLink>
      </div>

      <ControlPanel v-if="is_calc" />
      <ScannerControlPanel
        v-if="route.name === 'char-scanner' && scanner_shown"
      />
    </template>

    <template #main>
      <!-- the calculator is remounted per character, so nothing of the last one carries over -->
      <RouterView v-slot="{ Component }">
        <component
          :is="Component"
          :key="is_calc ? active_profile.id : String(route.name)"
        />
      </RouterView>
      <div class="min-h-300"></div>
    </template>
  </Sidebar>
</template>
