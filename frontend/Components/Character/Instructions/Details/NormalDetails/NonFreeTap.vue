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
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import { artisan_function } from "@/Utils/HoningUtil";
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
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const optimizer_working = computed(get_optimizer_working);

const starting_artisan = ref(
  locale_to_fixed(props.upgrade.starting_artisan * 100, 2, true) || "0.00",
);
watch(
  () => props.upgrade.starting_artisan,
  () => {
    starting_artisan.value = locale_to_fixed(
      props.upgrade.starting_artisan * 100,
      2,
      true,
    );
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

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
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
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

const tier = computed(() => active_profile.value.tier);

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

function write_normal_progress() {
  starting_artisan.value = clamp_percentage(starting_artisan.value);
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];

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

function clear_used_budget() {
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];
  this_keyed.used_materials = null;
}

function manual_artisan_change() {
  roster_config.value.is_details_update = true;
  // console.log(using_slider.value);
  taps_since_last_input.value = 0;
  using_slider.value = false;
  // console.log(using_slider.value);
  clear_used_budget();

  starting_artisan.value = clean_percentage_input(
    starting_artisan.value,
    0,
    true,
  );
  if (parse_locale_float(starting_artisan.value) > FLOAT_TOL) {
    // for the topmost upgrade, which may have expanded false still (and might get sorted down )
    active_profile.value.keyed_upgrades[upgrade_key.value].expanded = true;
  }
  write_normal_progress();
  start_eval_hist();
  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}

function manual_chance_change() {
  roster_config.value.is_details_update = true;
  taps_since_last_input.value = 0;
  using_slider.value = false;
  clear_used_budget();

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
        100 * props.upgrade.base_chance * 2 + props.upgrade.extra_chance * 100,
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
  write_normal_progress();
  start_eval_hist();
  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}

function reset_due_to_optimizer_run() {
  taps_since_last_input.value = 0;
  using_slider.value = true;
  roster_config.value.budget_snapshot = null;
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];
  this_keyed.taps_since_last_input = 0;
  clear_used_budget();
}

watch(
  () => optimizer_working.value,
  () => {
    // only do this as we start the optimizer,  shouldn't really matter but whatever
    if (optimizer_working.value) {
      reset_due_to_optimizer_run();
    }
  },
);

// changes in bound budgets will start the optimizer so it'll be gucci
// this is for when autostart is turned off only
watch(
  [
    () => active_profile.value.bound_budgets[tier.value].data,
    () => active_profile.value.bound_budgets[tier.value].enabled,
  ],
  () => {
    if (active_profile.value.auto_start_optimizer) {
      // console.log(roster_config.value.budget_snapshot);
      if (!roster_config.value.is_details_update) {
        reset_due_to_optimizer_run();
        console.log("non-slider change");
      }
      // console.log("change");
    }
  },
  { deep: true },
);

function slider_input() {
  roster_config.value.is_details_update = true;
  using_slider.value = true;

  // this kind of disallows any non-integer inputs by instantly setting invalid intermediate inputs to 0, but its like fine i think since it's not a big number or anything
  taps_since_last_input.value = clamp(
    0,
    Number(taps_since_last_input.value),
    props.upgrade.normal_dist.length - 1,
  );

  if (taps_since_last_input.value > 0) {
    // for the topmost upgrade, which may have expanded false still (and might get sorted down )
    active_profile.value.keyed_upgrades[upgrade_key.value].expanded = true;
  }
  starting_artisan.value = artisan_function(
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
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];

  this_keyed.used_materials = compute_used_materials(
    props.upgrade,
    taps_since_last_input.value,
    juice_info.value,
    0,
    0,
    true,
  );
  apply_remaining_mats();
  write_normal_progress();
  start_eval_hist();
  // console.log(taps_since_last_input.value, typeof taps_since_last_input.value);
  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}

function confirm() {
  start_all_workers();
  reset_due_to_optimizer_run(); // this is explicitly here because if it's already running then we need to trigger it manunally
}

function reset() {
  roster_config.value.is_details_update = true;

  taps_since_last_input.value = 0;
  starting_artisan.value = "0.00";
  current_chance_percentage.value = locale_to_fixed(
    props.upgrade.base_chance * 100 + props.upgrade.extra_chance * 100,
    2,
  );
  using_slider.value = false; // to force it to use the current_chance route
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];

  this_keyed.state = []; // clearing state because like the previous entries aren't here anymore, whatever state we have is kinda bogus. keyed_to_array is special cased to not overwrite this

  write_normal_progress();

  clear_used_budget();
  apply_remaining_mats();
  using_slider.value = true;

  if (
    props.upgrade.starting_artisan > 0 ||
    props.upgrade.starting_num_taps > 0
  ) {
    start_all_workers(); // starting optimizer cos the instruction needs to change
  } else {
    start_eval_hist();
  }

  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}

function succeed_click() {
  roster_config.value.is_details_update = true;
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];

  this_keyed.used_materials = compute_used_materials(
    props.upgrade,
    taps_since_last_input.value,
    juice_info.value,
    0,
    0,
    false, // so 0 also applies unlock cost when pressed explicitly
  );
  apply_remaining_mats();
  mark_upgrade_as_done(props.upgrade); // this will trigger optimizer run

  nextTick(() => {
    roster_config.value.is_details_update = false;
  });
}

const non_special_grid: GridConfig = {
  grid_template_columns: "175px 100px 120px",
};
</script>

<template>
  <div
    class="grid w-full pl-5"
    :style="{
      gridTemplateColumns: non_special_grid.grid_template_columns,
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
            :min="0"
            :max="upgrade.normal_dist.length - 1"
          />
          <span v-else class="mb-px ml-1">N/A</span>
        </span>
        <!-- 
        Actually we need a separate variable to track, because rn starting_num_taps can be overwritten by chance percentage -> num tap conversion, which has to cap out at 10
        <span
          v-if="using_slider && upgrade.starting_num_taps > 0"
          class="annotation max-h-0 overflow-visible text-right"
        >
          ({{ taps_since_last_input + upgrade.starting_num_taps }} total)</span
        > -->
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
      v-model:starting_artisan="starting_artisan"
      v-model:current_chance_percentage="current_chance_percentage"
      :base_chance="upgrade.base_chance"
      :extra_chance="upgrade.extra_chance"
      :optimizer_working="optimizer_working"
      :taps_since_last_input="Number(taps_since_last_input)"
      :normal_dist_length="upgrade.normal_dist.length"
      :using_slider="using_slider"
      :show_hints="true"
      @manual_artisan_change="manual_artisan_change"
      @manual_chance_change="manual_chance_change"
      @confirm="confirm"
    />

    <div class="button-row">
      <button
        class="reset-button"
        @click="reset"
        v-if="
          upgrade.starting_artisan > 0 ||
          upgrade.starting_num_taps > 0 ||
          parse_locale_float(starting_artisan) > FLOAT_TOL ||
          Math.abs(
            parse_locale_float(current_chance_percentage) -
              upgrade.base_chance * 100 -
              upgrade.extra_chance * 100,
          ) > FLOAT_TOL
        "
      >
        Reset this upgrade
      </button>
      <div v-else class="contents">
        <button
          class="reset-button w-full self-center"
          v-if="perform_order != 0"
          @click="active_profile.keyed_upgrades[upgrade_key].expanded = false"
        >
          Hide
        </button>
      </div>
    </div>
  </div>
</template>
