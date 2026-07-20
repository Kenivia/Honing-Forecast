<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { storeToRefs } from "pinia";
import { watch } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";
import ControlPanel from "@/Components/Character/ControlPanel.vue";
import Sidebar from "@/Components/Common/Sidebar.vue";
import Guide from "@/Components/Character/Guide.vue";
import Setup from "./InventoryScanner/Setup.vue";
import Calc from "./Calc.vue";

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
        active_profile.value.optimizer_worker_bundle.cancel();
        active_profile.value.histogram_worker_bundle.cancel();
        // active_profile.value.evaluation_worker_bundle.cancel()
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
</script>
<template>
  <Sidebar
    :width="route.path.endsWith('calc') ? 1255 : 1201"
    :header="active_profile.char_name"
  >
    <template #sidebar="{ close }">
      <div class="flex flex-col">
        <RouterLink to="guide" class="side-bar-link" @click="close">
          Guide
        </RouterLink>
        <RouterLink to="calc" class="side-bar-link" @click="close">
          Calc
        </RouterLink>
        <RouterLink to="scanner" class="side-bar-link" @click="close">
          Scanner setup
        </RouterLink>
      </div>

      <ControlPanel v-if="route.path.endsWith('calc')" />
    </template>

    <template #main>
      <Guide v-if="route.path.endsWith('guide')" />
      <Calc v-if="route.path.endsWith('calc')" />
      <Setup v-if="route.path.endsWith('scanner')" />
      <div class="min-h-30"></div>
    </template>
  </Sidebar>
</template>
