<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/_stores/RosterConfig"
import { ALL_LABELS, BUNDLE_SIZE, SERCA_SYNC_MAP, SERCA_TO_T4, SYNCED_LABELS } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { computed, nextTick, ref, watch, watchEffect } from "vue"
import { input_column_to_num } from "@/Utils/InputColumn"
import { force_rerender } from "./MarketUtil"

const roster_store = useRosterStore()
const { roster_config, active_roster_mats_owned, active_tradable_mats_owned } = storeToRefs(roster_store)



watchEffect(() => {
    let t4_price = input_column_to_num(roster_store.roster_config.mats_prices[0])
    let serca_price = input_column_to_num(roster_store.roster_config.mats_prices[1])
    roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map((_, index) => Math.min(t4_price[index] * 5, serca_price[index]))
})
const t4_better = computed(() => {
    let t4_price = input_column_to_num(roster_config.value.mats_prices[0])
    let serca_price = input_column_to_num(roster_config.value.mats_prices[1])
    return ALL_LABELS[1].map((_, index) => t4_price[index] * 5 < serca_price[index])
})

const T4_indices_to_watch = SERCA_SYNC_MAP.map(({ T4_index }) => T4_index)

watch(
    // one way sync from T4 to Serca, the ui modifies the T4 copy
    () =>
        T4_indices_to_watch.flatMap((T4_index) => [
            roster_store.roster_config.mats_prices[0].data[T4_index],
            roster_store.active_tradable_mats_owned[0].data[T4_index],
            roster_store.active_roster_mats_owned[0].data[T4_index],
        ]),
    () => {
        for (const { serca_index, T4_index } of SERCA_SYNC_MAP) {
            roster_config.value.mats_prices[1].data[serca_index] = roster_config.value.mats_prices[0].data[T4_index]
            roster_store.active_tradable_mats_owned[1].data[serca_index] = roster_store.active_tradable_mats_owned[0].data[T4_index]
            roster_store.active_roster_mats_owned[1].data[serca_index] = roster_store.active_roster_mats_owned[0].data[T4_index]
        }
        force_rerender()
    },
    { deep: false, immediate: true },
)
</script>

<template>
    <div v-if="roster_config.market_rerender_trigger" class="hf-outer-budget-grid" ">
        <div class="hf-tier-grid-scroll">
            <div class="hf-roster-inputs-tier-4" :style="{ gridRow: `span ${String(ALL_LABELS[0].length + 1)}` }">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px; color: var(--hf-graph-roster-color)">Roster Bound Mats</span>
                    <span style="color: var(--hf-graph-tradable-color)">Tradable Mats</span>
                    <span>Market price</span>

                    <!-- <span v-if="customLeftovers">Left</span> -->
                </div>
                <div v-for="(label, row) in ALL_LABELS[0]" :key="`roster-input-t4-${label}`" class="hf-mats-row">
                    <MaterialCell
                        :input_column="active_roster_mats_owned[0]"
                        :row="row"
                        :label="label"
                        :setter="
                            (val) => {
                                active_roster_mats_owned[0].data[row] = val
                            }
                        "
                        input_color="var(--hf-graph-roster-color)"
                        :hide_tick="true"
                    />
                    <MaterialCell
                        :input_column="active_tradable_mats_owned[0]"
                        :row="row"
                        :setter="
                            (val) => {
                                active_tradable_mats_owned[0].data[row] = val
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

        <div class="hf-tier-grid-scroll">
            <div class="hf-roster-inputs-serca" :style="{ gridRow: `span ${String(ALL_LABELS[1].length + 1)}` }">
                <div class="hf-table-title-row">
                    <span style="text-align: right; padding-right: 15px; color: var(--hf-graph-roster-color)">Roster Bound Mats</span>
                    <span style="color: var(--hf-graph-tradable-color)">Tradable Mats</span>
                    <span>Market price</span>
                    <span style="color: var(--hf-gold)">Effective price</span>

                    <!-- <span v-if="customLeftovers">Left</span> -->
                </div>
                <div v-for="(label, row) in ALL_LABELS[1]" :key="`roster-input-serca-${label}`" class="hf-mats-row">
                    <MaterialCell
                        :input_column="active_roster_mats_owned[row in SERCA_TO_T4 ? 0 : 1]"
                        :row="row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row"
                        :label="label"
                        :setter="
                            (val) => {
                                active_roster_mats_owned[row in SERCA_TO_T4 ? 0 : 1].data[row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row] = val
                            }
                        "
                        input_color="var(--hf-graph-roster-color)"
                        :hide_tick="true"
                    />
                    <MaterialCell
                        :input_column="active_tradable_mats_owned[row in SERCA_TO_T4 ? 0 : 1]"
                        :row="row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row"
                        :setter="
                            (val) => {
                                active_tradable_mats_owned[row in SERCA_TO_T4 ? 0 : 1].data[row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row] = val
                            }
                        "
                        input_color="var(--hf-graph-tradable-color)"
                    />
                    <MaterialCell
                        :input_column="roster_config.mats_prices[row in SERCA_TO_T4 ? 0 : 1]"
                        :row="row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row"
                        :setter="
                            (val) => {
                                roster_config.mats_prices[row in SERCA_TO_T4 ? 0 : 1].data[row in SERCA_TO_T4 ? SERCA_TO_T4[row] : row] = val
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
</template>
<style>
.hf-outer-budget-grid {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: space-around;
}

.hf-tier-grid-scroll {
    min-width: 0;
    overflow-x: auto;
    overflow-y: hidden;
    padding-bottom: 4px;
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
}

.hf-roster-inputs-serca {
    display: grid;
    grid-template-columns: 250px 120px 120px 130px;
    align-items: center;
    gap: 8px;
    background: var(--hf-bg-panel);
    border: 1px solid var(--hf-border-subtle);
    border-radius: 8px;
    width: max-content;
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
    border-bottom: 1px solid var(--border-medium);
    min-height: 0;
}

.hf-roster-inputs-serca .hf-table-title-row,
.hf-roster-inputs-serca .hf-mats-row {
    display: grid;
    grid-column: 1 / -1;
    grid-template-columns: 250px 120px 120px 120px;
    align-items: center;
    border-bottom: 1px solid var(--border-medium);
    min-height: 0;
}
</style>
