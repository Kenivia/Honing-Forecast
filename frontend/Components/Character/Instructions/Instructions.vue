<script setup lang="ts">
import { CharProfile, useProfilesStore } from "@/stores/CharacterProfile"
import { uesRosterStore } from "@/stores/RosterConfig"
import { PIECE_NAMES } from "@/Utils/Constants"
import { iconPath } from "@/Utils/Helpers"
import { Upgrade, UpgradeStatus } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { computed } from "vue"
import InstructionRow from "./InstructionRow.vue"

const profile_store = useProfilesStore()
const { active_profile } = storeToRefs(useProfilesStore())
const { roster_config } = storeToRefs(uesRosterStore())

function sortUpgradeIndices(list: number[], upgradeArr: Upgrade[], specialInvalidIndex: number) {
    const output: number[] = []
    const copy = upgradeArr.slice()

    for (const [originalIndex, upgradeIndex] of list.entries()) {
        if (!upgradeArr[upgradeIndex]) continue
        if (originalIndex >= specialInvalidIndex) {
            if (!output.includes(upgradeIndex)) {
                output.push(upgradeIndex)
            }
            continue
        }

        const currentUpgrade = upgradeArr[upgradeIndex]
        for (const [index, candidate] of copy.entries()) {
            if (!candidate) continue
            const candidateSucceeded = Boolean(candidate.succeeded)
            const samePiece = candidate.piece_type === currentUpgrade.piece_type
            const sameType = Boolean(candidate.is_normal_honing) === Boolean(currentUpgrade.is_normal_honing)
            const ordered = candidate.upgrade_index <= currentUpgrade.upgrade_index
            if (sameType && samePiece && ordered && !candidateSucceeded && !output.includes(index)) {
                output.push(index)
            }
        }
    }

    return output
}
</script>
<template>
    <section class="hf-card hf-optimizer-card">
        <div class="hf-compact-row">
            <section class="hf-card hf-tap-card">
                <div class="hf-card-header">
                    <div class="hf-card-title"><span class="hf-card-title-dot" />Tap Instructions</div>
                    <span class="hf-card-hint">Top to bottom execution</span>
                </div>
                <div class="hf-card-body">
                    <p class="hf-copy">Empty state means no book and no breath on that tap.</p>
                    <p class="hf-copy">Enable progress updates in Controls for better optimization and live progression tracking.</p>

                    <div v-if="orderedUpgradeIndices.length" class="hf-upgrade-editor">
                        <div v-for="(upgradeIndex, orderIndex) in orderedUpgradeIndices" :key="`editor-${upgradeIndex}`" class="hf-upgrade-row">
                            <InstructionRow />
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </section>
</template>
