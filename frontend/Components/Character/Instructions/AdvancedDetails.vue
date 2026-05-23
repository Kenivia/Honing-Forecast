<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { start_all_workers, start_eval_hist } from "../CharWorkerUtils";
import "./details.css";
import { get_optimizer_working } from "./InstructionUtils";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
}>();
const optimizer_working = computed(get_optimizer_working);

// In Rust start_xp ranges from 0 to 100 (each bar = 10 xp instead of 100 in game)
const current_adv_upgrade = ref(
  props.upgrade.adv_config
    ? Math.floor(props.upgrade.adv_config.start_xp / 10) +
        props.upgrade.upgrade_index * 10
    : 0,
);
const current_adv_xp = ref(
  props.upgrade.adv_config
    ? (props.upgrade.adv_config.start_xp -
        Math.floor(props.upgrade.adv_config.start_xp / 10) * 10) *
        10
    : 0,
);
const current_grace_progress = ref(props.upgrade.adv_config.start_balls);
const next_free = ref(props.upgrade.adv_config?.next_free ?? false);
const next_big = ref(props.upgrade.adv_config?.next_big ?? false);

watch(
  [
    () => props.upgrade.adv_config.start_xp,
    () => props.upgrade.adv_config.start_balls,
    () => props.upgrade.adv_config.next_big,
    () => props.upgrade.adv_config.next_free,
  ],
  () => {
    current_adv_upgrade.value = props.upgrade.adv_config
      ? Math.floor(props.upgrade.adv_config.start_xp / 10) +
        props.upgrade.upgrade_index * 10
      : 0;
    ((current_adv_xp.value = props.upgrade.adv_config
      ? (props.upgrade.adv_config.start_xp -
          Math.floor(props.upgrade.adv_config.start_xp / 10) * 10) *
        10
      : 0),
      (current_grace_progress.value = props.upgrade.adv_config.start_balls));
    next_free.value = props.upgrade.adv_config?.next_free ?? false;
    next_big.value = props.upgrade.adv_config?.next_big ?? false;
  },
);

function write_adv_progress() {
  current_adv_upgrade.value = Math.max(
    props.upgrade.upgrade_index * 10,
    Math.min(
      (props.upgrade.upgrade_index + 1) * 10 - 1,
      current_adv_upgrade.value,
    ),
  );
  current_adv_xp.value =
    Math.floor(Math.max(0, Math.min(90, current_adv_xp.value)) / 10) * 10;
  // console.log(current_grace_progress.value)
  current_grace_progress.value = Math.min(
    6,
    Math.max(0, current_grace_progress.value),
  );

  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].adv_progress = [
    (current_adv_upgrade.value - props.upgrade.upgrade_index * 10) * 10 +
      current_adv_xp.value / 10,
    current_grace_progress.value,
    next_free.value,
    next_big.value,
  ];
  start_eval_hist();
}

const must_show = ref(false);

watch(
  [
    () => props.upgrade.starting_artisan,
    () => props.upgrade.is_normal_honing,
    () => props.upgrade.adv_config.start_balls,
    () => props.upgrade.adv_config.start_xp,
    () => props.upgrade.adv_config.next_big,
    () => props.upgrade.adv_config.next_free,
  ],
  () => {
    if (props.upgrade.is_normal_honing) {
      must_show.value = props.upgrade.starting_artisan > 0;
    } else {
      must_show.value =
        props.upgrade.adv_config.start_balls > 0 ||
        props.upgrade.adv_config.start_xp > 0 ||
        props.upgrade.adv_config.next_big ||
        props.upgrade.adv_config.next_free;
    }
  },
  { immediate: true },
);
</script>
<template>
  <div class="outer-details-grid">
    <div class="label-number-grid">
      <div class="label-number-row">
        <span class="text-right">Current upgrade:</span>
        <input
          class="generic-input number-border ml-2 w-13"
          type="number"
          v-model.number="current_adv_upgrade"
          :min="upgrade.upgrade_index * 10"
          :max="(upgrade.upgrade_index + 1) * 10 - 1"
          @change="write_adv_progress"
        />
      </div>
      <div class="label-number-row">
        <span class="text-right">Current xp:</span>
        <input
          class="generic-input number-border ml-2 w-13"
          type="number"
          v-model.number="current_adv_xp"
          min="0"
          max="90"
          step="10"
          style="justify-self: flex-start"
          @change="write_adv_progress"
        />
      </div>
      <div class="label-number-row">
        <span class="text-right">Grace progress:</span>
        <input
          class="generic-input number-border ml-2 w-13"
          type="number"
          v-model.number="current_grace_progress"
          min="0"
          max="6"
          @change="write_adv_progress"
        />
      </div>
      <div
        class="label-number-row"
        v-if="
          current_grace_progress === 0 &&
          (current_adv_xp > 0 ||
            current_adv_upgrade > upgrade.upgrade_index * 10)
        "
      >
        <span class="text-right"> Next is free (Chisel):</span>
        <input
          class="ml-2 h-3.5 w-3.5 self-center"
          type="checkbox"
          v-model="next_free"
          @change="write_adv_progress"
        />
      </div>
      <div
        class="label-number-row"
        v-if="current_grace_progress === 6 && upgrade.upgrade_index >= 2"
      >
        <span class="text-right"> Naber's Awl </span>
        <input
          class="ml-2 h-3.5 w-3.5 self-center"
          type="checkbox"
          v-model="next_big"
          @change="write_adv_progress"
        />
      </div>
      <div
        v-if="
          !(
            current_grace_progress === 0 &&
            (current_adv_xp > 0 ||
              current_adv_upgrade > upgrade.upgrade_index * 10)
          ) && !(current_grace_progress === 6 && upgrade.upgrade_index >= 2)
        "
        class="h-6"
      ></div>
    </div>
    <div v-if="!optimizer_working" class="self-center">
      <button @click="start_all_workers" class="generic-button confirm-button">
        Confirm & re-run optimizer
      </button>
    </div>
    <div v-if="optimizer_working" class="h-fit max-w-20 self-center text-wrap">
      Optimizer working ({{
        active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(
          2,
        )
      }}%)
    </div>
  </div>
</template>
