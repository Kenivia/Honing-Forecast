import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import {
  OneUpgradeInput,
  to_upgrade_key,
  Upgrade,
} from "@/Utils/KeyedUpgrades";

// Hand-built stand-ins for the wasm result shapes the instruction components read.

// Rust's chances by failed taps: a tenth of base more per failure, up to double
export const fail_chances = (base: number, extra = 0) =>
  Array.from({ length: 11 }, (_, n) => Math.min(1, base * (1 + n / 10) + extra));

// artisan gains each tap's chance, so a 25% base reaches it on the 5th tap
export function fake_upgrade(over: Partial<Upgrade> = {}): Upgrade {
  return {
    piece_index: 0,
    piece_type_usize: 0,
    upgrade_index: 10,
    is_normal_honing: true,
    // slot 0 is the free tap, then taps 1..5; the 5th is the pity at base 25%
    normal_dist: [0, 0.2, 0.2, 0.2, 0.2, 0.2],
    state: [[0], [0], [], [], []],
    succeeded: false,
    unlocked: false,
    starting_artisan: 0,
    starting_num_taps: 0,
    adv_config: {
      start_xp: 0,
      start_balls: 0,
      next_free: false,
      next_big: false,
      double_balls: true,
      is_30_40: false,
      grace_juice_target: 0,
      non_grace_juice_target: 0,
      grace_scroll_target: 0,
      non_grace_scroll_target: 0,
    },
    cost_dist: [],
    base_chance: 0.25,
    tap_artisan: [0, 0.25, 0.525, 0.825, 1.15],
    fail_chances: fail_chances(0.25),
    // T4 armor +11's
    special_cost: 12,
    // Red, Blue, Leaps, Shards, Fusion, Gold, Silver
    unlock_costs: [0, 0, 0, 1000, 0, 0, 0],
    costs: [10, 20, 3, 0, 1, 5, 100],
    ...over,
  };
}

export function fake_adv(over: Partial<Upgrade> = {}): Upgrade {
  return fake_upgrade({
    is_normal_honing: false,
    upgrade_index: 1,
    normal_dist: [],
    state: [[0], [0]],
    special_cost: 0,
    ...over,
  });
}

// ids 0 (juice, 5 per tap) and 2 (book, 1 per tap) at every level, for normal and advanced
export function fake_juice_info(normal_rate = 0) {
  const entry = { normal_rate, normal_amt_used: 5, adv_amt_used: 7 };
  const levels = [...Array(30).keys()];
  const per_piece = <T>(f: () => T) => [0, 1, 2].map(f);
  return {
    all_juices: [0, 1, 2, 3].map(() =>
      per_piece(() => new Map(levels.map((l) => [l, entry]))),
    ),
    normal_uindex_to_id: per_piece(() => levels.map(() => [0, 2])),
    adv_uindex_to_id: per_piece(() => levels.map(() => [0, 2])),
  };
}

export function fresh_stores() {
  setActivePinia(createPinia());
  return { store: useRosterStore(), runtime: useRuntimeStore() };
}

export function key_of(u: Upgrade, tier = 0) {
  return to_upgrade_key(
    u.piece_index,
    u.upgrade_index,
    u.is_normal_honing,
    tier,
  );
}

// a keyed entry for each upgrade, as grids_to_keyed would make it
export function add_keyed(
  upgrades: Upgrade[],
  over: Partial<OneUpgradeInput> = {},
) {
  const profile = useRosterStore().active_profile;
  for (const u of upgrades) {
    profile.keyed_upgrades[key_of(u, profile.tier)] = {
      piece_index: u.piece_index,
      upgrade_index: u.upgrade_index,
      is_normal_honing: u.is_normal_honing,
      starting_artisan: 0,
      starting_num_taps: 0,
      state: null,
      unlocked: false,
      adv_progress: u.is_normal_honing ? null : [0, 0, false, false],
      expanded: false,
      taps_since_last_input: 0,
      used_materials: null,
      double_balls: true,
      ...over,
    };
  }
}

// both workers' results, the histogram's carrying its own copy of the state bundle
export function set_results(
  upgrade_arr: Upgrade[],
  special_state = upgrade_arr.map((_, i) => i),
  special_invalid_index = 0,
  latest_special_probs = upgrade_arr.map(() => 0),
) {
  const runtime = useRuntimeStore();
  const bundle = {
    upgrade_arr,
    special_state,
    special_invalid_index,
    latest_special_probs,
    chest_states: new Map(),
    prep_output: { table: { labels: [] }, materials: [] },
  };
  runtime.optimizer.result = bundle;
  runtime.histogram.result = {
    state_bundle: structuredClone(bundle),
    juice_info: fake_juice_info(),
  };
}
