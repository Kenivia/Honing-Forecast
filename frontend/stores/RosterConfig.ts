import { ALL_LABELS, DEFAULT_TIER, STORAGE_KEY, TIER_LABELS } from "@/Utils/Constants"
import { debounce } from "@/Utils/Helpers"
import { create_input_column, fill_new_tiers_with_default, InputColumn, InputType } from "@/Utils/Interfaces"
import { defineStore } from "pinia"

export interface RosterConfig {
    mats_prices: InputColumn[] // mats_prices[tier].data[row] = "123"
    roster_mats_owned: InputColumn[] // Same as in char profile, the tier distinction is because there's different number of mats (rows) for each tier
    tradable_mats_owned: InputColumn[]
    tier: number
    cumulative_graph: boolean
    selected_shard_bag_size: number
    region: string
    effective_serca_price: number[] // This is the one that's actually used (instead of mats_prices) for serca mats in build_material_info
    latest_market_data: Record<string, [number, any]> // [timestamp, raw_response_data]
}
export const useRosterStore = defineStore("roster", {
    state: () => ({
        data: null as RosterConfig | null,
    }),
    getters: {
        roster_config: (state): RosterConfig => state.data,
    },
    actions: {
        init() {
            this.data = load_roster_config() // we have to use a data field instead of just doing this = load_roster_config() cos that's how vue works ig
        },
    },
})

export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
    mats_prices: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)), // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
    roster_mats_owned: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)),
    tradable_mats_owned: ALL_LABELS.map((this_labels) => create_input_column(InputType.Int, this_labels)),
    tier: DEFAULT_TIER,
    cumulative_graph: true,
    selected_shard_bag_size: 3000,
    region: "NAE",
    effective_serca_price: ALL_LABELS[1].map(() => 0),
    latest_market_data: {},
}

export function load_roster_config(): RosterConfig {
    const raw = localStorage.getItem(STORAGE_KEY + "_roster")
    if (!raw) return DEFAULT_ROSTER_CONFIG
    let out = JSON.parse(raw)
    fill_new_tiers_with_default(out.mats_prices)
    fill_new_tiers_with_default(out.roster_mats_owned)
    fill_new_tiers_with_default(out.tradable_mats_owned)

    return { ...DEFAULT_ROSTER_CONFIG, ...out }
}
export function write_roster_config(state) {
    localStorage.setItem(STORAGE_KEY + "_roster", JSON.stringify(state.data))
}

export const debounced_write_roster_config = debounce(write_roster_config, 500)
