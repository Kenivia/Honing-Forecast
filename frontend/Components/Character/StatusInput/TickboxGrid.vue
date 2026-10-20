<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  PIECE_NAMES,
  NORMAL_COLS,
  NUM_PIECES,
  ADV_COLS,
  NUM_ADV_PIECES,
} from "@/Utils/Constants";
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { grid_change_callback } from "../CharWorkerUtils";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { click_cell, click_column, column_status } from "@/Utils/StatusGrid";
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

function change_col_and_update_keyed(col: number) {
  click_column(
    relevant_grid.value,
    col,
    active_profile.value.lock_fetched_done,
  );
  grid_change_callback();
}

function change_one_and_update_keyed(row: number, col: number) {
  click_cell(
    relevant_grid.value[row],
    col,
    active_profile.value.lock_fetched_done,
  );
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

// accessible name, e.g. "Helmet +15: Want"
function cell_label(row: number, col: number): string {
  const level = (col + 1) * (is_normal ? 1 : 10);
  const status = cell_class(relevant_grid.value[row][col]) || "NotYet";
  return `${PIECE_NAMES[row]} +${level}: ${status}`;
}

// a tick when done, the level on the last wanted one
function cell_text(row: number, col: number): string {
  const cells = relevant_grid.value[row];
  if (
    cells[col] === UpgradeStatus.Done ||
    cells[col] === UpgradeStatus.FetchedDone
  )
    return "✓";
  return cells[col] === UpgradeStatus.Want &&
    (col + 1 === COLS || cells[col + 1] === UpgradeStatus.NotYet)
    ? `+ ${(col + 1) * (is_normal ? 1 : 10)}`
    : "";
}

function cell_cursor(status: UpgradeStatus): string {
  return status === UpgradeStatus.FetchedDone &&
    active_profile.value.lock_fetched_done
    ? "not-allowed"
    : "pointer";
}
</script>

<template>
  <div
    class="flex flex-row flex-nowrap px-4 py-2"
    role="group"
    :aria-label="is_normal ? 'Normal honing' : 'Advanced honing'"
  >
    <div class="w-27.5">
      <div class="flex h-7 items-center justify-end" style="font-size: x-small">
        Toggle whole column ->
      </div>
      <div
        v-for="index in NUM_ADV_PIECES"
        :key="index"
        class="flex h-7 items-center justify-end"
      >
        <LabeledPieceIcon :piece="PIECE_NAMES[index - 1]" />
      </div>
      <div
        v-if="active_profile.tier == 1"
        class="mt-1 flex h-14 items-center justify-end gap-7 border-t border-(--border-main) pt-1"
      >
        <LabeledPieceIcon :piece="PIECE_NAMES[NUM_PIECES - 1]" />
      </div>
    </div>
    <div class="items-start overflow-x-auto">
      <div
        class="cell-grid mb-1 grid"
        :style="{ gridTemplateColumns: `repeat(${col_indices.length}, 26px)` }"
      >
        <button
          v-for="col in col_indices"
          :key="`${grid_type}-header-${col}`"
          class="cell"
          :class="cell_class(column_status(relevant_grid, col - 1))"
          :style="{
            cursor: cell_cursor(column_status(relevant_grid, col - 1)),
          }"
          @click="change_col_and_update_keyed(col - 1)"
        >
          +{{ col * (grid_type === "normal" ? 1 : 10) }}
        </button>
      </div>
      <div
        v-for="row in NUM_ADV_PIECES"
        :key="`${grid_type}-row-${row}`"
        class="mb-0.5 grid w-fit"
        :style="{ gridTemplateColumns: `repeat(${col_indices.length}, 26px)` }"
      >
        <button
          v-for="col in col_indices"
          :key="`${grid_type}-${row}-${col}`"
          class="cell"
          :class="cell_class(relevant_grid[row - 1][col - 1])"
          :aria-label="cell_label(row - 1, col - 1)"
          :style="{ cursor: cell_cursor(relevant_grid[row - 1][col - 1]) }"
          @click="change_one_and_update_keyed(row - 1, col - 1)"
        >
          {{ cell_text(row - 1, col - 1) }}
        </button>
      </div>
      <div
        v-if="active_profile.tier == 1"
        class="mt-1 mb-0.5 grid w-fit border-t border-(--border-main) pt-1"
        :style="{ gridTemplateColumns: `repeat(${col_indices.length}, 26px)` }"
      >
        <button
          v-for="col in COLS"
          :key="`${grid_type}-${NUM_PIECES}-${col}`"
          class="cell"
          :class="cell_class(relevant_grid[NUM_PIECES - 1][col - 1])"
          :aria-label="cell_label(NUM_PIECES - 1, col - 1)"
          :style="{
            cursor: cell_cursor(relevant_grid[NUM_PIECES - 1][col - 1]),
          }"
          @click="change_one_and_update_keyed(NUM_PIECES - 1, col - 1)"
        >
          +{{ col }}
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
