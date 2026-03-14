<script setup lang="ts">
import { uesRosterStore } from "@/stores/RosterConfig"
import { ALL_LABELS, BUNDLE_SIZE } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "../MaterialCell.vue"

const { roster_config } = storeToRefs(uesRosterStore())
</script>

<template>
    <div class="hf-roster-inputs">
        <div class="hf-table-title-row">
            <span style="text-align: right; padding-right: 15px">Roster Bound Mats</span>
            <span>Tradable Mats</span>
            <span>Market price</span>
            <!-- <span v-if="customLeftovers">Left</span> -->
        </div>
        <div v-for="(label, index) in ALL_LABELS" :key="`roster-input-${label}`" class="hf-mats-row">
            <MaterialCell
                :input_column="roster_config.roster_mats_owned"
                :row="index"
                :show_label="true"
                :setter="
                    (val) => {
                        roster_config.roster_mats_owned.data[index] = val
                    }
                "
            />
            <MaterialCell
                :input_column="roster_config.tradable_mats_owned"
                :row="index"
                :show_label="false"
                :setter="
                    (val) => {
                        roster_config.tradable_mats_owned.data[index] = val
                    }
                "
            />
            <MaterialCell
                :input_column="roster_config.mats_prices"
                :row="index"
                :show_label="false"
                :setter="
                    (val) => {
                        roster_config.mats_prices.data[index] = val
                    }
                "
                :suffix="BUNDLE_SIZE[index] > 1 ? 'x' + BUNDLE_SIZE[index].toLocaleString('en-US') : ''"
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
