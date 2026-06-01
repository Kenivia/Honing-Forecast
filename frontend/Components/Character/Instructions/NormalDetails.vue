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
import { start_all_workers, start_eval_hist } from "../CharWorkerUtils";
import { artisan_function } from "@/Utils/HoningUtil";
import "./details.css";
import { get_optimizer_working } from "./InstructionUtils";
import {
  compute_used_materials,
  mark_upgrade_as_done,
  apply_remaining_mats,
} from "./SuccessUtils";
import { FLOAT_TOL } from "@/Utils/Constants";
import { GridConfig } from "@/Utils/GridStyling";

const { active_profile, roster_config } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  free_tap_this_upgrade: boolean;
  index_in_special_state: number;
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
  restore_budget_snapshot();

  starting_artisan.value = clean_percentage_input(
    starting_artisan.value,
    0,
    true,
  );
  write_normal_progress();
  start_eval_hist();
}

function manual_chance_change() {
  // console.log(event.target.value, event, current_chance_percentage.value);
  taps_since_last_input.value = 0;
  using_slider.value = false;
  restore_budget_snapshot();

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

function reset_after_optimizer_run() {
  taps_since_last_input.value = 0;
  using_slider.value = true;
  roster_config.value.budget_snapshot = null;
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
  () => optimizer_working.value,
  () => {
    // only do this as we start the optimizer,  shouldn't really matter but whatever
    if (optimizer_working.value) {
      reset_after_optimizer_run();
    }
  },
);

const tier = computed(() => active_profile.value.tier);

// IM PRE SURE this isn't needed now because any change will start the optimizer so it'll be gucci
// I'll leave it in place because ig autostart can be turned off, might remove that later tho lowkey
watch(
  [
    () => active_profile.value.bound_budgets[tier.value].data,
    () => active_profile.value.bound_budgets[tier.value].enabled,
  ],
  () => {
    console.log(roster_config.value.budget_snapshot);
    if (!roster_config.value.is_slider_update) {
      reset_after_optimizer_run();
      console.log("non-slider change");
    }
    console.log("change");
  },
  { deep: true },
);

function slider_input() {
  roster_config.value.is_slider_update = true;
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
  current_chance_percentage.value = locale_to_fixed(
    100 *
      (props.upgrade.base_chance +
        0.1 *
          props.upgrade.base_chance *
          Math.min(
            10,
            taps_since_last_input.value + props.upgrade.starting_num_taps,
          )),
    2,
  );
  const this_keyed =
    active_profile.value.keyed_upgrades[
      to_upgrade_key(
        props.upgrade.piece_type,
        props.upgrade.upgrade_index,
        props.upgrade.is_normal_honing,
        active_profile.value.tier,
      )
    ];

  this_keyed.used_materials = compute_used_materials(
    props.upgrade,
    taps_since_last_input.value,
    juice_info.value,
    0,
    0,
    true,
  );
  apply_remaining_mats(props.upgrade);
  write_normal_progress();
  start_eval_hist();
  nextTick(() => {
    roster_config.value.is_slider_update = false;
  });
}

function slider_change() {
  // write_normal_progress();
  // start_eval_hist();
}

function confirm() {
  start_all_workers();
  // reset_after_optimizer_run();
}
const taps_since_last_input = ref(0);

const using_slider = ref(true);

function restore_budget_snapshot() {
  if (roster_config.value.budget_snapshot !== null) {
    active_profile.value.bound_budgets =
      roster_config.value.budget_snapshot.bound_budgets;
    roster_config.value.roster_mats_owned[active_profile.value.roster_id] =
      roster_config.value.budget_snapshot.roster_mats;
    roster_config.value.tradable_mats_owned[active_profile.value.roster_id] =
      roster_config.value.budget_snapshot.tradable_mats;
  }
}
function reset() {
  restore_budget_snapshot();
  taps_since_last_input.value = 0;
  starting_artisan.value = "0";
  current_chance_percentage.value = locale_to_fixed(
    props.upgrade.base_chance * 100,
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
  } else {
    start_eval_hist();
  }
}

function succeed_click() {
  const this_keyed =
    active_profile.value.keyed_upgrades[
      to_upgrade_key(
        props.upgrade.piece_type,
        props.upgrade.upgrade_index,
        props.upgrade.is_normal_honing,
        active_profile.value.tier,
      )
    ];

  this_keyed.used_materials = compute_used_materials(
    props.upgrade,
    taps_since_last_input.value,
    juice_info.value,
    0,
    0,
    false, // so this is just the unlock cost
  );
  apply_remaining_mats(props.upgrade);
  mark_upgrade_as_done(props.upgrade);
}

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
function special_success_click() {
  active_profile.value.special_budget.data[0] =
    new_special_leaps.value.toLocaleString();
  const this_keyed =
    active_profile.value.keyed_upgrades[
      to_upgrade_key(
        props.upgrade.piece_type,
        props.upgrade.upgrade_index,
        props.upgrade.is_normal_honing,
        active_profile.value.tier,
      )
    ];

  this_keyed.used_materials = compute_used_materials(
    props.upgrade,
    0,
    juice_info.value,
    0,
    0,
    false, // so this is just the unlock cost
  );
  apply_remaining_mats(props.upgrade);
  mark_upgrade_as_done(props.upgrade);
}

function special_fail_click() {
  active_profile.value.special_budget.data[0] = "0";
  start_all_workers();
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

const expand_free_tap_artisan = ref(
  (props.upgrade.starting_artisan > 0 || props.upgrade.starting_num_taps > 0) &&
    props.free_tap_this_upgrade,
);
watch(
  () =>
    (props.upgrade.starting_artisan > 0 ||
      props.upgrade.starting_num_taps > 0) &&
    props.free_tap_this_upgrade,
  () => {
    expand_free_tap_artisan.value =
      (props.upgrade.starting_artisan > 0 ||
        props.upgrade.starting_num_taps > 0) &&
      props.free_tap_this_upgrade;
  },
);
const non_special_grid: GridConfig = {
  grid_template_columns: "175px 100px 120px",
};
const special_grid: GridConfig = {
  grid_template_columns: "1fr 80px 100px ",
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
        class="barebone-button w-fit text-(--text-muted)"
      >
        Show input anyway
      </button>
    </div>

    <div v-else class="contents">
      <div
        v-if="!free_tap_this_upgrade"
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
            :disabled="optimizer_working"
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
        <span class="stat-label">Current artisan energy:</span>
        <div class="flex flex-row">
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
          <div
            class="question-mark"
            v-tooltip.left="
              'You can also input artisan directly here. However, costs will not be auto-deducted'
            "
          />
        </div>
        <div class="button-row">
          <button
            @click="confirm"
            class="generic-button w-20!"
            :disabled="
              optimizer_working ||
              taps_since_last_input === upgrade.normal_dist.length - 1
            "
            :style="{
              opacity:
                taps_since_last_input === upgrade.normal_dist.length - 1
                  ? 0.5
                  : 1,
            }"
          >
            Confirm
          </button>
          <div
            class="question-mark"
            v-tooltip.left="
              'Use the slider to update artisan & deduct costs, then press Confirm to re-run the optimizer.'
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
      <div
        v-else
        class="col-span-2 grid"
        :style="{
          gridTemplateColumns: special_grid.grid_template_columns,
        }"
      >
        <span class="self-center pr-1 text-right"
          >Succeeded with this many special leaps remaining:</span
        >
        <input
          class="generic-input w-18!"
          v-model="new_special_leaps"
          type="number"
          :min="0"
          :max="parse_locale_int(active_profile.special_budget.data[0])"
        />

        <button
          class="generic-button button-row w-20! text-(--achieved)!"
          @click="special_success_click"
        >
          Succeed
        </button>

        <span class="self-center pr-1 text-right"
          >({{ locale_to_fixed(upgrade.this_special_chance * 100, 2) }}% chance
          to succeed
          {{ index_in_special_state === 0 ? "" : "this AND everything above" }})
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
          @click="
            () => (active_profile.keyed_upgrades[upgrade_key].expanded = false)
          "
        >
          Hide
        </button>
        <button
          class="annotation col-span-2 pt-2 underline"
          @click="
            () => {
              expand_free_tap_artisan = !expand_free_tap_artisan;
            }
          "
          :disabled="optimizer_working"
          :style="{ cursor: optimizer_working ? 'not-allowed' : 'pointer' }"
        >
          {{
            expand_free_tap_artisan
              ? "Hide Artisan input"
              : "Show artisan input"
          }}
        </button>
      </div>
      <div
        v-if="expand_free_tap_artisan"
        class="mt-1 grid w-max pl-5"
        :style="{
          gridTemplateColumns: non_special_grid.grid_template_columns,
        }"
      >
        <span class="stat-label">Current artisan energy:</span>
        <div class="flex flex-row">
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
        </div>
        <div class="button-row">
          <button
            @click="confirm"
            class="generic-button w-20!"
            :disabled="
              optimizer_working ||
              taps_since_last_input === upgrade.normal_dist.length - 1
            "
            :style="{
              opacity:
                taps_since_last_input === upgrade.normal_dist.length - 1
                  ? 0.5
                  : 1,
            }"
          >
            Confirm
          </button>
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
      </div>
    </div>
  </div>
</template>
