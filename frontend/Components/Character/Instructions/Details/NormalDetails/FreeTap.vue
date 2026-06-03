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
import { computed, ref, watch } from "vue";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import "@/Components/Character/Instructions/Details/details.css";
import { get_optimizer_working } from "@/Components/Character/Instructions/InstructionUtils";
import {
  compute_used_materials,
  mark_upgrade_as_done,
  apply_remaining_mats,
} from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import { GridConfig } from "@/Utils/GridStyling";
import ManualArtisanInput from "@/Components/Character/Instructions/Details/ManualArtisanInput.vue";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  index_in_special_state: number;
}>();

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const optimizer_working = computed(get_optimizer_working);

const upgrade_key = computed(() =>
  to_upgrade_key(
    props.upgrade.piece_type,
    props.upgrade.upgrade_index,
    props.upgrade.is_normal_honing,
    active_profile.value.tier,
  ),
);

const new_special_leaps = ref(
  parse_locale_int(active_profile.value.special_budget.data[0]),
);
watch(
  () => active_profile.value.special_budget.data[0],
  () => {
    new_special_leaps.value = parse_locale_int(
      active_profile.value.special_budget.data[0],
    );
  },
);

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
      props.upgrade.base_chance) *
      100,
    2,
  ) || "0.00",
);
watch(
  () => props.upgrade.starting_num_taps,
  () => {
    current_chance_percentage.value = locale_to_fixed(
      ((Math.min(10, props.upgrade.starting_num_taps) / 10) *
        props.upgrade.base_chance +
        props.upgrade.base_chance) *
        100,
      2,
    );
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

const taps_since_last_input = ref(0);
const using_slider = ref(true);

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

function manual_artisan_change() {
  taps_since_last_input.value = 0;
  using_slider.value = false;
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];
  this_keyed.used_materials = null;
  apply_remaining_mats();

  starting_artisan.value = clean_percentage_input(
    starting_artisan.value,
    0,
    true,
  );
  write_normal_progress();
  start_eval_hist();
}

function manual_chance_change() {
  taps_since_last_input.value = 0;
  using_slider.value = false;
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];
  this_keyed.used_materials = null;
  apply_remaining_mats();

  current_chance_percentage.value = clean_percentage_input(
    locale_to_fixed(
      clamp(
        100 * props.upgrade.base_chance,
        parse_locale_float(
          locale_to_fixed(
            ((current_chance_to_num_taps.value / 10) *
              props.upgrade.base_chance +
              props.upgrade.base_chance) *
              100,
            2,
          ),
        ),
        100 * props.upgrade.base_chance * 2,
      ),
      2,
    ),
    props.upgrade.base_chance * 100,
  );
  write_normal_progress();
  start_eval_hist();
}

function confirm() {
  start_all_workers();
}

function special_succeed_click() {
  active_profile.value.special_budget.data[0] =
    new_special_leaps.value.toLocaleString();
  const this_keyed = active_profile.value.keyed_upgrades[upgrade_key.value];

  this_keyed.used_materials = compute_used_materials(
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

const special_grid: GridConfig = {
  grid_template_columns: "1fr 65px 100px ",
};
</script>

<template>
  <div
    class="col-span-2 grid"
    :style="{
      gridTemplateColumns: special_grid.grid_template_columns,
    }"
  >
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
        v-model:starting_artisan="starting_artisan"
        v-model:current_chance_percentage="current_chance_percentage"
        :base_chance="upgrade.base_chance"
        :extra_chance="upgrade.extra_chance"
        :optimizer_working="optimizer_working"
        :taps_since_last_input="taps_since_last_input"
        :normal_dist_length="upgrade.normal_dist.length"
        :using_slider="using_slider"
        :show_hints="false"
        @manual_artisan_change="manual_artisan_change"
        @manual_chance_change="manual_chance_change"
        @confirm="confirm"
      />
    </div>
  </div>
</template>
