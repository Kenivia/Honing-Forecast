<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  get_piece_name,
  get_icon_path,
  get_piece_icon,
  get_piece_bg,
  toOrdinal,
} from "@/Utils/Helpers";
import { SPECIAL_LEAP_LABELS } from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import ActualInstructions from "@/Components/Character/Instructions/ActualInstructions.vue";
import NormalDetailsWrapper from "@/Components/Character/Instructions/Details/NormalDetails/NormalDetailsWrapper.vue";
import {
  get_any_overwritten,
  get_optimizer_working,
} from "@/Components/Character/Instructions/InstructionUtils";
import AdvancedDetails from "@/Components/Character/Instructions/Details/AdvancedDetails.vue";
import HoverTip from "@/Components/Common/HoverTip.vue";
import UpgradeTip from "@/Components/Character/Instructions/UpgradeTip.vue";

const { active_profile, roster_config } = storeToRefs(useRosterStore());

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
const optimizer_working = computed(get_optimizer_working);
</script>

<template>
  <HoverTip
    :sticky="false"
    placement="right"
    :max-width="400"
    class="flex flex-col items-center"
  >
    <span v-if="upgrade.is_normal_honing">{{
      get_piece_name(upgrade) + " +" + String(upgrade.upgrade_index + 1)
    }}</span>

    <span v-if="!upgrade.is_normal_honing"
      >{{ get_piece_name(upgrade) + " Advanced " }}
    </span>
    <span v-if="!upgrade.is_normal_honing">{{
      " +" +
      String(upgrade.upgrade_index * 10) +
      " - " +
      String((upgrade.upgrade_index + 1) * 10)
    }}</span>
    <img
      :src="
        get_piece_icon(
          get_piece_name(upgrade),
          active_profile.class_name,
          active_profile.tier,
        )
      "
      :alt="get_piece_name(upgrade)"
      :style="get_piece_bg(get_piece_name(upgrade), upgrade.upgrade_index + 1)"
      class="generic-icon piece-icon h-8 w-8"
    />
    <template #tip><UpgradeTip :upgrade="upgrade" /></template>
  </HoverTip>

  <div
    v-if="upgrade.is_normal_honing"
    :style="{ opacity: !optimizer_working ? 1 : 0.5 }"
  >
    <div class="text-4xl">
      {{ toOrdinal(props.perform_order + 1) }}
    </div>
  </div>

  <div
    v-if="upgrade.is_normal_honing"
    class="flex flex-col items-center"
    :style="{ opacity: !optimizer_working ? 1 : 0.5 }"
  >
    <div
      class="can-disable-icon-wrapper"
      :class="{
        disabled: !free_tap_this_upgrade,
        ticked: free_tap_this_upgrade,
      }"
    >
      <img
        :src="get_icon_path(SPECIAL_LEAP_LABELS[active_profile.tier])"
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

  <!-- on a free-tap row the juice instructions only apply once the free taps fail -->
  <div class="flex min-w-0 flex-row items-center">
    <div
      v-if="free_tap_this_upgrade && !roster_config.hide_free_tap_hint"
      class="annotation flex shrink-0 flex-col items-center gap-1 px-2 text-nowrap"
    >
      <span>&gt;&gt;&gt;&gt;&gt;&gt;&gt;&gt;&gt;&gt;</span>
      <span>Only if all</span>
      <span>free taps failed</span>
      <span>&gt;&gt;&gt;&gt;&gt;&gt;&gt;&gt;&gt;&gt;</span>
      <button
        class="annotation cursor-pointer underline"
        @click="roster_config.hide_free_tap_hint = true"
      >
        OK
      </button>
    </div>
    <ActualInstructions class="min-w-0" :upgrade="props.upgrade" />
  </div>

  <div class="contents" v-if="!any_overwritten">
    <NormalDetailsWrapper
      v-if="upgrade.is_normal_honing"
      :upgrade="props.upgrade"
      :perform_order="props.perform_order"
      :free_tap_this_upgrade="free_tap_this_upgrade"
      :index_in_special_state="index_in_special_state"
      :key="
        to_upgrade_key(
          props.upgrade.piece_index,
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
    }"
  >
    <span> &lt;&lt;&lt; FOR COMPARISON PURPOSE ONLY, DO NOT FOLLOW!</span>
  </div>
</template>
