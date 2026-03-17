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

// function sortUpgradeIndices(list: number[], upgradeArr: Upgrade[], specialInvalidIndex: number) {
//     const output: number[] = []
//     const copy = upgradeArr.slice()

//     for (const [originalIndex, upgradeIndex] of list.entries()) {
//         if (!upgradeArr[upgradeIndex]) continue
//         if (originalIndex >= specialInvalidIndex) {
//             if (!output.includes(upgradeIndex)) {
//                 output.push(upgradeIndex)
//             }
//             continue
//         }

//         const currentUpgrade = upgradeArr[upgradeIndex]
//         for (const [index, candidate] of copy.entries()) {
//             if (!candidate) continue
//             const candidateSucceeded = Boolean(candidate.succeeded)
//             const samePiece = candidate.piece_type === currentUpgrade.piece_type
//             const sameType = Boolean(candidate.is_normal_honing) === Boolean(currentUpgrade.is_normal_honing)
//             const ordered = candidate.upgrade_index <= currentUpgrade.upgrade_index
//             if (sameType && samePiece && ordered && !candidateSucceeded && !output.includes(index)) {
//                 output.push(index)
//             }
//         }
//     }

//     return output
// }

function sort_upgrades(): Upgrade[] {
    if (!active_profile.value.optimizer_worker_bundle.result) {
        return []
    }

    let output: Upgrade[] = []
    let upgrade_arr: Upgrade[] = active_profile.value.optimizer_worker_bundle.result.upgrade_arr
    for (let index = 0; index < output.length; index++) {
        upgrade_arr[index].this_special_chance = active_profile.value.optimizer_worker_bundle.result.latest_special_probs[index]
    }
    let copy = upgrade_arr.slice()
    let special_state = active_profile.value.optimizer_worker_bundle.result.special_state
    let special_invalid_index = active_profile.value.optimizer_worker_bundle.result.special_invalid_index
    // console.log(list.slice(), special_invalid_index)              this_upgrade.this_special_chance = active_profile.value.optimizer_worker_bundle.result.

    for (const [index_in_special_state, index_in_upgrade_arr] of special_state.entries()) {
        // console.log(original_index, u_index, output)
        if (index_in_special_state >= special_invalid_index) {
            // console.log(output.slice(), u_index, u_index in output)
            if (!output.includes(index_in_upgrade_arr)) {
                output.push(upgrade_arr[index_in_upgrade_arr])
            }
        } else {
            let this_upgrade = upgrade_arr[index_in_upgrade_arr]
            for (const [index, upgrade] of copy.entries()) {
                if (
                    upgrade.upgrade_index <= this_upgrade.upgrade_index &&
                    upgrade.is_normal_honing &&
                    upgrade.piece_type == this_upgrade.piece_type &&
                    !upgrade.succeeded &&
                    !output.includes(upgrade)
                ) {
                    output.push(this_upgrade)

                    // console.log(u_index, index, this_upgrade.upgrade_index)
                }
            }
        }
    }

    // console.log(output)
    return output
}

let sorted_upgrade_arr = computed(sort_upgrades)
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
                    <div v-if="active_profile.optimizer_worker_bundle.result" class="hf-upgrade-instruction">
                        <div v-for="(upgrade, perform_order) in sorted_upgrade_arr" :key="`instructions-${sorted_upgrade_arr}`" class="hf-upgrade-row">
                            <InstructionRow
                                :upgrade="upgrade"
                                :perform_order="perform_order"
                                :special_invalid_index="active_profile.optimizer_worker_bundle.result.special_invalid_index"
                            />
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </section>
</template>
