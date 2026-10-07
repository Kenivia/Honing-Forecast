import { ALL_LABELS, FALLBACK_PRICES } from "@/Utils/Constants";
import {
  create_input_column,
  input_column_to_num,
  InputColumn,
  InputType,
} from "@/Utils/InputColumn";

import { defineStore } from "pinia";
import { CharProfile, new_char_profile } from "./CharacterProfile";

import {
  DEFAULT_SHARD_INFO,
  MarketRegions,
  ShardInfo,
  start_fetch,
} from "@/Utils/MarketDataFetcher";

import { load_roster_config } from "./ConfigStorage";

// Everything in here is persisted. Session-only state lives in RuntimeState.
export interface RosterConfig {
  mats_prices: Record<MarketRegions, InputColumn[]>; // mats_prices[region][tier]
  roster_mats_owned: Record<number, InputColumn[]>; // the tier distinction is because there's a different number of mats (rows) per tier
  tradable_mats_owned: Record<number, InputColumn[]>;

  all_regions: Record<number, MarketRegions>;

  cumulative_graph: boolean;

  shard_infos: Record<MarketRegions, ShardInfo>;

  latest_market_data: Partial<Record<MarketRegions, [number, any]>>; // [timestamp, raw_response_data]

  profiles: CharProfile[];
  active_profile_index: number;
  last_seen_version: string;

  enabled_annotations: boolean[];
  show_all_rows: boolean;

  auto_deduct_costs: boolean;
  auto_fetch: boolean;
}

export function create_default_owned_input_column(): InputColumn[] {
  return ALL_LABELS.map((this_labels) =>
    create_input_column(InputType.Int, this_labels),
  );
}

function default_prices(region: MarketRegions): InputColumn[] {
  return ALL_LABELS.map((this_labels, tier) =>
    create_input_column(
      InputType.Int,
      this_labels,
      region === "Custom"
        ? this_labels.map((label) => (label === "Gold" ? "1" : "0"))
        : FALLBACK_PRICES[tier].map((price) => price.toLocaleString()),
    ),
  );
}

// A function rather than a const so callers always get a fresh tree, and so this module
// has no load-time dependency on ConfigStorage.
export function default_roster_config(): RosterConfig {
  return {
    mats_prices: {
      nae: default_prices("nae"),
      euc: default_prices("euc"),
      Custom: default_prices("Custom"),
    },
    roster_mats_owned: { 0: create_default_owned_input_column() },
    tradable_mats_owned: { 0: create_default_owned_input_column() },
    all_regions: { 0: "nae" },
    cumulative_graph: true,
    shard_infos: {
      nae: structuredClone(DEFAULT_SHARD_INFO),
      euc: structuredClone(DEFAULT_SHARD_INFO),
      Custom: structuredClone(DEFAULT_SHARD_INFO),
    },
    latest_market_data: {},
    profiles: [new_char_profile()],
    active_profile_index: 0,
    last_seen_version: "v0.0.0",
    enabled_annotations: [true, true, false, false],
    show_all_rows: false,
    auto_deduct_costs: true,
    auto_fetch: true,
  };
}

export const useRosterStore = defineStore("roster", {
  state: () => ({
    roster_config: default_roster_config(),
  }),
  getters: {
    active_profile: (state): CharProfile =>
      state.roster_config.profiles[state.roster_config.active_profile_index],

    active_roster_mats_owned(state): InputColumn[] {
      return state.roster_config.roster_mats_owned[
        this.active_profile.roster_id
      ];
    },
    active_tradable_mats_owned(state): InputColumn[] {
      return state.roster_config.tradable_mats_owned[
        this.active_profile.roster_id
      ];
    },
    active_region(state): MarketRegions {
      return state.roster_config.all_regions[this.active_profile.roster_id];
    },
    active_mats_prices(state): InputColumn[] {
      return state.roster_config.mats_prices[this.active_region];
    },

    all_profiles: (state): CharProfile[] => state.roster_config.profiles,
    roster_ids: (state): number[] =>
      [...new Set(state.roster_config.profiles.map((x) => x.roster_id))].sort(
        (a, b) => a - b,
      ),
    enabled_annotations: (state): boolean[] =>
      state.roster_config.enabled_annotations,

    effective_serca_price(): number[] {
      const t4_price = input_column_to_num(this.active_mats_prices[0]);
      const serca_price = input_column_to_num(this.active_mats_prices[1]);
      return ALL_LABELS[1].map((_, index) =>
        Math.min(t4_price[index] * 5, serca_price[index]),
      );
    },
  },

  actions: {
    init() {
      this.roster_config = load_roster_config();
    },
    replace_config(config: RosterConfig) {
      this.roster_config = config;
    },
    switch_profile(id: number) {
      this.roster_config.active_profile_index = id;
    },
    add_profile(profile: CharProfile) {
      this.roster_config.profiles.push(profile);
    },

    reset_active_profile() {
      const { char_name, roster_id } = this.active_profile;
      this.roster_config.profiles[this.roster_config.active_profile_index] = {
        ...new_char_profile(),
        char_name,
        roster_id,
      };
    },

    active_region_change(event: Event) {
      const new_region = (event.target as HTMLSelectElement)
        .value as MarketRegions;
      this.roster_config.all_regions[this.active_profile.roster_id] =
        new_region;
      start_fetch(new_region);
    },
  },
});
