<script setup lang="ts">
import { useRosterStore as useRosterStore } from "@/Stores/RosterConfig"
import { ALL_LABELS, BUDGET_NARROW_WIDTH, BUNDLE_SIZE, SERCA_SYNC_MAP, SERCA_TO_T4, SYNCED_LABELS, TIER_OPTIONS } from "@/Utils/Constants"
import { storeToRefs } from "pinia"
import MaterialCell from "@/Components/Common/MaterialCell.vue"
import { computed, nextTick, ref, watch, watchEffect } from "vue"
import { SelectButton } from "primevue"
import { useMediaIsNarrow } from "@/Utils/WindowSize"
import TierConvertButton from "../Common/TierConvertButton.vue"
import { fetch_callback, useTimedFetch } from "@/Utils/MarketDataFetcher"
import { input_column_to_num, parse_input } from "@/Utils/InputColumn"
import Sidebar from "../Common/Sidebar.vue"
import BudgetGrid from "./BudgetGrid.vue"

const roster_store = useRosterStore()
const { roster_config, active_roster_mats_owned, active_tradable_mats_owned, all_profiles, roster_ids, active_profile } = storeToRefs(roster_store)

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
            active_roster_mats_owned.value[1].data[serca_index] = (
                input_column_to_num(active_roster_mats_owned.value[1])[T4_index] +
                parse_input(active_tradable_mats_owned.value[0], T4_index, String(input_column_to_num(active_tradable_mats_owned.value[0])[T4_index] * 0.2))
            ).toLocaleString()
            active_tradable_mats_owned.value[0].data[T4_index] = "0"
            active_roster_mats_owned.value[1].data[serca_index] = (
                input_column_to_num(active_roster_mats_owned.value[1])[T4_index] +
                parse_input(active_roster_mats_owned.value[0], T4_index, String(input_column_to_num(active_roster_mats_owned.value[0])[T4_index] * 0.2))
            ).toLocaleString()
            active_roster_mats_owned.value[0].data[T4_index] = "0"
        }
    }
}
watchEffect(() => {
    let t4_price = input_column_to_num(roster_store.roster_config.mats_prices[0])
    let serca_price = input_column_to_num(roster_store.roster_config.mats_prices[1])
    roster_store.roster_config.effective_serca_price = ALL_LABELS[1].map((_, index) => Math.min(t4_price[index] * 5, serca_price[index]))
})

function find_representative(): Record<string, number> {
    let out = {}
    let seen = {}
    let roster_index = 1
    for (const [profile_index, profile] of all_profiles.value.entries()) {
        if (!seen.hasOwnProperty(profile.roster_id)) {
            seen[profile.roster_id] = roster_index
            let name = "Roster " + String(roster_index)
            out[name] = profile_index
            roster_index += 1
        }
    }
    return out
}
const representative_profile_indices = computed(find_representative)
const selected_roster = computed(() => {
    let out = Object.entries(representative_profile_indices.value).find(([, v]) => all_profiles.value[v].roster_id === active_profile.value.roster_id)[0]
    // console.log(out)
    return out
})
function change_roster(event) {
    // console.log(representative_profile_indices.value[event.target.value])
    roster_store.switchProfile(representative_profile_indices.value[event.target.value])
    forceRerender()
}

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
        forceRerender()
    },
    { deep: false, immediate: true },
)
</script>

<template>
    <Sidebar header="Market & mats">
        <template #sidebar="{ close }">
            <!-- <SelectButton
                v-if="isNarrow"
                v-model="roster_config.tier"
                :options="TIER_OPTIONS"
                option-label="label"
                option-value="value"
                class="hf-roster-tier-select"
                :option-disabled="(data) => data.value === roster_config.tier"
            /> -->

            <div style="display: flex; flex-direction: column; align-items: center">
                <div style="display: flex; flex-direction: row; align-items: center">
                    <select v-model="roster_config.region" @change="start_fetch(roster_config.region)">
                        <option>NAE</option>
                        <option>EUC</option>
                    </select>
                    <span style="width: 20px; text-wrap-mode: nowrap">
                        {{ !disabled && roster_config.latest_market_data && !roster_config.market_fetch_failed ? "✅" : disabled ? "" : "Failed" }}
                    </span>
                </div>
                <button :disabled="disabled" @click="() => start_fetch(roster_config.region)" style="width: 140px">
                    {{ !disabled ? "Fetch Market Data" : "Fetching..." }}
                </button>
            </div>

            <div class="hf-shard-size-selector">
                <div style="display: flex; flex-direction: row; align-items: center">
                    <label>Shard bag size:</label>
                    <select v-model.number="roster_config.selected_shard_bag_size" class="hf-shard-size-select">
                        <option value="1000">x1000</option>
                        <option value="2000">x2000</option>
                        <option value="3000">x3000</option>
                    </select>
                </div>
                <label>(Best one will be auto selected)</label>
            </div>
            <div v-if="roster_ids.length > 1">
                <span> Active Roster: </span>
                <select :value="selected_roster" class="hf-shard-size-select" @change="change_roster">
                    <option v-for="(profile_index, name) in representative_profile_indices" :value="name">{{ name }}</option>
                </select>
            </div>
            <TierConvertButton
                label-text="Convert owned T4 Roster & Tradable to T4.5 Serca mats (5 to 1 ratio)"
                tooltip-text="Red, Blue, and Leaps (not abidos)"
                @change-tier="convert_roster_mats_to_serca"
            ></TierConvertButton>
        </template>

        <template #main key="market"> <BudgetGrid /> </template>
    </Sidebar>
</template>
<style>
.hf-shard-size-selector {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    font: 500 12px/1 var(--hf-font-body);
    color: var(--hf-text-muted);
    flex-direction: column;
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
} /* Container */
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
