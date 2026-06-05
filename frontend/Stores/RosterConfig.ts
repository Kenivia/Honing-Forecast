import {
  ALL_LABELS,
  DEFAULT_TIER,
  FALLBACK_PRICES,
  STORAGE_KEY,
} from "@/Utils/Constants";
import { debounce } from "@/Utils/Helpers";
import {
  create_input_column,
  InputColumn,
  InputType,
  validate_input_column_array,
} from "@/Utils/InputColumn";

import { defineStore } from "pinia";
import {
  CharProfile,
  default_char_profile,
  recreate_char_profile,
  validate_char_profile,
} from "./CharacterProfile";

import { BudgetSnapshot } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import { MarketRegions, start_fetch } from "@/Utils/MarketDataFetcher";

export interface RosterConfig {
  mats_prices: Record<MarketRegions, InputColumn[]>;
  // mats_prices: InputColumn[]; // mats_prices[tier].data[row] = "123"
  roster_mats_owned: Record<number, InputColumn[]>; // Same as in char profile, the tier distinction is because there's different number of mats (rows) for each tier
  tradable_mats_owned: Record<number, InputColumn[]>;

  all_regions: Record<number, MarketRegions>;

  tier: number;
  cumulative_graph: boolean;
  selected_shard_bag_size: number;
  effective_serca_price: number[]; // This is the one that's actually used (instead of mats_prices) for serca mats in build_material_info
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
      console.log(
        state.roster_config.all_regions,
        state.roster_config.all_regions[active_profile.roster_id],
      );
      return state.roster_config.all_regions[active_profile.roster_id];
    },

    active_mats_prices: (state): InputColumn[] => {
      const active_profile =
        state.roster_config.profiles[state.roster_config.active_profile_index];
      const active_region =
        state.roster_config.all_regions[active_profile.roster_id];

      // if (!state.roster_config.mats_prices[active_region]) {
      //   state.roster_config.mats_prices[active_region] =

      //   create_input_column(InputType.Float,ALL_LABELS ) ;
      // }
      console.log(
        state.roster_config.mats_prices,
        active_region,
        state.roster_config.all_regions,
        active_profile.roster_id,
        state.roster_config.mats_prices[active_region],
      );
      return state.roster_config.mats_prices[active_region];
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
        recreate_char_profile(default_char_profile);
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

      console.log(
        "setting",
        active_profile.roster_id,
        "to",
        new_region,
        this.roster_config.all_regions,
      );
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
  }, // was gonna use Float here but ig it makes more sense to do int, leaving float in place cos why not
  roster_mats_owned: { 0: create_default_owned_input_column() },
  tradable_mats_owned: { 0: create_default_owned_input_column() },
  tier: DEFAULT_TIER,
  cumulative_graph: true,
  selected_shard_bag_size: 3000,
  effective_serca_price: ALL_LABELS[1].map(() => 0),
  latest_market_data: {},

  profiles: [recreate_char_profile(default_char_profile)],
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
};

export function load_roster_config(): RosterConfig {
  const raw = localStorage.getItem(STORAGE_KEY + "_roster");

  let out = (() => {
    try {
      return JSON.parse(raw) ?? DEFAULT_ROSTER_CONFIG;
    } catch {
      return DEFAULT_ROSTER_CONFIG;
    }
  })();

  const old_char_profiles = localStorage.getItem(
    "HF_UI_STATE_V3_char_profiles",
  );
  if (old_char_profiles !== null) {
    try {
      let parsed = JSON.parse(old_char_profiles);
      out.profiles = parsed.profiles;
    } catch {
      out.profiles = [recreate_char_profile(default_char_profile)];
    }

    localStorage.removeItem("HF_UI_STATE_V3_char_profiles");
  }
  const old_roster = localStorage.getItem("HF_UI_STATE_V3_roster");
  if (old_roster !== null) {
    try {
      let parsed = JSON.parse(old_roster);
      out.roster_mats_owned = { 0: parsed.roster_mats_owned };
      out.tradable_mats_owned = { 0: parsed.tradable_mats_owned };
    } catch {
      out.roster_mats_owned = { 0: create_default_owned_input_column() };
      out.tradable_mats_owned = { 0: create_default_owned_input_column() };
    }
    localStorage.removeItem("HF_UI_STATE_V3_roster");
  }

  // not sure when active_profile_index was introduced but was definitely before 1.2.0,
  out.active_profile_index = !out.active_profile_index
    ? 0
    : Math.max(0, Math.min(out.profiles.length - 1, out.active_profile_index));

  // just after 1.2.0
  if (out.region !== undefined) {
    const region: MarketRegions = out.region.toLowerCase();
    out.all_regions = DEFAULT_ROSTER_CONFIG.all_regions; // i mean profiles shouldn't be empty or anything but just in case ig
    for (let index = 0; index < out.profiles.length; index++) {
      out.all_regions[out.profiles[index].roster_id] = region;
    }
    // out.mats_prices = Object.fromEntries([[region, out.mats_prices]]); i don think this is needed cos like we're fetching anyway
    out.mats_prices = DEFAULT_ROSTER_CONFIG.mats_prices;
    delete out["region"];
  }
  out = standard_validation(out);
  return { ...DEFAULT_ROSTER_CONFIG, ...out };
}

// just making sure that things are correct, not really necessary i think but oh well
function standard_validation(out: any) {
  out.is_fetching = false;
  for (const key in out.active_mats_prices) {
    validate_input_column_array(
      out.active_mats_prices[key],
      DEFAULT_ROSTER_CONFIG.mats_prices["nae"],
    );
  }
  for (const key in out.roster_mats_owned) {
    validate_input_column_array(
      out.roster_mats_owned[key],
      DEFAULT_ROSTER_CONFIG.roster_mats_owned[0],
    );
    validate_input_column_array(
      out.tradable_mats_owned[key],
      DEFAULT_ROSTER_CONFIG.tradable_mats_owned[0],
    );
  }

  for (let i = 0; i < out.profiles.length; i++) {
    out.profiles[i] = validate_char_profile(out.profiles[i], out, i);
  }
  return out;
}
function stringifyOmit(obj: RosterConfig, keys: string[]): string {
  const omit = new Set(keys);
  return JSON.stringify(obj, (key, value) =>
    omit.has(key) ? undefined : value,
  );
}
export function write_roster_config(state) {
  try {
    // localStorage.setItem(
    //   STORAGE_KEY + "_roster",
    //   stringifyOmit(state.roster_config, [
    //     "optimizer_worker_bundle",
    //     "histogram_worker_bundle",
    //     "optimizer_override",
    //     "budget_snapshot",
    //     "is_slider_update",
    //   ]),
    // );
  } catch {
    console.log(JSON.stringify(state.roster_config));
  }
}

export const debounced_write_roster_config = debounce(write_roster_config, 500);
