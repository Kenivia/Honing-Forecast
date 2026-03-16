<script setup lang="ts">
import { uesRosterStore as useRosterStore } from "@/stores/RosterConfig"
import { ALL_LABELS, BUDGET_NARROW_WIDTH, BUNDLE_SIZE, NARROW_WIDTH, TIER_OPTIONS } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { computed } from "vue"
import { SelectButton } from "primevue"
import { useMediaIsNarrow } from "@/Utils/WindowSize"

const { roster_config } = storeToRefs(useRosterStore())

const tier = computed(() => roster_config.value.tier)
const { isNarrow } = useMediaIsNarrow(BUDGET_NARROW_WIDTH)

const column_template = computed(() => (isNarrow ? "1fr" : "1fr 1fr"))
</script>

<template>
    <SelectButton
        v-if="isNarrow"
        v-model="roster_config.tier"
        :options="TIER_OPTIONS"
        option-label="label"
        option-value="value"
        class="hf-roster-tier-select"
        :option-disabled="(data) => (data.value === roster_config.tier ? true : false)"
    />
    <div class="hf-outer-budget-grid" :class="{ narrow: isNarrow }">
        <div v-if="!isNarrow || tier == 0" class="hf-roster-inputs-tier-4" :style="{ gridRow: `span ${String(ALL_LABELS[0].length + 1)}` }">
            <div class="hf-table-title-row">
                <span style="text-align: right; padding-right: 15px">Roster Bound Mats</span>
                <span>Tradable Mats</span>
                <span>Market price</span>

                <!-- <span v-if="customLeftovers">Left</span> -->
            </div>
            <div v-for="(label, row) in ALL_LABELS[0]" :key="`roster-input-${label}`" class="hf-mats-row">
                <MaterialCell
                    :input_column="roster_config.roster_mats_owned[0]"
                    :row="row"
                    :show_label="true"
                    :setter="
                        (val) => {
                            roster_config.roster_mats_owned[0].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.tradable_mats_owned[0]"
                    :row="row"
                    :show_label="false"
                    :setter="
                        (val) => {
                            roster_config.tradable_mats_owned[0].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.mats_prices[0]"
                    :row="row"
                    :show_label="false"
                    :setter="
                        (val) => {
                            roster_config.mats_prices[0].data[row] = val
                        }
                    "
                    :suffix="BUNDLE_SIZE[row] > 1 ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US') : ''"
                />
            </div>
        </div>

        <div v-if="!isNarrow || tier == 1" class="hf-roster-inputs-serca" :style="{ gridRow: `span ${String(ALL_LABELS[1].length + 1)}` }">
            <div class="hf-table-title-row">
                <span style="text-align: right; padding-right: 15px">Roster Bound Mats</span>
                <span>Tradable Mats</span>
                <span>Market price</span>

                <!-- <span v-if="customLeftovers">Left</span> -->
            </div>
            <div v-for="(label, row) in ALL_LABELS[1]" :key="`roster-input-${label}`" class="hf-mats-row">
                <MaterialCell
                    :input_column="roster_config.roster_mats_owned[1]"
                    :row="row"
                    :show_label="true"
                    :setter="
                        (val) => {
                            roster_config.roster_mats_owned[1].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.tradable_mats_owned[1]"
                    :row="row"
                    :show_label="false"
                    :setter="
                        (val) => {
                            roster_config.tradable_mats_owned[1].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.mats_prices[1]"
                    :row="row"
                    :show_label="false"
                    :setter="
                        (val) => {
                            roster_config.mats_prices[1].data[row] = val
                        }
                    "
                    :suffix="BUNDLE_SIZE[row] > 1 ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US') : ''"
                />
            </div>
        </div>
    </div>
</template>
<style>
.hf-outer-budget-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: subgrid; /* rows are shared */
}
.hf-outer-budget-grid.narrow {
    grid-template-columns: 1fr;
}
.hf-roster-inputs-tier-4 {
    display: grid;
    grid-template-columns: 250px 120px 120px;
    align-items: center; /* optional: vertically center each cell */
    gap: 8px; /* optional: spacing between cells */
    grid-template-rows: subgrid;
}
.hf-roster-inputs-serca {
    display: grid;
    grid-template-columns: 250px 120px 120px;
    align-items: center; /* optional: vertically center each cell */
    gap: 8px; /* optional: spacing between cells */
    grid-template-rows: subgrid;
}
/* Container */
.hf-roster-tier-select.hf-roster-tier-select {
    display: inline-flex;
    gap: 6px;
    background: transparent;
    border: none;
    padding: 0;
}

/* Each button */
.hf-roster-tier-select .p-togglebutton.p-togglebutton {
    color: var(--hf-text-muted);
    background: transparent;
    border: 1px solid var(--hf-border-subtle);
    /* border-radius: 999px; */
    font: 500 11px/1 var(--hf-font-body);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 7px 12px;
    cursor: pointer;
    transition:
        border-color 0.2s ease,
        color 0.2s ease,
        background-color 0.2s ease,
        box-shadow 0.2s ease;
    box-shadow: none;
    outline: none;
}

/* Divider */
.hf-roster-tier-select .p-togglebutton.p-togglebutton:not(:first-child)::before {
    content: "";
    position: absolute;
    left: -4px;
    top: 20%;
    height: 60%;
    width: 1px;
    background: var(--hf-border-subtle);
    opacity: 0.6;
    pointer-events: none;
}

/* Hover (unselected) */
.hf-roster-tier-select .p-togglebutton.p-togglebutton:not(.p-togglebutton-checked):hover {
    border-color: var(--hf-gold-dim);
    color: var(--hf-gold);
    background: rgba(201, 168, 76, 0.08);
}

/* Selected */
.hf-roster-tier-select .p-togglebutton.p-togglebutton-checked.p-togglebutton-checked {
    color: var(--hf-gold);
    background: rgba(201, 168, 76, 0.15);
    border-color: var(--hf-gold-dim);
    box-shadow: 0 0 0 1px var(--hf-gold-dim);
    cursor: default;
}

/* Selected hover suppression */
.hf-roster-tier-select .p-togglebutton.p-togglebutton-checked.p-togglebutton-checked:hover {
    background: rgba(201, 168, 76, 0.15);
    border-color: var(--hf-gold-dim);
}

/* Focus ring */
.hf-roster-tier-select .p-togglebutton.p-togglebutton:focus-visible {
    outline: 2px solid var(--hf-gold-dim);
    outline-offset: 2px;
}
</style>
