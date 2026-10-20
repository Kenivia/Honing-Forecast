import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));

import { useRosterStore } from "@/Stores/RosterConfig";
import {
  AdvOverride,
  NormalOverride,
  OptimizerOverride,
} from "@/WasmInterface/Overrides";
import {
  build_payload,
  payload_for,
  snapshot_payload,
  special_sort_override,
  states_payload,
} from "@/WasmInterface/PayloadBuilder";
import type { Snapshot } from "@/Utils/Scans";
import { hash } from "@/WasmInterface/OptimizerJobs";
import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import type { CharProfile } from "@/Stores/CharacterProfile";
import {
  ALL_LABELS,
  ALL_WORTHLESS,
  bundle_key,
  SPECIAL_LEAP_LABELS,
} from "@/Utils/Constants";
import type { InputColumn } from "@/Utils/InputColumn";
import { UpgradeStatus, type Upgrade } from "@/Utils/KeyedUpgrades";
import type { ManifestChest } from "@/Utils/Chest";

let store: ReturnType<typeof useRosterStore>;
let runtime: ReturnType<typeof useRuntimeStore>;
let profile: CharProfile;

beforeEach(() => {
  setActivePinia(createPinia());
  store = useRosterStore();
  runtime = useRuntimeStore();
  profile = store.active_profile;
});

const set = (column: InputColumn, label: string, value: number) =>
  (column.values[label] = String(value));

// keyed upgrades from tickbox cells, [piece, column] of the normal and advanced grids
function want(normal: [number, number][], adv: [number, number][] = []) {
  for (const [row, col] of normal)
    profile.normal_grid[row][col] = UpgradeStatus.Want;
  for (const [row, col] of adv) profile.adv_grid[row][col] = UpgradeStatus.Want;
  grid_change_callback(true);
}

const upgrade = (
  piece_index: number,
  upgrade_index: number,
  is_normal_honing: boolean,
  state: number[][] = [],
  piece_type_usize = piece_index === 5 ? 1 : 0,
) =>
  ({
    piece_index,
    upgrade_index,
    is_normal_honing,
    piece_type_usize,
    state,
  }) as unknown as Upgrade;

// juice / book ids an upgrade may use, by [piece type][upgrade index]
const juice_info = (ids: number[]) => ({
  normal_uindex_to_id: [{ 5: ids }, { 5: ids }],
  adv_uindex_to_id: [{ 0: [0] }, { 0: [0] }],
});

const manifest_chest = (title: string, band: number): ManifestChest => ({
  key: title,
  title,
  band,
  count: 3,
  options: [{ Red: 100 }, { Blue: 200, Gold: 5 }],
  obtain_all: false,
});

describe("build_payload: materials", () => {
  it("labels are the tier's row order and key the materials map", () => {
    for (const tier of [0, 1]) {
      profile.tier = tier;
      const p = build_payload();
      expect(p.material_labels).toEqual(ALL_LABELS[tier]);
      expect(Object.keys(p.materials).sort()).toEqual(
        [...ALL_LABELS[tier]].sort(),
      );
      expect(p.tier).toBe(tier);
      expect(p.special_label).toBe(SPECIAL_LEAP_LABELS[tier]);
    }
  });

  it("owned amounts per band and per-unit prices of the selected bundle", () => {
    set(profile.bound_budgets, "Red", 10);
    set(store.active_roster_mats_owned, "Red", 20);
    set(store.active_tradable_mats_owned, "Red", 30);
    expect(build_payload().materials.Red).toEqual({
      bound: 10,
      roster: 20,
      tradable: 30,
      // fallback 647 per 100
      market_price: 6.47,
      taxed_price: 6.14,
    });
  });

  it("gold is never tradable", () => {
    set(store.active_tradable_mats_owned, "Gold", 500);
    expect(build_payload().materials.Gold).toMatchObject({
      tradable: 0,
      market_price: 1,
    });
  });

  it("a disabled material has zero prices, no tradable stock and the sentinel bound", () => {
    // shards start disabled
    set(store.active_tradable_mats_owned, "Shards", 1000);
    expect(build_payload().materials.Shards).toEqual({
      bound: 999999999,
      roster: 0,
      tradable: 0,
      taxed_price: 0,
      market_price: 0,
    });
    profile.bound_budgets.enabled.Red = false;
    expect(build_payload().materials.Red).toMatchObject({
      taxed_price: 0,
      market_price: 0,
    });
  });

  it("a Serca character counts its T4 stock and the cheaper of buying or converting", () => {
    profile.tier = 1;
    set(profile.bound_budgets, "Red", 12);
    set(profile.bound_budgets, "Serca Red", 1);
    set(store.active_roster_mats_owned, "Red", 3);
    set(store.active_tradable_mats_owned, "Red", 4);
    const red = build_payload().materials["Serca Red"];
    expect([red.bound, red.roster, red.tradable]).toEqual([3, 1, 0]);
    // 5 Red at 6.47 beat 34.94
    expect(red.market_price).toBeCloseTo(32.35);
  });

  it("a Serca material whose T4 source is disabled does not count it", () => {
    profile.tier = 1;
    set(profile.bound_budgets, "Red", 50);
    profile.bound_budgets.enabled.Red = false;
    expect(build_payload().materials["Serca Red"].bound).toBe(0);
  });

  it("the sell tax is taken on the bundle, then split per unit", () => {
    const blue = build_payload().materials.Blue;
    // 10 per 100 sells for 9
    expect(blue.market_price).toBeCloseTo(0.1);
    expect(blue.taxed_price).toBeCloseTo(0.09);
  });
});

describe("build_payload: plans and budget", () => {
  it("plan 0 is all worthless and the user's plan is clamped to non-decreasing", () => {
    profile.band_values = ["Market", "Worthless", "TaxedSell"];
    const p = build_payload();
    expect(p.plans).toEqual([ALL_WORTHLESS, ["Market", "Market", "Market"]]);
    expect(p.optimizer_plan).toBe(1);
    expect(profile.band_values).toEqual(["Market", "Worthless", "TaxedSell"]);
  });

  it("free taps are the tier's special leaps, a Serca character's T4 ones at the ratio", () => {
    set(profile.special_budget, "Special Leap", 12);
    set(profile.special_budget, "Serca Special Leap", 1);
    expect(build_payload().special_budget).toBe(12);
    profile.tier = 1;
    expect(build_payload().special_budget).toBe(1 + 2);
  });

  it("express and the fixed settings", () => {
    profile.express_event = true;
    const p = build_payload();
    // event temporarily forced off
    expect(p.express_event).toBe(false);
    expect([p.num_threads, p.metric_type, p.min_resolution]).toEqual([1, 1, 1]);
  });
});

describe("build_payload: chests and optimizer state", () => {
  it("sends the character's and the roster's stacks by key, all of the tier", () => {
    profile.chests = [manifest_chest("Own", 0)];
    store.roster_config.roster_chests[profile.roster_id] = [
      manifest_chest("Roster", 1),
    ];
    for (const tier of [0, 1]) {
      profile.tier = tier;
      const p = build_payload();
      const stacks = Object.entries(p.chests);
      expect(stacks).toHaveLength(2);
      for (const [key, stack] of stacks) {
        expect(store.processed_chests.map((c) => c.key)).toContain(key);
        expect(stack.count).toBe(3);
        for (const option of stack.options)
          for (const label of Object.keys(option))
            expect([...ALL_LABELS[tier], SPECIAL_LEAP_LABELS[tier]]).toContain(
              label,
            );
      }
    }
  });

  it("without an optimizer result there is no state to feed back", () => {
    const p = build_payload();
    expect(p.special_state).toBeUndefined();
    expect(p.chest_states).toBeUndefined();
  });

  it("feeds back the optimizer's free-tap order and chest states", () => {
    const chest_states = new Map([["k", [1, 2]]]);
    runtime.optimizer.result = {
      upgrade_arr: [],
      special_state: [1, 0],
      chest_states,
    };
    const p = build_payload();
    expect(p.special_state).toEqual([1, 0]);
    expect(p.chest_states).toBe(chest_states);
  });

  it("can be posted to a worker", () => {
    want([[0, 5]], [[0, 0]]);
    profile.chests = [manifest_chest("Own", 0)];
    runtime.adv_cache = new Map([["a", 1]]);
    runtime.optimizer.result = {
      upgrade_arr: [upgrade(0, 5, true, [[0]])],
      special_state: [0, 1],
      chest_states: new Map(),
    };
    runtime.optimizer.result.prep_output = {
      juice_info: juice_info([0, 1, 2]),
    };
    expect(() =>
      structuredClone(build_payload(profile.optimizer_override)),
    ).not.toThrow();
  });
});

describe("build_payload: upgrades", () => {
  it("T4 has no vambrace and Serca no advanced honing", () => {
    want(
      [
        [0, 5],
        [6, 5],
      ],
      [[0, 0]],
    );
    const t4 = build_payload().upgrade_info;
    expect(t4.map((u) => [u.piece_index, u.is_normal_honing])).toEqual([
      [0, true],
      [0, false],
    ]);
    expect(t4.find((u) => !u.is_normal_honing).double_balls).toBe(true);
    expect(t4.find((u) => u.is_normal_honing).double_balls).toBe(false);

    profile.tier = 1;
    grid_change_callback(true);
    const serca = build_payload().upgrade_info;
    expect(serca.every((u) => u.is_normal_honing)).toBe(true);
    expect(serca.map((u) => u.piece_index).sort()).toEqual([0, 6]);
  });

  it("the optimizer's state is sent from the taps since the last input on", () => {
    want([[0, 5]]);
    const keyed = Object.values(profile.keyed_upgrades)[0];
    keyed.taps_since_last_input = 1;
    keyed.starting_num_taps = 1;
    runtime.optimizer.result = {
      upgrade_arr: [upgrade(0, 5, true, [[0], [1], [2]])],
    };
    runtime.optimizer.result.prep_output = {
      juice_info: juice_info([0, 1, 2]),
    };
    const sent = build_payload().upgrade_info[0];
    expect(sent.state).toEqual([[1], [2]]);
    expect(sent.unlocked).toBe(true);
    // the store's copy is untouched
    expect(keyed.state).toBeNull();
  });

  it("typed artisan counts as unlocked before any optimizer result", () => {
    want([[0, 5]]);
    expect(build_payload().upgrade_info[0].unlocked).toBe(false);
    Object.values(profile.keyed_upgrades)[0].starting_artisan = 0.3;
    expect(build_payload().upgrade_info[0].unlocked).toBe(true);
  });

  it("no state is fed back without the juice info of its optimizer run", () => {
    want([[0, 5]]);
    runtime.optimizer.result = { upgrade_arr: [upgrade(0, 5, true, [[0]])] };
    expect(build_payload().upgrade_info[0].state).toBeNull();
  });

  it("a reset state is sent empty once, then the optimizer's is used again", () => {
    want([[0, 5]]);
    const keyed = Object.values(profile.keyed_upgrades)[0];
    keyed.state = [];
    runtime.optimizer.result = { upgrade_arr: [upgrade(0, 5, true, [[0]])] };
    runtime.optimizer.result.prep_output = {
      juice_info: juice_info([0, 1, 2]),
    };
    expect(build_payload().upgrade_info[0].state).toEqual([]);
    expect(keyed.state).toBeNull();
    expect(build_payload().upgrade_info[0].state).toEqual([[0]]);
  });

  it("normal overrides fill or empty juices (ids 0, 1) and books (the rest)", () => {
    want([[0, 5]]);
    runtime.optimizer.result = {
      upgrade_arr: [upgrade(0, 5, true, [[2], [0, 3]])],
    };
    runtime.optimizer.result.prep_output = {
      juice_info: juice_info([0, 1, 2, 3]),
    };
    const with_override = (juice: NormalOverride, book: NormalOverride) =>
      build_payload({
        ...profile.optimizer_override,
        normal: { juice, book },
      }).upgrade_info[0].state.map((tap) => [...new Set(tap)].sort());
    expect(with_override(NormalOverride.Full, NormalOverride.Empty)).toEqual([
      [0, 1],
      [0, 1],
    ]);
    expect(with_override(NormalOverride.Empty, NormalOverride.Full)).toEqual([
      [2, 3],
      [2, 3],
    ]);
    expect(
      with_override(NormalOverride.Optimizer, NormalOverride.Optimizer),
    ).toEqual([[2], [0, 3]]);
  });

  it("advanced overrides pick the first, grace or last strategy", () => {
    want([], [[0, 0]]);
    runtime.optimizer.result = {
      upgrade_arr: [upgrade(0, 0, false, [[3], [4]])],
    };
    runtime.optimizer.result.prep_output = { juice_info: juice_info([]) };
    const with_override = (juice: AdvOverride, scroll: AdvOverride) =>
      build_payload({
        ...profile.optimizer_override,
        advanced: { juice, scroll },
      } as OptimizerOverride).upgrade_info[0].state;
    expect(with_override(AdvOverride.Optimizer, AdvOverride.Optimizer)).toEqual(
      [[3], [4]],
    );
    expect(with_override(AdvOverride.Grace, AdvOverride.Empty)).toEqual([
      [13],
      [0],
    ]);
    expect(with_override(AdvOverride.Full, AdvOverride.Grace)).toEqual([
      [18],
      [13],
    ]);
  });
});

describe("special_sort_override", () => {
  // armor +3, weapon +2, armor +5, advanced, weapon +4
  const arr = [
    upgrade(0, 3, true),
    upgrade(5, 2, true),
    upgrade(0, 5, true),
    upgrade(1, 1, false),
    upgrade(5, 4, true),
  ];
  const order = [0, 1, 2, 3, 4];

  it("leaves the optimizer's order alone", () => {
    const same = special_sort_override(order, arr, {
      optimizer: true,
      weapon_first: true,
      highest_first: true,
    });
    expect(same).toBe(order);
    expect(special_sort_override(order, arr)).toBe(order);
  });

  it("lowest first: normal honing, then the preferred piece type, then the level", () => {
    const sort = (weapon_first: boolean) =>
      special_sort_override(order, arr, {
        optimizer: false,
        weapon_first,
        highest_first: false,
      });
    expect(sort(true)).toEqual([1, 4, 0, 2, 3]);
    expect(sort(false)).toEqual([0, 2, 1, 4, 3]);
    expect(order).toEqual([0, 1, 2, 3, 4]);
  });

  it("highest first: each piece's top upgrade first", () => {
    expect(
      special_sort_override(order, arr, {
        optimizer: false,
        weapon_first: true,
        highest_first: true,
      }),
    ).toEqual([4, 2, 1, 0, 3]);
  });
});

describe("payload_for", () => {
  const prices = { [bundle_key("Red", 100)]: 700 };
  const of = (
    over: Partial<{
      bound: Record<string, number>;
      special: Record<string, number>;
    }> = {},
  ) =>
    payload_for(
      profile,
      over.bound ?? { Red: 5 },
      { Red: 10 },
      { Blue: 20 },
      over.special ?? { "Special Leap": 3 },
      [],
      prices,
      {},
    );

  it("is plain, with no optimizer state and no UI-only upgrade fields", () => {
    want([[0, 5]], [[0, 0]]);
    const p = of();
    expect(JSON.parse(JSON.stringify(p))).toEqual(p);
    expect("special_state" in p).toBe(false);
    expect("chest_states" in p).toBe(false);
    expect(p.adv_cache).toBeNull();
    expect(p.upgrade_info).toHaveLength(2);
    for (const u of p.upgrade_info) {
      expect(Object.keys(u)).not.toContain("expanded");
      expect(Object.keys(u)).not.toContain("used_materials");
      expect(Object.keys(u)).not.toContain("taps_since_last_input");
    }
  });

  it("uses the given budgets and prices; a missing price is zero", () => {
    const p = of();
    expect(p.materials.Red).toEqual({
      bound: 5,
      roster: 10,
      tradable: 0,
      market_price: 7,
      taxed_price: 6.65,
    });
    expect(p.materials.Blue).toMatchObject({ tradable: 20, market_price: 0 });
    expect(p.special_budget).toBe(3);
    expect(of({ special: {} }).special_budget).toBe(0);
  });

  it("prices a material by the bundle chosen for it", () => {
    profile.bound_budgets.enabled.Shards = true;
    const p = payload_for(
      profile,
      {},
      {},
      {},
      {},
      [],
      { [bundle_key("Shards", 1000)]: 100, [bundle_key("Shards", 2000)]: 100 },
      { Shards: 2000 },
    );
    expect(p.materials.Shards.market_price).toBeCloseTo(0.05);
  });

  it("merges a Serca character's T4 free taps", () => {
    profile.tier = 1;
    expect(
      of({ special: { "Special Leap": 12, "Serca Special Leap": 1 } })
        .special_budget,
    ).toBe(3);
  });

  it("hashes the same for the same inputs", () => {
    want([[0, 5]]);
    expect(hash(JSON.stringify(of()))).toBe(hash(JSON.stringify(of())));
    expect(hash(JSON.stringify(of({ bound: { Red: 6 } })))).not.toBe(
      hash(JSON.stringify(of())),
    );
  });
});

describe("snapshot_payload", () => {
  it("is payload_for of the character's entry and the snapshot's roster and prices", () => {
    const snapshot = {
      roster_owned: { Red: 10 },
      tradable: { Blue: 20 },
      prices: { [bundle_key("Red", 100)]: 700 },
      bundles: {},
      chars: {
        [profile.id]: { bound: { Red: 5 }, special: { "Special Leap": 3 } },
      },
    } as unknown as Snapshot;
    expect(snapshot_payload(profile, snapshot, [])).toEqual(
      payload_for(
        profile,
        { Red: 5 },
        { Red: 10 },
        { Blue: 20 },
        { "Special Leap": 3 },
        [],
        { [bundle_key("Red", 100)]: 700 },
        {},
      ),
    );
    // a character not in it has no bound stock or free taps
    snapshot.chars = {};
    const p = snapshot_payload(profile, snapshot, []);
    expect(p.materials.Red.bound).toBe(0);
    expect(p.special_budget).toBe(0);
  });
});

describe("states_payload", () => {
  it("applies an override to the states and the free-tap order", () => {
    want([[0, 5]]);
    const base = payload_for(profile, {}, {}, {}, {}, [], {}, {});
    const state = {
      upgrade_arr: [upgrade(0, 5, true, [[0, 2]]), upgrade(5, 5, true)],
      special_state: [0, 1],
      prep_output: { juice_info: juice_info([0, 1, 2]) },
    };
    const p = states_payload(base, profile, state, {
      ...profile.optimizer_override,
      normal: { juice: NormalOverride.Empty, book: NormalOverride.Optimizer },
      special: { optimizer: false, weapon_first: true, highest_first: false },
    });
    expect(p.upgrade_info[0].state).toEqual([[2]]);
    expect(p.special_state).toEqual([1, 0]);
  });

  it("adds an optimizer result's states and keeps the rest", () => {
    want([[0, 5]]);
    const base = payload_for(profile, {}, {}, {}, {}, [], {}, {});
    const state = {
      upgrade_arr: [upgrade(0, 5, true, [[1], [0]])],
      special_state: [0],
      chest_states: new Map([["k", [1]]]),
      adv_cache: new Map(),
      prep_output: { juice_info: juice_info([0, 1]) },
    };
    const p = states_payload(base, profile, state);
    expect(p.materials).toBe(base.materials);
    expect(p.plans).toBe(base.plans);
    expect(p.upgrade_info[0].state).toEqual([[1], [0]]);
    expect(p.special_state).toBe(state.special_state);
    expect(p.chest_states).toBe(state.chest_states);
    expect(p.adv_cache).toBe(state.adv_cache);
    // nothing of the calculator's runtime is read or written
    expect(runtime.optimizer.result).toBeNull();
  });
});
