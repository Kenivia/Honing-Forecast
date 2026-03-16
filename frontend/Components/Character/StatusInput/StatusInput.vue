<script setup lang="ts">
import { useProfilesStore } from "@/stores/CharacterProfile"
import TickboxGrid from "./TickboxGrid.vue"
import { storeToRefs } from "pinia"
import { achieved_ilevel, check_adv_all_done, check_eligibility, check_ilevel_all_good, pending_ilevel } from "@/Utils/Helpers"
import { computed } from "vue"
import { TIER_OPTIONS } from "@/Utils/Constants"
import { SelectButton } from "primevue"
import HoverTooltip from "@/Components/Common/HoverTooltip.vue"
import TierSelectButton from "@/Components/Common/TierSelectButton.vue"

const { active_profile } = storeToRefs(useProfilesStore())
const tooltip_text = computed(() => {
    return active_profile.value.tier == 0
        ? check_eligibility()
            ? "Eligible for conversion to T4.5 Serca gear"
            : "Cannot convert to Serca because: " +
              [!check_adv_all_done() ? "Adv honing not all +40" : null, !(check_ilevel_all_good() === true) ? "Gear not all at least + 20" : null]
                  .filter((x) => x !== null)
                  .join(", \n")
        : check_eligibility()
          ? "Can go back to T4"
          : "Cannot convert back to T4 because +" + String(check_ilevel_all_good()) + " did not come directly from T4 transfer"
})
const tier_label_text = computed(() => {
    return active_profile.value.tier == 0 ? "Convert to T4.5 Serca" : "Revert back to T4"
})
function change_tier() {
    if (check_eligibility()) {
        active_profile.value.tier = active_profile.value.tier == 0 ? 1 : 0
    }
}
</script>

<template>
    <div class="hf-honing-row">
        <section class="hf-normal-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Normal Honing</div>
                <label class="hf-achieved-ilevel">Achieved ilevel: {{ achieved_ilevel() }}</label>
                <label class="hf-pending-ilevel">Pending ilevel: {{ pending_ilevel() }}</label>

                <TierSelectButton :labelText="tier_label_text" :tooltipText="tooltip_text" :checkEligibility="check_eligibility" @change-tier="change_tier" />
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="normal" />
            </div>
        </section>

        <section v-if="active_profile.tier == 0" class="hf-advanced-card">
            <div class="hf-card-header">
                <div class="hf-card-title"><span class="hf-card-title-dot" />Advanced Honing</div>
            </div>
            <div class="hf-card-body">
                <TickboxGrid grid_type="adv" />
            </div>
        </section>
    </div>
</template>
<style>
.hf-pending-ilevel {
    color: var(--checkbox-checked-bg);
}

.hf-achieved-ilevel {
    color: var(--checkbox-done-bg);
}
.hf-normal-card {
    min-width: 0;
    width: min(100%, 800px);
}
.hf-advanced-card {
    width: min(100%, 262px);
    min-width: 0;
}
.hf-honing-row {
    width: min(100%, 1074px);
    display: flex;
    gap: 12px;
    align-items: center;
    min-width: 0;
    flex-wrap: wrap;
}
</style>
