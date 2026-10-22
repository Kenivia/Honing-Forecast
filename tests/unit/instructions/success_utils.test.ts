import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/Components/Character/CharWorkerUtils", () => ({
  grid_change_callback: vi.fn(),
  start_all_workers: vi.fn(),
  start_eval_hist: vi.fn(),
}));
import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";
import {
  apply_remaining_mats,
  compute_remaining_materials,
  deduct,
  finish_upgrade,
  compute_used_materials,
  make_budget_snapshot,
  mark_upgrade_as_done,
  set_special_remaining,
  settle_deductions,
  spend_band,
} from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import { ALL_LABELS, SPECIAL_LEAP_LABELS } from "@/Utils/Constants";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { new_char_profile } from "@/Stores/CharacterProfile";
import {
  add_keyed,
  fake_adv,
  fake_juice_info,
  fake_upgrade,
  fresh_stores,
  key_of,
} from "./fake";

const L = ALL_LABELS[0];
const juice = L[7];
const book = L[9];
const used = (u, taps, pretend) =>
  compute_used_materials(u, taps, fake_juice_info(), pretend, 0);

describe("compute_used_materials", () => {
  it("charges the unlock, base costs per tap and juice per juiced tap", () => {
    const out = used(fake_upgrade(), 2, true);
    expect(out.Shards).toBe(1000);
    expect([out.Red, out.Blue, out.Silver]).toEqual([20, 40, 200]);
    expect(out[juice]).toBe(10);
    expect(out[book]).toBe(0);
    expect(Object.keys(out)).toEqual(L);
  });

  it("does not charge the unlock while the slider sits at 0", () => {
    const out = used(fake_upgrade(), 0, true);
    expect(Object.values(out).every((v) => v === 0)).toBe(true);
  });

  it("charges only the unlock on 0 taps when not pretending (free tap success)", () => {
    const out = used(fake_upgrade(), 0, false);
    expect(out.Shards).toBe(1000);
    expect(out.Red).toBe(0);
  });

  it("skips the unlock once taps were recorded", () => {
    expect(used(fake_upgrade({ starting_num_taps: 3 }), 2, true).Shards).toBe(
      0,
    );
  });

  it("never charges juice on the pity tap", () => {
    const u = fake_upgrade({ state: [[0], [0], [0], [0], [0]] });
    const out = used(u, 5, true);
    expect(out[juice]).toBe(4 * 5);
    expect(out.Red).toBe(5 * 10);
  });

  it("does not charge the unlock of an upgrade with typed artisan", () => {
    const u = fake_upgrade({ starting_artisan: 0.3 });
    expect(used(u, 2, true).Shards).toBe(0);
  });
});

describe("spend_band", () => {
  it("takes own stock, then whole conversions across sources in order", () => {
    expect(spend_band(1, [3, 8], 5, 4)).toEqual({
      own: 0,
      sources: [0, 1],
      cost: 1,
    });
  });

  it("leaves sources alone when they do not make one unit", () => {
    expect(spend_band(0, [4], 5, 1)).toEqual({ own: 0, sources: [4], cost: 1 });
  });
});

describe("compute_remaining_materials", () => {
  beforeEach(() => fresh_stores());

  it("snapshots the store when none is given, and ignores negative costs", () => {
    const { store } = fresh_stores();
    set_cell(store.active_profile.bound_budgets, "Red", "50");
    set_cell(store.active_roster_mats_owned, "Red", "7");
    const out = compute_remaining_materials({ Red: 55, Blue: -10 });
    expect([out.bound_budgets.Red, out.roster_mats.Red]).toEqual([0, 2]);
    expect(out.bound_budgets.Blue).toBe(0);
  });

  it("a disabled material is spent from its unlimited bound stock only", () => {
    const { store } = fresh_stores();
    expect(store.active_profile.bound_budgets.enabled.Shards).toBe(false);
    set_cell(store.active_roster_mats_owned, "Shards", "100");
    const out = compute_remaining_materials({ Shards: 5000 });
    expect(out.roster_mats.Shards).toBe(100);
  });

  it("make_budget_snapshot is a copy", () => {
    const { store } = fresh_stores();
    const snap = make_budget_snapshot();
    set_cell(store.active_profile.bound_budgets, "Red", "9");
    expect(snap.bound_budgets.values.Red).toBe("0");
  });
});

describe("apply_remaining_mats", () => {
  it("does nothing with auto deduction off", () => {
    const { store, runtime } = fresh_stores();
    store.roster_config.auto_deduct_costs = false;
    apply_remaining_mats();
    expect(runtime.budget_snapshots).toEqual({});
  });

  it("deducts the sum over upgrades from the snapshot, not cumulatively", () => {
    const { store, runtime } = fresh_stores();
    const p = store.active_profile;
    set_cell(p.bound_budgets, "Red", "1000");
    set_cell(p.bound_budgets, "Shards", "1000"); // disabled: not written
    const a = fake_upgrade();
    const b = fake_upgrade({ piece_index: 1 });
    add_keyed([a, b]);
    p.keyed_upgrades[key_of(a)].used_materials = { Red: 100, Shards: 1 };
    p.keyed_upgrades[key_of(b)].used_materials = { Red: 30 };
    apply_remaining_mats();
    expect(runtime.budget_snapshots[p.id]).toBeDefined();
    expect(input_column_to_num(p.bound_budgets).Red).toBe(870);
    // the slider moved back: measured again from the snapshot
    p.keyed_upgrades[key_of(a)].used_materials = { Red: 10 };
    apply_remaining_mats();
    expect(input_column_to_num(p.bound_budgets).Red).toBe(960);
    expect(p.bound_budgets.values.Shards).toBe("1,000");
  });

  it("a snapshot of one character is never written into another", () => {
    const { store } = fresh_stores();
    store.add_profile({
      ...new_char_profile(),
      roster_id: store.active_profile.roster_id,
    });
    set_cell(store.active_profile.bound_budgets, "Red", "1000");
    apply_remaining_mats();
    store.switch_profile(1);
    set_cell(store.active_profile.bound_budgets, "Red", "5");
    apply_remaining_mats();
    expect(input_column_to_num(store.active_profile.bound_budgets).Red).toBe(5);
  });
});

describe("deduct and finish_upgrade", () => {
  function two_upgrades() {
    const { store, runtime } = fresh_stores();
    const p = store.active_profile;
    set_cell(p.bound_budgets, "Red", "1000");
    const a = fake_upgrade({ unlock_costs: [7, 0, 0, 0, 0, 0, 0] });
    const b = fake_upgrade({ piece_index: 1 });
    add_keyed([a, b]);
    const red = () => input_column_to_num(p.bound_budgets).Red;
    return { p, runtime, a, b, red, keyed: (u) => p.keyed_upgrades[key_of(u)] };
  }

  it("retakes the snapshot when nothing is outstanding", () => {
    const { p, a, red, keyed } = two_upgrades();
    deduct(keyed(a), { Red: 100 });
    deduct(keyed(a), { Red: 0 });
    expect(red()).toBe(1000);
    // an edit while nothing is deducted is kept
    set_cell(p.bound_budgets, "Red", "500");
    deduct(keyed(a), { Red: 10 });
    expect(red()).toBe(490);
  });

  it("a success is folded into the snapshot, so later deductions keep it", () => {
    const { runtime, p, a, b, red, keyed } = two_upgrades();
    deduct(keyed(b), { Red: 30 });
    finish_upgrade(a, 2, fake_juice_info());
    expect(red()).toBe(1000 - 30 - 7 - 20);
    expect(keyed(a).used_materials).toBeNull();
    expect(p.normal_grid[0][10]).toBe(UpgradeStatus.Done);
    // b's slider moves back to 0: a's cost stays deducted
    deduct(keyed(b), { Red: 0 });
    expect(red()).toBe(1000 - 7 - 20);
    expect(
      input_column_to_num(runtime.budget_snapshots[p.id].bound_budgets).Red,
    ).toBe(973);
  });
});

describe("settle_deductions", () => {
  function two_chars() {
    const { store, runtime } = fresh_stores();
    store.add_char(store.active_profile.roster_id);
    const [a, b] = store.roster_config.profiles;
    set_cell(store.active_roster_mats_owned, "Red", "1000");
    const roster_red = () =>
      input_column_to_num(store.active_roster_mats_owned).Red;
    const keyed_of = () => {
      const u = fake_upgrade();
      add_keyed([u]);
      return store.active_profile.keyed_upgrades[key_of(u)];
    };
    return { store, runtime, a, b, roster_red, keyed_of };
  }

  it("keeps the deduction and forgets the snapshot and the usage", () => {
    const { store, runtime, a, roster_red, keyed_of } = two_chars();
    const keyed = keyed_of();
    deduct(keyed, { Red: 100 });
    keyed.taps_since_last_input = 2;
    settle_deductions(a);
    expect(roster_red()).toBe(900);
    expect(runtime.budget_snapshots[a.id]).toBeUndefined();
    expect([keyed.used_materials, keyed.taps_since_last_input]).toEqual([
      null,
      0,
    ]);
    // a later deduction starts from the budgets as they are
    deduct(keyed, { Red: 10 });
    expect(roster_red()).toBe(890);
    expect(store.active_profile.id).toBe(a.id);
  });

  it("two characters of a roster deducting in turn both count", () => {
    const { store, a, b, roster_red, keyed_of } = two_chars();
    const on_a = keyed_of();
    deduct(on_a, { Red: 100 });
    settle_deductions(a);
    store.point_active_at(b.id);
    const on_b = keyed_of();
    deduct(on_b, { Red: 30 });
    settle_deductions(b);
    store.point_active_at(a.id);
    deduct(on_a, { Red: 5 });
    expect(roster_red()).toBe(1000 - 100 - 30 - 5);
  });

  it("an edit made elsewhere after leaving is kept", () => {
    const { store, a, roster_red, keyed_of } = two_chars();
    const keyed = keyed_of();
    deduct(keyed, { Red: 100 });
    settle_deductions(a);
    // e.g. the market page or Choices' Add to budget
    set_cell(store.active_roster_mats_owned, "Red", "2000");
    deduct(keyed, { Red: 10 });
    expect(roster_red()).toBe(1990);
  });
});

describe("set_special_remaining", () => {
  it("a T4 character's free taps become the remaining count", () => {
    const { store } = fresh_stores();
    const col = store.active_profile.special_budget;
    set_cell(col, SPECIAL_LEAP_LABELS[0], "10");
    set_special_remaining(4);
    expect(col.values[SPECIAL_LEAP_LABELS[0]]).toBe("4");
  });

  it("more remaining than owned changes nothing", () => {
    const { store } = fresh_stores();
    const col = store.active_profile.special_budget;
    set_cell(col, SPECIAL_LEAP_LABELS[0], "10");
    set_special_remaining(25);
    expect(col.values[SPECIAL_LEAP_LABELS[0]]).toBe("10");
  });
});

describe("mark_upgrade_as_done", () => {
  it("marks the right grid cell done and reruns", () => {
    const { store } = fresh_stores();
    const p = store.active_profile;
    mark_upgrade_as_done(fake_upgrade({ piece_index: 2, upgrade_index: 7 }));
    expect(p.normal_grid[2][7]).toBe(UpgradeStatus.Done);
    mark_upgrade_as_done(fake_adv({ piece_index: 5, upgrade_index: 1 }));
    expect(p.adv_grid[5][1]).toBe(UpgradeStatus.Done);
    expect(grid_change_callback).toHaveBeenCalledTimes(2);
  });
});
