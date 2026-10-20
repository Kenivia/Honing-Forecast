export interface Support {
  support: number[];
}
export interface AdvConfig {
  start_xp: number;
  start_balls: number;
  next_free: boolean;
  next_big: boolean;
  double_balls: boolean;
  is_30_40: boolean;

  grace_juice_target: number;
  non_grace_juice_target: number;
  grace_scroll_target: number;
  non_grace_scroll_target: number;
}

export interface Upgrade {
  piece_index: number;
  piece_type_usize: number;
  upgrade_index: number;
  is_normal_honing?: boolean;
  // is_weapon?: boolean;
  normal_dist?: number[];
  adv_dists?: number[][];
  state?: OneState[];
  succeeded?: boolean;
  unlocked?: boolean;
  starting_artisan?: number;
  starting_num_taps?: number;
  adv_config: AdvConfig;
  cost_dist: Support[];
  artisan_rate: number;
  base_chance: number;
  extra_chance: number;
  unlock_costs: number[];
  costs: number[];

  // added for UI purpose, not in rust (it's wiped after optimizer run)
  this_special_chance?: number;
}
export type OneState = number[];
export type AdvProgress = [number, number, boolean, boolean]; // current xp(0 to 100 or 99 ig), current balls ( 0 to 6), next_free, next_big

// ========================================================================================
// Status grid (TickboxGrid) stuff
export enum UpgradeStatus {
  Done,
  Want,
  NotYet,
  FetchedDone,
}

export type StatusGrid = UpgradeStatus[][];

// ========================================================================================
// These are to interface between UI and rust

// an array of this is passed into rust
export interface OneUpgradeInput {
  piece_index: number;
  upgrade_index: number;
  is_normal_honing: boolean;
  starting_artisan: number | null;
  starting_num_taps: number | null;
  state: OneState[] | null;
  unlocked: boolean;
  adv_progress: AdvProgress | null;
  double_balls: boolean;

  // This is here because 1. it needs to persist through optimizser runs and 2. extra fields are ignored by serde so its perfect
  expanded: boolean;
  taps_since_last_input: number;
  used_materials: Record<string, number> | null;
}

export type OneUpgradeKey = `${number},${number},${"true" | "false"},${number}`;
export type KeyedUpgrades = Record<OneUpgradeKey, OneUpgradeInput>; // This is modified by UI
export function get_upgrade_map(
  upgrade_arr: Upgrade[],
  tier: number,
): Map<string, Upgrade> {
  const upgrade_map = new Map<string, Upgrade>();
  if (upgrade_arr != null) {
    for (const upgrade of upgrade_arr) {
      const key = to_upgrade_key(
        upgrade.piece_index,
        upgrade.upgrade_index,
        upgrade.is_normal_honing,
        tier,
      );
      upgrade_map.set(key, upgrade);
    }
  }
  return upgrade_map;
}
export function to_upgrade_key(
  piece_index: number,
  upgrade_index: number,
  is_normal_honing: boolean,
  tier: number,
): OneUpgradeKey {
  return `${piece_index},${upgrade_index},${is_normal_honing},${tier}`;
}
export function grids_to_keyed(
  normal_grid: StatusGrid,
  adv_grid: StatusGrid,
  all_keyed: KeyedUpgrades,
  tier: number,
) {
  const new_keyed: KeyedUpgrades = {};
  for (const [grid, is_normal_honing] of [
    [normal_grid, true],
    [adv_grid, false],
  ] as const) {
    for (const [piece_index, row] of grid.entries()) {
      for (const [upgrade_index, upgrade_status] of row.entries()) {
        // non-want ones are discarded
        if (upgrade_status !== UpgradeStatus.Want) continue;
        const key = to_upgrade_key(
          piece_index,
          upgrade_index,
          is_normal_honing,
          tier,
        );
        new_keyed[key] = is_one_upgrade(all_keyed[key])
          ? all_keyed[key]
          : {
              piece_index,
              upgrade_index,
              is_normal_honing,
              starting_artisan: 0,
              starting_num_taps: 0,
              state: null,
              unlocked: false,
              // a fresh array each, as the import writes into it
              adv_progress: is_normal_honing ? null : [0, 0, false, false],
              expanded: false,
              taps_since_last_input: 0,
              used_materials: null,
              double_balls: true,
            };
      }
    }
  }
  return new_keyed;
}
function is_one_upgrade(obj: unknown): obj is OneUpgradeInput {
  if (typeof obj !== "object" || obj === null) return false;

  const o = obj as Record<string, unknown>;

  if (typeof o.piece_index !== "number") return false;
  if (typeof o.upgrade_index !== "number") return false;
  if (typeof o.is_normal_honing !== "boolean") return false;
  if (typeof o.unlocked !== "boolean") return false;
  if (typeof o.expanded !== "boolean") return false;
  if (typeof o.taps_since_last_input !== "number") return false;

  if (
    o.starting_artisan !== undefined &&
    o.starting_artisan !== null &&
    typeof o.starting_artisan !== "number"
  )
    return false;

  if (o.state !== undefined && o.state !== null) {
    if (!Array.isArray(o.state)) return false;
    if (
      !o.state.every(
        (one_state) =>
          Array.isArray(one_state) &&
          one_state.every((x) => typeof x === "number"),
      )
    ) {
      return false;
    }
  }

  if (o.adv_progress !== undefined && o.adv_progress !== null) {
    const adv = o.adv_progress;
    if (
      !Array.isArray(adv) ||
      adv.length !== 4 ||
      typeof adv[0] !== "number" ||
      typeof adv[1] !== "number" ||
      typeof adv[2] !== "boolean" ||
      typeof adv[3] !== "boolean"
    )
      return false;
  }

  return o.double_balls === undefined || typeof o.double_balls === "boolean";
}
