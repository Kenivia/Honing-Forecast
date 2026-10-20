import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import {
  adv_streaks,
  instruction_rows,
  normal_streaks,
  sort_upgrades,
  streak_lines,
} from "@/Components/Character/Instructions/InstructionUtils";
import { chance_percent, chance_to_taps } from "@/Utils/HoningUtil";
import { fake_adv, fake_juice_info, fake_upgrade } from "./fake";

// Pure logic pulled out of Instructions.vue, ActualInstructions.vue and NormalDetails.vue.

const bundle = (upgrade_arr, special_state, special_invalid_index, probs) =>
  ({
    upgrade_arr,
    special_state,
    special_invalid_index,
    latest_special_probs: probs,
  }) as any;

describe("sort_upgrades", () => {
  const helm11 = fake_upgrade({ piece_index: 0, upgrade_index: 10 });
  const helm12 = fake_upgrade({ piece_index: 0, upgrade_index: 11 });
  const shoulder = fake_upgrade({ piece_index: 1, upgrade_index: 10 });
  const adv = fake_adv({ piece_index: 0, upgrade_index: 2 });
  const b = bundle(
    [helm11, helm12, shoulder, adv],
    [2, 1, 0, 3],
    2,
    [0.3, 0.1, 0, 0],
  );

  it("puts lower levels of a free-tapped piece first, then the tail", () => {
    const rows = sort_upgrades(b);
    expect(rows.map(([u]) => [u.piece_index, u.upgrade_index])).toEqual([
      [1, 10],
      [0, 10],
      [0, 11],
      [0, 2],
    ]);
    // index in special_state and perform order
    expect(rows.map(([, pos, order]) => [pos, order])).toEqual([
      [0, 0],
      [2, 1],
      [1, 2],
      [3, 3],
    ]);
    expect(rows.map(([u]) => u.this_special_chance)).toEqual([0.3, 0, 0.1, 0]);
  });

  it("copies rather than writes the result's upgrades", () => {
    sort_upgrades(b);
    expect(helm11.this_special_chance).toBeUndefined();
  });

  it("nothing before a result", () => {
    expect(sort_upgrades(null)).toEqual([]);
  });

  it("instruction_rows splits sections, advanced by level", () => {
    const adv1 = fake_adv({ piece_index: 1, upgrade_index: 1 });
    const two = bundle([adv, adv1], [0, 1], 0, [0, 0]);
    expect(instruction_rows(two, false).map(([u]) => u.upgrade_index)).toEqual([
      1, 2,
    ]);
    expect(instruction_rows(b, true)).toHaveLength(3);
  });
});

describe("streaks", () => {
  const info = fake_juice_info();

  it("normal: equal taps grouped, the pity on its own", () => {
    const u = fake_upgrade();
    const streaks = normal_streaks(u, info);
    expect(streaks.map((s) => [s.ids, s.count, s.pity])).toEqual([
      [[0], 2, false],
      [[], 2, false],
      [[], 1, true],
    ]);
    expect(streak_lines(u, streaks, info)).toEqual([
      ["x2 taps", "until 52.50% artisan"],
      ["x2 taps", "until 100.00% artisan"],
      ["Pity", "occurs on the 5th tap"],
    ]);
  });

  it("normal: the same ids in another order are one streak", () => {
    const u = fake_upgrade({ state: [[0, 2], [2, 0], [], [], []] });
    expect(normal_streaks(u, info)[0].count).toBe(2);
  });

  const adv = (juice: number, scroll: number) =>
    adv_streaks(fake_adv({ state: [[juice], [scroll]] }), [0, 2]).map((s) => [
      s.ids,
      s.count,
      s.grace,
    ]);

  it("advanced: both, then the longer one alone", () => {
    // juice index 13 = all grace, scroll index 5 = first 5 grace
    expect(adv(13, 5)).toEqual([
      [[0, 2], 5, true],
      [[0], 255, true],
    ]);
    // juice index 8 = first 8 grace, scroll index 3 = first 3
    expect(adv(8, 3)).toEqual([
      [[0, 2], 3, true],
      [[0], 5, true],
    ]);
  });

  it("advanced: grace and non-grace, and nothing", () => {
    // juice index 15 = all grace then 10 non-grace, scroll index 18 = all of both
    expect(adv(15, 18)).toEqual([
      [[0, 2], 255, true],
      [[0, 2], 10, false],
      [[2], 255, false],
    ]);
    expect(adv(0, 0)).toEqual([[[], 255, true]]);
  });

  it("advanced lines", () => {
    const u = fake_adv();
    const lines = (streaks) => streak_lines(u, streaks, info);
    expect(
      lines([
        { ids: [0], count: 255, grace: true, pity: false },
        { ids: [0], count: 10, grace: false, pity: false },
        { ids: [0], count: 3, grace: true, pity: false },
        { ids: [], count: 255, grace: true, pity: false },
      ]),
    ).toEqual([
      ["All", "Grace"],
      ["Any 10", "non-Grace"],
      ["First 3", "Grace"],
      ["Nothing", "on Grace"],
    ]);
  });
});

describe("chance helpers", () => {
  const u = fake_upgrade({ base_chance: 0.1, extra_chance: 0.05 });

  it("chance_percent adds 10% of base per failure up to 10, plus the extra", () => {
    const got = [0, 3, 10, 15].map((n) => chance_percent(u, n));
    [15, 18, 25, 25].forEach((v, i) => expect(got[i]).toBeCloseTo(v, 9));
    expect(chance_percent(fake_upgrade({ base_chance: 0.6 }), 10)).toBe(100);
  });

  it("chance_to_taps is its inverse, rounded and clamped", () => {
    expect([15, 18, 18.4, 99, 3].map((p) => chance_to_taps(u, p))).toEqual([
      0, 3, 3, 10, 0,
    ]);
  });
});
