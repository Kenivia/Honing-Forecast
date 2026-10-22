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
import { toRaw } from "vue";
import { CharProfile, new_char_profile } from "./CharacterProfile";
import { format_char_name } from "@/Utils/Helpers";

import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";

import { load_roster_config } from "./ConfigStorage";
import { process_chests, type Chest, type ManifestChest } from "@/Utils/Chest";
import type { ScanPage, Snapshot } from "@/Utils/Scans";
import type { CustomChoice } from "@/Utils/Choices";

// Everything in here is persisted. Session-only state lives in RuntimeState.
export interface RosterConfig {
  // Prices are per bundle, so this column is keyed by (material, bundle size); the rest
  // are keyed by material label. All of them cover every tier.
  mats_prices: Record<MarketRegions, InputColumn>;
  // which bundle the user buys, per material, for anything sold in several sizes
  selected_bundles: Record<MarketRegions, Record<string, number>>;
  roster_mats_owned: Record<string, InputColumn>;
  tradable_mats_owned: Record<string, InputColumn>;
  // chests of a roster that are not bound to one character
  roster_chests: Record<string, ManifestChest[]>;
  // every page the scanner exported, as last exported, and the budget snapshots
  scan_pages: ScanPage[];
  snapshots: Snapshot[];
  // income per week typed by hand, [roster bound, tradable]
  roster_income: Record<string, Record<string, number>[]>;
  // the Choices page's custom choice
  custom_choices: Record<string, CustomChoice>;

  all_regions: Record<string, MarketRegions>;

  cumulative_graph: boolean;

  latest_market_data: Partial<Record<MarketRegions, [number, any]>>; // [timestamp, raw_response_data]

  profiles: CharProfile[];
  active_profile_index: number;
  last_seen_version: string;

  show_all_rows: boolean;

  auto_deduct_costs: boolean;
  auto_fetch: boolean;
  // budget snapshots are uploaded unless the user opts out (Utils/Analytics.ts)
  analytics: boolean;
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
  const roster_id = crypto.randomUUID();
  return {
    mats_prices: {
      nae: default_prices("nae"),
      euc: default_prices("euc"),
      Custom: default_prices("Custom"),
    },
    roster_mats_owned: { [roster_id]: create_default_owned_input_column() },
    tradable_mats_owned: { [roster_id]: create_default_owned_input_column() },
    roster_chests: {},
    scan_pages: [],
    snapshots: [],
    roster_income: {},
    custom_choices: {},
    all_regions: { [roster_id]: "nae" },
    cumulative_graph: true,
    selected_bundles: {
      nae: default_selected_bundles(),
      euc: default_selected_bundles(),
      Custom: default_selected_bundles(),
    },
    latest_market_data: {},
    profiles: [{ ...new_char_profile(), roster_id }],
    active_profile_index: 0,
    last_seen_version: "v0.0.0",
    show_all_rows: false,
    auto_deduct_costs: true,
    auto_fetch: true,
    analytics: true,
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
    // -1 when no character has that name
    profile_index_by_name:
      (state) =>
      (name: string): number =>
        state.roster_config.profiles.findIndex((p) => p.char_name === name),
    // in order of first appearance; add_profile keeps a roster's characters together
    roster_ids: (state): string[] => [
      ...new Set(state.roster_config.profiles.map((x) => x.roster_id)),
    ],

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
    switch_profile(index: number) {
      this.roster_config.active_profile_index = index;
    },
    // after the roster's last character, so deleting one never reorders the rosters
    add_profile(profile: CharProfile) {
      const profiles = this.roster_config.profiles;
      const id = this.active_profile?.id;
      const last = profiles.findLastIndex(
        (p) => p.roster_id === profile.roster_id,
      );
      profiles.splice(last < 0 ? profiles.length : last + 1, 0, profile);
      if (id) this.point_active_at(id);
    },
    // points the active index at the character with `id`, wherever it now is
    point_active_at(id: string) {
      const index = this.roster_config.profiles.findIndex((p) => p.id === id);
      if (index >= 0) this.roster_config.active_profile_index = index;
    },

    // returns the store's (reactive) copy
    add_char(roster_id: string): CharProfile {
      const profiles = this.roster_config.profiles;
      const profile = {
        ...new_char_profile(),
        roster_id,
        char_name: format_char_name("Newchar", profiles.length, profiles),
      };
      this.add_profile(profile);
      return profiles.find((p) => p.id === profile.id);
    },
    add_roster() {
      const config = this.roster_config;
      const roster_id = crypto.randomUUID();
      config.roster_mats_owned[roster_id] = create_default_owned_input_column();
      config.tradable_mats_owned[roster_id] =
        create_default_owned_input_column();
      config.all_regions[roster_id] = "nae";
      this.add_char(roster_id);
    },
    // a separate, identical character: chests and income settings are copied too
    duplicate_profile(index: number) {
      const profiles = this.roster_config.profiles;
      this.add_profile({
        ...structuredClone(toRaw(profiles[index])),
        id: crypto.randomUUID(),
        char_name: format_char_name("Newchar", profiles.length, profiles),
      });
    },
    move_profile(i: number, j: number) {
      const profiles = this.roster_config.profiles;
      const id = this.active_profile.id;
      [profiles[i], profiles[j]] = [profiles[j], profiles[i]];
      this.point_active_at(id);
    },
    // Deleting the active character moves to the one before it. A roster's last character
    // takes the roster's data with it.
    delete_profile(index: number) {
      const config = this.roster_config;
      const active = config.active_profile_index;
      const keep =
        index === active
          ? (config.profiles[index - 1] ?? config.profiles[index + 1])
          : config.profiles[active];
      const [gone] = config.profiles.splice(index, 1);
      this.point_active_at(keep.id);

      config.scan_pages = config.scan_pages.filter(
        (p) => p.char_id !== gone.id,
      );
      const roster_id = gone.roster_id;
      if (config.profiles.some((p) => p.roster_id === roster_id)) return;
      delete config.roster_mats_owned[roster_id];
      delete config.tradable_mats_owned[roster_id];
      delete config.all_regions[roster_id];
      delete config.roster_chests[roster_id];
      delete config.roster_income[roster_id];
      delete config.custom_choices[roster_id];
      config.scan_pages = config.scan_pages.filter(
        (p) => p.roster_id !== roster_id,
      );
      config.snapshots = config.snapshots.filter(
        (s) => s.roster_id !== roster_id,
      );
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
