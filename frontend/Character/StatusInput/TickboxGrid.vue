<script setup lang="ts">
import { useProfilesStore } from "@/stores/CharacterProfile"
import { PIECE_NAMES } from "@/Utils/Constants"
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
    if (relevant_grid.every((row: UpgradeStatus[]) => row[col] == UpgradeStatus.Done)) {
        return UpgradeStatus.Done
    }
    if (relevant_grid.every((row: UpgradeStatus[]) => row[col] == UpgradeStatus.Want)) {
        return UpgradeStatus.Want
    }
    return UpgradeStatus.NotYet
}
function toggleTopCol(col: number) {
    for (let row = 0; row < TOP_ROWS; row++) {
        next[row][col] = newState
    }
    topGrid.value = next
    stopGridDrag()
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
            <div class="hf-cell-grid hf-cell-grid-head" :style="{ gridTemplateColumns: `repeat(${TOP_COLS}, 26px)` }">
                <button
                    v-for="col in TOP_COLS"
                    :key="`top-col-${col}`"
                    class="hf-cell hf-cell-header"
                    :class="{ selected: check_all_same(col - 1) }"
                    @click="toggleTopCol(col - 1)"
                >
                    +{{ col }}
                </button>
            </div>
            <div v-for="row in TOP_ROWS" :key="`top-row-${row}`" class="hf-cell-grid" :style="{ gridTemplateColumns: `repeat(${TOP_COLS}, 26px)` }">
                <button
                    v-for="col in TOP_COLS"
                    :key="`top-${row}-${col}`"
                    class="hf-cell"
                    :class="{ selected: topGrid[row - 1][col - 1] }"
                    @pointerdown.prevent="startTopDrag(row - 1, col - 1, $event)"
                    @pointerenter="dragTopCell(row - 1, col - 1)"
                    @click.prevent="onTopCellClick(row - 1, col - 1, $event)"
                />
            </div>
        </div>
    </div>
</template>
