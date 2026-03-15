<script setup lang="ts">
import { uesRosterStore as useRosterStore } from "@/stores/RosterConfig"
import { ALL_LABELS, BUNDLE_SIZE, TIER_OPTIONS } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { computed } from "vue"
import { SelectButton } from "primevue"

const { roster_config } = storeToRefs(useRosterStore())

const tier = computed(() => roster_config.value.tier)
</script>

<template>
    <SelectButton v-model="roster_config.tier" :options="TIER_OPTIONS" option-label="label" option-value="value" class="hf-header-link-btn" />
    <div class="hf-roster-inputs">
        <div class="hf-table-title-row">
            <span style="text-align: right; padding-right: 15px">Roster Bound Mats</span>
            <span>Tradable Mats</span>
            <span>Market price</span>

            <!-- <span v-if="customLeftovers">Left</span> -->
        </div>
        <div v-for="(label, row) in ALL_LABELS[tier]" :key="`roster-input-${label}`" class="hf-mats-row">
            <MaterialCell
                :input_column="roster_config.roster_mats_owned[tier]"
                :row="row"
                :show_label="true"
                :setter="
                    (val) => {
                        roster_config.roster_mats_owned[tier].data[row] = val
                    }
                "
            />
            <MaterialCell
                :input_column="roster_config.tradable_mats_owned[tier]"
                :row="row"
                :show_label="false"
                :setter="
                    (val) => {
                        roster_config.tradable_mats_owned[tier].data[row] = val
                    }
                "
            />
            <MaterialCell
                :input_column="roster_config.mats_prices[tier]"
                :row="row"
                :show_label="false"
                :setter="
                    (val) => {
                        roster_config.mats_prices[tier].data[row] = val
                    }
                "
                :suffix="BUNDLE_SIZE[row] > 1 ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US') : ''"
            />
        </div>
    </div>
</template>
<style>
.hf-roster-inputs {
    display: grid;
    grid-template-columns: 250px 120px 120px;
    align-items: center; /* optional: vertically center each cell */
    gap: 8px; /* optional: spacing between cells */
}
</style>
