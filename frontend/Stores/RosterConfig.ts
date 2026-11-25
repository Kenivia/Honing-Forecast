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
import {
  clean_char_name,
  default_char_labels,
  route_labels,
} from "@/Utils/Helpers";

import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";
import { linked_now } from "@/Utils/BibleAuth";
import { market_to_uwuowo_region } from "@/Components/Common/Uwuowo/UwuowoUtils";

import {
  load_roster_config,
  moved_in_save,
  moved_save,
  use_moved_save,
} from "./ConfigStorage";
import { settle_import } from "@/Utils/MoveReport";
import { process_chests, type Chest, type ManifestChest } from "@/Utils/Chest";
import type { ScanCurrencies, ScanPage, Snapshot } from "@/Utils/Scans";
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
  // every page the scanner exported, slot by slot, as last exported; the currencies likewise;
  // and the budget snapshots
  scan_pages: ScanPage[];
  scan_currencies: ScanCurrencies[];
  snapshots: Snapshot[];
  // income per week typed by hand, [roster bound, tradable]
  roster_income: Record<string, Record<string, number>[]>;
  // the Choices page's custom choice
  custom_choices: Record<string, CustomChoice>;

  all_regions: Record<string, MarketRegions>;
  // names given by the user; a roster without one is "Roster N"
  roster_names: Record<string, string>;

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
  show_suggestions: boolean;
  hide_free_tap_hint: boolean;
}

export function create_default_owned_input_column(): InputColumn {
  return create_input_column(InputType.Int, ALL_MATERIAL_LABELS);
}

// JSON, not deep-equal: values here come from different realms under jsdom
const equal = (a: unknown, b: unknown) =>
  JSON.stringify(a) === JSON.stringify(b);

// a fresh install's lone default character with nothing ticked, typed or added, and a roster with no mats
export function fresh_profile_id(config: RosterConfig): string | null {
  const p = config.profiles[0];
  const base = new_char_profile();
  const empty = create_default_owned_input_column();
  const fresh =
    config.profiles.length === 1 &&
    p.fetched_at === 0 &&
    p.char_name === "" &&
    (
      [
        "normal_grid",
        "adv_grid",
        "bound_budgets",
        "special_budget",
        "chests",
        "forbidden_chests",
      ] as const
    ).every((k) => equal(p[k], base[k])) &&
    equal(config.roster_mats_owned[p.roster_id], empty) &&
    equal(config.tradable_mats_owned[p.roster_id], empty) &&
    !config.roster_chests[p.roster_id]?.length;
  return fresh ? p.id : null;
}

// typed materials or chests (not what a fetch sets); a roster's last character counts its roster's too
export function has_edited_mats(config: RosterConfig, p: CharProfile): boolean {
  const base = new_char_profile();
  const empty = create_default_owned_input_column();
  const own = (
    ["bound_budgets", "special_budget", "chests", "forbidden_chests"] as const
  ).some((k) => !equal(p[k], base[k]));
  const last = !config.profiles.some(
    (x) => x.roster_id === p.roster_id && x.id !== p.id,
  );
  return (
    own ||
    (last &&
      (!equal(config.roster_mats_owned[p.roster_id], empty) ||
        !equal(config.tradable_mats_owned[p.roster_id], empty) ||
        !!config.roster_chests[p.roster_id]?.length))
  );
}

export function default_prices(region: MarketRegions): InputColumn {
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
    scan_currencies: [],
    snapshots: [],
    roster_income: {},
    custom_choices: {},
    all_regions: { [roster_id]: "nae" },
    roster_names: {},
    cumulative_graph: false,
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
    show_suggestions: true,
    hide_free_tap_hint: false,
  };
}

// the region segment of character routes
export const REGION_SLUGS: Record<MarketRegions, string> = {
  nae: "NA",
  euc: "EU",
  Custom: "Custom",
};

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
    // the active chests this character may open
    allowed_chests(): ManifestChest[] {
      const forbidden = this.active_profile.forbidden_chests;
      return this.active_chests.filter(
        (chest) => !forbidden.includes(chest.key),
      );
    },
    // what the payload sends: the allowed chests as the character's tier takes them
    processed_chests(): Chest[] {
      return process_chests(this.allowed_chests, this.active_profile.tier);
    },
    // the forbidden ones, only to be shown: each processed alone, keeping its stored key
    forbidden_processed_chests(): Chest[] {
      const forbidden = this.active_profile.forbidden_chests;
      return this.active_chests
        .filter((chest) => forbidden.includes(chest.key))
        .flatMap((chest) =>
          process_chests([chest], this.active_profile.tier).map((out) => ({
            ...out,
            key: chest.key,
          })),
        );
    },
    active_region(state): MarketRegions {
      return state.roster_config.all_regions[this.active_profile.roster_id];
    },
    active_mats_prices(state): InputColumn {
      return state.roster_config.mats_prices[this.active_region];
    },

    all_profiles: (state): CharProfile[] => state.roster_config.profiles,
    // profile id -> generated label, for unnamed characters only
    default_char_labels: (state): Record<string, string> =>
      default_char_labels(
        state.roster_config.profiles,
        state.roster_config.all_regions,
      ),
    // a name is shown as typed, an empty one as its default (also what a char is routed by)
    char_default_label() {
      return (p: CharProfile): string =>
        this.default_char_labels[p.id] ?? "Newchar";
    },
    char_label() {
      return (p: CharProfile): string =>
        p.char_name || this.char_default_label(p);
    },
    // profile id -> url key: the label, with -2, -3, ... for a repeat in its region
    route_labels(): Record<string, string> {
      return route_labels(
        this.roster_config.profiles,
        this.roster_config.all_regions,
        this.char_label,
      );
    },
    // signed in and the character is in one of the user's lostark.bible rosters (reactive)
    char_linked() {
      return (p: CharProfile): boolean => {
        const region = market_to_uwuowo_region(
          this.roster_config.all_regions[p.roster_id],
        );
        return region !== null && linked_now(region, p.char_name) !== null;
      };
    },
    // any of the roster's characters is linked
    roster_linked() {
      return (roster_id: string): boolean =>
        this.roster_config.profiles.some(
          (p) => p.roster_id === roster_id && this.char_linked(p),
        );
    },
    // whether another character of the region has the same name
    name_taken() {
      return (p: CharProfile): boolean =>
        p.char_name !== "" && this.route_labels[p.id] !== p.char_name;
    },
    // -1 when no character has that key; the first of any region (old /chars/<name> links)
    profile_index_by_name() {
      return (name: string): number =>
        this.roster_config.profiles.findIndex(
          (p) => this.route_labels[p.id] === name,
        );
    },
    // keys are unique per region only, so a character's route carries both
    char_params() {
      return (p: CharProfile): { region: string; characterName: string } => ({
        region: REGION_SLUGS[this.roster_config.all_regions[p.roster_id]],
        characterName: this.route_labels[p.id],
      });
    },
    profile_index_by_route() {
      return (region: string, name: string): number =>
        this.roster_config.profiles.findIndex(
          (p) =>
            this.route_labels[p.id] === name &&
            REGION_SLUGS[this.roster_config.all_regions[p.roster_id]] ===
              region,
        );
    },
    // in order of first appearance; add_profile keeps a roster's characters together
    roster_ids: (state): string[] => [
      ...new Set(state.roster_config.profiles.map((x) => x.roster_id)),
    ],
    roster_default_label() {
      return (roster_id: string): string =>
        `Roster ${this.roster_ids.indexOf(roster_id) + 1}`;
    },
    roster_label() {
      return (roster_id: string): string =>
        this.roster_config.roster_names[roster_id] ||
        this.roster_default_label(roster_id);
    },

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
    // forbids a stored chest on the active character, or lifts it
    toggle_forbid(key: string) {
      const forbidden = this.active_profile.forbidden_chests;
      const index = forbidden.indexOf(key);
      if (index >= 0) forbidden.splice(index, 1);
      else forbidden.push(key);
    },
    // a save moved in from the old domain replaces a fresh one, else waits for its banner (Sidebar.vue)
    init() {
      let config = load_roster_config();
      const save = moved_in_save();
      const applied =
        !!save && !!fresh_profile_id(config) && !config.snapshots.length;
      if (applied) {
        use_moved_save(save);
        config = load_roster_config();
      } else moved_save.value = save;
      settle_import(!save || applied);
      this.roster_config = config;
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
      const profile = { ...new_char_profile(), roster_id };
      this.add_profile(profile);
      return profiles.find((p) => p.id === profile.id);
    },
    // returns the new roster's first character
    add_roster(region: MarketRegions = "nae"): CharProfile {
      const config = this.roster_config;
      const roster_id = crypto.randomUUID();
      config.roster_mats_owned[roster_id] = create_default_owned_input_column();
      config.tradable_mats_owned[roster_id] =
        create_default_owned_input_column();
      config.all_regions[roster_id] = region;
      return this.add_char(roster_id);
    },
    // a separate, identical character: chests and income settings are copied too
    duplicate_profile(index: number): CharProfile {
      const profiles = this.roster_config.profiles;
      const id = crypto.randomUUID();
      this.add_profile({
        ...structuredClone(toRaw(profiles[index])),
        id,
        char_name: "",
      });
      return this.roster_config.profiles.find((p) => p.id === id);
    },
    move_profile(i: number, j: number) {
      const profiles = this.roster_config.profiles;
      const id = this.active_profile.id;
      [profiles[i], profiles[j]] = [profiles[j], profiles[i]];
      this.point_active_at(id);
    },
    // puts a roster's characters in the order of `ids`, in the slots they already take
    reorder_roster(roster_id: string, ids: string[]) {
      const profiles = this.roster_config.profiles;
      const id = this.active_profile.id;
      const by_id = new Map(profiles.map((p) => [p.id, p]));
      let k = 0;
      this.roster_config.profiles = profiles.map((p) =>
        p.roster_id === roster_id ? by_id.get(ids[k++]) : p,
      );
      this.point_active_at(id);
    },
    // renames take what was typed; a blank one goes back to the default label
    rename_char(id: string, name: string) {
      this.roster_config.profiles.find((p) => p.id === id).char_name =
        clean_char_name(name);
    },
    rename_roster(roster_id: string, name: string) {
      const trimmed = name.trim();
      if (trimmed) this.roster_config.roster_names[roster_id] = trimmed;
      else delete this.roster_config.roster_names[roster_id];
    },
    // moves the roster at position `from` to position `to`, its characters in one block
    move_roster(from: number, to: number) {
      const ids = [...this.roster_ids];
      if (from === to || !ids[from] || !ids[to]) return;
      const id = this.active_profile.id;
      const [moved] = ids.splice(from, 1);
      ids.splice(to, 0, moved);
      const profiles = this.roster_config.profiles;
      this.roster_config.profiles = ids.flatMap((r) =>
        profiles.filter((p) => p.roster_id === r),
      );
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
      config.scan_currencies = config.scan_currencies.filter(
        (p) => p.char_id !== gone.id,
      );
      const roster_id = gone.roster_id;
      if (config.profiles.some((p) => p.roster_id === roster_id)) return;
      delete config.roster_mats_owned[roster_id];
      delete config.tradable_mats_owned[roster_id];
      delete config.all_regions[roster_id];
      delete config.roster_names[roster_id];
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

    set_region(roster_id: string, region: MarketRegions) {
      this.roster_config.all_regions[roster_id] = region;
    },
    active_region_change(event: Event) {
      const new_region = (event.target as HTMLSelectElement)
        .value as MarketRegions;
      this.set_region(this.active_profile.roster_id, new_region);
      start_fetch(new_region);
    },
  },
});
