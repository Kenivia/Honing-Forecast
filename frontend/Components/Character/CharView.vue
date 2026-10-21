<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { computed, onUnmounted, watch } from "vue";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import ControlPanel from "@/Components/Character/ControlPanel.vue";
import ScannerControlPanel from "@/Components/Character/InventoryScanner/ScannerControlPanel.vue";
import Sidebar from "@/Components/Common/Sidebar.vue";
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

onUnmounted(() => {
  // kill workers when going to market / roster view
  runtime.cancel_active();
});
</script>
<template>
  <Sidebar :width="is_calc ? 1255 : 1201" :header="active_profile.char_name">
    <template #sidebar="{ close }">
      <div class="flex flex-col">
        <RouterLink to="guide" class="side-bar-link" @click="close">
          Guide
        </RouterLink>
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
          Scanner<span class="new-tag ml-2">beta</span>
        </RouterLink>
      </div>

      <ControlPanel v-if="is_calc" />
      <ScannerControlPanel v-if="route.name === 'char-scanner'" />
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
