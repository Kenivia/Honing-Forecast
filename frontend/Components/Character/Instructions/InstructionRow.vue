<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_piece_name, get_icon_path, toOrdinal } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import ActualInstructions from "@/Components/Character/Instructions/ActualInstructions.vue";
import NormalDetails from "@/Components/Character/Instructions/Details/NormalDetails/NormalDetails.vue";
import { get_any_overwritten } from "@/Components/Character/Instructions/InstructionUtils";
import AdvancedDetails from "@/Components/Character/Instructions/Details/AdvancedDetails.vue";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  index_in_special_state: number;
  special_invalid_index: number;
}>();

const any_overwritten = computed(get_any_overwritten);

const free_tap_this_upgrade = computed(() => {
  return (
    props.index_in_special_state < props.special_invalid_index &&
    props.upgrade.this_special_chance > 0
  );
});
</script>

<template>
  <div class="flex flex-col items-center">
    <span v-if="upgrade.is_normal_honing">{{
      get_piece_name(upgrade) + " +" + String(upgrade.upgrade_index + 1)
    }}</span>

    <span v-if="!upgrade.is_normal_honing"
      >{{ get_piece_name(upgrade) + " Advanced " }}
    </span>
    <span v-if="!upgrade.is_normal_honing">{{
      " +" +
      String(upgrade.upgrade_index * 10 + 1) +
      " - " +
      String((upgrade.upgrade_index + 1) * 10)
    }}</span>
    <img
      :src="get_icon_path(get_piece_name(upgrade))"
      :alt="get_piece_name(upgrade)"
      class="generic-icon h-8 w-8"
    />
  </div>

  <div v-if="upgrade.is_normal_honing">
    <div class="text-4xl">
      {{ toOrdinal(props.perform_order + 1) }}
    </div>
  </div>

  <div v-if="upgrade.is_normal_honing" class="flex flex-col items-center">
    <div
      class="can-disable-icon-wrapper"
      :class="{
        disabled: !free_tap_this_upgrade,
        ticked: free_tap_this_upgrade,
      }"
    >
      <img
        :src="
          get_icon_path(
            (active_profile.tier == 1 ? 'Serca ' : '') +
              active_profile.special_budget.keys[0],
          )
        "
        :alt="get_piece_name(upgrade)"
        class="generic-icon h-12 w-12"
        :class="{ disabled: !free_tap_this_upgrade }"
      />
    </div>

    <span class="annotation">
      {{
        free_tap_this_upgrade
          ? "Free tap this until you run out or succeed"
          : "Do not use special leaps on this upgrade"
      }}
    </span>
  </div>

  <ActualInstructions v-if="!free_tap_this_upgrade" :upgrade="props.upgrade" />

  <div class="contents" v-if="!any_overwritten">
    <NormalDetails
      v-if="upgrade.is_normal_honing"
      :upgrade="props.upgrade"
      :perform_order="props.perform_order"
      :free_tap_this_upgrade="free_tap_this_upgrade"
      :index_in_special_state="index_in_special_state"
      :key="
        to_upgrade_key(
          props.upgrade.piece_type,
          props.upgrade.upgrade_index,
          props.upgrade.is_normal_honing,
          active_profile.tier,
        )
      "
    />
    <AdvancedDetails
      v-if="!upgrade.is_normal_honing"
      :upgrade="props.upgrade"
    />
  </div>
  <div
    v-else
    class="flex w-full flex-col text-left"
    :style="{
      backgroundColor: any_overwritten ? 'var(--warning-dark)' : 'transparent',
      gridColumn: free_tap_this_upgrade ? 'span 2' : 'span 1',
    }"
  >
    <span> &lt;&lt;&lt; FOR COMPARISON PURPOSE ONLY, DO NOT FOLLOW!</span>
  </div>
</template>
