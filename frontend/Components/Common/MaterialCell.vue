<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { useRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES, NORMAL_COLS as NORMAL_COLS, NUM_PIECES as NORMAL_ROWS, ALL_LABELS } from "@/Utils/Constants"
import { cssVar, iconPath } from "@/Utils/Helpers"
import { InputColumn, get_modified_cell, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed, ref, Ref, watchEffect } from "vue"

const props = defineProps<{
    input_column: InputColumn | number[]
    row: number
    label?: string
    setter?: (val: string) => void // optional so read-only columns don't need it
    suffix?: string
    input_color?: string
    is_percentage?: boolean
    hide_tick?:boolean
}>()

const resolved_color = computed(() => {
    return cssVar(props.input_color, props.input_color)
})
const this_data = ref(String(!Array.isArray(props.input_column) ? (props.input_column as InputColumn).data[props.row] : props.input_column[props.row]))
</script>

<template>
    <div class="hf-material-cell">
        <input v-if="!hide_tick && label && !Array.isArray(input_column)" type="checkbox" v-model="(input_column as InputColumn).enabled[row]" />
        <label v-if="label" class="hf-row-label">
            <span>{{ label }}</span>
            <img :src="iconPath(label)" :alt="label" />
        </label>
        <input
            v-if="!Array.isArray(input_column)"
            type="text"
            class="hf-material-cell-input"
            :style="{ color: resolved_color }"
            v-model="this_data"
            @change="
                ((console.log('change'), (this_data = get_modified_cell(input_column, row, $event))), setter(get_modified_cell(input_column, row, $event)))
            "
        />
        <label v-else class="hf-material-cell-result" :style="{ color: resolved_color }" type="text">{{
            is_percentage
                ? (input_column[row] * 100).toFixed(2) + "%"
                : input_column[row].toLocaleString("en-US", {
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
    --hf-cell-input-width: 100px;
    --hf-cell-label-width: 150px;
    --hf-cell-icon-size: 32px;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 6px;
    font-size: 16px;
    min-width: 0;
    text-align: right;
    padding-right: 8px;
}

input.hf-material-cell-input {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 6px;
    font-size: 16px;
    min-width: 0;
    text-align: left;
    padding-right: 8px;
    width: var(--hf-cell-input-width);
}

.hf-material-cell-result {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    font-size: 16px;
    min-width: 0;
    text-align: left;
    padding-right: 8px;
    width: var(--hf-cell-input-width);
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
    width: var(--hf-cell-label-width);
}

.hf-row-label img {
    width: var(--hf-cell-icon-size);
    height: var(--hf-cell-icon-size);
    object-fit: contain;
}

@media (max-width: 900px) {
    .hf-material-cell {
        --hf-cell-input-width: 74px;
        --hf-cell-label-width: 104px;
        --hf-cell-icon-size: 24px;
        gap: 4px;
        font-size: 14px;
        padding-right: 4px;
    }

    input.hf-material-cell-input,
    .hf-material-cell-result {
        font-size: 14px;
        padding-right: 4px;
    }

    .hf-row-label {
        font-size: 12px;
        padding-right: 4px;
    }

    .hf-material-cell-suffix {
        font-size: 10px;
        padding-right: 4px;
    }
}
</style>
