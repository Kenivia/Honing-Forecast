<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { PIECE_NAMES, TOP_COLS as NORMAL_COLS, TOP_ROWS as NORMAL_ROWS } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"

const profile_store = useProfilesStore()

const active_profile: CharProfile = profile_store.activeProfile()
const props = defineProps<{
    grid_type: "normal" | "adv"
}>()

const relevant_grid = props.grid_type == "normal" ? active_profile.normal_grid : active_profile.adv_grid

function check_all_same(col: number) {
    if (relevant_grid.data.every((row: UpgradeStatus[]) => row[col] == UpgradeStatus.Done)) {
        return UpgradeStatus.Done
    }
    if (relevant_grid.data.every((row: UpgradeStatus[]) => row[col] == UpgradeStatus.Want)) {
        return UpgradeStatus.Want
    }
    return UpgradeStatus.NotYet
}

function change_one(row: number, col: number) {
    let current = relevant_grid[row][col]
    if (current == UpgradeStatus.NotYet) {
        if (col > 0 && relevant_grid[row][col - 1] == UpgradeStatus.NotYet) {
            for (let c = 0; c <= col; c++) {
                relevant_grid[row][c] = UpgradeStatus.Done
            }
        } else {
            relevant_grid[row][col] = UpgradeStatus.Want
        }
    } else if (current == UpgradeStatus.Want) {
        for (let c = 0; c <= col; c++) {
            relevant_grid[row][c] = UpgradeStatus.Done
        }
    } else if (current == UpgradeStatus.Done) {
        for (let c = col; c < relevant_grid[row].length; c++) {
            relevant_grid[row][c] = UpgradeStatus.NotYet
        }
    }
}
</script>
<template>
    <div class="hf-grid-content">
        <div class="hf-label-col">
            <div class="hf-label-row" />
            <div v-for="piece in PIECE_NAMES" :key="piece" class="hf-label-row">
                <div class="hf-equip-label">
                    <span>{{ piece }}</span>
                    <img :src="iconPath(piece)" :alt="piece" />
                </div>
            </div>
        </div>
        <div ref="normalGridScrollRef" class="hf-grid-scroll">
            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${NORMAL_COLS}, 26px)` }">
                <button v-for="col in NORMAL_COLS" :key="`top-col-${col}`" class="hf-cell hf-cell-header" :class="{ selected: check_all_same(col - 1) }">
                    +{{ col }}
                </button>
            </div>
            <div v-for="row in NORMAL_ROWS" :key="`top-row-${row}`" class="hf-cell-grid" :style="{ gridTemplateColumns: `repeat(${NORMAL_COLS}, 26px)` }">
                <button v-for="col in NORMAL_COLS" :key="`top-${row}-${col}`" class="hf-cell" :class="{ selected: relevant_grid[row - 1][col - 1] }" />
                <!-- @pointerdown.prevent="startTopDrag(row - 1, col - 1, $event)"
                    @pointerenter="dragTopCell(row - 1, col - 1)"
                    @click.prevent="onTopCellClick(row - 1, col - 1, $event)" -->
            </div>
        </div>
    </div>
</template>
