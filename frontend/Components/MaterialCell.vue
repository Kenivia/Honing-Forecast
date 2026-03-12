<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES, NORMAL_COLS as NORMAL_COLS, NUM_PIECES as NORMAL_ROWS, ALL_LABELS } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { InputColumn, modifyInputColumn, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"

const props = defineProps<{
    input_columns: (InputColumn | number[])[]
    index: number
}>()
const label: string = ALL_LABELS[props.index]
</script>

<template>
    <label class="hf-row-label">
        <span>{{ label }}</span>
        <img :src="iconPath(label)" :alt="label" />
    </label>
    <div v-for="col in input_columns">
        <input
            v-if="label !== 'Special Leap' && !Array.isArray(col)"
            type="text"
            :value="col[index] ?? ''"
            @change="modifyInputColumn(col as InputColumn, index, $event)"
        />
        <text v-else-if="label !== 'Special Leap'" type="text" :value="col[index] ?? ''" />
        <div v-else class="hf-input-placeholder" />
    </div>
    <!-- <input v-if="customLeftovers" type="text" :value="matsLeftover[label]" @input="setRecordValue(matsLeftover, label, $event)" /> -->
</template>
