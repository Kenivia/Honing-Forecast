<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import NormalDetails from "./NormalDetails.vue";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  free_tap_this_upgrade: boolean;
  index_in_special_state: number;
}>();

const upgrade_key = computed(() =>
  to_upgrade_key(
    props.upgrade.piece_type,
    props.upgrade.upgrade_index,
    props.upgrade.is_normal_honing,
    active_profile.value.tier,
  ),
);

const actual_expanded = computed(
  () =>
    active_profile.value.keyed_upgrades[upgrade_key.value].expanded ||
    props.perform_order == 0,
);
</script>

<template>
  <div v-if="free_tap_this_upgrade && !actual_expanded">
    <!-- placeholder  to push details to the right -->
  </div>
  <div
    v-if="!actual_expanded"
    class="flex w-full flex-col items-center"
    :style="{
      gridColumn: 'span 1',
      opacity: 1,
    }"
  >
    <span
      v-if="perform_order != 0"
      class="text-nowrap text-(--text-muted)"
      :style="{ color: 'var(--dont-click)' }"
      >You should do the upgrade{{ perform_order == 1 ? "" : "s" }} above first
    </span>
    <button
      @click="
        () => {
          active_profile.keyed_upgrades[upgrade_key].expanded = true;
        }
      "
      class="barebone-button w-fit text-(--text-muted)"
    >
      Show input anyway
    </button>
  </div>
  <div v-else class="contents">
    <NormalDetails
      :is_free_tap="free_tap_this_upgrade"
      :perform_order="perform_order"
      :upgrade="upgrade"
      :index_in_special_state="index_in_special_state"
    />
  </div>
</template>
