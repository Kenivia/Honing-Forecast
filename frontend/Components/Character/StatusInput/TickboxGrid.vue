<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  PIECE_NAMES,
  NORMAL_COLS,
  NUM_PIECES as NORMAL_ROWS,
  ADV_COLS,
} from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { grid_change_callback } from "../CharWorkerUtils";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import LabeledPieceIcon from "@/Components/Common/LabeledPieceIcon.vue";

const { active_profile } = storeToRefs(useRosterStore());
const props = defineProps<{
  grid_type: "normal" | "adv";
}>();

const is_normal = props.grid_type === "normal";
const COLS = is_normal ? NORMAL_COLS : ADV_COLS; // these two are not reactive because like they aint changing

const relevant_grid = computed(() =>
  is_normal ? active_profile.value.normal_grid : active_profile.value.adv_grid,
);
const offset = computed(() =>
  !is_normal ? 0 : active_profile.value.tier === 0 ? 10 : 11,
);
const col_indices = computed(() =>
  Array.from({ length: COLS - offset.value }, (_, k) => k + 1 + offset.value),
);

function is_cell_locked(row: number, col: number): boolean {
  return (
    relevant_grid.value[row][col] === UpgradeStatus.FetchedDone &&
    active_profile.value.lock_fetched_done
  );
}

function set_cell(row: number, col: number, status: UpgradeStatus) {
  relevant_grid.value[row][col] = is_cell_locked(row, col)
    ? UpgradeStatus.FetchedDone
    : status;
}

function check_all_same(col: number): UpgradeStatus {
  const grid = relevant_grid.value;
  if (
    grid.every((row: UpgradeStatus[]) => row[col] === UpgradeStatus.FetchedDone)
  )
    return UpgradeStatus.FetchedDone;
  if (
    grid.every(
      (row: UpgradeStatus[]) =>
        row[col] === UpgradeStatus.Done ||
        row[col] === UpgradeStatus.FetchedDone,
    )
  )
    return UpgradeStatus.Done;
  if (
    grid.every(
      (row: UpgradeStatus[]) =>
        row[col] === UpgradeStatus.Want ||
        row[col] === UpgradeStatus.Done ||
        row[col] === UpgradeStatus.FetchedDone,
    )
  )
    return UpgradeStatus.Want;
  return UpgradeStatus.NotYet;
}

function change_one(
  row: number,
  col: number,
  current = relevant_grid.value[row][col],
) {
  if (
    current === UpgradeStatus.FetchedDone &&
    active_profile.value.lock_fetched_done
  )
    return;

  if (current === UpgradeStatus.NotYet) {
    let seen_want = false;
    let all_not_yet = true;

    for (let index = 0; index <= col; index++) {
      const cell = relevant_grid.value[row][index];
      if (cell === UpgradeStatus.Want) {
        seen_want = true;
        all_not_yet = false;
      } else if (
        cell === UpgradeStatus.Done ||
        cell === UpgradeStatus.FetchedDone
      ) {
        all_not_yet = false;
      }
      if (seen_want) set_cell(row, index, UpgradeStatus.Want);
    }

    if (all_not_yet) {
      for (let index = 0; index <= col; index++) {
        set_cell(row, index, UpgradeStatus.Done);
      }
    } else {
      for (let index = 0; index <= col; index++) {
        if (relevant_grid.value[row][index] === UpgradeStatus.NotYet)
          set_cell(row, index, UpgradeStatus.Want);
      }
      for (
        let index = col + 1;
        index < relevant_grid.value[0].length;
        index++
      ) {
        if (
          relevant_grid.value[row][index] === UpgradeStatus.Done ||
          relevant_grid.value[row][index] === UpgradeStatus.FetchedDone
        )
          set_cell(row, index, UpgradeStatus.Want);
      }
    }
    set_cell(row, col, UpgradeStatus.Want);
  } else if (current === UpgradeStatus.Want) {
    for (let index = 0; index <= col; index++) {
      set_cell(row, index, UpgradeStatus.Done);
    }
  } else {
    for (let index = col; index < relevant_grid.value[row].length; index++) {
      set_cell(row, index, UpgradeStatus.NotYet);
    }
  }
}

function change_col_and_update_keyed(col: number) {
  const current = check_all_same(col);
  for (const [row] of relevant_grid.value.entries()) {
    change_one(row, col, current);
  }
  grid_change_callback();
}

function change_one_and_update_keyed(row: number, col: number) {
  change_one(row, col);
  grid_change_callback();
}

function cell_class(status: UpgradeStatus): string {
  const locked = active_profile.value.lock_fetched_done;
  if (status === UpgradeStatus.FetchedDone && locked) return "FetchedDone";
  if (status === UpgradeStatus.Done || status === UpgradeStatus.FetchedDone)
    return "Done";
  if (status === UpgradeStatus.Want) return "Want";
  return "";
}

function cell_cursor(status: UpgradeStatus): string {
  return status === UpgradeStatus.FetchedDone &&
    active_profile.value.lock_fetched_done
    ? "not-allowed"
    : "pointer";
}
</script>

<template>
  <div class="fler-row flex flex-nowrap px-4 py-2">
    <div class="w-27.5">
      <div class="flex h-7 items-center justify-end" style="font-size: x-small">
        Toggle whole column ->
      </div>
      <div
        v-for="piece in PIECE_NAMES"
        :key="piece"
        class="flex h-7 items-center justify-end"
      >
        <LabeledPieceIcon :piece="piece" />
      </div>
    </div>
    <div ref="`${grid_type}_GridScrollRef`" class="items-start overflow-x-auto">
      <div
        class="cell-grid mb-1 grid"
        :style="{ gridTemplateColumns: `repeat(${col_indices.length}, 26px)` }"
      >
        <button
          v-for="col in col_indices"
          :key="`${grid_type}-header-${col}`"
          class="cell"
          :class="cell_class(check_all_same(col - 1))"
          :style="{ cursor: cell_cursor(check_all_same(col - 1)) }"
          @click="change_col_and_update_keyed(col - 1)"
        >
          +{{ col * (grid_type === "normal" ? 1 : 10) }}
        </button>
      </div>
      <div
        v-for="row in NORMAL_ROWS"
        :key="`${grid_type}-row-${row}`"
        class="mb-0.5 grid w-fit"
        :style="{ gridTemplateColumns: `repeat(${col_indices.length}, 26px)` }"
      >
        <button
          v-for="col in col_indices"
          :key="`${grid_type}-${row}-${col}`"
          class="cell"
          :class="cell_class(relevant_grid[row - 1][col - 1])"
          :style="{ cursor: cell_cursor(relevant_grid[row - 1][col - 1]) }"
          @click="change_one_and_update_keyed(row - 1, col - 1)"
        >
          {{
            relevant_grid[row - 1][col - 1] === UpgradeStatus.Done ||
            relevant_grid[row - 1][col - 1] === UpgradeStatus.FetchedDone
              ? "✓"
              : ""
          }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cell {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border-main);
  border-radius: 2px;
  background: transparent;
  font-size: 10px;
  cursor: pointer;
  line-height: 1;
  user-select: none;
  align-items: center;
  color: var(--text-muted);
  border-color: var(--border-muted);
}
.cell.FetchedDone {
  background: transparent;
  color: var(--achieved);
  border-color: transparent;
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
