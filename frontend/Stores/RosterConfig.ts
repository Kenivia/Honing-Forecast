import {
  ALL_BUNDLE_KEYS,
  ALL_MATERIAL_LABELS,
  bundle_key,
  default_selected_bundles,
  effective_unit_prices,
  FALLBACK_PRICES,
  unit_prices,
} from "@/Utils/Constants";
import {
  create_input_column,
  input_column_to_num,
  InputColumn,
  InputType,
} from "@/Utils/InputColumn";

import { defineStore } from "pinia";
import { CharProfile, new_char_profile } from "./CharacterProfile";

import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";

import { load_roster_config } from "./ConfigStorage";
import { process_chests, type Chest, type ManifestChest } from "@/Utils/Chest";
import type { ScanPage, Snapshot } from "@/Utils/Scans";

// Everything in here is persisted. Session-only state lives in RuntimeState.
export interface RosterConfig {
  // Prices are per bundle, so this column is keyed by (material, bundle size); the rest
  // are keyed by material label. All of them cover every tier.
  mats_prices: Record<MarketRegions, InputColumn>;
  // which bundle the user buys, per material, for anything sold in several sizes
  selected_bundles: Record<MarketRegions, Record<string, number>>;
  roster_mats_owned: Record<number, InputColumn>;
  tradable_mats_owned: Record<number, InputColumn>;
  // chests of a roster that are not bound to one character
  roster_chests: Record<number, ManifestChest[]>;
  // every page the scanner exported, as last exported, and the budget snapshots
  scan_pages: ScanPage[];
  snapshots: Snapshot[];

  all_regions: Record<number, MarketRegions>;

  cumulative_graph: boolean;

  latest_market_data: Partial<Record<MarketRegions, [number, any]>>; // [timestamp, raw_response_data]

  profiles: CharProfile[];
  active_profile_index: number;
  last_seen_version: string;

  enabled_annotations: boolean[];
  show_all_rows: boolean;

  auto_deduct_costs: boolean;
  auto_fetch: boolean;
}

export function create_default_owned_input_column(): InputColumn {
  return create_input_column(InputType.Int, ALL_MATERIAL_LABELS);
}

function default_prices(region: MarketRegions): InputColumn {
  return create_input_column(InputType.Int, ALL_BUNDLE_KEYS, {
    value: (key) =>
      region === "Custom"
        ? key === bundle_key("Gold", 1)
          ? "1"
          : "0"
        : FALLBACK_PRICES[key].toLocaleString(),
  });
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
    roster_chests: {},
    scan_pages: [],
    snapshots: [],
    all_regions: { 0: "nae" },
    cumulative_graph: true,
    selected_bundles: {
      nae: default_selected_bundles(),
      euc: default_selected_bundles(),
      Custom: default_selected_bundles(),
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

    active_roster_mats_owned(state): InputColumn {
      return state.roster_config.roster_mats_owned[
        this.active_profile.roster_id
      ];
    },
    active_tradable_mats_owned(state): InputColumn {
      return state.roster_config.tradable_mats_owned[
        this.active_profile.roster_id
      ];
    },
    // the character's own chests and its roster's
    active_chests(state): ManifestChest[] {
      return [
        ...this.active_profile.chests,
        ...(state.roster_config.roster_chests[this.active_profile.roster_id] ??
          []),
      ];
    },
    // what the payload sends: the active chests as the character's tier takes them
    processed_chests(): Chest[] {
      return process_chests(this.active_chests, this.active_profile.tier);
    },
    active_region(state): MarketRegions {
      return state.roster_config.all_regions[this.active_profile.roster_id];
    },
    active_mats_prices(state): InputColumn {
      return state.roster_config.mats_prices[this.active_region];
    },

    all_profiles: (state): CharProfile[] => state.roster_config.profiles,
    roster_ids: (state): number[] =>
      [...new Set(state.roster_config.profiles.map((x) => x.roster_id))].sort(
        (a, b) => a - b,
      ),
    enabled_annotations: (state): boolean[] =>
      state.roster_config.enabled_annotations,

    active_selected_bundles(state): Record<string, number> {
      return state.roster_config.selected_bundles[this.active_region];
    },
    // price of one unit, from each material's selected bundle
    active_unit_prices(): Record<string, number> {
      return unit_prices(
        input_column_to_num(this.active_mats_prices),
        this.active_selected_bundles,
      );
    },
    // cheapest way to obtain a unit: buy it, or make it from a lower tier
    active_effective_prices(): Record<string, number> {
      return effective_unit_prices(this.active_unit_prices);
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
      const { id, char_name, roster_id } = this.active_profile;
      this.roster_config.profiles[this.roster_config.active_profile_index] = {
        ...new_char_profile(),
        id,
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
