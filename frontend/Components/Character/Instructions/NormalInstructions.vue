<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed, ref, watch } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { Upgrade } from "@/Utils/KeyedUpgrades";
import { GridConfig } from "@/Utils/GridStyling";
import { Quaternary, Trinary } from "@/WasmInterface/PayloadBuilder";
import NormalRow from "./NormalRow.vue";
import { get_any_overwritten, get_optimizer_working } from "./InstructionUtils";

const { active_profile } = storeToRefs(useRosterStore());
const any_overwritten = computed(get_any_overwritten);
const relevant_result = computed(() => {
  // console.log("relevant", any_overwritten.value);
  return any_overwritten.value
    ? active_profile.value.histogram_worker_bundle.result.state_bundle
    : active_profile.value.optimizer_worker_bundle.result;
});
// This sorts the upgrades into an order that can actually be performed in game
// special_state is already guaranteed to be valid on the rust side, but it doesn't tell us how to do the non-special taps
// this gives a suggestion
function sort_upgrades(): [Upgrade, number, number][] {
  if (!relevant_result.value) {
    return [];
  }

  let output: number[] = [];
  let indices_in_special_state: number[] = [];
  let upgrade_arr: Upgrade[] = relevant_result.value.upgrade_arr;
  // let copy = upgrade_arr.slice()
  let special_state: number[] = structuredClone(
    relevant_result.value.special_state,
  );
  const special_chance_map = new Map();
  for (let index = 0; index < special_state.length; index++) {
    special_chance_map.set(
      special_state[index],
      relevant_result.value.latest_special_probs[index],
    );
  }

  let special_invalid_index = relevant_result.value.special_invalid_index;
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

  let out = output
    .map((x, perform_order) => {
      const upgrade = upgrade_arr[x];
      const index_in_special = special_state.findIndex((y) => y == x);
      return [
        { ...upgrade, this_special_chance: special_chance_map.get(x) }, // Shallow clone
        index_in_special,
        perform_order,
      ] as [Upgrade, number, number];
    })
    .filter((x) => x[0].is_normal_honing);
  // const special_override =
  //   active_profile.value.optimizer_override.special_state;
  // if (
  //   active_profile.value.optimizer_override === undefined ||
  //   special_override.optimizer
  // ) {
  //   return out;
  // }

  // const compare_function = (ua: Upgrade, ub: Upgrade): number => {
  //   // Primary: preferred type goes first
  //   const a_preferred = ua.is_weapon === special_override.weapon_first;
  //   const b_preferred = ub.is_weapon === special_override.weapon_first;
  //   if (a_preferred !== b_preferred) {
  //     return a_preferred ? -1 : 1;
  //   }

  //   // Tiebreak: by upgrade_index direction
  //   return special_override.highest_first
  //     ? ub.upgrade_index - ua.upgrade_index
  //     : ua.upgrade_index - ub.upgrade_index;
  // };

  // out.sort((A, B) => compare_function(A[0], B[0]));
  return out;
}

const sorted_upgrade_arr = ref(sort_upgrades());
//  i dont really understand why using computed for this doesn't update correctly but whatever
watch(
  () => relevant_result.value,
  () => {
    // console.log("sort", any_overwritten.value);
    sorted_upgrade_arr.value = sort_upgrades();
  },
  { deep: true, immediate: true },
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
    //  66 px fits Weapon, Shoulder still doesn't fit but whatever
    "minmax(66px, 70px) minmax(70px,110px) minmax(80px,100px) minmax(200px, max-content) 80px 370px ",
};

const optimizer_working = computed(get_optimizer_working);
</script>
<template>
  <section class="card-shell">
    <div class="card-header">
      <div class="card-title">
        Normal Honing Instructions
        {{ any_overwritten ? "(Not optimized)" : "" }}
        <span v-if="optimizer_working" class="text-(--text-main)">
          ({{
            active_profile.optimizer_worker_bundle.est_progress_percentage.toFixed(
              2,
            )
          }}%)</span
        >
      </div>
    </div>
    <div
      class="card-body outer-grid"
      :style="{
        '--grid-cols': grid.grid_template_columns,
      }"
    >
      <div class="mats-row">
        <span>Upgrade</span>
        <div class="flex w-full flex-row justify-center">
          <div class="ml-3 w-min text-wrap">Upgrade order</div>
          <div
            class="question-mark"
            v-tooltip.right="
              'You should do the upgrades in the below order. (Only the order you attempt free taps actually matter, there are many equivalent orderings.)'
            "
          />
        </div>
        <span>Special usage</span>
        <div
          class="flex w-full flex-row flex-nowrap justify-center gap-1 px-3 text-nowrap"
        >
          <span> Juice & book Instructions</span>
          <div
            class="question-mark"
            v-tooltip.right="
              'Juiced taps mean full-juice (use the maximum amount of Lava / Glacier Breath).'
            "
          />
        </div>
        <span>Succeed</span>
        <div class="flex w-full flex-row justify-center">
          Artisan input
          <div
            class="question-mark ml-2"
            v-tooltip.left="
              'This is optional - updating your progress may produce instructions that save slightly more gold.'
            "
          />
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
          class="mats-row h-fit! py-1!"
        >
          <NormalRow
            v-if="upgrade.is_normal_honing"
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
