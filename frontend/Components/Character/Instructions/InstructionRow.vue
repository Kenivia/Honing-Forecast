<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { ALL_LABELS, T4_JUICE_LABELS } from "@/Utils/Constants";
import { get_piece_name, get_icon_path, toOrdinal } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { computed, nextTick, ref, watch } from "vue";
import { grid_change_callback } from "../CharWorkerUtils";

import {
  aggregate_streaks,
  artisan_function,
  compute_remaininig_materials,
  compute_used_materials,
  cumulative_chance,
  streaks_to_text,
} from "./InstructionsUtil";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { Upgrade, UpgradeStatus } from "@/Utils/KeyedUpgrades";

const { active_profile } = storeToRefs(useRosterStore());
const { active_roster_mats_owned, active_tradable_mats_owned } =
  storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
  perform_order: number;
  index_in_special_state: number;
  special_invalid_index: number;
}>();

const free_tap_this_upgrade = computed(() => {
  return (
    props.index_in_special_state < props.special_invalid_index &&
    props.upgrade.this_special_chance > 0
  );
});

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const streaks = computed(() =>
  aggregate_streaks(props.upgrade, juice_info.value, taps_so_far.value),
);
const streak_texts = computed(() =>
  streaks_to_text(
    props.upgrade,
    streaks.value,
    juice_info.value,
    taps_so_far.value,
  ),
);

const taps_so_far = ref(props.upgrade.starting_num_taps || 0);
watch(
  () => props.upgrade.starting_num_taps,
  () => {
    taps_so_far.value = props.upgrade.starting_num_taps;
  }, // This watch is here to watch for when upgrade changes (optimizer shuffled order or tick /untick), in which case props.upgrade changes
);

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

// function write_normal_progress() {
//     taps_so_far.value = Math.max(0, Math.min(props.upgrade.normal_dist.length - 1, taps_so_far.value))
//     active_profile.value.keyed_upgrades[
//         to_upgrade_key(props.upgrade.piece_type, props.upgrade.upgrade_index, props.upgrade.is_normal_honing, active_profile.value.tier)
//     ][3] = taps_so_far.value
//     start_all_workers()
// }

// function write_adv_progress() {
//     current_adv_upgrade.value = Math.max(props.upgrade.upgrade_index * 10, Math.min((props.upgrade.upgrade_index + 1) * 10 - 1, current_adv_upgrade.value))
//     current_adv_xp.value = Math.floor(Math.max(0, Math.min(90, current_adv_xp.value)) / 10) * 10
//     // console.log(current_grace_progress.value)
//     current_grace_progress.value = Math.min(6, Math.max(0, current_grace_progress.value))

//     active_profile.value.keyed_upgrades[
//         to_upgrade_key(props.upgrade.piece_type, props.upgrade.upgrade_index, props.upgrade.is_normal_honing, active_profile.value.tier)
//     ][7] = [
//         (current_adv_upgrade.value - props.upgrade.upgrade_index * 10) * 10 + current_adv_xp.value / 10,
//         current_grace_progress.value,
//         next_free.value,
//         next_big.value,
//     ]
//     start_all_workers()
// }

function juice_icon_path(upgrade: Upgrade, juice: boolean) {
  let juice_info =
    active_profile.value.histogram_worker_bundle.result.juice_info;
  let relevant_id_map = upgrade.is_normal_honing
    ? juice_info.normal_uindex_to_id
    : juice_info.adv_uindex_to_id;

  let relevant_upgrade = relevant_id_map[upgrade.upgrade_index];
  if (relevant_upgrade.length === 0) {
    return "no juice avail";
  }
  return get_icon_path(
    T4_JUICE_LABELS[relevant_upgrade[juice ? 0 : relevant_upgrade.length - 1]][
      upgrade.is_weapon ? 0 : 1
    ],
  );
}

const progress_expanded = ref(false);
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
const show_success_modal = ref(false);
function onSucceedClick() {
  show_success_modal.value = true;
}

// ============================================== Popup related stuff =============================================

const succeed_without_deduct = ref(false);
const adv_juice_used = ref(0);
const adv_scroll_used = ref(0);

const used_materials = computed(() =>
  compute_used_materials(
    props.upgrade,
    taps_so_far.value,
    juice_info.value,
    adv_juice_used.value,
    adv_scroll_used.value,
  ),
);
const remaining_materials = computed(() =>
  compute_remaininig_materials(used_materials.value),
);
const visibleRows = computed(() => {
  const tier = active_profile.value.tier;
  if (!ALL_LABELS || !ALL_LABELS[tier]) return [];
  return ALL_LABELS[tier]
    .map((label, index) => ({ label, index, row: index }))
    .filter(
      (item) =>
        used_materials.value[item.index] > 0 &&
        active_profile.value.bound_budgets[tier].enabled[item.index],
    );
});

async function confirmSuccess() {
  if (!succeed_without_deduct.value) {
    const tier = active_profile.value.tier;

    used_materials.value.forEach((cost, index) => {
      if (cost <= 0) return;
      active_profile.value.bound_budgets[tier].data[index] =
        remaining_materials.value.bound_budgets[index].toLocaleString();
      active_roster_mats_owned.value[tier].data[index] =
        remaining_materials.value.roster_mats[index].toLocaleString();
      active_tradable_mats_owned.value[tier].data[index] =
        remaining_materials.value.tradable_mats[index].toLocaleString();
    });
  }
  if (props.upgrade.is_normal_honing) {
    active_profile.value.normal_grid[props.upgrade.piece_type][
      props.upgrade.upgrade_index
    ] = UpgradeStatus.Done;
  } else {
    active_profile.value.adv_grid[props.upgrade.piece_type][
      props.upgrade.upgrade_index
    ] = UpgradeStatus.Done;
  }
  grid_change_callback();

  show_success_modal.value = false;
  succeed_without_deduct.value = false;
  active_profile.value.material_rerender_trigger = false;
  await nextTick();
  active_profile.value.material_rerender_trigger = true;
}
</script>

<template>
  <div class="flex flex-col items-center">
    <span>{{
      (upgrade.is_normal_honing ? "" : "Advanced ") +
      get_piece_name(upgrade) +
      " +" +
      String((upgrade.upgrade_index + 1) * (upgrade.is_normal_honing ? 1 : 10))
    }}</span>
    <img
      :src="get_icon_path(get_piece_name(upgrade))"
      :alt="get_piece_name(upgrade)"
      class="h-8 w-8 object-contain"
    />
  </div>

  <div>
    <div class="text-4xl">
      {{ toOrdinal(props.perform_order + 1) }}
    </div>
    <span class="annotation"
      >Do this upgrade {{ toOrdinal(props.perform_order + 1) }}
    </span>
  </div>

  <div class="flex flex-col items-center">
    <img
      :src="
        get_icon_path(
          (active_profile.tier == 1 ? 'Serca ' : '') +
            active_profile.special_budget.keys[0],
        )
      "
      :alt="get_piece_name(upgrade)"
      class="h-12 w-12 object-contain"
    />
    <!-- TODO ADD BIG CROSS HERE FOR NO FREE TAP -->
    <span class="annotation">
      {{
        free_tap_this_upgrade
          ? "Free tap this until you run out"
          : "Do not use special tap on this upgrade"
      }}
    </span>
  </div>

  <div class="flex flex-row">
    <div
      v-for="(streak_text, i) in streak_texts"
      :key="i"
      class="flex flex-col items-center"
    >
      <img
        :src="juice_icon_path(upgrade, true)"
        alt="Top Mat"
        class="h-8 w-8 object-contain"
      />

      <div
        v-if="
          juice_icon_path(upgrade, false) !== juice_icon_path(upgrade, true)
        "
      >
        <img
          :src="juice_icon_path(upgrade, false)"
          alt="Bottom Mat"
          class="h-8 w-8 object-contain"
        />
      </div>
      <div class="annotation">
        <div v-html="streak_text.name_line"></div>
        <div v-html="streak_text.line1"></div>
        <div v-html="streak_text.line2"></div>
      </div>
    </div>
  </div>

  <div
    v-if="
      progress_expanded &&
      active_profile.optimizer_worker_bundle.status === 'busy'
    "
  >
    <span> Optimizer working...</span>
  </div>
  <div
    v-if="
      (progress_expanded || must_show) &&
      active_profile.optimizer_worker_bundle.status !== 'busy'
    "
  >
    <div>
      <div v-if="upgrade.is_normal_honing">
        <div>
          Current Artisan energy:
          {{ artisan_function(upgrade, taps_so_far, juice_info) }}%
        </div>
        <div>
          Cumulative chance:
          {{ cumulative_chance(upgrade, taps_so_far, juice_info) }}%
        </div>
      </div>
      <div v-else></div>
      <button @click="onSucceedClick">Succeed & deduct costs</button>
    </div>
  </div>

  <!--
  <Teleport to="body">
    <div
      v-if="show_success_modal"
      class="modal-overlay"
      @click="show_success_modal = false"
    >
      <div class="popup" @click.stop>
        <div v-if="upgrade.is_normal_honing" class="popup-header">
          <h3>Confirm Success</h3>
          <div class="input-row text-left">
            Final Artisan energy:
            {{
              artisan_function(
                upgrade,
                Math.max(0, taps_so_far - 1),
                juice_info,
              )
            }}%
          </div>
          <div class="input-row text-left">
            Cumulative chance:
            {{ cumulative_chance(upgrade, taps_so_far, juice_info) }}%
          </div>
        </div>
        <div
          v-if="upgrade.is_normal_honing"
          style="
            display: flex;
            align-items: center;
            justify-content: flex-end;
            flex-direction: row;
          "
        >
          <div class="input-row">
            <label>Taps to succeed</label>
            <input
              type="number"
              v-model.number="taps_so_far"
              min="0"
              :max="upgrade.normal_dist?.length - 1 || 100"
              @change="write_normal_progress"
            />
          </div>
          <div class="input-row">
        
            <input
              type="range"
              v-model.number="taps_so_far"
              min="0"
              :max="upgrade.normal_dist?.length - 1 || 100"
              class="slider"
              @change="write_normal_progress"
            />
          </div>
          <label class="check-label">
            <input type="checkbox" v-model="succeed_without_deduct" />
            Succeed without deducting cost
          </label>
        </div>
        <div v-if="!upgrade.is_normal_honing" class="popup-header">
          <h3>Confirm Success</h3>
          <label class="input-row">
            Total taps used <input v-model="taps_so_far" :min="0" :max="100" />
          </label>
          <label class="input-row">
            Juiced taps used
            <input v-model="adv_juice_used" :min="0" :max="100" />
          </label>
          <label class="input-row">
            Scroll taps used
            <input v-model="adv_scroll_used" :min="0" :max="100" />
          </label>
        </div>

        <div class="popup-grid">
          <div class="popup-title-row">
            <span style="color: var(--graph-average-color)"
              >Material Costs</span
            >
            <span style="color: var(--bound); text-align: left"
              >Char-Bound (after)</span
            >
            <span style="color: var(--roster); text-align: left"
              >Rester-Bound (after)
            </span>
            <span style="color: var(--tradable); text-align: left"
              >Tradable (after)</span
            >
          </div>

          <div
            v-for="{ label, row } in visibleRows"
            :key="`manifest-${label}`"
            class="mats-row"
          >
            <MaterialCell
              :input_column="used_materials"
              :row="row"
              :input_color="'--graph-average-color'"
              :label="label"
            />
            <MaterialCell
              :input_column="remaining_materials.bound_budgets"
              :row="row"
              :input_color="'--bound'"
            />
            <MaterialCell
              :input_column="remaining_materials.roster_mats"
              :row="row"
              :input_color="'--roster'"
            />
            <MaterialCell
              :input_column="remaining_materials.tradable_mats"
              :row="row"
              :input_color="'--tradable'"
            />
          </div>
          <div
            v-if="upgrade.is_normal_honing && taps_so_far == 0"
            class="mats-row"
          >
            <MaterialCell
              :input_column="active_profile.special_budget"
              :row="0"
              :setter="(val) => (active_profile.special_budget.data[0] = val)"
              :label="
                (active_profile.tier == 1 ? 'Serca ' : '') +
                active_profile.special_budget.keys[0]
              "
              :hide_tick="true"
            ></MaterialCell>
            <span style="justify-self: left">(after)</span>
          </div>
        </div>

        <div class="popup-actions">
          <button class="btn-cancel" @click="show_success_modal = false">
            Cancel
          </button>
          <button class="btn-confirm" @click="confirmSuccess">Confirm</button>
        </div>
      </div>
    </div>
  </Teleport>
  -->
</template>

<style scoped>
.annotation {
  color: var(--text-muted);
  font-size: x-small;
}
/*
 .upgrade-row {
  --icon-size: 36px;
  --font-primary: 1rem;
  --font-small: 0.8rem;

  display: flex;
  flex-direction: column;
  margin-bottom: 1rem;
  background: var(--bg-surface, #1e1e1e);
  border-radius: 8px;
  padding: 1rem;
}

.upgrade-topline {
  border-bottom: 1px solid var(--border-main, #333);
  padding-bottom: 0.5rem;
  margin-bottom: 1rem;
  font-weight: bold;
  font-size: 1.1rem;
}

.upgrade-content {
  display: flex;
  flex-wrap: wrap;
  gap: 1.5rem;
  align-items: stretch;
}
.left-controls {
  display: flex;
  gap: 1rem;
  align-items: flex-start;
}

.order-block,
.free-tap-actions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.order-circle {
  width: calc(
    var(--icon-size) * 2 + 0.25rem
  );
  height: calc(var(--icon-size) * 2 + 0.25rem);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: bold;
  background-color: var(--text-muted, #555);
  color: var(--bg-deep, #000);
}

.order-circle.is-free-tap {
  background-color: var(--free-tap, #4caf50);
}

.order-text,
.action-desc {
  font-size: var(--font-small);
  color: var(--text-muted, #aaa);
  text-align: center;
  text-wrap-mode: wrap;
}

.btn-all-failed {
  height: calc(var(--icon-size) * 3 + 0.25rem);
  background-color: var(--free-tap);
  color: var(--bg-deep, #000);
  border: none;
  border-radius: 8px;
  padding: 0 1rem;
  font-weight: bold;
  cursor: pointer;
  transition: filter 0.2s;
  text-wrap-mode: wrap;
  max-width: 100px;
}
.btn-all-failed:hover {
  filter: brightness(1.2);
}

.btn-expand {
  height: calc(var(--icon-size) * 1 + 0.25rem);
  background-color: var(--text-muted);
  color: var(--bg-deep, #000);
  border: none;
  border-radius: 8px;
  padding: 0 1rem;
  font-weight: bold;
  cursor: pointer;
  transition: filter 0.2s;
  justify-self: center;
}
.btn-expand:hover {
  filter: brightness(1.2);
}

.scrollable-instructions {
  display: flex;
  gap: 0.5rem;
  overflow-x: auto;
  padding-bottom: 0.5rem;
  flex: 1;
  min-width: 200px;
  max-width: 400px;
  transition: opacity 0.3s;
}

.scrollable-instructions.is-dimmed {
  opacity: 0.4;
}

.instruction-stack {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  min-width: 80px;
}

.icon-slot {
  width: var(--icon-size);
  height: var(--icon-size);
  border: 1px solid rgba(255, 255, 255, 0.1);
  background-color: var(--bg-raised);
}
.icon-slot.should-not-use {
  width: var(--icon-size);
  height: var(--icon-size);
  border: none;
}

.icon-slot img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.text-slot {
  text-align: center;
  margin-top: 0.5rem;
}

.line-primary {
  color: white;
  font-size: var(--font-primary);
}

.line-muted {
  color: var(--text-muted, #aaa);
  font-size: var(--font-small);
}


.right-section {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  flex: 1;
  min-width: 150px;
  max-width: 600px;
  flex-direction: row;
  align-items: center;
  justify-content: center;
}

.inputs-container {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  flex: 1;
  min-width: 200px;
}

.input-row {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0.5rem;
  height: 33%;
}

.text-left {
  justify-content: flex-start;
}

.input-row input[type="number"] {
  width: 60px;
  padding: 0.25rem;
  background: var(--bg-deep, #121212);
  border: 1px solid var(--border-main, #333);
  color: white;
  border-radius: 4px;
}

.grid-4 {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
}

.slider {
  width: 100%;
  cursor: pointer;
}

.btn-succeed {
  background-color: var(--btn-success, #2e7d32);
  color: var(
    --bg-deep,
    #fff
  );
  border: none;
  border-radius: 8px;
  padding: 1rem;
  font-weight: bold;
  cursor: pointer;
  flex-shrink: 0;
  transition: filter 0.2s;
}
.btn-succeed:hover {
  filter: brightness(1.2);
}
.popup-grid {
  display: grid;
  grid-template-columns: 250px 140px 140px 140px;
  align-items: center;
  row-gap: 0;
  overflow-y: auto;
  flex: 1;
}

.popup-title-row,
.mats-row {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: subgrid;
  align-items: center;
  border-bottom: 1px solid var(--border-main, #333);
  min-height: 40px;
  padding: 0.5rem 0;
  justify-items: right;
}

.popup-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid var(--border-main, #333);
}

.btn-confirm {
  background: var(--btn-success, #2e7d32);
  color: var(--bg-deep);
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
}
.btn-cancel {
  background: var(--cancel, #d32f2f);
  color: var(--text-bright,);
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
}

.check-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: var(--font-small);
  cursor: pointer;
}
  */
</style>
