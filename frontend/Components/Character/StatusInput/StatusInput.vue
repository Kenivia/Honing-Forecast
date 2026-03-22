<script setup lang="ts">
import { useProfilesStore } from "@/Stores/CharacterProfile"
import TickboxGrid from "./TickboxGrid.vue"
import { storeToRefs } from "pinia"
import { achieved_ilevel, check_adv_all_done, check_all_plus_20, check_revert_ilevel_ok, pending_ilevel } from "@/Utils/Helpers"
import { computed, watch } from "vue"
import { ADV_COLS, ALL_LABELS, NORMAL_COLS, NUM_PIECES, PLUS_TIER_CONVERSION } from "@/Utils/Constants"
import TierConvertButton from "@/Components/Common/TierConvertButton.vue"
import { input_column_to_num, parse_input, UpgradeStatus } from "@/Utils/Interfaces"
import ControlPanel from "../ControlPanel.vue"

const { active_profile } = storeToRefs(useProfilesStore())
const tooltip_text = computed(() => {
    return active_profile.value.tier == 0
        ? check_all_plus_20() && check_adv_all_done()
            ? "Eligible for conversion to T4.5 Serca gear"
            : "Warning! " +
              [!check_adv_all_done() ? "All Adv honing will be set to +40" : null, !check_all_plus_20() ? "All Gear will be set to +20 (T4)" : null]
                  .filter((x) => x !== null)
                  .join(", \n")
        : check_revert_ilevel_ok() === true
          ? "Can go back to T4"
          : "Cannot convert back to T4 because +" + String(check_revert_ilevel_ok()) + " cannot be mapped directly to a T4 upgrade"
})
const tier_label_text = computed(() => {
    return active_profile.value.tier == 0 ? "Convert to T4.5 Serca" : "Revert back to T4"
})
function change_tier() {
    let old_tier = active_profile.value.tier
    if (check_revert_ilevel_ok() === true) {
        active_profile.value.tier = active_profile.value.tier == 0 ? 1 : 0
    }
    let new_tier = active_profile.value.tier

    if (new_tier === null || old_tier === null || new_tier == old_tier) return
    if (ALL_LABELS.length != 2) {
        // This doesn't work for more tiers and should be updated when more tiers comes eventually
        throw new Error("conversion between more than 2 tiers not implemented yet")
    }

    active_profile.value.optimizer_worker_bundle?.cancel_and_clear_prev_result()
    active_profile.value.histogram_worker_bundle?.cancel_and_clear_prev_result()
    active_profile.value.evaluation_worker_bundle?.cancel_and_clear_prev_result()

    let num_array_old = input_column_to_num(active_profile.value.bound_budgets[old_tier])

    let multiplied_indices = [0, 1, 2, 4] // red, blue, leaps, fusion
    let multiplier = new_tier == 1 ? 0.2 : 5
    multiplied_indices.forEach(
        (index) =>
            (active_profile.value.bound_budgets[new_tier].data[index] = parse_input(
                active_profile.value.bound_budgets[old_tier],
                index,
                String(num_array_old[index] * multiplier),
            ).toLocaleString()),
    )
    // Special leaps also multiplied
    active_profile.value.special_budget.data[0] = parse_input(
        active_profile.value.special_budget,
        0,
        String(input_column_to_num(active_profile.value.special_budget)[0] * multiplier),
    ).toLocaleString()

    let stay_same_indices = [3, 5, 6, 7] // shards, gold, silver, red juice
    stay_same_indices.forEach((index) => (active_profile.value.bound_budgets[new_tier].data[index] = active_profile.value.bound_budgets[old_tier].data[index]))

    // special case for blue juice
    let new_num_juice_avail = (ALL_LABELS[new_tier].length - 7) / 2
    let new_index = 7 + new_num_juice_avail
    let old_num_juice_avail = (ALL_LABELS[old_tier].length - 7) / 2
    let old_index = 7 + old_num_juice_avail
    active_profile.value.bound_budgets[new_tier].data[new_index] = active_profile.value.bound_budgets[old_tier].data[old_index]

    // the rest have separate values between tiers

    for (let row = 0; row < NUM_PIECES; row++) {
        let highest_done = Math.max(new_tier == 1 ? 20 : 11, active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Done) + 1)
        let highest_want = Math.max(
            new_tier == 1 ? 20 : 11,
            active_profile.value.normal_grid[row].findLastIndex((value) => value == UpgradeStatus.Want || value == UpgradeStatus.Done) + 1,
        )
        let converted_done = PLUS_TIER_CONVERSION[old_tier][String(highest_done)]
        let converted_want = highest_want > 0 ? PLUS_TIER_CONVERSION[old_tier][String(highest_want)] : converted_done
        for (let col = 0; col < NORMAL_COLS; col++) {
            if (col < converted_done) {
                active_profile.value.normal_grid[row][col] = UpgradeStatus.Done
            } else if (col < converted_want) {
                active_profile.value.normal_grid[row][col] = UpgradeStatus.Want
            } else {
                active_profile.value.normal_grid[row][col] = UpgradeStatus.NotYet
            }
        }
    }
    if (new_tier == 1) {
        for (let row = 0; row < NUM_PIECES; row++) {
            for (let col = 0; col < ADV_COLS; col++) {
                active_profile.value.adv_grid[row][col] = UpgradeStatus.Done
            }
        }
    }
}
</script>

<template>
    <div class="hf-honing-row">
        <section class="hf-card-normal">
            <div class="hf-card-header">
                <div style="display: flex; gap: 20px">
                    <label class="hf-achieved-ilevel">Achieved ilevel: {{ achieved_ilevel(active_profile) }}</label>
                    <label class="hf-pending-ilevel">Pending ilevel: {{ pending_ilevel(active_profile) }}</label>
                </div>
                <TierConvertButton
                    :labelText="tier_label_text"
                    :tooltipText="tooltip_text"
                    :checkEligibility="() => check_revert_ilevel_ok() === true"
                    @change-tier="change_tier"
                    :show-tooltip-only-on-disabled="false"
                    :warning="active_profile.tier == 0 && !(check_all_plus_20() && check_adv_all_done())"
                />
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="normal" />
            </div>
        </section>
        <div class="hf-side-cards">
            <section v-if="active_profile.tier == 0" class="hf-card-adv">
                <div class="hf-card-header">
                    <div class="hf-card-title"><span class="hf-card-title-dot" />Advanced Honing</div>
                </div>
                <div class="hf-card-body">
                    <TickboxGrid grid_type="adv" />
                </div>
            </section>
            <ControlPanel />
        </div>
    </div>
</template>
<style scoped>
.hf-card-normal {
    background: var(--hf-bg-panel);
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    overflow: visible;
    width: fit-content;
    max-width: 100%;
    padding: 4px;
}

.hf-card-adv {
    background: var(--hf-bg-panel);
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    overflow: visible;
    width: 262px;
    min-width: 262px;
}

.hf-honing-row {
    width: min(100%, 1074px);
    display: flex;
    gap: 10px;
    align-items: start;
    min-width: 0;
    flex-wrap: wrap;
    justify-content: center;
}

.hf-side-cards {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
}

.hf-normal-card {
    min-width: 0;
    width: min(100%, 800px);
}
.hf-advanced-card {
    width: min(100%, 262px);
    min-width: 0;
}

.hf-card-normal :deep(.hf-card-header) {
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 8px;
}

@media (max-width: 1000px) {
    .hf-honing-row {
        width: 100%;
        align-items: stretch;
        justify-content: stretch;
        flex-direction: column;
    }

    .hf-card-normal {
        width: 100%;
    }

    .hf-side-cards {
        width: 100%;
        flex-direction: column;
    }

    .hf-card-adv {
        width: 100%;
        min-width: 0;
    }
}
</style>
