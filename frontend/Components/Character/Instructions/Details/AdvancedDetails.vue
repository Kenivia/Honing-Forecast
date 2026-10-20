<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import "@/Components/Character/Instructions/Details/details.css";
import { GridConfig } from "@/Utils/GridStyling";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import { mark_upgrade_as_done } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
}>();

const current_adv_upgrade = ref(0);
const current_adv_xp = ref(0);
const current_grace_progress = ref(0);
const next_free = ref(false);
const next_big = ref(false);

// In Rust start_xp ranges from 0 to 100 (each bar = 10 xp instead of 100 in game)
watch(
  [
    () => props.upgrade.adv_config.start_xp,
    () => props.upgrade.adv_config.start_balls,
    () => props.upgrade.adv_config.next_free,
    () => props.upgrade.adv_config.next_big,
  ],
  ([start_xp, start_balls, free, big]) => {
    const levels = Math.floor(start_xp / 10);
    current_adv_upgrade.value = levels + props.upgrade.upgrade_index * 10;
    current_adv_xp.value = (start_xp - levels * 10) * 10;
    current_grace_progress.value = start_balls;
    next_free.value = free;
    next_big.value = big;
  },
  { immediate: true },
);

// the game's Chisel and Naber's Awl, only offered in these states
const chisel_shown = computed(
  () =>
    current_grace_progress.value === 0 &&
    (current_adv_xp.value >= 60 ||
      current_adv_upgrade.value > props.upgrade.upgrade_index * 10),
);
const awl_shown = computed(
  () => current_grace_progress.value === 6 && props.upgrade.upgrade_index >= 2,
);

function write_adv_progress() {
  const base = props.upgrade.upgrade_index * 10;
  current_adv_upgrade.value = Math.max(
    base,
    Math.min(base + 9, current_adv_upgrade.value),
  );
  current_adv_xp.value =
    Math.floor(Math.max(0, Math.min(90, current_adv_xp.value)) / 10) * 10;
  current_grace_progress.value = Math.min(
    6,
    Math.max(0, current_grace_progress.value),
  );

  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_index,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].adv_progress = [
    (current_adv_upgrade.value - base) * 10 + current_adv_xp.value / 10,
    current_grace_progress.value,
    next_free.value && chisel_shown.value,
    next_big.value && awl_shown.value,
  ];
  start_eval_hist();
}

const grid: GridConfig = {
  grid_template_columns: "150px 70px 120px",
};
</script>
<template>
  <div
    class="grid w-full pl-5"
    :style="{
      gridTemplateColumns: grid.grid_template_columns,
      opacity: 1,
    }"
  >
    <span class="stat-label">Current upgrade:</span>
    <input
      class="generic-input number-border ml-2 w-13"
      type="number"
      v-model.number="current_adv_upgrade"
      :min="upgrade.upgrade_index * 10"
      :max="(upgrade.upgrade_index + 1) * 10 - 1"
      @change="write_adv_progress"
    />
    <div class="button-row">
      <button
        @click="() => mark_upgrade_as_done(props.upgrade)"
        class="generic-button w-20! text-(--achieved)!"
      >
        Succeed
      </button>
      <QuestionMark
        text="This just removes the upgrade. Costs are not deducted."
        position="left"
      />
    </div>

    <span class="text-right">Current xp:</span>
    <input
      class="generic-input number-border ml-2 w-13 self-start"
      type="number"
      v-model.number="current_adv_xp"
      min="0"
      max="90"
      step="10"
      @change="write_adv_progress"
    />
    <div class="button-row">
      <button @click="start_all_workers" class="generic-button w-20!">
        Confirm
      </button>
      <QuestionMark
        text="Input your current progress, then press Confirm. Costs are not deducted."
        position="left"
      />
    </div>
    <span class="text-right">Grace progress:</span>
    <input
      class="generic-input number-border ml-2 w-13"
      type="number"
      v-model.number="current_grace_progress"
      min="0"
      max="6"
      @change="write_adv_progress"
    />
    <div></div>
    <!-- placeholder for the button -->
    <div class="contents" v-if="chisel_shown">
      <span class="text-right"> Next is free (Chisel):</span>
      <input
        class="ml-2 h-3.5 w-3.5 self-center"
        type="checkbox"
        v-model="next_free"
        @change="write_adv_progress"
      />
    </div>
    <div class="contents" v-if="awl_shown">
      <span class="text-right"> Naber's Awl </span>
      <input
        class="ml-2 h-3.5 w-3.5 self-center"
        type="checkbox"
        v-model="next_big"
        @change="write_adv_progress"
      />
    </div>
    <div v-if="!chisel_shown && !awl_shown" class="h-6"></div>
  </div>
</template>
