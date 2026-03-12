<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES, NORMAL_COLS as NORMAL_COLS, NUM_PIECES as NORMAL_ROWS } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { InputColumn, modifyInputColumn, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"

const props = defineProps<{
    input_columns: (InputColumn | Record<string, number>)[]
    label: string
}>()
</script>

<template>
    <label class="hf-row-label">
        <span>{{ label }}</span>
        <img :src="iconPath(label)" :alt="label" />
    </label>
    <div v-for="col in input_columns">
        <input
            v-if="label !== 'Special Leap' && col instanceof InputColumn"
            type="text"
            :value="col[label] ?? ''"
            @change="modifyInputColumn(col as InputColumn, label, $event)"
        />
        <text v-else-if="label !== 'Special Leap'" type="text" :value="col[label] ?? ''" />
        <div v-else class="hf-input-placeholder" />
    </div>
    <!-- <input v-if="customLeftovers" type="text" :value="matsLeftover[label]" @input="setRecordValue(matsLeftover, label, $event)" /> -->
</template>
