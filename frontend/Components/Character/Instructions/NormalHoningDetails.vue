<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { DEFAULT_ARTISAN_MULTIPLIER, FLOAT_TOL } from "@/Utils/Constants";
import {
  clamp,
  clamp_percentage,
  clean_percentage_input,
} from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";

import { storeToRefs } from "pinia";

import { computed, ref, watch } from "vue";
import { start_all_workers, start_eval_hist } from "../CharWorkerUtils";
import { artisan_function, cumulative_chance } from "@/Utils/HoningUtil";

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
  (props.upgrade.starting_artisan * 100).toFixed(2) || "0.00",
);
watch(
  () => props.upgrade.starting_artisan,
  () => {
    starting_artisan.value = (props.upgrade.starting_artisan * 100).toFixed(2);
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

const current_chance_percentage = ref(
  (
    ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
      props.upgrade.base_chance +
      props.upgrade.base_chance) *
    100
  ).toFixed(2) || "0.00",
);
watch(
  () => props.upgrade.starting_num_taps,
  () => {
    current_chance_percentage.value = (
      ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
        props.upgrade.base_chance +
        props.upgrade.base_chance) *
      100
    ).toFixed(2);
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

function write_normal_progress() {
  starting_artisan.value = clamp_percentage(starting_artisan.value);
  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].starting_artisan = parse_locale_float(starting_artisan.value) / 100;

  active_profile.value.keyed_upgrades[
    to_upgrade_key(
      props.upgrade.piece_type,
      props.upgrade.upgrade_index,
      props.upgrade.is_normal_honing,
      active_profile.value.tier,
    )
  ].starting_num_taps = using_slider.value
    ? taps_since_last_input.value
    : current_chance_to_num_taps.value;
}
const current_chance_to_num_taps = computed(() => {
  return Math.round(
    (parse_locale_float(
      clean_percentage_input(
        current_chance_percentage.value,
        props.upgrade.base_chance * 100,
      ),
    ) /
      (100 - props.upgrade.base_chance) /
      props.upgrade.base_chance) *
      10,
  );
});

function manual_artisan_change() {
  taps_since_last_input.value = 0;
  using_slider.value = false;
  starting_artisan.value = clean_percentage_input(starting_artisan.value, 0);
  write_normal_progress();
  start_eval_hist();
}

function manual_chance_change(event) {
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
    ).toFixed(2),
    props.upgrade.base_chance * 100,
  );
  write_normal_progress();
  start_eval_hist();
}

function slider_input() {
  using_slider.value = true;

  // this kind of disallows any non-integer inputs by instantly setting invalid intermediate inputs to 0, but its like fine i think since it's not a big number or anything
  taps_since_last_input.value = Number(taps_since_last_input.value);

  starting_artisan.value = artisan_function(
    props.upgrade,
    Number(taps_since_last_input.value) +
      Number(props.upgrade.starting_num_taps),
    juice_info.value,
  );
  current_chance_percentage.value = (
    100 *
    (props.upgrade.base_chance +
      0.1 *
        props.upgrade.base_chance *
        Math.min(
          10,
          taps_since_last_input.value + props.upgrade.starting_num_taps,
        ))
  ).toFixed(2);
  write_normal_progress();
  start_eval_hist();
}

function slider_change() {
  write_normal_progress();
  start_eval_hist();
}

function confirm() {
  taps_since_last_input.value = 0;
  start_all_workers();
}
const taps_since_last_input = ref(0);

const optimizer_working = computed(
  () => active_profile.value.optimizer_worker_bundle.status === "busy",
);
const using_slider = ref(true);
const expanded = ref(
  props.upgrade.starting_num_taps > 0 || // SHOULDN"T need to check this but technically the user can put in starting num 0 and some non-zero artisan so yea why not
    props.upgrade.starting_artisan > 0,
);
watch(
  () => [
    props.upgrade.starting_num_taps,
    props.upgrade.starting_artisan,
    props.perform_order,
  ],
  () => {
    expanded.value =
      props.upgrade.starting_num_taps > 0 || props.upgrade.starting_artisan > 0;
  },
);
const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const should_click = computed(
  () =>
    props.perform_order == 0 &&
    !props.free_tap_this_upgrade &&
    !optimizer_working.value,
);
</script>
<template>
  <div class="flex w-full flex-col px-3">
    <div v-if="!expanded" class="contents">
      <button
        @click="
          () => {
            if (!optimizer_working) {
              expanded = true;
            }
          }
        "
        class="barebone-button"
        :style="{
          '--btn-hover-bg': should_click
            ? 'var(--bg-very-bright)'
            : 'var(--bg-medium)',
          color: optimizer_working
            ? 'var(--warning-dark)'
            : should_click
              ? 'var(--text-main)'
              : 'var(--dont-click)',
          cursor: optimizer_working ? 'not-allowed' : 'pointer',
        }"
      >
        Expand Artisan input
      </button>

      <div
        class="flex w-fit flex-col content-start self-center text-left"
        v-if="!optimizer_working"
      >
        <span
          v-if="perform_order != 0"
          class="annotation"
          :style="{
            color: should_click ? 'var(--text-main)' : 'var(--dont-click)',
          }"
          >You should do the upgrades above this first</span
        >
        <span
          v-if="free_tap_this_upgrade"
          class="annotation"
          :style="{
            color: should_click ? 'var(--text-main)' : 'var(--dont-click)',
          }"
          >You should attempt
          <span class="text-(--free-tap)">free taps</span> on this first</span
        >
        <span
          v-if="perform_order != 0 || free_tap_this_upgrade"
          class="annotation"
          :style="{
            color: should_click ? 'var(--text-main)' : 'var(--dont-click)',
          }"
        >
        </span>
      </div>
      <span v-else class="text-(--warning-dark)">
        Optimizer working ({{
          active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(
            2,
          )
        }}%)</span
      >
    </div>

    <div v-if="expanded" class="contents">
      <div
        class="flex min-w-full flex-row flex-nowrap justify-between"
        :style="{ opacity: using_slider ? 1 : 0.5 }"
      >
        <!-- min-w-32.75 is exact amount needed for fitting '219'  -->
        <div class="flex max-w-37 min-w-37 flex-col">
          <span class="w-full text-left text-nowrap"> Taps since last </span>
          <span class="flex w-full flex-row text-left text-nowrap">
            optimizer run:
            <input
              v-if="using_slider"
              class="ml-1 w-full border-b border-(--border-muted)"
              v-model="taps_since_last_input"
              type="number"
              @input="slider_input"
              @change="slider_change"
            />
            <span v-else class="mb-px ml-1">N/A</span>
            <!-- margin bottom here to match the input's height -->
          </span>
        </div>
        <input
          v-if="!optimizer_working"
          v-model="taps_since_last_input"
          class="h w-45"
          :min="0"
          :max="upgrade.normal_dist.length - 1"
          type="range"
          @input="slider_input"
          @change="slider_change"
        />
        <div
          v-if="!optimizer_working"
          class="question-mark min-w-fit shrink-0"
          v-tooltip.left="
            'The slider assumes that you followed the Juice & Book Instructions. If you didn\'t follow the instructions, input your artisan and stuff directly below instead.'
          "
        ></div>
      </div>
      <div class="grid grid-cols-[1fr_max-content]">
        <div class="grid h-fit grid-cols-[1fr_90px]">
          <div class="col-span-2 grid h-fit grid-cols-subgrid">
            <span class="text-right"> Current artisan energy: </span>
            <div class="pl-2">
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
                @change="manual_artisan_change"
              />
            </div>
          </div>
          <div class="col-span-2 grid h-fit grid-cols-subgrid">
            <span class="text-right"> Current base chance: </span>
            <div class="flex flex-row pl-2">
              <input
                class="generic-input w-10 border-transparent! border-b-(--border-very-muted)! pr-0!"
                v-model="current_chance_percentage"
                :min="upgrade.base_chance * 100"
                :max="upgrade.base_chance * 100 * 2"
                @change="manual_chance_change"
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
          <div
            class="col-span-2 grid h-fit grid-cols-subgrid text-(--text-muted)"
          >
            <span class="text-right"> Cumulative chance: </span>
            <div class="flex flex-row content-start pl-2">
              <span class="generic-input border-0!">
                {{
                  using_slider
                    ? cumulative_chance(
                        upgrade,
                        Number(taps_since_last_input) +
                          Number(upgrade.starting_num_taps),
                        juice_info,
                      )
                    : "N/A"
                }}</span
              >
              <span>%</span>
            </div>
          </div>
        </div>
        <div v-if="!optimizer_working">
          <button
            @click="confirm"
            class="generic-button max-w-20! self-end! py-1! text-base/tight text-wrap! text-(--gold)!"
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
