<script setup lang="ts">
import { storeToRefs } from "pinia";
import { ref, watch } from "vue";
import InstructionRow from "./InstructionRow.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { Upgrade, UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { PIECE_NAMES } from "@/Utils/Constants";
import { GridConfig } from "@/Utils/GridStyling";

// This sorts the upgrades into an order that can actually be performed in game
// special_state is already guaranteed to be valid on the rust side, but it doesn't tell us how to do the non-special taps
// this gives a suggestion
function sort_upgrades(): [Upgrade, number, number][] {
  const { active_profile } = storeToRefs(useRosterStore());

  if (!active_profile.value.optimizer_worker_bundle.result) {
    return [];
  }

  let output: number[] = [];
  let indices_in_special_state: number[] = [];
  let upgrade_arr: Upgrade[] =
    active_profile.value.optimizer_worker_bundle.result.upgrade_arr;
  // let copy = upgrade_arr.slice()
  let special_state: number[] =
    active_profile.value.optimizer_worker_bundle.result.special_state;
  const special_chance_map = new Map();
  for (let index = 0; index < special_state.length; index++) {
    special_chance_map.set(
      special_state[index],
      active_profile.value.optimizer_worker_bundle.result.latest_special_probs[
        index
      ],
    );
  }

  let special_invalid_index =
    active_profile.value.optimizer_worker_bundle.result.special_invalid_index;
  for (const [
    index_in_special_state,
    index_in_upgrade_arr,
  ] of special_state.entries()) {
    // console.log(original_index, u_index, output)
    if (index_in_special_state >= special_invalid_index) {
      // console.log(output.slice(), u_index, u_index in output)
      if (!output.includes(index_in_upgrade_arr)) {
        output.push(index_in_upgrade_arr);
      }
    } else {
      let this_upgrade = upgrade_arr[index_in_upgrade_arr];
      for (const [index, upgrade] of upgrade_arr.entries()) {
        if (
          upgrade.upgrade_index <= this_upgrade.upgrade_index &&
          upgrade.is_normal_honing &&
          upgrade.piece_type == this_upgrade.piece_type &&
          !upgrade.succeeded &&
          !output.includes(index)
        ) {
          output.push(index);
          indices_in_special_state.push(
            special_state.findIndex((x) => x == index),
          );
        }
      }
    }
  }

  return output.map((x, perform_order) => {
    const upgrade = upgrade_arr[x];
    const index_in_special = special_state.findIndex((y) => y == x);
    return [
      { ...upgrade, this_special_chance: special_chance_map.get(x) }, // Shallow clone
      index_in_special,
      perform_order,
    ] as [Upgrade, number, number];
  });
  // .sort(
  //   ([upgrade1], [upgrade2]) =>
  //     (upgrade1 as Upgrade).piece_type - (upgrade2 as Upgrade).piece_type,
  // );
}
const { active_profile } = storeToRefs(useRosterStore());
const sorted_upgrade_arr = ref(sort_upgrades());
watch(
  () => active_profile.value.optimizer_worker_bundle.result?.upgrade_arr,
  () => {
    sorted_upgrade_arr.value = sort_upgrades();
  },
  { deep: true },
);

// const upgrade_map = computed(() =>
//   get_upgrade_map(
//     active_profile.value.optimizer_worker_bundle.result?.upgrade_arr ?? null,
//     active_profile.value.tier,
//   ),
// );
// const lowest_arr = computed(() =>
//   PIECE_NAMES.map(
//     (_, piece_type) =>
//       upgrade_map.value.get(
//         to_upgrade_key(
//           piece_type,
//           (
//             active_profile.value.normal_grid[piece_type] as UpgradeStatus[]
//           ).findIndex((value) => value == UpgradeStatus.Want),
//           true,
//           active_profile.value.tier,
//         ),
//       ) ?? null,
//   ),
// );

// const lowest_upgrade_index = computed(() =>
//   Math.min(...lowest_arr.value.map((x) => x?.upgrade_index ?? 999)),
// );

const grid: GridConfig = {
  grid_template_columns:
    "minmax(50px, 0.25fr) minmax(90px, 0.35fr) minmax(80px, 0.35fr) minmax(200px, 1fr) 80px 250px ",
  grid_row_span: `span ${6}`,
};
</script>
<template>
  <section class="card-shell">
    <div class="card-header">
      <div class="card-title">Tap Instructions</div>
      <span class="card-hint">Go from top to bottom</span>
    </div>
    <div
      class="card-body outer-grid"
      :style="{
        '--grid-cols': grid.grid_template_columns,
        // gridRow: grid.grid_row_span,
      }"
    >
      <div class="mats-row">
        <span>Upgrade</span>
        <div class="flex w-full flex-row justify-center">
          <div class="ml-3 w-min text-wrap">Upgrade order</div>
          <div
            class="question-mark"
            v-tooltip.right="
              'Only the order that you attempt Free taps actually matter, this is one of many equivalent orderings.'
            "
          >
            ?
          </div>
        </div>
        <span>Special usage</span>
        <div class="flex w-full flex-row justify-center">
          Juice & book Instructions
          <div
            class="question-mark ml-2"
            v-tooltip.left="
              'All juices (Lava & Glacier Breath) should be used at the max amount.'
            "
          >
            ?
          </div>
        </div>
        <span>Succeed</span>
        <div class="flex w-full flex-row justify-center">
          Artisan input
          <div
            class="question-mark ml-2"
            v-tooltip.left="
              'Only use this if you are starting from some non-zero artisan. There is no need to update your progress after every tap.'
            "
          >
            ?
          </div>
        </div>
      </div>
      <div
        v-if="
          active_profile.optimizer_worker_bundle.result &&
          active_profile.histogram_worker_bundle.result
        "
        class="contents"
      >
        <div
          v-for="[
            upgrade,
            index_in_special_state,
            perform_order,
          ] in sorted_upgrade_arr"
          :key="`instructions-${upgrade.upgrade_index}-${upgrade.piece_type}-${upgrade.is_normal_honing}`"
          class="mats-row h-30!"
        >
          <InstructionRow
            :upgrade="upgrade"
            :perform_order="perform_order"
            :special_invalid_index="
              active_profile.optimizer_worker_bundle.result
                .special_invalid_index
            "
            :index_in_special_state="index_in_special_state"
          />
        </div>
      </div>
    </div>
  </section>
</template>

<!-- <InstructionRow
            :upgrade="upgrade"
            :perform_order="perform_order"
            :special_invalid_index="
              active_profile.optimizer_worker_bundle.result
                .special_invalid_index
            "
            :index_in_special_state="index_in_special_state"
          /> -->
