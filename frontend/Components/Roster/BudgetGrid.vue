<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/stores/RosterConfig"
import { ALL_LABELS, BUDGET_NARROW_WIDTH, BUNDLE_SIZE, NARROW_WIDTH, SYNCED_LABELS, TIER_OPTIONS } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { computed, nextTick, ref, watchEffect } from "vue"
import { SelectButton } from "primevue"
import { useMediaIsNarrow } from "@/Utils/WindowSize"
import { input_column_to_num, parse_input } from "@/Utils/Interfaces"
import TierConvertButton from "../Common/TierConvertButton.vue"
import { fetch_callback, useTimedFetch } from "@/Utils/MarketDataFetcher"

const { roster_config } = storeToRefs(useRosterStore())

const tier = computed(() => roster_config.value.tier)
const { isNarrow } = useMediaIsNarrow(BUDGET_NARROW_WIDTH)

const { disabled, start_fetch } = useTimedFetch((result, selected, price) => {
    fetch_callback(result, selected, price)
    forceRerender()
})
const re_render_trigger = ref(true)
const forceRerender = async () => {
    re_render_trigger.value = false
    await nextTick()
    re_render_trigger.value = true
}
watchEffect(() => {
    // one way sync from T4 to Serca, the uui modifies the T4 copy
    for (let serca_index = 0; serca_index < ALL_LABELS[1].length; serca_index++) {
        if (SYNCED_LABELS.includes(ALL_LABELS[1][serca_index])) {
            let T4_index = ALL_LABELS[0].findIndex((x) => x == ALL_LABELS[1][serca_index].replace("Serca ", ""))
            roster_config.value.mats_prices[1].data[serca_index] = roster_config.value.mats_prices[0].data[T4_index]
            roster_config.value.tradable_mats_owned[1].data[serca_index] = roster_config.value.tradable_mats_owned[0].data[T4_index]
            roster_config.value.roster_mats_owned[1].data[serca_index] = roster_config.value.roster_mats_owned[0].data[T4_index]
        }
    }
})
function convert_roster_mats_to_serca() {
    for (let serca_index = 0; serca_index < ALL_LABELS[1].length; serca_index++) {
        if (!SYNCED_LABELS.includes(ALL_LABELS[1][serca_index])) {
            let T4_index = ALL_LABELS[0].findIndex((x) => x == ALL_LABELS[1][serca_index].replace("Serca ", ""))

            // all become roster bound
            roster_config.value.roster_mats_owned[1].data[serca_index] = (
                input_column_to_num(roster_config.value.roster_mats_owned[1])[T4_index] +
                parse_input(
                    roster_config.value.tradable_mats_owned[0],
                    T4_index,
                    String(input_column_to_num(roster_config.value.tradable_mats_owned[0])[T4_index] * 0.2),
                )
            ).toLocaleString()
            roster_config.value.tradable_mats_owned[0].data[T4_index] = "0"
            roster_config.value.roster_mats_owned[1].data[serca_index] = (
                input_column_to_num(roster_config.value.roster_mats_owned[1])[T4_index] +
                parse_input(
                    roster_config.value.roster_mats_owned[0],
                    T4_index,
                    String(input_column_to_num(roster_config.value.roster_mats_owned[0])[T4_index] * 0.2),
                )
            ).toLocaleString()
            roster_config.value.roster_mats_owned[0].data[T4_index] = "0"
        }
    }
}

const t4_better = computed(() => {
    let t4_price = input_column_to_num(roster_config.value.mats_prices[0])
    let serca_price = input_column_to_num(roster_config.value.mats_prices[1])
    return ALL_LABELS[1].map((_, index) => t4_price[index] * 5 < serca_price[index])
})

watchEffect(() => {
    let t4_price = input_column_to_num(roster_config.value.mats_prices[0])
    let serca_price = input_column_to_num(roster_config.value.mats_prices[1])
    roster_config.value.effective_serca_price = ALL_LABELS[1].map((_, index) => Math.min(t4_price[index] * 5, serca_price[index]))
})
</script>

<template>
    <TierConvertButton
        label-text="Convert owned T4 Roster & Tradable to T4.5 Serca mats"
        tooltip-text="Converts Red, Blue, and Leaps (not abidos) to Serca (5:1 ratio)"
        @change-tier="convert_roster_mats_to_serca"
    ></TierConvertButton>
    <SelectButton
        v-if="isNarrow"
        v-model="roster_config.tier"
        :options="TIER_OPTIONS"
        option-label="label"
        option-value="value"
        class="hf-roster-tier-select"
        :option-disabled="(data) => data.value === roster_config.tier"
    />
    <div>
        <select v-model="roster_config.region">
            <option>NAE</option>
            <option>EUC</option>
        </select>
        <button :disabled="disabled" @click="() => start_fetch(roster_config.region)">
            {{ disabled ? "Please wait..." : "Fetch Market Data" }}
        </button>
    </div>

    <div class="hf-shard-size-selector">
        <label>Shard bag size:</label>
        <select v-model.number="roster_config.selected_shard_bag_size" class="hf-shard-size-select">
            <option value="1000">x1000</option>
            <option value="2000">x2000</option>
            <option value="3000">x3000</option>
        </select>
        <label>(Best one will be auto selected)</label>
    </div>
    <div v-if="re_render_trigger" class="hf-outer-budget-grid" :class="{ narrow: isNarrow }">
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
                    :label="label"
                    :setter="
                        (val) => {
                            roster_config.roster_mats_owned[0].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.tradable_mats_owned[0]"
                    :row="row"
                    :setter="
                        (val) => {
                            roster_config.tradable_mats_owned[0].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.mats_prices[0]"
                    :row="row"
                    :setter="
                        (val) => {
                            roster_config.mats_prices[0].data[row] = val
                        }
                    "
                    :suffix="
                        label === 'Shards'
                            ? 'x' + roster_config.selected_shard_bag_size.toString()
                            : BUNDLE_SIZE[row] > 1
                              ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US')
                              : ''
                    "
                />
            </div>
        </div>

        <div v-if="!isNarrow || tier == 1" class="hf-roster-inputs-serca" :style="{ gridRow: `span ${String(ALL_LABELS[1].length + 1)}` }">
            <div class="hf-table-title-row">
                <span style="text-align: right; padding-right: 15px">Roster Bound Mats</span>
                <span>Tradable Mats</span>
                <span>Market price</span>
                <span>Effective price</span>

                <!-- <span v-if="customLeftovers">Left</span> -->
            </div>
            <div v-for="(label, row) in ALL_LABELS[1]" :key="`roster-input-${label}`" class="hf-mats-row">
                <MaterialCell
                    :input_column="roster_config.roster_mats_owned[SYNCED_LABELS.includes(label) ? 0 : 1]"
                    :row="row"
                    :label="label"
                    :setter="
                        (val) => {
                            roster_config.roster_mats_owned[SYNCED_LABELS.includes(label) ? 0 : 1].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.tradable_mats_owned[SYNCED_LABELS.includes(label) ? 0 : 1]"
                    :row="row"
                    :setter="
                        (val) => {
                            roster_config.tradable_mats_owned[SYNCED_LABELS.includes(label) ? 0 : 1].data[row] = val
                        }
                    "
                />
                <MaterialCell
                    :input_column="roster_config.mats_prices[SYNCED_LABELS.includes(label) ? 0 : 1]"
                    :row="row"
                    :setter="
                        (val) => {
                            roster_config.mats_prices[SYNCED_LABELS.includes(label) ? 0 : 1].data[row] = val
                        }
                    "
                    :suffix="
                        label === 'Shards'
                            ? 'x' + roster_config.selected_shard_bag_size.toString()
                            : BUNDLE_SIZE[row] > 1
                              ? 'x' + BUNDLE_SIZE[row].toLocaleString('en-US')
                              : ''
                    "
                />
                <MaterialCell
                    v-if="!SYNCED_LABELS.includes(label)"
                    :input_column="roster_config.effective_serca_price"
                    :row="row"
                    :suffix="t4_better[row] ? 'Convert T4' : 'Buy Serca '"
                />
            </div>
        </div>
    </div>
</template>
<style>
.hf-shard-size-selector {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    font: 500 12px/1 var(--hf-font-body);
    color: var(--hf-text-muted);
}

.hf-shard-size-select {
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--hf-border-subtle);
    color: var(--hf-text-muted);
    font: 500 12px/1 var(--hf-font-body);
    cursor: pointer;
    transition:
        border-color 0.2s ease,
        color 0.2s ease,
        background-color 0.2s ease;
    outline: none;
}

.hf-shard-size-select:hover {
    border-color: var(--hf-gold-dim);
    color: var(--hf-gold);
}

.hf-shard-size-select:focus-visible {
    outline: 2px solid var(--hf-gold-dim);
    outline-offset: 2px;
    border-color: var(--hf-gold-dim);
}

.hf-table-title-row,
.hf-mats-row {
    display: grid;
    grid-column: 1 / -1; /* span all 3 columns */
    grid-template-columns: subgrid; /* inherit parent column definitions */
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
    min-height: 0px;
}
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
    grid-template-columns: 250px 120px 120px 120px;
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
