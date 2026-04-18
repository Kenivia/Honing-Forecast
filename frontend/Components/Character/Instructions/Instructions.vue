<script setup lang="ts">
import { Upgrade } from "@/Utils/Interfaces"
import { storeToRefs } from "pinia"
import { ref, watch } from "vue"
import InstructionRow from "./InstructionRow.vue"
import { useRosterStore } from "@/Stores/RosterConfig"

const { active_profile } = storeToRefs(useRosterStore())

// This sorts the upgrades into an order that can actually be performed in game
// special_state is already guaranteed to be valid on the rust side, but it doesn't tell us how to do the non-special taps
// this gives a suggestion
function sort_upgrades(): [Upgrade, number][] {
    if (!active_profile.value.optimizer_worker_bundle.result) {
        return []
    }

    let output: number[] = []
    let indices_in_special_state: number[] = []
    let upgrade_arr: Upgrade[] = active_profile.value.optimizer_worker_bundle.result.upgrade_arr
    // let copy = upgrade_arr.slice()
    let special_state: number[] = active_profile.value.optimizer_worker_bundle.result.special_state
    const special_chance_map = new Map()
    for (let index = 0; index < special_state.length; index++) {
        special_chance_map.set(special_state[index], active_profile.value.optimizer_worker_bundle.result.latest_special_probs[index])
    }

    let special_invalid_index = active_profile.value.optimizer_worker_bundle.result.special_invalid_index
    for (const [index_in_special_state, index_in_upgrade_arr] of special_state.entries()) {
        // console.log(original_index, u_index, output)
        if (index_in_special_state >= special_invalid_index) {
            // console.log(output.slice(), u_index, u_index in output)
            if (!output.includes(index_in_upgrade_arr)) {
                output.push(index_in_upgrade_arr)
            }
        } else {
            let this_upgrade = upgrade_arr[index_in_upgrade_arr]
            for (const [index, upgrade] of upgrade_arr.entries()) {
                if (
                    upgrade.upgrade_index <= this_upgrade.upgrade_index &&
                    upgrade.is_normal_honing &&
                    upgrade.piece_type == this_upgrade.piece_type &&
                    !upgrade.succeeded &&
                    !output.includes(index)
                ) {
                    output.push(index)
                    indices_in_special_state.push(special_state.findIndex((x) => x == index))
                }
            }
        }
    }

    return output.map((x) => {
        const upgrade = upgrade_arr[x]
        const index_in_special = special_state.findIndex((y) => y == x)
        return [
            { ...upgrade, this_special_chance: special_chance_map.get(x) }, // Shallow clone
            index_in_special,
        ]
    })
}

const sorted_upgrade_arr = ref(sort_upgrades())
watch(
    () => active_profile.value.optimizer_worker_bundle.result?.upgrade_arr,
    () => {
        sorted_upgrade_arr.value = sort_upgrades()
        // console.log("resorted", sorted_upgrade_arr.value.length)
    },
    { deep: true },
)
</script>
<template>
    <section class="hf-card hf-instructions-pane">
        <div class="hf-card-header">
            <div class="hf-card-title"><span class="hf-card-title-dot" />Tap Instructions</div>
            <span class="hf-card-hint">Go from top to bottom</span>
        </div>
        <div class="hf-card-body">
            <div v-if="active_profile.optimizer_worker_bundle.result">
                <div
                    v-for="([upgrade, index_in_special_state], perform_order) in sorted_upgrade_arr"
                    :key="`instructions-${upgrade.upgrade_index}-${upgrade.piece_type}-${upgrade.is_normal_honing}`"
                >
                    <InstructionRow
                        :upgrade="upgrade"
                        :perform_order="perform_order"
                        :special_invalid_index="active_profile.optimizer_worker_bundle.result.special_invalid_index"
                        :index_in_special_state="index_in_special_state"
                    />
                </div>
            </div>
        </div>
    </section>
</template>
<style>
.hf-instructions-pane {
    width: min(100%, 700px);
    overflow-x: auto;
    overflow-y: scroll;
    max-height: 1500px;
}
</style>
