<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_piece_name, get_icon_path, toOrdinal } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";

import { Upgrade } from "@/Utils/KeyedUpgrades";
import ActualInstructions from "./ActualInstructions.vue";
import NormalHoningDetails from "./NormalHoningDetails.vue";
import SuccessPopup from "./SuccessPopup.vue";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  index_in_special_state: number;
  special_invalid_index: number;
}>();

const free_tap_this_upgrade = computed(() => {
  return (
    props.index_in_special_state < props.special_invalid_index &&
    props.upgrade.this_special_chance > 0
  );
});

const show_success_modal = ref(false);
function onSucceedClick() {
  show_success_modal.value = true;
}
</script>

<template>
  <div class="flex flex-col items-center">
    <span>{{
      (upgrade.is_normal_honing ? "" : "Advanced ") +
      get_piece_name(upgrade) +
      " +" +
      String((upgrade.upgrade_index + 1) * (upgrade.is_normal_honing ? 1 : 10))
    }}</span>
    <img
      :src="get_icon_path(get_piece_name(upgrade))"
      :alt="get_piece_name(upgrade)"
      class="h-8 w-8 object-contain"
    />
  </div>

  <div>
    <div class="text-4xl">
      {{ toOrdinal(props.perform_order + 1) }}
    </div>
  </div>

  <div class="flex flex-col items-center">
    <img
      :src="
        get_icon_path(
          (active_profile.tier == 1 ? 'Serca ' : '') +
            active_profile.special_budget.keys[0],
        )
      "
      :alt="get_piece_name(upgrade)"
      class="h-12 w-12 object-contain"
    />
    <!-- TODO ADD BIG CROSS HERE FOR NO FREE TAP -->
    <span class="annotation">
      {{
        free_tap_this_upgrade
          ? "Free tap this until you run out or succeed"
          : "Do not use special tap on this upgrade"
      }}
    </span>
  </div>

  <ActualInstructions :upgrade="props.upgrade" />

  <button @click="onSucceedClick" class="generic-button text-wrap!">
    Succeed & deduct costs
  </button>
  <div class="flex flex-col items-center">
    <NormalHoningDetails
      v-if="upgrade.is_normal_honing"
      :upgrade="props.upgrade"
    />
  </div>
  <SuccessPopup
    :upgrade="props.upgrade"
    v-model="show_success_modal"
  ></SuccessPopup>
</template>

<style scoped></style>
