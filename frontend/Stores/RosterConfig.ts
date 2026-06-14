import { ALL_LABELS, DEFAULT_TIER, FALLBACK_PRICES } from "@/Utils/Constants";
import {
  create_input_column,
  input_column_to_num,
  InputColumn,
  InputType,
} from "@/Utils/InputColumn";

import { defineStore } from "pinia";
import {
  CharProfile,
  DEFAULT_CHAR_PROFILE_NO_WORKER,
  init_workers,
} from "./CharacterProfile";

import { BudgetSnapshot } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import {
  DEFAULT_SHARD_INFO,
  MarketRegions,
  ShardInfo,
  start_fetch,
} from "@/Utils/MarketDataFetcher";

import { load_roster_config } from "./ConfigStorage";

export const CURRENT_STORAGE_KEY = "HF_CONFIG_V6_COMPRESSED";

export interface RosterConfig {
  mats_prices: Record<MarketRegions, InputColumn[]>;
  // mats_prices: InputColumn[]; // mats_prices[tier].data[row] = "123"
  roster_mats_owned: Record<number, InputColumn[]>; // Same as in char profile, the tier distinction is because there's different number of mats (rows) for each tier
  tradable_mats_owned: Record<number, InputColumn[]>;

  all_regions: Record<number, MarketRegions>;

  tier: number;
  cumulative_graph: boolean;

  shard_infos: Record<MarketRegions, ShardInfo>;

  latest_market_data: Partial<Record<MarketRegions, [number, any]>>; // [timestamp, raw_response_data]

  profiles: CharProfile[];
  active_profile_index: number;
  last_seen_version: string;

  enabled_annotations: boolean[];
  show_all_rows: boolean;
  market_fetch_failed: boolean;

  budget_snapshot: BudgetSnapshot | null;
  is_details_update: boolean;

  auto_deduct_costs: boolean;
  is_fetching: boolean;

  auto_fetch: boolean;
}
export const useRosterStore = defineStore("roster", {
  state: () => ({
    roster_config: DEFAULT_ROSTER_CONFIG,
  }),
  getters: {
    active_roster_mats_owned: (state): InputColumn[] => {
      let active_profile =
        state.roster_config.profiles[state.roster_config.active_profile_index];
      return state.roster_config.roster_mats_owned[active_profile.roster_id];
    },
    active_tradable_mats_owned: (state): InputColumn[] => {
      let active_profile =
        state.roster_config.profiles[state.roster_config.active_profile_index];
      // console.log(state.roster_config.active_profile_index, active_profile, state.roster_config.tradable_mats_owned)
      return state.roster_config.tradable_mats_owned[active_profile.roster_id];
    },

    active_profile: (state): CharProfile => {
      // console.log(state.roster_config.profiles)
      return state.roster_config.profiles[
        state.roster_config.active_profile_index
      ];
    },

    all_profiles: (state): CharProfile[] => {
      return state.roster_config.profiles;
    },
    roster_ids: (state): number[] => {
      return [
        ...new Set(state.roster_config.profiles.map((x) => x.roster_id)),
      ].sort((a, b) => a - b);
    },
    enabled_annotations: (state): boolean[] => {
      return state.roster_config.enabled_annotations;
    },
    active_region: (state): MarketRegions => {
      const active_profile =
        state.roster_config.profiles[state.roster_config.active_profile_index];
      // console.log(
      //   state.roster_config.all_regions,
      //   state.roster_config.all_regions[active_profile.roster_id],
      // );
      return state.roster_config.all_regions[active_profile.roster_id];
    },

    active_mats_prices: (state): InputColumn[] => {
      const active_profile =
        state.roster_config.profiles[state.roster_config.active_profile_index];
      const active_region =
        state.roster_config.all_regions[active_profile.roster_id];
      return state.roster_config.mats_prices[active_region];
    },

    effective_serca_price: (state): number[] => {
      const active_profile =
        state.roster_config.profiles[state.roster_config.active_profile_index];
      const active_region =
        state.roster_config.all_regions[active_profile.roster_id];

      const active_mats_prices = state.roster_config.mats_prices[active_region];
      const t4_price = input_column_to_num(active_mats_prices[0]);
      const serca_price = input_column_to_num(active_mats_prices[1]);
      return ALL_LABELS[1].map((_, index) =>
        Math.min(t4_price[index] * 5, serca_price[index]),
      );
    },
  },

  actions: {
    init() {
      this.roster_config = load_roster_config();
    },
    switch_profile(id: number) {
      this.roster_config.active_profile_index = id;
    },
    add_profile(profile: CharProfile) {
      this.roster_config.profiles.push(profile);
    },

    reset_active_profile() {
      const name =
        this.roster_config.profiles[this.roster_config.active_profile_index]
          .char_name;
      const roster_id =
        this.roster_config.profiles[this.roster_config.active_profile_index]
          .roster_id;
      this.roster_config.profiles[this.roster_config.active_profile_index] =
        init_workers(DEFAULT_CHAR_PROFILE_NO_WORKER);
      this.roster_config.profiles[
        this.roster_config.active_profile_index
      ].char_name = name;
      this.roster_config.profiles[
        this.roster_config.active_profile_index
      ].roster_id = roster_id;
    },

    active_region_change(event: any) {
      const new_region = (event.target as HTMLSelectElement)
        .value as MarketRegions;
      const active_profile: CharProfile =
        this.roster_config.profiles[this.roster_config.active_profile_index];

      // console.log(
      //   "setting",
      //   active_profile.roster_id,
      //   "to",
      //   new_region,
      //   this.roster_config.all_regions,
      // );
      this.roster_config.all_regions[active_profile.roster_id] = new_region;
      start_fetch(new_region);
    },
  },
});

export function create_default_owned_input_column(): InputColumn[] {
  return ALL_LABELS.map((this_labels) =>
    create_input_column(InputType.Int, this_labels),
  );
}
export const DEFAULT_ROSTER_CONFIG: RosterConfig = {
  mats_prices: {
    nae: ALL_LABELS.map((this_labels, index) =>
      create_input_column(
        InputType.Int,
        this_labels,
        FALLBACK_PRICES[index].map((price) => price.toLocaleString()),
      ),
    ),
    euc: ALL_LABELS.map((this_labels, index) =>
      create_input_column(
        InputType.Int,
        this_labels,
        FALLBACK_PRICES[index].map((price) => price.toLocaleString()),
      ),
    ),
    Custom: ALL_LABELS.map((this_labels, index) =>
      create_input_column(
        InputType.Int,
        this_labels,
        FALLBACK_PRICES[index].map((_, p_index) =>
          this_labels[p_index] === "Gold" ? "1" : "0",
        ),
      ),
    ),
  }, // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
  roster_mats_owned: { 0: create_default_owned_input_column() },
  tradable_mats_owned: { 0: create_default_owned_input_column() },
  tier: DEFAULT_TIER,
  cumulative_graph: true,
  shard_infos: {
    nae: structuredClone(DEFAULT_SHARD_INFO),
    euc: structuredClone(DEFAULT_SHARD_INFO),
    Custom: structuredClone(DEFAULT_SHARD_INFO),
  },
  latest_market_data: {},

  profiles: [init_workers(DEFAULT_CHAR_PROFILE_NO_WORKER)],
  active_profile_index: 0,
  last_seen_version: "v0.0.0",
  enabled_annotations: [true, true, false, false],
  show_all_rows: false,
  market_fetch_failed: true,
  budget_snapshot: null,
  is_details_update: false,
  auto_deduct_costs: true,
  all_regions: { 0: "nae" },
  is_fetching: false,
  auto_fetch: true,
};
