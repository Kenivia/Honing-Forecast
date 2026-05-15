<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { ARTISAN_RATE, FLOAT_TOL } from "@/Utils/Constants";
import {
  clamp,
  clamp_percentage,
  clean_percentage_input,
} from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";

import { storeToRefs } from "pinia";

import { computed, ref, watch } from "vue";
import { start_all_workers } from "../CharWorkerUtils";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  free_tap_this_upgrade: boolean;
}>();

// const juice_info = computed(() => {
//   return active_profile.value.histogram_worker_bundle.result.juice_info;
// });

const starting_artisan = ref(
  parse_locale_float((props.upgrade.starting_artisan * 100).toFixed(2)) || 0,
);
watch(
  () => props.upgrade.starting_artisan,
  () => {
    starting_artisan.value = parse_locale_float(
      (props.upgrade.starting_artisan * 100).toFixed(2),
    );
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

const current_chance_percentage = ref(
  ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
    props.upgrade.base_chance +
    props.upgrade.base_chance) *
    100 || 0,
);
watch(
  () => props.upgrade.starting_num_taps,
  () => {
    current_chance_percentage.value =
      ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
        props.upgrade.base_chance +
        props.upgrade.base_chance) *
      100;
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

function write_normal_progress() {
  taps_since_last_input.value = 0;
  starting_artisan.value = clamp_percentage(starting_artisan.value);
  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].starting_artisan = starting_artisan.value / 100;

  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].starting_num_taps = current_chance_to_num_taps.value;

  start_all_workers();
}
const current_chance_to_num_taps = computed(() => {
  return Math.round(
    ((clean_percentage_input(
      current_chance_percentage.value,
      props.upgrade.base_chance * 100,
    ) /
      100 -
      props.upgrade.base_chance) /
      props.upgrade.base_chance) *
      10,
  );
});

function clean_artisan() {
  taps_since_last_input.value = 0;
  using_slider.value = false;
  starting_artisan.value = clean_percentage_input(starting_artisan.value, 0);
}

function clean_chance(event) {
  console.log(event.target.value, event, current_chance_percentage.value);
  taps_since_last_input.value = 0;
  using_slider.value = false;

  current_chance_percentage.value = clean_percentage_input(
    clamp(
      100 * props.upgrade.base_chance,
      parse_locale_float(
        (
          ((current_chance_to_num_taps.value / 10) * props.upgrade.base_chance +
            props.upgrade.base_chance) *
          100
        ).toFixed(2),
      ),

      100 * props.upgrade.base_chance * 2,
    ),
    props.upgrade.base_chance * 100,
  );
}

const taps_since_last_input = ref(0);

const optimizer_working = computed(
  () => active_profile.value.optimizer_worker_bundle.status === "busy",
);
const using_slider = ref(true);
const expanded = ref(
  props.upgrade.starting_num_taps > 0 || // SHOULDN"T need to check this but technically the user can put in starting num 0 and some non-zero artisan so yea why not
    props.upgrade.starting_artisan > 0 ||
    props.perform_order == 0,
);
watch(
  () => [
    props.upgrade.starting_num_taps,
    props.upgrade.starting_artisan,
    props.perform_order,
  ],
  () => {
    expanded.value =
      props.upgrade.starting_num_taps > 0 ||
      props.upgrade.starting_artisan > 0 ||
      props.perform_order == 0;
  },
);
</script>
<template>
  <div class="flex w-full flex-col px-3">
    <div v-if="!expanded" class="contents">
      <button
        @click="() => (expanded = true)"
        class="generic-button"
        :style="{
          opacity: perform_order == 0 && !free_tap_this_upgrade ? 1 : 0.5,
        }"
      >
        Expand Artisan input
      </button>
      <!-- <span v-if="perform_order != 0" class="annotation"
        >You should do the upgrades from top to bottom.</span
      > -->
    </div>
    <div v-if="expanded" class="contents">
      <div
        class="flex min-w-full flex-row flex-nowrap justify-between"
        :style="{ opacity: using_slider ? 1 : 0.5 }"
      >
        <!-- min-w-32.75 is exact amount needed for fitting '219'  -->
        <div class="flex max-w-32.75 min-w-32.75 flex-col">
          <span class="w-full text-left text-nowrap"> Taps since last </span>
          <span class="flex w-full flex-row text-left text-nowrap">
            optimizer run:
            <input
              class="ml-1 w-full border-b border-(--border-muted)"
              v-model="taps_since_last_input"
            />
          </span>
        </div>
        <input
          v-if="!optimizer_working"
          v-model="taps_since_last_input"
          class="h w-45"
          :min="0"
          :max="upgrade.normal_dist.length - 1"
          type="range"
          @input="
            () => {
              using_slider = true;
              starting_artisan = taps_since_last_input;
            }
          "
        />
        <div
          v-if="!optimizer_working"
          class="question-mark min-w-fit shrink-0"
          v-tooltip.left="
            'The slider assumes that you followed the Juice & Book Instructions. If you didn\'t follow the instructions, input your artisan and stuff directly below instead.'
          "
        ></div>
      </div>
      <div class="flex w-full flex-row content-center justify-between">
        <div class="flex flex-col justify-center">
          <div class="flex flex-row flex-nowrap justify-start gap-2">
            <span class="w-fit"> Current artisan energy: </span>

            <div class="flex flex-row">
              <input
                class="generic-input w-13 border-transparent! border-b-(--border-very-muted)!"
                :style="{
                  backgroundColor: using_slider
                    ? 'transparent'
                    : 'var(--bg-bright)',
                }"
                v-model="starting_artisan"
                inputmode="decimal"
                :disabled="optimizer_working"
                @change="clean_artisan"
              />
            </div>
          </div>
          <div class="flex flex-row flex-nowrap justify-start gap-2">
            <span class="w-fit"> Current base chance: </span>

            <div class="flex flex-row">
              <input
                class="generic-input w-10 border-transparent! border-b-(--border-very-muted)!"
                v-model="current_chance_percentage"
                :min="upgrade.base_chance * 100"
                :max="upgrade.base_chance * 100 * 2"
                @change="clean_chance"
                :disabled="optimizer_working"
                inputmode="decimal"
                :style="{
                  backgroundColor: using_slider
                    ? 'transparent'
                    : 'var(--bg-bright)',
                }"
              />
              <span>%</span>
            </div>
          </div>
        </div>
        <div v-if="!optimizer_working">
          <button
            @click="write_normal_progress"
            class="generic-button max-w-20! self-end! text-wrap! text-(--gold)!"
          >
            Confirm & re-run optimizer
          </button>
        </div>
        <div v-if="optimizer_working" class="h-19.5 max-w-20 text-wrap">
          Optimizer working...
        </div>
      </div>
    </div>
  </div>
</template>
