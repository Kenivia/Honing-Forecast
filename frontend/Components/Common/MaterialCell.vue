<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES, NORMAL_COLS as NORMAL_COLS, NUM_PIECES as NORMAL_ROWS, ALL_LABELS } from "@/Utils/Constants"
import { cssVar, iconPath } from "@/Utils/Helpers"
import { InputColumn, get_modified_cell, UpgradeStatus, forbid_non_numeric } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed, Ref } from "vue"

const props = defineProps<{
    input_column: InputColumn | number[]
    row: number
    label?: string
    setter?: (val: string) => void // optional so read-only columns don't need it
    suffix?: string
    input_color?: string
}>()

const resolved_color = computed(() => {
    // console.log(cssVar(props.input_color, props.input_color))
    return cssVar(props.input_color, props.input_color)
})
</script>

<template>
    <div class="hf-material-cell">
        <label v-if="label" class="hf-row-label">
            <span>{{ label }}</span>
            <img :src="iconPath(label)" :alt="label" />
        </label>

        <input
            v-if="!Array.isArray(input_column)"
            type="text"
            class="hf-material-cell-input"
            :style="{ color: resolved_color }"
            :value="input_column.data[row]"
            @change="setter(get_modified_cell(input_column, row, $event))"
            @input="setter(forbid_non_numeric(input_column, row, $event))"
        />
        <label v-else-if="label !== 'Special Leap'" class="hf-material-cell-result" :style="{ color: resolved_color }" type="text">{{
            input_column[row].toLocaleString("en-US", {
                minimumFractionDigits: 0, // show decimals for small K/M/B
                maximumFractionDigits: 0,
            })
        }}</label>
        <label class="hf-material-cell-suffix" v-if="suffix">{{ suffix }}</label>
    </div>
    <!-- <input v-if="customLeftovers" type="text" :value="matsLeftover[label]" @input="setRecordValue(matsLeftover, label, $event)" /> -->
</template>
<style>
.hf-material-cell {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 6px;
    font-size: 18px;
    min-width: 0;
    text-align: right;
    padding-right: 8px;
}

input.hf-material-cell-input {
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
    gap: 6px;
    font-size: 18px;
    min-width: 0;
    text-align: left;
    padding-right: 8px;
    width: 100px;
}

.hf-material-cell-result {
    display: flex;
    align-items: flex-start;
    justify-content: flex-start;
    gap: 6px;
    font-size: 18px;
    min-width: 0;
    text-align: left;
    padding-right: 8px;
    width: 100px;
}

.hf-material-cell-suffix {
    color: var(--text-very-muted);
    font-size: 12px;
    min-width: 0;
    text-align: left;
    padding-right: 8px;
}
.hf-row-label {
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 13px;
    min-width: 0;
    text-align: right;
    padding-right: 8px;
    width: 150px;
}

.hf-row-label img {
    width: 32px;
    height: 32px;
    object-fit: contain;
}
</style>
