import {
  ALL_LABELS,
  ALL_WORTHLESS,
  BandPlan,
  clamp_band_plan,
  GRACE_FIRST_N,
  JOINED_ADV_JUICE,
  NUM_ADV_PIECES,
  SPECIAL_LEAP_LABELS,
  ADV_ALWAYS_DOUBLE_BALLS,
  CONVERTIBLE_MATERIALS,
  ALL_BUNDLE_KEYS,
  unit_prices,
  effective_unit_prices,
  sell_tax,
} from "@/Utils/Constants";

import { toRaw } from "vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  get_upgrade_map,
  OneUpgradeInput,
  Upgrade,
} from "@/Utils/KeyedUpgrades";
import { input_column_to_num } from "@/Utils/InputColumn";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { Bag, type Chest } from "@/Utils/Chest";
import type { CharProfile } from "@/Stores/CharacterProfile";
import type { Snapshot } from "@/Utils/Scans";
import {
  AdvOverride,
  NormalOverride,
  type AdvStateOverride,
  type OptimizerOverride,
  type SpecialOverride,
  type StateOverride,
} from "./Overrides";

// I don't think it's possible to directly export this struct from rust to javascript because of all the vectors,
// so it's copied & pasted here
// Owned amounts and per-unit prices for one material. Mirrors Rust's OneMaterial.
export interface OneMaterial {
  bound: number;
  roster: number;
  tradable: number;
  taxed_price: number;
  market_price: number;
}

// an upgrade as Rust takes it, without the fields only the page uses
export type SentUpgrade = Omit<
  OneUpgradeInput,
  "expanded" | "taps_since_last_input" | "used_materials"
>;

export interface Payload {
  // row order; the one place the order Rust's cost tables assume is declared
  material_labels: string[];
  materials: Record<string, OneMaterial>;
  // every plan to evaluate. Index 0 is always ALL_WORTHLESS, so the histogram always
  // reports plain gold spent alongside whatever the user picked.
  plans: BandPlan[];
  optimizer_plan: number;
  upgrade_info: SentUpgrade[];
  special_budget: number;
  special_state?: number[];
  // every chest stack of process_chests, by its key
  chests: Record<string, { count: number; band: number; options: Bag[] }>;
  // how many of each stack go to each option, from the last optimizer run, by the same key
  chest_states?: Map<string, number[]>;
  // what a chest's special leaps are called
  special_label: string;
  tier: number;
  express_event: boolean;
  min_resolution: number;
  num_threads: number;
  metric_type: number;
  adv_cache: any;
}

// what of an optimizer result is fed back into the next payload
interface FedState {
  upgrade_arr?: Upgrade[];
  special_state?: number[];
  chest_states?: Map<string, number[]>;
  adv_cache?: any;
  prep_output?: { juice_info: any };
}

// the advanced strategy index each override picks
const ADV_OVERRIDE_INDEX = {
  [AdvOverride.Empty]: 0,
  [AdvOverride.Grace]: GRACE_FIRST_N.length - 1,
  [AdvOverride.Full]: JOINED_ADV_JUICE.length - 1,
};

// ids 0 and 1 are juices, the rest books; Full takes every id the upgrade can use
function normal_tap(
  tap: number[],
  usable: number[],
  override?: StateOverride,
): number[] {
  if (override === undefined) return tap;
  const setting = (id: number) => (id <= 1 ? override.juice : override.book);
  return [
    ...tap.filter((id) => setting(id) === NormalOverride.Optimizer),
    ...usable.filter((id) => setting(id) === NormalOverride.Full),
  ];
}

function adv_state(state: number[][], override?: AdvStateOverride) {
  return state.map((one_state, index) => {
    const setting = index === 0 ? override?.juice : override?.scroll;
    return [
      setting === undefined || setting === AdvOverride.Optimizer
        ? one_state[0]
        : ADV_OVERRIDE_INDEX[setting],
    ];
  });
}

// The profile's upgrades as Rust takes them, each with the state of `fed` from the taps since the
// last input on. Pure: the stored upgrades are not written.
function upgrade_info(
  profile: CharProfile,
  fed?: FedState,
  override?: OptimizerOverride,
): SentUpgrade[] {
  const tier = profile.tier;
  const upgrade_map = get_upgrade_map(fed?.upgrade_arr, tier);
  const juice_info = fed?.prep_output?.juice_info;
  return (
    Object.entries(profile.keyed_upgrades)
      // shouldn't be necessary, but it went wrong once
      .filter(([, u]) => tier === 0 || u.is_normal_honing)
      .filter(([, u]) => tier === 1 || u.piece_index < NUM_ADV_PIECES)
      .map(([key, keyed]) => {
        const {
          expanded: _expanded,
          used_materials: _used,
          taps_since_last_input: taps,
          ...out
        } = structuredClone(toRaw(keyed));
        out.double_balls =
          !out.is_normal_honing &&
          (ADV_ALWAYS_DOUBLE_BALLS ||
            (out.upgrade_index < 2 && profile.express_event) ||
            (out.upgrade_index >= 2 && profile.pretend_30_40_x2_grace));
        out.unlocked = out.is_normal_honing
          ? out.starting_artisan > 0 || out.starting_num_taps > 0
          : out.adv_progress !== null &&
            (out.adv_progress[0] > 0 || out.adv_progress[1] > 0);
        const upgrade = upgrade_map.get(key);
        if (!(upgrade?.state?.length > 0 && juice_info)) return out;

        // a reset upgrade (state []) is sent empty
        if (out.state?.length === 0) return out;
        const usable = (
          upgrade.is_normal_honing
            ? juice_info.normal_uindex_to_id
            : juice_info.adv_uindex_to_id
        )[upgrade.piece_type_usize][upgrade.upgrade_index];
        out.state = upgrade.is_normal_honing
          ? upgrade.state
              .slice(taps)
              .map((tap) => normal_tap(tap, usable, override?.normal))
          : adv_state(upgrade.state, override?.advanced);
        return out;
      })
  );
}

// The order the user picked in place of the optimizer's: normal honing first, then (highest first)
// each piece's top upgrade, then the preferred piece type, then (lowest first) by level.
export function special_sort_override(
  special_state: number[],
  upgrade_arr: Upgrade[],
  special_override?: SpecialOverride,
): number[] {
  if (
    special_override === undefined ||
    special_state === undefined ||
    special_override.optimizer
  ) {
    return special_state;
  }
  const { weapon_first, highest_first } = special_override;
  const peak = new Map<number, number>();
  for (const idx of special_state) {
    const u = upgrade_arr[idx];
    peak.set(
      u.piece_index,
      Math.max(peak.get(u.piece_index) ?? -1, u.upgrade_index),
    );
  }
  const rank = (idx: number) => {
    const u = upgrade_arr[idx];
    return [
      u.is_normal_honing ? 0 : 1,
      highest_first && peak.get(u.piece_index) !== u.upgrade_index ? 1 : 0,
      (u.piece_type_usize === 1) === weapon_first ? 0 : 1,
      highest_first ? 0 : u.upgrade_index,
    ];
  };
  return [...special_state].sort((a, b) => {
    const [ra, rb] = [rank(a), rank(b)];
    const at = ra.findIndex((x, i) => x !== rb[i]);
    return at === -1 ? 0 : ra[at] - rb[at];
  });
}

// A lower tier's stock counted as the tier's materials it converts to, which is how Rust sees a
// Serca character's T4: bound into bound, roster-bound and tradable into roster-bound (converting
// binds it). Floored per band, like the market page's convert button. A disabled material on
// either side is left out.
export function merge_lower_tier(
  tier: number,
  bound: Record<string, number>,
  roster: Record<string, number>,
  tradable: Record<string, number>,
  enabled: Record<string, boolean> = {},
) {
  const out = {
    bound: { ...bound },
    roster: { ...roster },
    tradable: { ...tradable },
  };
  for (const mat of CONVERTIBLE_MATERIALS) {
    const { label, ratio } = mat.from;
    if (!mat.tiers.includes(tier)) continue;
    if (enabled[mat.label] === false || enabled[label] === false) continue;
    const units = (amount: number) => Math.floor(amount / ratio);
    out.bound[mat.label] = (bound[mat.label] ?? 0) + units(bound[label] ?? 0);
    out.roster[mat.label] =
      (roster[mat.label] ?? 0) +
      units((roster[label] ?? 0) + (tradable[label] ?? 0));
  }
  return out;
}

function materials_of(
  tier: number,
  bound: Record<string, number>,
  roster_owned: Record<string, number>,
  tradable_owned: Record<string, number>,
  enabled: Record<string, boolean>,
  listed: Record<string, number>,
  taxed: Record<string, number>,
): Record<string, OneMaterial> {
  const merged = merge_lower_tier(
    tier,
    bound,
    roster_owned,
    tradable_owned,
    enabled,
  );
  const effective = effective_unit_prices(listed);
  const out: Record<string, OneMaterial> = {};
  for (const label of ALL_LABELS[tier]) {
    // A disabled material is one the user will not trade, which `bound` already models
    // as an effectively infinite stock. Zero prices keep it out of the gold entirely, so
    // what its leftovers are "worth" cannot depend on the plan.
    const traded = enabled[label];
    out[label] = {
      bound: merged.bound[label] ?? 0,
      roster: merged.roster[label] ?? 0,
      // gold is never sold
      tradable: !traded || label === "Gold" ? 0 : (merged.tradable[label] ?? 0),
      taxed_price: traded ? taxed[label] : 0,
      market_price: traded ? effective[label] : 0,
    };
  }
  return out;
}

// The payload of a profile's current upgrades and settings over given budgets, chests and prices
// (per bundle key, with the bundle bought), with no optimizer state. Plain, so it can be hashed
// and posted. The calculator's payload is this plus states_payload.
export function payload_for(
  profile: CharProfile,
  bound: Record<string, number>,
  roster: Record<string, number>,
  tradable: Record<string, number>,
  special: Record<string, number>,
  chests: Chest[],
  prices: Record<string, number>,
  bundles: Record<string, number>,
): Payload {
  const tier = profile.tier;
  const bundle_prices = {
    ...Object.fromEntries(ALL_BUNDLE_KEYS.map((key) => [key, 0])),
    ...prices,
  };
  return JSON.parse(
    JSON.stringify({
      material_labels: ALL_LABELS[tier],
      materials: materials_of(
        tier,
        bound,
        roster,
        tradable,
        profile.bound_budgets.enabled,
        unit_prices(bundle_prices, bundles ?? {}),
        // the fee is taken per bundle sold
        unit_prices(bundle_prices, bundles ?? {}, sell_tax),
      ),
      // Rust asserts the plan does not decrease, and the dropdowns only disable the values
      // that would break that rather than preventing them.
      plans: [ALL_WORTHLESS, clamp_band_plan(profile.band_values)],
      optimizer_plan: 1,
      upgrade_info: upgrade_info(profile),
      // a Serca character's T4 special leaps count too
      special_budget:
        merge_lower_tier(tier, special, {}, {}).bound[
          SPECIAL_LEAP_LABELS[tier]
        ] ?? 0,
      express_event: false, // temporary overwrite while the event does nothing
      tier,
      min_resolution: profile.min_resolution,
      num_threads: 1,
      metric_type: 1,
      // Amounts stay fractional and Rust floors the total.
      chests: Object.fromEntries(
        chests.map((chest) => [
          chest.key,
          { count: chest.count, band: chest.band, options: chest.options },
        ]),
      ),
      special_label: SPECIAL_LEAP_LABELS[tier],
      adv_cache: null,
    }),
  );
}

// a snapshot's budgets and prices with the given chests, for one character
export function snapshot_payload(
  profile: CharProfile,
  snapshot: Snapshot,
  chests: Chest[],
): Payload {
  const char = snapshot.chars[profile.id];
  return payload_for(
    profile,
    char?.bound ?? {},
    snapshot.roster_owned,
    snapshot.tradable,
    char?.special ?? {},
    chests,
    snapshot.prices,
    snapshot.bundles,
  );
}

// `payload` with what an optimizer result chose, overridden where the user asked
export function states_payload(
  payload: Payload,
  profile: CharProfile,
  state: FedState,
  override?: OptimizerOverride,
): Payload {
  return {
    ...payload,
    upgrade_info: upgrade_info(profile, state, override),
    special_state: special_sort_override(
      state.special_state,
      state.upgrade_arr,
      override?.special,
    ),
    chest_states: state.chest_states,
    adv_cache: state.adv_cache,
  };
}

// the active profile over the store's budgets, chests and prices, with no state
function store_payload(): Payload {
  const store = useRosterStore();
  const profile = store.active_profile;
  return payload_for(
    profile,
    input_column_to_num(profile.bound_budgets),
    input_column_to_num(store.active_roster_mats_owned),
    input_column_to_num(store.active_tradable_mats_owned),
    input_column_to_num(profile.special_budget),
    store.processed_chests,
    input_column_to_num(store.active_mats_prices),
    store.active_selected_bundles,
  );
}

export const build_materials = () => store_payload().materials;

// The calculator's payload, with the last optimizer result's states fed back so the next run
// starts from them.
export function build_payload(override?: OptimizerOverride): Payload {
  const runtime = useRuntimeStore();
  const profile = useRosterStore().active_profile;
  const out = states_payload(
    store_payload(),
    profile,
    { ...runtime.optimizer.result, adv_cache: toRaw(runtime.adv_cache) },
    override,
  );
  // a reset upgrade is sent empty once, then takes the optimizer's state again
  for (const keyed of Object.values(profile.keyed_upgrades))
    if (keyed.state?.length === 0) keyed.state = null;
  return out;
}
