<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  PIECE_NAMES,
  NORMAL_COLS as NORMAL_COLS,
  NUM_PIECES as NORMAL_ROWS,
  ADV_COLS,
} from "@/Utils/Constants";
import { get_icon_path } from "@/Utils/Helpers";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { grid_change_callback } from "../CharWorkerUtils";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";

const { active_profile } = storeToRefs(useRosterStore());
const props = defineProps<{
  grid_type: "normal" | "adv";
}>();

const COLS = props.grid_type == "normal" ? NORMAL_COLS : ADV_COLS;
const relevant_grid = computed(() =>
  props.grid_type === "normal"
    ? active_profile.value.normal_grid
    : active_profile.value.adv_grid,
);
function check_all_same(col: number) {
  if (
    relevant_grid.value.every(
      (row: UpgradeStatus[]) => row[col] == UpgradeStatus.Done,
    )
  ) {
    return UpgradeStatus.Done;
  }
  if (
    relevant_grid.value.every(
      (row: UpgradeStatus[]) =>
        row[col] == UpgradeStatus.Want || row[col] == UpgradeStatus.Done,
    )
  ) {
    return UpgradeStatus.Want;
  }
  return UpgradeStatus.NotYet;
}
function change_col_and_update_keyed(col: number) {
  let current = check_all_same(col);
  for (const [row] of relevant_grid.value.entries()) {
    change_one(row, col, current);
  }
  grid_change_callback();
}

// The idea is that on the box that the user ticks, the it will always cycle:
// NotYet -> Want -> Done -> NotYet
// The rest is convenienc / making things make sense
function change_one(
  row: number,
  col: number,
  current = relevant_grid.value[row][col],
) {
  if (current == UpgradeStatus.NotYet) {
    let left_is_not_yet = true;
    let no_done = true;
    for (const [index, cell] of relevant_grid.value[row].entries()) {
      if (index > col) {
        break;
      }
      if (cell == UpgradeStatus.Want) {
        left_is_not_yet = false;
        no_done = false;
      }
      if (cell == UpgradeStatus.Done) {
        no_done = false;
      }
      if (!left_is_not_yet) {
        relevant_grid.value[row][index] = UpgradeStatus.Want;
      }
    }
    if (no_done) {
      for (const [index] of relevant_grid.value[row].entries()) {
        if (index > col) {
          break;
        }
        relevant_grid.value[row][index] = UpgradeStatus.Done;
      }
    } else {
      for (const [index] of relevant_grid.value[row].entries()) {
        if (index > col) {
          break;
        }
        if (relevant_grid.value[row][index] == UpgradeStatus.NotYet) {
          relevant_grid.value[row][index] = UpgradeStatus.Want;
        }
      }
      for (
        let index = col + 1;
        index < relevant_grid.value[0].length;
        index += 1
      ) {
        if (relevant_grid.value[row][index] == UpgradeStatus.Done) {
          relevant_grid.value[row][index] = UpgradeStatus.Want;
        }
      }
    }
    relevant_grid.value[row][col] = UpgradeStatus.Want;
  } else if (current == UpgradeStatus.Want) {
    for (let c = 0; c <= col; c++) {
      relevant_grid.value[row][c] = UpgradeStatus.Done;
    }
  } else if (current == UpgradeStatus.Done) {
    let right_is_not_yet = true;
    for (let index = relevant_grid.value[row].length - 1; index >= 0; index--) {
      if (index < col) {
        break;
      }
      if (!right_is_not_yet) {
        relevant_grid.value[row][index] = UpgradeStatus.Want;
      }
    }
    if (right_is_not_yet) {
      for (
        let index = relevant_grid.value[row].length - 1;
        index >= 0;
        index--
      ) {
        if (index < col) {
          break;
        }
        relevant_grid.value[row][index] = UpgradeStatus.NotYet;
      }
      relevant_grid.value[row][col] = UpgradeStatus.NotYet;
    } else {
      relevant_grid.value[row][col] = UpgradeStatus.Want;
    }
  }
}
function change_one_and_update_keyed(
  row: number,
  col: number,
  current = relevant_grid.value[row][col],
) {
  change_one(row, col, current);
  grid_change_callback();
}
</script>
<template>
  <div class="fler-row flex flex-nowrap">
    <div class="w-27.5">
      <div class="flex h-7 items-center justify-end" style="font-size: x-small">
        Toggle whole column ->
      </div>
      <div
        v-for="piece in PIECE_NAMES"
        :key="piece"
        class="flex h-7 items-center justify-end"
      >
        <div
          class="inline-flex w-full items-center justify-end gap-1.5 text-right text-sm text-(--text-secondary)"
        >
          <span class="text-(--text-muted)">{{ piece }}</span>
          <img
            :src="get_icon_path(piece)"
            :alt="piece"
            class="h-6.75 w-6.75 object-contain"
          />
        </div>
      </div>
    </div>
    <div ref="`${grid_type}_GridScrollRef`" class="items-start overflow-x-auto">
      <div
        class="cell-grid mb-1 grid"
        :style="{
          gridTemplateColumns: `repeat(${grid_type == 'normal' ? NORMAL_COLS : ADV_COLS}, 26px)`,
        }"
      >
        <button
          v-for="col in COLS"
          :key="`${grid_type}-header-${col}`"
          class="cell"
          :class="{
            Done: check_all_same(col - 1) == UpgradeStatus.Done,
            Want: check_all_same(col - 1) == UpgradeStatus.Want,
          }"
          @click="change_col_and_update_keyed(col - 1)"
        >
          +{{ col * (grid_type == "normal" ? 1 : 10) }}
        </button>
      </div>
      <div
        v-for="row in NORMAL_ROWS"
        :key="`${grid_type}-row-${row}`"
        class="mb-0.5 grid w-fit"
        :style="{
          gridTemplateColumns: `repeat(${grid_type == 'normal' ? NORMAL_COLS : ADV_COLS}, 26px)`,
        }"
      >
        <button
          v-for="col in COLS"
          :key="`${grid_type}-${row}-${col}`"
          class="cell"
          :class="{
            Done: relevant_grid[row - 1][col - 1] == UpgradeStatus.Done,
            Want: relevant_grid[row - 1][col - 1] == UpgradeStatus.Want,
          }"
          :style="{ color: 'white' }"
          @click="change_one_and_update_keyed(row - 1, col - 1)"
        >
          {{ relevant_grid[row - 1][col - 1] == UpgradeStatus.Done ? "✓" : "" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cell {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border-medium);
  border-radius: 2px;
  background: transparent;
  font-size: 10px;
  cursor: pointer;
  line-height: 1;
  user-select: none;
  align-items: center;
  color: var(--text-muted);
  border-color: var(--border-subtle);
}
.cell.Done {
  background: var(--achieved);
  color: var(--text-void);
  border-color: transparent;
}
.cell.Want {
  background: var(--pending);
  color: var(--text-void);
  border-color: transparent;
}
</style>
