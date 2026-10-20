<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  clamp,
  clamp_percentage,
  clean_percentage_input,
  locale_to_fixed,
} from "@/Utils/Helpers";
import { parse_locale_float } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import {
  artisan_number,
  artisan_string,
  chance_percent,
  chance_to_taps,
} from "@/Utils/HoningUtil";
import "@/Components/Character/Instructions/Details/details.css";
import {
  clear_snapshot,
  compute_used_materials,
  deduct,
  finish_upgrade,
  set_special_remaining,
  special_total,
} from "./SuccessUtils";
import { FLOAT_TOL } from "@/Utils/Constants";
import { GridConfig } from "@/Utils/GridStyling";
import ManualArtisanInput from "@/Components/Character/Instructions/Details/ManualArtisanInput.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { get_optimizer_working } from "@/Components/Character/Instructions/InstructionUtils";
const runtime = useRuntimeStore();

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  is_free_tap: boolean;
  index_in_special_state?: number;
}>();

const juice_info = computed(() => runtime.histogram.result.juice_info);

const optimizer_working = computed(get_optimizer_working);
const upgrade_key = computed(() =>
  to_upgrade_key(
    props.upgrade.piece_index,
    props.upgrade.upgrade_index,
    props.upgrade.is_normal_honing,
    active_profile.value.tier,
  ),
);
const this_keyed = computed(
  () => active_profile.value.keyed_upgrades[upgrade_key.value],
);

const taps_since_last_input = ref(0);
const using_slider = ref(true);
const starting_artisan_string = ref("0.00");
const starting_artisan_number = ref(0);
const current_chance_percentage = ref("");
const chance_string = (failed: number) =>
  locale_to_fixed(chance_percent(props.upgrade, failed), 2);

// props.upgrade changes when the optimizer reorders or a tick changes
watch(
  () => props.upgrade.starting_artisan,
  (artisan) => {
    starting_artisan_string.value = locale_to_fixed(artisan * 100, 2, true);
    starting_artisan_number.value = artisan;
  },
  { immediate: true },
);
watch(
  [() => props.upgrade.starting_num_taps, () => props.upgrade.extra_chance],
  () => {
    current_chance_percentage.value = chance_string(
      props.upgrade.starting_num_taps,
    );
  },
  { immediate: true },
);

const current_chance_to_num_taps = computed(() =>
  chance_to_taps(
    props.upgrade,
    parse_locale_float(
      clean_percentage_input(
        current_chance_percentage.value,
        chance_percent(props.upgrade, 0),
      ),
    ),
  ),
);

// reset as the optimizer starts, and on a budget edit when auto start is off
function post_opt_run_reset() {
  taps_since_last_input.value = 0;
  using_slider.value = true;
  this_keyed.value.taps_since_last_input = 0;
  this_keyed.value.used_materials = null;
  clear_snapshot();
}
watch(
  () => runtime.optimizer.run_counter,
  () => {
    if (optimizer_working.value) post_opt_run_reset();
  },
);
watch(
  [
    () => active_profile.value.bound_budgets.values,
    () => active_profile.value.bound_budgets.enabled,
  ],
  () => {
    if (
      !active_profile.value.auto_start_optimizer &&
      !runtime.is_details_update
    )
      post_opt_run_reset();
  },
  { deep: true },
);

///////////////////////////////////////////////////////////////////////////////////////////
// non-freetap branch
///////////////////////////////////////////////////////////////////////////////////////////

function write_normal_progress() {
  starting_artisan_number.value = clamp(0, starting_artisan_number.value, 1);
  starting_artisan_string.value = clamp_percentage(
    (starting_artisan_number.value * 100).toLocaleString(),
    true,
  );

  this_keyed.value.starting_artisan = starting_artisan_number.value;
  this_keyed.value.starting_num_taps = using_slider.value
    ? Math.min(
        taps_since_last_input.value + props.upgrade.starting_num_taps,
        10,
      )
    : current_chance_to_num_taps.value;
  this_keyed.value.taps_since_last_input = taps_since_last_input.value; // 0 off the slider
}

function change_wrapper(
  slider: boolean,
  func: () => void,
  worker_func = start_eval_hist,
) {
  runtime.is_details_update = true;
  // non-integer intermediate inputs become 0
  taps_since_last_input.value = slider
    ? clamp(
        0,
        Number(taps_since_last_input.value),
        props.upgrade.normal_dist.length - 1,
      )
    : 0;
  using_slider.value = slider;
  deduct(
    this_keyed.value,
    slider
      ? compute_used_materials(
          props.upgrade,
          taps_since_last_input.value,
          juice_info.value,
          true,
          active_profile.value.tier,
        )
      : null,
  );
  func();
  write_normal_progress();
  worker_func();
  nextTick(() => {
    runtime.is_details_update = false;
  });
}

// the topmost upgrade may still be collapsed, and may get sorted down
const expand = () => (this_keyed.value.expanded = true);

function manual_artisan_change() {
  change_wrapper(false, () => {
    starting_artisan_string.value = clean_percentage_input(
      starting_artisan_string.value,
      0,
      true,
    );
    starting_artisan_number.value =
      parse_locale_float(starting_artisan_string.value) / 100;
    if (starting_artisan_number.value > FLOAT_TOL) expand();
  });
}

function manual_chance_change() {
  change_wrapper(false, () => {
    const failed = current_chance_to_num_taps.value;
    current_chance_percentage.value = chance_string(failed);
    if (failed > 0) expand();
  });
}

function confirm() {
  start_all_workers();
  post_opt_run_reset(); // a run already going would not tick run_counter
}

function slider_input() {
  change_wrapper(true, () => {
    const taps = taps_since_last_input.value;
    if (taps > 0) expand();
    starting_artisan_string.value = artisan_string(
      props.upgrade,
      taps,
      juice_info.value,
    );
    starting_artisan_number.value = artisan_number(
      props.upgrade,
      taps,
      juice_info.value,
    );
    current_chance_percentage.value = chance_string(
      taps + props.upgrade.starting_num_taps,
    );
  });
}

function reset() {
  change_wrapper(
    false,
    () => {
      starting_artisan_string.value = "0.00";
      starting_artisan_number.value = 0;
      current_chance_percentage.value = chance_string(0);
      // the old state no longer fits; keyed_to_array is special cased to not overwrite this
      this_keyed.value.state = [];
    },
    () => {
      using_slider.value = true;
      if (
        props.upgrade.starting_artisan > 0 ||
        props.upgrade.starting_num_taps > 0
      ) {
        start_all_workers(); // the instructions need to change
      } else {
        start_eval_hist();
      }
    },
  );
}

function succeed_click() {
  finish_upgrade(
    props.upgrade,
    Number(taps_since_last_input.value),
    juice_info.value,
  );
}

///////////////////////////////////////////////////////////////////////////////////////////
// free tap branch
///////////////////////////////////////////////////////////////////////////////////////////

const expand_free_tap_artisan = ref(false);
watch(
  () =>
    props.upgrade.starting_artisan > 0 || props.upgrade.starting_num_taps > 0,
  (started) => (expand_free_tap_artisan.value = started),
  { immediate: true },
);

// an empty input leaves the free taps as they are
const new_special_leaps = ref<number | "">(0);
watch(special_total, (total) => (new_special_leaps.value = total), {
  immediate: true,
});

function special_succeed_click() {
  if (new_special_leaps.value !== "")
    set_special_remaining(Number(new_special_leaps.value));
  finish_upgrade(props.upgrade, 0, juice_info.value); // just the unlock
}

function special_fail_click() {
  set_special_remaining(0);
  start_all_workers();
}

const non_special_grid: GridConfig = {
  grid_template_columns: "175px 100px 120px",
};

const special_grid: GridConfig = {
  grid_template_columns: "1fr 65px 100px ",
};
</script>

<template>
  <div
    :class="is_free_tap ? 'col-span-2 grid' : 'grid w-full pl-5'"
    :style="{
      gridTemplateColumns: is_free_tap
        ? special_grid.grid_template_columns
        : non_special_grid.grid_template_columns,
    }"
  >
    <template v-if="!is_free_tap">
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
              :min="0"
              :max="upgrade.normal_dist.length - 1"
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
        />
      </div>

      <div class="button-row">
        <button
          class="generic-button w-20! text-(--achieved)!"
          @click="succeed_click"
        >
          Succeed
        </button>
        <QuestionMark
          text="Use the slider to indicate how many taps it took, then press Succeed. Costs are automatically deducted."
          position="left"
        />
      </div>

      <ManualArtisanInput
        v-model:starting_artisan="starting_artisan_string"
        v-model:current_chance_percentage="current_chance_percentage"
        :upgrade="upgrade"
        :taps_since_last_input="Number(taps_since_last_input)"
        :normal_dist_length="upgrade.normal_dist.length"
        :using_slider="using_slider"
        :show_hints="true"
        :perform_order="perform_order"
        :show_hide="true"
        @manual_artisan_change="manual_artisan_change"
        @manual_chance_change="manual_chance_change"
        @confirm="confirm"
        @reset="reset"
      />
    </template>

    <template v-else>
      <span class="self-center pr-1 text-right">Special leaps remaining:</span>
      <input
        class="generic-input w-18!"
        v-model="new_special_leaps"
        type="number"
        :min="0"
        :max="special_total()"
      />

      <button
        class="generic-button button-row w-20! text-(--achieved)!"
        @click="special_succeed_click"
      >
        Succeed
      </button>

      <span class="self-center pr-1 text-right"
        >({{ locale_to_fixed(upgrade.this_special_chance * 100, 2) }}% chance to
        succeed{{
          index_in_special_state === 0 ? "" : " this upgrade AND the above"
        }})
      </span>
      <button
        class="generic-button col-span-2 mr-5 text-(--free-tap)!"
        @click="special_fail_click"
      >
        All free taps failed
      </button>

      <button
        class="reset-button min-w-full pt-2! text-right"
        v-if="perform_order != 0"
        @click="active_profile.keyed_upgrades[upgrade_key].expanded = false"
      >
        Hide
      </button>
      <div v-else>
        <!-- placeholder to make the other button go in the right place -->
      </div>
      <button
        class="annotation col-span-2 pt-2 underline"
        @click="
          () => {
            expand_free_tap_artisan = !expand_free_tap_artisan;
          }
        "
      >
        {{
          expand_free_tap_artisan ? "Hide Artisan input" : "Show artisan input"
        }}
      </button>

      <div
        v-if="expand_free_tap_artisan"
        class="col-span-3 mt-1 grid w-max justify-self-end pl-5"
        :style="{
          gridTemplateColumns: special_grid.grid_template_columns,
        }"
      >
        <ManualArtisanInput
          v-model:starting_artisan="starting_artisan_string"
          v-model:current_chance_percentage="current_chance_percentage"
          :upgrade="upgrade"
          :taps_since_last_input="taps_since_last_input"
          :normal_dist_length="upgrade.normal_dist.length"
          :using_slider="using_slider"
          :show_hints="false"
          :perform_order="perform_order"
          :show_hide="false"
          @manual_artisan_change="manual_artisan_change"
          @manual_chance_change="manual_chance_change"
          @confirm="confirm"
          @reset="reset"
        />
      </div>
    </template>
  </div>
</template>
