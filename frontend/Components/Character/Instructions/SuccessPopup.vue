<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import { Upgrade, UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { storeToRefs } from "pinia";
import { computed, nextTick, ref } from "vue";
import {
  compute_remaininig_materials,
  compute_used_materials,
} from "./SuccessUtils";
import { grid_change_callback } from "../CharWorkerUtils";
import { artisan_function, cumulative_chance } from "@/Utils/HoningUtil";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { ALL_LABELS } from "@/Utils/Constants";

const { active_profile } = storeToRefs(useRosterStore());

const props = defineProps<{
  upgrade: Upgrade;
}>();

const show_success_modal = defineModel<boolean>({ required: true });

const succeed_without_deduct = ref(false);
const adv_juice_used = ref(0);
const adv_scroll_used = ref(0);
const { active_roster_mats_owned, active_tradable_mats_owned } =
  storeToRefs(useRosterStore());

const juice_info = computed(() => {
  return active_profile.value.histogram_worker_bundle.result.juice_info;
});

const used_materials = computed(() =>
  compute_used_materials(
    props.upgrade,
    0,
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

const taps_so_far = ref(0);
</script>
<template>
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
            />
          </div>
          <div class="input-row">
            <input
              type="range"
              v-model.number="taps_so_far"
              min="0"
              :max="upgrade.normal_dist?.length - 1 || 100"
              class="slider"
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
</template>
