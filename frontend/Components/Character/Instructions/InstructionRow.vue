<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { get_piece_name, get_icon_path, toOrdinal } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import ActualInstructions from "./ActualInstructions.vue";
import NormalDetails from "./NormalDetails.vue";
import SuccessPopup from "./SuccessPopup.vue";
import { get_any_overwritten, get_optimizer_working } from "./InstructionUtils";
import AdvancedDetails from "./AdvancedDetails.vue";

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

const show_success_modal = ref(false);
function onSucceedClick() {
  show_success_modal.value = true;
}
const optimizer_working = computed(get_optimizer_working);
const should_click = computed(
  () =>
    props.perform_order == 0 &&
    !free_tap_this_upgrade.value &&
    !optimizer_working.value,
);
</script>

<template>
  <div class="flex flex-col items-center">
    <span>{{
      (upgrade.is_normal_honing ? "" : "Advanced ") +
      get_piece_name(upgrade) +
      " +" +
      String(upgrade.upgrade_index * (upgrade.is_normal_honing ? 1 : 10) + 1) +
      (upgrade.is_normal_honing
        ? ""
        : " - " +
          String(
            (upgrade.upgrade_index + 1) * (upgrade.is_normal_honing ? 1 : 10),
          ))
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
          : "Do not use special tap on this upgrade"
      }}
    </span>
  </div>

  <ActualInstructions :upgrade="props.upgrade" />

  <div class="contents" v-if="!any_overwritten">
    <div class="flex w-full flex-col">
      <button
        v-if="perform_order == 0 || !upgrade.is_normal_honing"
        @click="onSucceedClick"
        class="generic-button w-full! text-wrap! text-(--achieved)!"
        :style="{
          color: optimizer_working
            ? 'var(--warning-dark)'
            : should_click
              ? 'var(--text-main)'
              : 'var(--dont-click)',
          cursor: optimizer_working ? 'not-allowed' : 'pointer',
          opacity:
            (props.perform_order == 0 || !upgrade.is_normal_honing) &&
            !optimizer_working
              ? 1
              : 0.5,
        }"
      >
        Succeed
      </button>
      <button
        v-if="
          free_tap_this_upgrade &&
          perform_order == 0 &&
          upgrade.is_normal_honing
        "
        @click="onSucceedClick"
        class="generic-button text-wrap! text-(--free-tap-muted)!"
      >
        All free-taps failed
      </button>
      <span
        v-if="perform_order != 0 && upgrade.is_normal_honing"
        class="annotation"
        :style="{
          color: should_click ? 'var(--text-main)' : 'var(--dont-click)',
        }"
        >You should do the upgrades above this first</span
      >
    </div>
    <NormalDetails
      v-if="upgrade.is_normal_honing"
      :upgrade="props.upgrade"
      :perform_order="props.perform_order"
      :free_tap_this_upgrade="free_tap_this_upgrade"
    />
    <AdvancedDetails
      v-if="!upgrade.is_normal_honing"
      :upgrade="props.upgrade"
      :perform_order="props.perform_order"
      :free_tap_this_upgrade="free_tap_this_upgrade"
    />

    <SuccessPopup :upgrade="props.upgrade" v-model="show_success_modal" />
  </div>
  <div
    v-else
    class="col-span-2 flex w-full flex-col text-left"
    :style="{
      backgroundColor: any_overwritten ? 'var(--warning-dark)' : 'transparent',
    }"
  >
    <span> &lt;&lt;&lt; FOR COMPARISON PURPOSE ONLY, DO NOT FOLLOW!</span>
  </div>
</template>
