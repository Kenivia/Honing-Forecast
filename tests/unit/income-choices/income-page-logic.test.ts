import { describe, expect, it } from "vitest";
import {
  chain_of,
  has_band,
  next_in_chain,
  projected,
  range_of,
  steps_of,
  WEEK_MS,
} from "@/Utils/Income";
import { raids_of, signed_gold, type Choice } from "@/Utils/Choices";
import { snap } from "./helpers";

// The page logic pulled out of Income.vue and Choices.vue.
describe("chain_of / next_in_chain", () => {
  const entry = (at: number, extra = {}, result?: object) => ({
    snapshot: snap(at, "me", {}, extra),
    job: { hash: String(at), result },
  });

  it("every snapshot oldest first, ignored only when selected, then what comes after", () => {
    const b = entry(2, { kind: "Before export" });
    const a = entry(1);
    const d = entry(4, { ignored: true });
    const c = entry(3, { kind: "After export" });
    const now = entry(6);
    const entries = [b, a, d, c];
    expect(chain_of(entries, null, [now])).toEqual([a, b, c, now]);
    expect(chain_of(entries, d.snapshot, [now])).toEqual([a, b, c, d, now]);
  });

  it("the first not done, from the latest state before it", () => {
    const chain = [entry(1, {}, {}), entry(2, {}, {}), entry(3), entry(4)];
    const states = { "1": "one" };
    expect(next_in_chain(chain, (hash) => states[hash])).toEqual({
      entry: chain[2],
      from: "one",
    });
    expect(next_in_chain(chain.slice(0, 2), () => null)).toBeNull();
    expect(next_in_chain(chain.slice(2), () => "x").from).toBeUndefined();
  });
});

describe("projected", () => {
  it("adds weeks of income per band, free taps char bound, never below 0", () => {
    const now = snap(
      1000,
      "me",
      { Red: 10 },
      {
        roster_owned: { Silver: 5 },
        tradable: { Gold: 3 },
      },
    );
    now.chars.me.special = { "Serca Special Leap": 1 };
    const out = projected(
      now,
      "me",
      "Serca Special Leap",
      [{ Red: 2.4, "Serca Special Leap": 1 }, { Silver: 100 }, { Gold: -1 }],
      2,
    );
    expect(out.at).toBe(1000 + 2 * WEEK_MS);
    expect(out.chars.me.bound).toEqual({ Red: 15 });
    expect(out.chars.me.special).toEqual({ "Serca Special Leap": 3 });
    expect(out.roster_owned).toEqual({ Silver: 205 });
    expect(out.tradable).toEqual({ Gold: 1 });
    expect(now.chars.me.bound).toEqual({ Red: 10 });
    expect(
      projected(now, "me", "x", [{}, {}, { Gold: -5 }], 1).tradable,
    ).toEqual({
      Gold: 0,
    });
  });
});

describe("range_of / steps_of", () => {
  it("from now, five weeks or more ahead", () => {
    expect(range_of(0, 2)).toEqual([0, 5 * WEEK_MS]);
    expect(range_of(0, 12)).toEqual([0, 12 * WEEK_MS]);
  });

  it("steps within runs and their days", () => {
    const day = 864e5;
    expect(steps_of([])).toBeNull();
    expect(steps_of([{ at: 0 }])).toBeNull();
    expect(
      steps_of([
        { at: 0 },
        { at: 2 * day },
        { at: 5 * day, breaks: true },
        { at: 6 * day },
      ]),
    ).toEqual({ steps: 2, days: 3 });
  });
});

describe("has_band", () => {
  it("silver is roster bound only, free taps char bound only", () => {
    expect([0, 1, 2].map((b) => has_band("Silver", b))).toEqual([
      false,
      true,
      false,
    ]);
    for (const label of ["Special Leap", "Serca Special Leap"])
      expect([0, 1, 2].map((b) => has_band(label, b))).toEqual([
        true,
        false,
        false,
      ]);
    expect([0, 1, 2].map((b) => has_band("Gold", b))).toEqual([
      true,
      true,
      true,
    ]);
  });
});

describe("Choices helpers", () => {
  it("signed_gold rounds before the sign", () => {
    expect([1499.6, -600, -0.4, 0.4, 0].map(signed_gold)).toEqual([
      "+1,500",
      "−600",
      "0",
      "0",
      "0",
    ]);
  });

  it("raids_of groups, lists difficulties and sorts newest first, ties in data order", () => {
    const choice = (group: string, difficulty?: string, item_level?: number) =>
      ({
        id: group + difficulty,
        group,
        title: "Gate 1",
        band: 0,
        options: [],
        difficulty,
        item_level,
      }) as Choice;
    const raids = raids_of([
      choice("Old", "Hard", 1600),
      choice("New", "Normal", 1700),
      choice("New", "Hard", 1720),
      choice("Plain"),
      choice("Tie", "Hard", 1600),
    ]);
    expect(raids.map((r) => r.name)).toEqual(["New", "Old", "Tie", "Plain"]);
    expect(raids[0].difficulties.map((d) => d.name)).toEqual([
      "Normal",
      "Hard",
    ]);
    expect(raids[0].members).toHaveLength(2);
  });
});
