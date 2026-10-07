<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { computed, onUnmounted, watch } from "vue";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import ControlPanel from "@/Components/Character/ControlPanel.vue";
import ScannerControlPanel from "@/Components/Character/InventoryScanner/ScannerControlPanel.vue";
import Sidebar from "@/Components/Common/Sidebar.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
const runtime = useRuntimeStore();

const route = useRoute();
const router = useRouter();

const roster_store = useRosterStore();
const { active_profile, all_profiles } = storeToRefs(roster_store);

const match = all_profiles.value.findIndex(
  (c) => c.char_name === (route.params.characterName as string),
);
if (match >= 0) {
  roster_store.switch_profile(match);
} else {
  router.replace({
    name: "char",
    params: { characterName: all_profiles.value[0].char_name },
  });
  roster_store.switch_profile(0);
}
watch(
  () => route.params.characterName as string,
  (name) => {
    const match = all_profiles.value.findIndex((c) => c.char_name === name);
    if (match >= 0) {
      if (roster_store.roster_config.active_profile_index !== match) {
        // this happens one invalid names (routre param written to by the one-off code, triggering the watcher) i believe, idk how to prevent that but this works
        runtime.cancel_active();
        // reset the scanner, the capture stream itself is kept
        runtime.cropper?.cancel_and_clear_prev_result();
        roster_store.switch_profile(match);
      }
    } else {
      router.replace({
        name: "char",
        params: { characterName: all_profiles.value[0].char_name },
      });
      roster_store.switch_profile(0);
    }
  },
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
        <RouterLink to="setup" class="side-bar-link" @click="close">
          Scanner setup
        </RouterLink>
        <RouterLink to="scanner" class="side-bar-link" @click="close">
          Scanner
        </RouterLink>
      </div>

      <ControlPanel v-if="is_calc" />
      <ScannerControlPanel v-if="route.name === 'char-scanner'" />
    </template>

    <template #main>
      <RouterView />
      <div class="min-h-300"></div>
    </template>
  </Sidebar>
</template>
