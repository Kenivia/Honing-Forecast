<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  clamp,
  clamp_percentage,
  clean_percentage_input,
  locale_to_fixed,
} from "@/Utils/Helpers";
import { parse_locale_float, parse_locale_int } from "@/Utils/InputColumn";
import { to_upgrade_key, Upgrade } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import { artisan_number, artisan_string } from "@/Utils/HoningUtil";
import "@/Components/Character/Instructions/Details/details.css";
import { get_optimizer_working } from "@/Components/Character/Instructions/InstructionUtils";
import {
  compute_used_materials,
  mark_upgrade_as_done,
  apply_remaining_mats,
} from "./SuccessUtils";
import { FLOAT_TOL } from "@/Utils/Constants";
import { GridConfig } from "@/Utils/GridStyling";
import ManualArtisanInput from "@/Components/Character/Instructions/Details/ManualArtisanInput.vue";

const { active_profile, roster_config } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  is_free_tap: boolean;
  index_in_special_state?: number;
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const optimizer_working = computed(get_optimizer_working);
const tier = computed(() => active_profile.value.tier);
const this_keyed = computed(
  () => active_profile.value.keyed_upgrades[upgrade_key.value],
);

const taps_since_last_input = ref(0);
const using_slider = ref(true);

const upgrade_key = computed(() =>
  to_upgrade_key(
    props.upgrade.piece_type,
    props.upgrade.upgrade_index,
    props.upgrade.is_normal_honing,
    active_profile.value.tier,
  ),
);

const starting_artisan_string = ref(
  locale_to_fixed(props.upgrade.starting_artisan * 100, 2, true) || "0.00",
);

const starting_artisan_number = ref(props.upgrade.starting_artisan);

const current_chance_percentage = ref(
  locale_to_fixed(
    ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
      props.upgrade.base_chance +
      props.upgrade.base_chance +
      props.upgrade.extra_chance) *
      100,
    2,
  ),
);

const current_chance_to_num_taps = computed(() => {
  return Math.round(
    (parse_locale_float(
      clean_percentage_input(
        current_chance_percentage.value,
        props.upgrade.base_chance * 100 + props.upgrade.extra_chance * 100,
      ),
    ) -
      props.upgrade.base_chance * 100 -
      props.upgrade.extra_chance * 100) /
      (props.upgrade.base_chance * 10),
  );
});

watch(
  () => props.upgrade.starting_artisan,
  () => {
    starting_artisan_string.value = locale_to_fixed(
      props.upgrade.starting_artisan * 100,
      2,
      true,
    );

    starting_artisan_number.value = props.upgrade.starting_artisan;
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

watch(
  [() => props.upgrade.starting_num_taps, () => props.upgrade.extra_chance],
  () => {
    current_chance_percentage.value = locale_to_fixed(
      ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
        props.upgrade.base_chance +
        props.upgrade.base_chance +
        props.upgrade.extra_chance) *
        100,
      2,
    );
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes (i think it might not be necesary? idk)
);

///////////////////////////////////////////////////////////////////////////////////////////
// watchers triggered whenever optimizer runs (to reset)
///////////////////////////////////////////////////////////////////////////////////////////

function clear_used_budget() {
  this_keyed.value.used_materials = null;
}

function post_opt_run_reset() {
  taps_since_last_input.value = 0;
  using_slider.value = true;
  this_keyed.value.taps_since_last_input = 0;
  roster_config.value.budget_snapshot = null;
  clear_used_budget();
}

watch(
  () => active_profile.value.optimizer_worker_bundle.run_counter,
  () => {
    // only do this as we start the optimizer,  shouldn't really matter but whatever
    if (optimizer_working.value) {
      post_opt_run_reset();
    }
  },
  { deep: true },
);

// changes in bound budgets will start the optimizer so it'll be gucci
// this is for when autostart is turned off only
watch(
  [
    () => active_profile.value.bound_budgets[tier.value].data,
    () => active_profile.value.bound_budgets[tier.value].enabled,
  ],
  () => {
    if (!active_profile.value.auto_start_optimizer) {
      if (!roster_config.value.is_details_update) {
        post_opt_run_reset();
      }
    }
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

  this_keyed.value.taps_since_last_input = taps_since_last_input.value; // which is 0 when using_slice is false
}

function set_used_materials(pretend_zero_no_unlock: boolean) {
  this_keyed.value.used_materials = compute_used_materials(
    props.upgrade,
    taps_since_last_input.value,
    juice_info.value,
    0,
    0,
    pretend_zero_no_unlock,
  );
}

function change_wrapper(
  slider: boolean,
  func: () => void,
  worker_func = start_eval_hist,
) {
  roster_config.value.is_details_update = true;

  taps_since_last_input.value = slider
    ? clamp(
        0,
        Number(taps_since_last_input.value),
        props.upgrade.normal_dist.length - 1,
      ) // this kind of disallows any non-integer inputs by instantly setting invalid intermediate inputs to 0, but its like fine i think since it's not a big number or anything
    : 0;

  using_slider.value = slider;
  if (slider) {
    set_used_materials(true);
  } else {
    clear_used_budget();
  }

  apply_remaining_mats();

  func();

  write_normal_progress();
  worker_func();
  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}
function manual_artisan_change() {
  change_wrapper(false, () => {
    starting_artisan_string.value = clean_percentage_input(
      starting_artisan_string.value,
      0,
      true,
    );
    starting_artisan_number.value = parse_locale_float(
      starting_artisan_string.value,
    );
    if (starting_artisan_number.value > FLOAT_TOL) {
      // for the topmost upgrade, which may have expanded false still (and might get sorted down )
      active_profile.value.keyed_upgrades[upgrade_key.value].expanded = true;
    }
  });
}

function manual_chance_change() {
  change_wrapper(false, () => {
    current_chance_percentage.value = clean_percentage_input(
      locale_to_fixed(
        clamp(
          100 * props.upgrade.base_chance + props.upgrade.extra_chance * 100,
          parse_locale_float(
            locale_to_fixed(
              ((current_chance_to_num_taps.value / 10) *
                props.upgrade.base_chance +
                props.upgrade.base_chance) *
                100,
              2,
            ),
          ),
          100 * props.upgrade.base_chance * 2 +
            props.upgrade.extra_chance * 100,
        ),
        2,
      ),
      props.upgrade.base_chance * 100 + props.upgrade.extra_chance * 100,
    );
    if (
      parse_locale_float(current_chance_percentage.value) >
      props.upgrade.base_chance * 100 + props.upgrade.extra_chance * 100
    ) {
      // for the topmost upgrade, which may have expanded false still (and might get sorted down )
      active_profile.value.keyed_upgrades[upgrade_key.value].expanded = true;
    }
  });
}

function confirm() {
  start_all_workers();
  post_opt_run_reset(); // this is explicitly here because if it's already running then we need to trigger it manunally
}

function slider_input() {
  change_wrapper(true, () => {
    if (taps_since_last_input.value > 0) {
      // for the topmost upgrade, which may have expanded false still (and might get sorted down )
      active_profile.value.keyed_upgrades[upgrade_key.value].expanded = true;
    }
    starting_artisan_string.value = artisan_string(
      props.upgrade,
      Number(taps_since_last_input.value),
      juice_info.value,
    );
    starting_artisan_number.value = artisan_number(
      props.upgrade,
      Number(taps_since_last_input.value),
      juice_info.value,
    );

    current_chance_percentage.value = locale_to_fixed(
      100 *
        (props.upgrade.base_chance +
          props.upgrade.extra_chance +
          0.1 *
            props.upgrade.base_chance *
            Math.min(
              10,
              taps_since_last_input.value + props.upgrade.starting_num_taps,
            )),
      2,
    );
  });
}

function reset() {
  change_wrapper(
    false,
    () => {
      starting_artisan_string.value = "0.00";
      starting_artisan_number.value = 0;
      current_chance_percentage.value = locale_to_fixed(
        props.upgrade.base_chance * 100 + props.upgrade.extra_chance * 100,
        2,
      );

      active_profile.value.keyed_upgrades[upgrade_key.value].state = []; // clearing state because like the previous entries aren't here anymore, whatever state we have is kinda bogus. keyed_to_array is special cased to not overwrite this
    },
    () => {
      using_slider.value = true;
      if (
        props.upgrade.starting_artisan > 0 ||
        props.upgrade.starting_num_taps > 0
      ) {
        start_all_workers(); // starting optimizer cos the instruction needs to change
      } else {
        start_eval_hist();
      }
    },
  );
}

// not gonna use change_wrapper cos its so different
function succeed_click() {
  roster_config.value.is_details_update = true;
  set_used_materials(false); // deduct unlock costs (just like free tap)
  apply_remaining_mats();

  post_opt_run_reset(); // THIS IS NEEDED BECAUSE this component is deleted when that happens and the watch statement wont catch it

  mark_upgrade_as_done(props.upgrade); // this will call optimizer
  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}

///////////////////////////////////////////////////////////////////////////////////////////
// free tap branch
///////////////////////////////////////////////////////////////////////////////////////////

const expand_free_tap_artisan = ref(
  props.upgrade.starting_artisan > 0 || props.upgrade.starting_num_taps > 0,
);
watch(
  () =>
    props.upgrade.starting_artisan > 0 || props.upgrade.starting_num_taps > 0,
  () => {
    expand_free_tap_artisan.value =
      props.upgrade.starting_artisan > 0 || props.upgrade.starting_num_taps > 0;
  },
);

const new_special_leaps = computed(() =>
  parse_locale_int(active_profile.value.special_budget.data[0]),
);

function special_succeed_click() {
  active_profile.value.special_budget.data[0] =
    new_special_leaps.value.toLocaleString();

  this_keyed.value.used_materials = compute_used_materials(
    props.upgrade,
    0,
    juice_info.value,
    0,
    0,
    false, // so this is just the unlock cost
  );
  apply_remaining_mats();
  mark_upgrade_as_done(props.upgrade);
}

function special_fail_click() {
  active_profile.value.special_budget.data[0] = "0";
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
        <div
          class="question-mark"
          v-tooltip.left="
            'Use the slider to indicate how many taps it took, then press Succeed. Costs are automatically deducted.'
          "
        />
      </div>

      <ManualArtisanInput
        v-model:starting_artisan="starting_artisan_string"
        v-model:current_chance_percentage="current_chance_percentage"
        :upgrade="upgrade"
        :optimizer_working="optimizer_working"
        :taps_since_last_input="taps_since_last_input"
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
        :max="parse_locale_int(active_profile.special_budget.data[0])"
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
          :optimizer_working="optimizer_working"
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
