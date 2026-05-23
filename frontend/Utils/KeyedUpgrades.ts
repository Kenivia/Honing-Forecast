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
  piece_type: number;
  upgrade_index: number;
  is_normal_honing?: boolean;
  is_weapon?: boolean;
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
export type OneState = [boolean, number]; // juice, bookid
export type AdvProgress = [number, number, boolean, boolean]; // current xp(0 to 100 or 99 ig), current balls ( 0 to 6), next_free, next_big

// ========================================================================================

// ========================================================================================
// Status grid (TickboxGrid) stuff
export enum UpgradeStatus {
  Done,
  Want,
  NotYet,
}

export type BoolGrid = boolean[][];
export type StatusGrid = UpgradeStatus[][];

// ========================================================================================
// These are to interface between UI and rust

//                    'bound','tradable', leftover(bound), tradable sell price, market price

export type BudgetPricePair = [number, number];
export type OneMaterialInput = BudgetPricePair[]; // an array of this is passed into rust

//                        piece type, upgrade index, is_normal_honing, normal_progress, state, unlocked, succeeded, adv_progress
export type OldOneUpgrade = [
  number,
  number,
  boolean,
  number | null,
  OneState[] | null,
  boolean,
  boolean,
  AdvProgress | null,
];

// an array of this is passed into rust
export interface OneUpgradeInput {
  piece_type: number;
  upgrade_index: number;
  is_normal_honing: boolean;
  starting_artisan?: number;
  starting_num_taps?: number;
  state?: OneState[];
  unlocked: boolean;
  adv_progress?: AdvProgress;

  // This is here because 1. it needs to persist through optimizser runs and 2. extra fields are ignored by serde so its perfect
  expanded: boolean;
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
        upgrade.piece_type,
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
  piece_type: number,
  upgrade_index: number,
  is_normal_honing: boolean,
  tier: number,
): OneUpgradeKey {
  return `${piece_type},${upgrade_index},${is_normal_honing},${tier}`;
}
export function grids_to_keyed(
  normal_grid: StatusGrid,
  adv_grid: StatusGrid,
  all_keyed: KeyedUpgrades,
  tier: number,
) {
  let new_keyed: KeyedUpgrades = {};

  const grid_configs = [
    { grid: normal_grid, is_normal_honing: true, default_adv_progress: null },
    {
      grid: adv_grid,
      is_normal_honing: false,
      default_adv_progress: [0, 0, false, false] as AdvProgress,
    },
  ];

  for (const { grid, is_normal_honing, default_adv_progress } of grid_configs) {
    for (const [piece_type, row] of grid.entries()) {
      for (const [upgrade_index, upgrade_status] of row.entries()) {
        const key = to_upgrade_key(
          piece_type,
          upgrade_index,
          is_normal_honing,
          tier,
        );
        if (upgrade_status === UpgradeStatus.Want) {
          if (key in all_keyed && is_one_upgrade(all_keyed[key])) {
            new_keyed[key] = all_keyed[key];
          } else if (is_old_one_upgrade(all_keyed[key])) {
            const [
              piece_type,
              upgrade_index,
              is_normal_honing,
              _normal_progress,
              state,
              unlocked,
              _succeeded,
              adv_progress,
            ] = all_keyed[key] as unknown as OldOneUpgrade;
            new_keyed[key] = {
              piece_type: piece_type,
              upgrade_index: upgrade_index,
              is_normal_honing: is_normal_honing,
              starting_artisan: 0,
              starting_num_taps: 0, // TODO technically can reverse-engineer a artisan / num taps here, but that'll need thet actual parsed Upgrade object (unless I re-implement all the logic in typescript) so maybe its not worth the hassle
              state: state,
              unlocked: unlocked,
              adv_progress: adv_progress,
              expanded: false,
            };
          } else {
            new_keyed[key] = {
              piece_type,
              upgrade_index,
              is_normal_honing,
              starting_artisan: 0,
              starting_num_taps: 0,
              state: null,
              unlocked: false,
              adv_progress: default_adv_progress,
              expanded: false,
            };
          }
        } else {
          // note we don't delete unused ones, so when user undos 78987something it'll still be there
        }
      }
    }
  }

  return new_keyed;
}
function is_one_upgrade(obj: unknown): obj is OneUpgradeInput {
  if (typeof obj !== "object" || obj === null) return false;

  const o = obj as Record<string, unknown>;

  if (typeof o.piece_type !== "number") return false;
  if (typeof o.upgrade_index !== "number") return false;
  if (typeof o.is_normal_honing !== "boolean") return false;
  if (typeof o.unlocked !== "boolean") return false;
  if (typeof o.expanded !== "boolean") return false;

  if (
    o.starting_artisan !== undefined &&
    typeof o.starting_artisan !== "number"
  )
    return false;

  if (o.state !== undefined) {
    if (!Array.isArray(o.state)) return false;
    for (const entry of o.state) {
      if (
        !Array.isArray(entry) ||
        entry.length !== 2 ||
        typeof entry[0] !== "boolean" ||
        typeof entry[1] !== "number"
      )
        return false;
    }
  }

  if (o.adv_progress !== undefined) {
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
  return true;
}
function is_old_one_upgrade(foo: unknown): foo is OneUpgradeInput {
  if (!Array.isArray(foo) || foo.length !== 8) return false;

  const [f0, f1, f2, f3, f4, f5, f6, f7] = foo;

  return (
    typeof f0 === "number" &&
    typeof f1 === "number" &&
    typeof f2 === "boolean" &&
    (f3 === null || typeof f3 === "number") &&
    (f4 === null || Array.isArray(f4)) &&
    (f4 === null ||
      f4.every(
        (x) =>
          x.length == 2 &&
          typeof x[0] === "boolean" &&
          typeof x[1] === "number",
      )) &&
    typeof f5 === "boolean" &&
    typeof f6 === "boolean" &&
    (f7 === null ||
      (typeof f7[0] === "number" &&
        typeof f7[1] === "number" &&
        typeof f7[2] === "boolean" &&
        typeof f7[3] === "boolean"))
  );
}
