<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES } from "@/Utils/Constants"
import { get_piece_name, iconPath } from "@/Utils/Helpers"
import { Upgrade, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed } from "vue"
import InstructionRow from "./InstructionRow.vue"

const profile_store = useProfilesStore()
const { active_profile } = storeToRefs(useProfilesStore())
const { roster_config } = storeToRefs(uesRosterStore())

const props = defineProps<{
    orderIndex: number
}>()
</script>
<template>
    <div class="hf-upgrade-order">{{ props.orderIndex + 1 }}</div>
    <div class="hf-upgrade-meta">
        <div class="hf-upgrade-topline">
            <span class="hf-upgrade-name">{{ get_piece_name(upgradeArr[upgradeIndex]) }}</span>
            <span v-if="isFreeTapUpgrade(upgradeIndex)" class="hf-upgrade-free"> Free tap {{ formatSig(freeTapChance(upgradeIndex) * 100, 3) }}% </span>
        </div>
        <span class="hf-upgrade-plan">{{ getNextTapInstruction(upgradeIndex) }}</span>

        <div class="hf-upgrade-actions-row" v-if="getNextTapPlan(upgradeIndex)?.status === 'active'">
            <span class="hf-action-chip tap"> Tap {{ getNextTapPlan(upgradeIndex)?.tapIndex }}/{{ getNextTapPlan(upgradeIndex)?.pityLength }} </span>
            <span v-if="getNextTapPlan(upgradeIndex)?.useJuice" class="hf-action-chip juice">
                <img :src="iconPath(getNextTapPlan(upgradeIndex)?.juiceLabel ?? '')" :alt="getNextTapPlan(upgradeIndex)?.juiceLabel ?? ''" />
                {{ getNextTapPlan(upgradeIndex)?.juiceLabel }}
            </span>
            <span v-if="(getNextTapPlan(upgradeIndex)?.bookTier ?? 0) > 0" class="hf-action-chip book">
                <img :src="iconPath(getNextTapPlan(upgradeIndex)?.bookLabel ?? '')" :alt="getNextTapPlan(upgradeIndex)?.bookLabel ?? ''" />
                {{ getNextTapPlan(upgradeIndex)?.bookLabel }}
            </span>
            <span v-if="!getNextTapPlan(upgradeIndex)?.useJuice && (getNextTapPlan(upgradeIndex)?.bookTier ?? 0) === 0" class="hf-action-chip muted">
                No juice / no book
            </span>
        </div>
    </div>

    <div v-if="allowManualState" class="hf-upgrade-controls">
        <label>
            Progress
            <input
                type="number"
                min="0"
                :max="getPityLength(upgradeArr[upgradeIndex])"
                :value="getUpgradeProgress(upgradeIndex)"
                @input="onUpgradeProgressInput(upgradeIndex, $event)"
            />
        </label>
        <button class="hf-mini-btn" @click="toggleUpgradeSucceeded(upgradeIndex)">
            {{ getUpgradeSucceeded(upgradeIndex) ? "Undo" : "Succeed" }}
        </button>
    </div>
    <div v-else class="hf-upgrade-controls hf-upgrade-controls-readonly">
        <span class="hf-upgrade-readonly-note">Progress updates disabled</span>
    </div>
</template>
