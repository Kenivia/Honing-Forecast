<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  clamp,
  clamp_percentage,
  clean_percentage_input,
} from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed, nextTick, Ref, ref, watch } from "vue";
import { start_all_workers, start_eval_hist } from "../CharWorkerUtils";
import { artisan_function } from "@/Utils/HoningUtil";
import "./details.css";
import { get_optimizer_working } from "./InstructionUtils";
import {
  BudgetSnapshot,
  compute_remaininig_materials,
  compute_used_materials,
  RemainingMats,
  snapshot_budgets,
} from "./SuccessUtils";
import { StateBundle } from "@/WasmInterface/WasmWorker";
import { FLOAT_TOL } from "@/Utils/Constants";
import { GridConfig } from "@/Utils/GridStyling";

const {
  active_profile,
  active_roster_mats_owned,
  active_tradable_mats_owned,
  roster_config,
} = storeToRefs(useRosterStore());
const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  free_tap_this_upgrade: boolean;
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

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
  const this_keyed =
    active_profile.value.keyed_upgrades[
      to_upgrade_key(
        props.upgrade.piece_type,
        props.upgrade.upgrade_index,
        props.upgrade.is_normal_honing,
        active_profile.value.tier,
      )
    ];

  this_keyed.starting_artisan =
    parse_locale_float(starting_artisan.value) / 100;

  this_keyed.starting_num_taps = using_slider.value
    ? Math.min(
        taps_since_last_input.value + props.upgrade.starting_num_taps,
        10,
      )
    : current_chance_to_num_taps.value;

  this_keyed.taps_since_last_input = taps_since_last_input.value; // which is 0 when using_slice is false
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

function manual_chance_change() {
  // console.log(event.target.value, event, current_chance_percentage.value);
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

const previous_budgets: Ref<null | BudgetSnapshot> = ref(null);

const is_slider_update = ref(false);

function reset_after_optimizer_run() {
  taps_since_last_input.value = 0;
  using_slider.value = true;
  previous_budgets.value = null;
  const this_keyed =
    active_profile.value.keyed_upgrades[
      to_upgrade_key(
        props.upgrade.piece_type,
        props.upgrade.upgrade_index,
        props.upgrade.is_normal_honing,
        active_profile.value.tier,
      )
    ];

  this_keyed.taps_since_last_input = 0;
}
watch(
  [
    () => active_profile.value.bound_budgets[active_profile.value.tier].data,
    () => active_profile.value.bound_budgets[active_profile.value.tier].enabled,
  ],
  () => {
    if (!is_slider_update.value) {
      reset_after_optimizer_run();
      // console.log("non-slider change");
    }
    // console.log("change");
  },
  { deep: true },
);
const used_materials = computed(() =>
  compute_used_materials(
    props.upgrade,
    taps_since_last_input.value,
    juice_info.value,
    0,
    0,
  ),
);
const tier = computed(() => active_profile.value.tier);
function slider_input() {
  is_slider_update.value = true;
  using_slider.value = true;

  // this kind of disallows any non-integer inputs by instantly setting invalid intermediate inputs to 0, but its like fine i think since it's not a big number or anything
  taps_since_last_input.value = clamp(
    0,
    Number(taps_since_last_input.value),
    props.upgrade.normal_dist.length - 1,
  );

  starting_artisan.value = artisan_function(
    props.upgrade,
    Number(taps_since_last_input.value),
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

  if (previous_budgets.value === null) {
    previous_budgets.value = snapshot_budgets();
    // console.log("snap", toRaw(previous_budgets.value.bound_budgets[0].data));
  }
  // console.log(
  //   "calc",
  //   props.upgrade.starting_num_taps,
  //   taps_since_last_input.value,
  //   toRaw(previous_budgets.value.bound_budgets[0].data),
  // );
  const remaining_materials: RemainingMats = compute_remaininig_materials(
    used_materials.value,
    previous_budgets.value,
  );

  // console.log(used_materials, remaining_materials.bound_budgets);

  used_materials.value.forEach((_, index) => {
    if (active_profile.value.bound_budgets[tier.value].enabled[index]) {
      active_profile.value.bound_budgets[tier.value].data[index] =
        remaining_materials.bound_budgets[index].toLocaleString();
    }
    if (active_roster_mats_owned.value[tier.value].enabled[index]) {
      active_roster_mats_owned.value[tier.value].data[index] =
        remaining_materials.roster_mats[index].toLocaleString();
    }
    if (active_tradable_mats_owned.value[tier.value].enabled[index]) {
      active_tradable_mats_owned.value[tier.value].data[index] =
        remaining_materials.tradable_mats[index].toLocaleString();
    }
  });

  write_normal_progress();
  start_eval_hist();
  nextTick(() => {
    is_slider_update.value = false;
  });
}

function slider_change() {
  // write_normal_progress();
  // start_eval_hist();
}

function confirm() {
  start_all_workers();
  reset_after_optimizer_run();
}
const taps_since_last_input = ref(0);

const optimizer_working = computed(get_optimizer_working);
const using_slider = ref(true);

const should_click = computed(
  () =>
    props.perform_order == 0 &&
    !props.free_tap_this_upgrade &&
    !optimizer_working.value,
);
function reset() {
  if (previous_budgets.value !== null) {
    active_profile.value.bound_budgets = previous_budgets.value.bound_budgets;
    roster_config.value.roster_mats_owned[active_profile.value.roster_id] =
      previous_budgets.value.roster_mats;
    roster_config.value.tradable_mats_owned[active_profile.value.roster_id] =
      previous_budgets.value.tradable_mats;
  }

  taps_since_last_input.value = 0;
  starting_artisan.value = "0";
  current_chance_percentage.value = (props.upgrade.base_chance * 100).toFixed(
    2,
  );
  using_slider.value = false; // to force it to use the current_chance route
  write_normal_progress();
  using_slider.value = true;
  if (
    props.upgrade.starting_artisan > 0 ||
    props.upgrade.starting_num_taps > 0
  ) {
    start_all_workers(); // starting optimizer cos the instruction needs to change
  }
}

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
const grid: GridConfig = {
  grid_template_columns: "175px 100px 120px",
};
</script>
<template>
  <div class="contents">
    <div v-if="free_tap_this_upgrade && !actual_expanded" />
    <!-- ^ this is a placeholder for  actualinstructions when it's a freetap -->

    <div
      v-if="!actual_expanded"
      class="flex w-full min-w-98.75 flex-col items-center"
    >
      <!-- <span
        v-if="free_tap_this_upgrade && perform_order == 0"
        class="text-nowrap text-(--text-muted)"
        :style="{ color: 'var(--dont-click)' }"
        >You should attempt <span class="text-(--free-tap)">free taps</span> on
        this before normal honing</span
      > -->
      <span
        v-if="perform_order != 0"
        class="text-nowrap text-(--text-muted)"
        :style="{ color: 'var(--dont-click)' }"
        >You should do the upgrade{{ perform_order == 1 ? "" : "s" }} above
        first
      </span>
      <button
        @click="
          () => {
            active_profile.keyed_upgrades[upgrade_key].expanded = true;
          }
        "
        class="barebone-button w-fit"
        :style="{
          '--btn-hover-bg': should_click
            ? 'var(--bg-very-bright)'
            : 'var(--bg-main)',
          color: should_click ? 'var(--text-main)' : 'var(--dont-click)',
          cursor: 'pointer',
        }"
      >
        Show input anyway
      </button>
    </div>

    <div v-else class="contents">
      <div
        v-if="!free_tap_this_upgrade"
        class="grid w-full pl-5"
        :style="{
          cursor: optimizer_working ? 'not-allowed' : '',
          opacity: !optimizer_working ? 1 : 0.5,
          gridTemplateColumns: grid.grid_template_columns,
        }"
      >
        <div
          class="col-span-2 flex flex-row flex-nowrap justify-between gap-1"
          :style="{ opacity: using_slider ? 1 : 0.5 }"
        >
          <div class="flex max-w-37 min-w-37 flex-col">
            <span class="w-full text-left text-nowrap">Taps since last</span>
            <span class="flex w-full flex-row text-left text-nowrap">
              optimizer run:
              <input
                v-if="using_slider"
                class="ml-1 w-full border-b border-(--border-muted)"
                v-model="taps_since_last_input"
                type="number"
                @input="slider_input"
                @change="slider_change"
                :min="0"
                :max="upgrade.normal_dist.length - 1"
                :disabled="optimizer_working"
              />
              <span v-else class="mb-px ml-1">N/A</span>
            </span>
          </div>
          <input
            v-model="taps_since_last_input"
            class="w-full"
            :min="0"
            :max="upgrade.normal_dist.length - 1"
            type="range"
            @input="slider_input"
            @change="slider_change"
            :disabled="optimizer_working"
          />
        </div>

        <div class="button-row">
          <button
            class="generic-button w-20! text-(--achieved)!"
            :style="{
              color: optimizer_working
                ? 'var(--warning-dark)'
                : should_click
                  ? 'var(--text-main)'
                  : 'var(--dont-click)',
              cursor: optimizer_working ? 'not-allowed' : 'pointer',
              opacity: !optimizer_working ? 1 : 0.5,
            }"
          >
            Succeed
          </button>
          <div
            class="question-mark"
            v-tooltip.left="
              'Use the slider to indicate how many taps it took, then press Succeed. Costs are automatically deducted.'
            "
          />
        </div>
        <span class="stat-label">Current artisan energy:</span>
        <div class="stat-input">
          <input
            class="generic-input number-border w-13"
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
        <div class="button-row">
          <button @click="confirm" class="generic-button w-20! text-(--gold)!">
            Confirm
          </button>
          <div
            class="question-mark"
            v-tooltip.left="
              'Use the slider to update artisan & deduct costs, then press Confirm to re-run the optimizer. This may produce instructions that save slightly more gold (No need to do this after every tap, just when you feel like it).'
            "
          />
        </div>

        <span class="stat-label">Current base chance:</span>
        <div class="stat-input">
          <input
            class="generic-input number-border w-10 pr-0!"
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
        <div v-if="!optimizer_working" class="button-row">
          <button
            class="reset-button"
            @click="reset"
            :disabled="optimizer_working"
            v-if="
              upgrade.starting_artisan > 0 ||
              upgrade.starting_num_taps > 0 ||
              parse_locale_float(starting_artisan) > FLOAT_TOL ||
              Math.abs(
                parse_locale_float(current_chance_percentage) -
                  upgrade.base_chance * 100,
              ) > FLOAT_TOL
            "
            :style="{ cursor: optimizer_working ? 'not-allowed' : 'pointer' }"
          >
            Reset this upgrade
          </button>
          <div v-else class="contents">
            <button
              class="reset-button w-full self-center"
              v-if="perform_order != 0"
              @click="
                () =>
                  (active_profile.keyed_upgrades[upgrade_key].expanded = false)
              "
            >
              Hide
            </button>
          </div>
        </div>
      </div>
      <div v-else class="col-span-2">
        <div class="flex flex-row">
          <span>Succeeded with this many special leaps remaining:</span>
          <input class="generic-input w-25!" />
          <button
            class="generic-button w-20! text-(--achieved)!"
            :style="{
              color: optimizer_working
                ? 'var(--warning-dark)'
                : should_click
                  ? 'var(--text-main)'
                  : 'var(--dont-click)',
              cursor: optimizer_working ? 'not-allowed' : 'pointer',
              opacity: !optimizer_working ? 1 : 0.5,
            }"
          >
            Succeed
          </button>
        </div>
        <button
          class="generic-button text-(--free-tap)!"
          :style="{
            color: optimizer_working
              ? 'var(--warning-dark)'
              : should_click
                ? 'var(--text-main)'
                : 'var(--dont-click)',
            cursor: optimizer_working ? 'not-allowed' : 'pointer',
            opacity: props.perform_order == 0 && !optimizer_working ? 1 : 0.5,
          }"
        >
          All free taps failed
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stat-label {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  text-align: right;
}

.stat-input {
  display: flex;
  flex-direction: row;
  align-items: center;
  padding-left: 0.5rem;
}
.button-row {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-around;
}
</style>
