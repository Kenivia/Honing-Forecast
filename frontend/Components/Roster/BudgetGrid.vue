<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig"
import { ALL_LABELS, BUDGET_NARROW_WIDTH, BUNDLE_SIZE, SYNCED_LABELS, TIER_OPTIONS } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { computed, nextTick, ref, watchEffect } from "vue"
import { SelectButton } from "primevue"
import { useMediaIsNarrow } from "@/Utils/WindowSize"
import TierConvertButton from "../Common/TierConvertButton.vue"
import { fetch_callback, useTimedFetch } from "@/Utils/MarketDataFetcher"
import { input_column_to_num, parse_input } from "@/Utils/InputColumn"

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
</script>

<template>
    <SelectButton
        v-if="isNarrow"
        v-model="roster_config.tier"
        :options="TIER_OPTIONS"
        option-label="label"
        option-value="value"
        class="hf-roster-tier-select"
        :option-disabled="(data) => data.value === roster_config.tier"
    />
    <div style="display: flex; flex-direction: row; align-items: center">
        <select v-model="roster_config.region">
            <option>NAE</option>
            <option>EUC</option>
        </select>
        <button :disabled="disabled" @click="() => start_fetch(roster_config.region)" style="width: 140px">
            {{ !disabled ? "Fetch Market Data" : "Fetching..." }}
        </button>
        <span style="width: 20px; text-wrap-mode: nowrap">
            {{ !disabled && roster_config.latest_market_data ? "✅" : disabled ? "" : "Failed" }}
        </span>
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
        <div v-if="!isNarrow || tier == 0" class="hf-tier-grid-scroll">
            <div class="hf-roster-inputs-tier-4" :style="{ gridRow: `span ${String(ALL_LABELS[0].length + 1)}` }">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px; color: var(--hf-graph-roster-color)">Roster Bound Mats</span>
                    <span style="color: var(--hf-graph-tradable-color)">Tradable Mats</span>
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
                        input_color="var(--hf-graph-roster-color)"
                        :hide_tick="true"
                    />
                    <MaterialCell
                        :input_column="roster_config.tradable_mats_owned[0]"
                        :row="row"
                        :setter="
                            (val) => {
                                roster_config.tradable_mats_owned[0].data[row] = val
                            }
                        "
                        input_color="var(--hf-graph-tradable-color)"
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
        </div>

        <div v-if="!isNarrow || tier == 1" class="hf-tier-grid-scroll">
            <div class="hf-roster-inputs-serca" :style="{ gridRow: `span ${String(ALL_LABELS[1].length + 1)}` }">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px; color: var(--hf-graph-roster-color)">Roster Bound Mats</span>
                    <span style="color: var(--hf-graph-tradable-color)">Tradable Mats</span>
                    <span>Market price</span>
                    <span style="color: var(--hf-gold)">Effective price</span>

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
                        input_color="var(--hf-graph-roster-color)"
                        :hide_tick="true"
                    />
                    <MaterialCell
                        :input_column="roster_config.tradable_mats_owned[SYNCED_LABELS.includes(label) ? 0 : 1]"
                        :row="row"
                        :setter="
                            (val) => {
                                roster_config.tradable_mats_owned[SYNCED_LABELS.includes(label) ? 0 : 1].data[row] = val
                            }
                        "
                        input_color="var(--hf-graph-tradable-color)"
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
                        input_color="var(--hf-gold)"
                    />
                </div>
            </div>
        </div>
    </div>
    <TierConvertButton
        label-text="Convert owned T4 Roster & Tradable to T4.5 Serca mats (5:1 ratio)"
        tooltip-text="Converts Red, Blue, and Leaps (not abidos) to Serca"
        @change-tier="convert_roster_mats_to_serca"
    ></TierConvertButton>
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

.hf-outer-budget-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    align-items: start;
}

.hf-outer-budget-grid.narrow {
    grid-template-columns: 1fr;
}

.hf-tier-grid-scroll {
    min-width: 0;
    width: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    padding-bottom: 4px;
    -webkit-overflow-scrolling: touch;
}

.hf-roster-inputs-tier-4 {
    display: grid;
    grid-template-columns: 250px 120px 120px;
    align-items: center;
    gap: 8px;
    background: var(--hf-bg-panel);
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    width: max-content;
    min-width: 100%;
}

.hf-roster-inputs-serca {
    display: grid;
    grid-template-columns: 250px 120px 120px 120px;
    align-items: center;
    gap: 8px;
    background: var(--hf-bg-panel);
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    width: max-content;
    min-width: 100%;
}

.hf-roster-inputs-tier-4 .hf-material-cell,
.hf-roster-inputs-serca .hf-material-cell {
    --hf-cell-input-width: 86px;
    --hf-cell-label-width: 136px;
    --hf-cell-icon-size: 28px;
}

.hf-roster-inputs-tier-4 .hf-table-title-row,
.hf-roster-inputs-tier-4 .hf-mats-row {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: 250px 120px 120px;
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
    min-height: 0;
}

.hf-roster-inputs-serca .hf-table-title-row,
.hf-roster-inputs-serca .hf-mats-row {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: 250px 120px 120px 120px;
    align-items: center;
    border-bottom: 1px solid var(--separator-color);
    min-height: 0;
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

@media (max-width: 900px) {
    .hf-shard-size-selector {
        flex-wrap: wrap;
        align-items: center;
        gap: 8px;
    }

    .hf-roster-inputs-tier-4 {
        grid-template-columns: 170px 90px 90px;
        gap: 4px;
    }

    .hf-roster-inputs-tier-4 .hf-table-title-row,
    .hf-roster-inputs-tier-4 .hf-mats-row {
        grid-template-columns: 170px 90px 90px;
    }

    .hf-roster-inputs-serca {
        grid-template-columns: 150px 76px 76px 76px;
        gap: 4px;
    }

    .hf-roster-inputs-serca .hf-table-title-row,
    .hf-roster-inputs-serca .hf-mats-row {
        grid-template-columns: 150px 76px 76px 76px;
    }

    .hf-roster-inputs-tier-4 .hf-material-cell,
    .hf-roster-inputs-serca .hf-material-cell {
        --hf-cell-input-width: 64px;
        --hf-cell-label-width: 88px;
        --hf-cell-icon-size: 20px;
    }

    .hf-table-title-row {
        font-size: 11px;
    }
}
</style>
