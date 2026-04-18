<script setup lang="ts">
import { useRosterStore } from "@/Stores/RosterConfig"
import { PIECE_NAMES, NORMAL_COLS as NORMAL_COLS, NUM_PIECES as NORMAL_ROWS, ADV_COLS } from "@/Utils/Constants"
import { get_icon_path } from "@/Utils/Helpers"
import { UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed, watch } from "vue"
import { grid_change_callback, start_all_workers } from "../CharWorkerUtils"

const { active_profile } = storeToRefs(useRosterStore())
const props = defineProps<{
    grid_type: "normal" | "adv"
}>()

const COLS = props.grid_type == "normal" ? NORMAL_COLS : ADV_COLS
const relevant_grid = computed(() => (props.grid_type === "normal" ? active_profile.value.normal_grid : active_profile.value.adv_grid))
function check_all_same(col: number) {
    if (relevant_grid.value.every((row: UpgradeStatus[]) => row[col] == UpgradeStatus.Done)) {
        return UpgradeStatus.Done
    }
    if (relevant_grid.value.every((row: UpgradeStatus[]) => row[col] == UpgradeStatus.Want || row[col] == UpgradeStatus.Done)) {
        return UpgradeStatus.Want
    }
    return UpgradeStatus.NotYet
}
function change_col_and_update_keyed(col: number) {
    let current = check_all_same(col)
    for (const [row] of relevant_grid.value.entries()) {
        change_one(row, col, current)
    }
    grid_change_callback()
}

// The idea is that on the box that the user ticks, the it will always cycle:
// NotYet -> Want -> Done -> NotYet
// The rest is convenienc / making things make sense
function change_one(row: number, col: number, current = relevant_grid.value[row][col]) {
    if (current == UpgradeStatus.NotYet) {
        let left_is_not_yet = true
        let no_done = true
        for (const [index, cell] of relevant_grid.value[row].entries()) {
            if (index > col) {
                break
            }
            if (cell == UpgradeStatus.Want) {
                left_is_not_yet = false
                no_done = false
            }
            if (cell == UpgradeStatus.Done) {
                no_done = false
            }
            if (!left_is_not_yet) {
                relevant_grid.value[row][index] = UpgradeStatus.Want
            }
        }
        if (no_done) {
            for (const [index, cell] of relevant_grid.value[row].entries()) {
                if (index > col) {
                    break
                }
                relevant_grid.value[row][index] = UpgradeStatus.Done
            }
        } else {
            for (const [index, cell] of relevant_grid.value[row].entries()) {
                if (index > col) {
                    break
                }
                if (relevant_grid.value[row][index] == UpgradeStatus.NotYet) {
                    relevant_grid.value[row][index] = UpgradeStatus.Want
                }
            }
            for (let index = col + 1; index < relevant_grid.value[0].length; index += 1) {
                if (relevant_grid.value[row][index] == UpgradeStatus.Done) {
                    relevant_grid.value[row][index] = UpgradeStatus.Want
                }
            }
        }
        relevant_grid.value[row][col] = UpgradeStatus.Want
    } else if (current == UpgradeStatus.Want) {
        for (let c = 0; c <= col; c++) {
            relevant_grid.value[row][c] = UpgradeStatus.Done
        }
    } else if (current == UpgradeStatus.Done) {
        let right_is_not_yet = true
        for (let index = relevant_grid.value[row].length - 1; index >= 0; index--) {
            if (index < col) {
                break
            }
            if (!right_is_not_yet) {
                relevant_grid.value[row][index] = UpgradeStatus.Want
            }
        }
        if (right_is_not_yet) {
            for (let index = relevant_grid.value[row].length - 1; index >= 0; index--) {
                if (index < col) {
                    break
                }
                relevant_grid.value[row][index] = UpgradeStatus.NotYet
            }
            relevant_grid.value[row][col] = UpgradeStatus.NotYet
        } else {
            relevant_grid.value[row][col] = UpgradeStatus.Want
        }
    }
}
function change_one_and_update_keyed(row: number, col: number, current = relevant_grid.value[row][col]) {
    change_one(row, col, current)
    grid_change_callback()
}

// shouldn't be needed anymore
// watch(
//     () => active_profile.value.tier,
//     () => {
//         // console.log("keyed update")
//         active_profile.value.keyed_upgrades = grids_to_keyed(
//             active_profile.value.normal_grid,
//             active_profile.value.adv_grid,
//             active_profile.value.keyed_upgrades,
//             active_profile.value.tier,
//         )
//     },
//     { deep: true, immediate: true },
// )

// Don't watch state changes betcause that's handled by start_eval_hist
watch(
    [
        // () => active_profile.value.bound_budgets[active_profile.value.tier].data,
        // () => active_profile.value.bound_budgets[active_profile.value.tier].enabled, now has direct callback in their materialCell

        () => active_profile.value.express_event,
        // () => active_profile.value.min_resolution, not used rn

        // () => active_roster_mats_owned,  // shouldn't be able to change on charview
        // () => active_tradable_mats_owned,
        // () =>

        // () => active_profile.value.keyed_upgrades,  dedicated callback
        // () => active_profile.value.special_budget.data,
        () => active_profile.value.optimizer_treatment_plan,
        () => active_profile.value.auto_start_optimizer,
    ],
    () => {
        // console.log("start", active_profile.value, roster_config.value)
        start_all_workers()
    },
    { deep: true, immediate: true },
)
</script>
<template>
    <div class="hf-grid-content">
        <div class="hf-label-col">
            <div class="hf-label-row" style="font-size: x-small">Toggle whole column -></div>
            <div v-for="piece in PIECE_NAMES" :key="piece" class="hf-label-row">
                <div class="hf-equip-label">
                    <span>{{ piece }}</span>
                    <img :src="get_icon_path(piece)" :alt="piece" />
                </div>
            </div>
        </div>
        <div ref="`${grid_type}_GridScrollRef`" class="hf-grid-scroll">
            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${NORMAL_COLS}, 26px)` }">
                <button
                    v-for="col in COLS"
                    :key="`${grid_type}-header-${col}`"
                    class="hf-cell"
                    :class="{ Done: check_all_same(col - 1) == UpgradeStatus.Done, Want: check_all_same(col - 1) == UpgradeStatus.Want }"
                    @click="change_col_and_update_keyed(col - 1)"
                >
                    +{{ col * (grid_type == "normal" ? 1 : 10) }}
                </button>
            </div>
            <div
                v-for="row in NORMAL_ROWS"
                :key="`${grid_type}-row-${row}`"
                class="hf-cell-grid"
                :style="{ gridTemplateColumns: `repeat(${NORMAL_COLS}, 26px)` }"
            >
                <button
                    v-for="col in COLS"
                    :key="`${grid_type}-${row}-${col}`"
                    class="hf-cell"
                    :class="{
                        Done: relevant_grid[row - 1][col - 1] == UpgradeStatus.Done,
                        Want: relevant_grid[row - 1][col - 1] == UpgradeStatus.Want,
                    }"
                    :style="{ color: 'white' }"
                    @click="change_one_and_update_keyed(row - 1, col - 1)"
                >
                    {{ relevant_grid[row - 1][col - 1] == UpgradeStatus.Done ? "✓" : "" }}
                </button>
                <!-- @pointerdown.prevent="startTopDrag(row - 1, col - 1, $event)"
                    @pointerenter="dragTopCell(row - 1, col - 1)"
                    @click.prevent="onTopCellClick(row - 1, col - 1, $event)" -->
            </div>
        </div>
    </div>
</template>
<style>
.hf-label-row {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
}

.hf-equip-label {
    width: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 14px;
    text-align: right;
}
.hf-equip-label img {
    width: 27px;
    height: 27px;
    object-fit: contain;
}

.hf-grid-scroll {
    align-items: start;
    overflow-x: auto;
    min-width: 0;
}
.hf-label-col {
    width: 110px;
    min-width: 110px;
}

.hf-grid-content {
    display: flex;
    gap: 4px;
    min-width: 0;
}

.hf-cell-grid {
    display: grid;
    gap: 0;
    margin-bottom: 2px;
}

.hf-cell-grid-head {
    margin-bottom: 4px;
}

.hf-cell {
    width: 26px;
    height: 26px;
    border: 1px solid var(--checkbox-border);
    border-radius: 2px;
    background: transparent;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    line-height: 1;
    user-select: none;
}

.hf-cell.Done {
    background: var(--checkbox-done-bg);
    color: var(--checkbox-done-text);
    border-color: var(--checkbox-done-border);
}

.hf-cell.Want {
    background: var(--checkbox-checked-bg);
    color: var(--checkbox-checked-text);
    border-color: var(--checkbox-checked-border);
}

.hf-cell-header {
    font-size: 10px;
}
</style>
