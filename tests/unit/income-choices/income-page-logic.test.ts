import { describe, expect, it } from "vitest";
import {
  estimate_line,
  has_band,
  manual_line,
  MIN_SPAN,
  queue_order,
  range_of,
  steps_of,
  WEEK_MS,
} from "@/Utils/Income";
import { raids_of, signed_gold, type Choice } from "@/Utils/Choices";
import { snap } from "./helpers";

// The page logic pulled out of Income.vue and Choices.vue.
describe("queue_order", () => {
  const entry = (at: number, extra = {}, result?: object) => ({
    snapshot: snap(at, "me", {}, extra),
    job: { result },
  });

  it("selected, now, used newest first, then Before export; each once, done ones out", () => {
    const a = entry(1);
    const b = entry(2, { kind: "Before export" });
    const c = entry(3, { kind: "After export" });
    const d = entry(4, { ignored: true });
    const e = entry(5, {}, { gold: 0 });
    const now = entry(6);
    const entries = [a, b, c, d, e];
    expect(queue_order(entries, null, null)).toEqual([c, a, b]);
    expect(queue_order(entries, a.snapshot, now)).toEqual([a, now, c, b]);
    expect(queue_order(entries, d.snapshot, null)).toEqual([d, c, a, b]);
  });
});

describe("range_of / steps_of", () => {
  it("centred on now, at least MIN_SPAN either side", () => {
    expect(range_of(undefined, 100)).toEqual([100 - MIN_SPAN, 100 + MIN_SPAN]);
    const old = 100 - 2 * MIN_SPAN;
    expect(range_of(old, 100)).toEqual([old, 100 + 2 * MIN_SPAN]);
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

describe("lines", () => {
  it("a run each, apart by a null, the last on to the end", () => {
    const line = estimate_line(
      [
        { x: 0, y: 0 },
        { x: WEEK_MS, y: 10 },
        { x: 2 * WEEK_MS, y: 100, breaks: true },
        { x: 3 * WEEK_MS, y: 110 },
      ],
      4 * WEEK_MS,
    );
    expect(line.map((p) => [p.x / WEEK_MS, p.y])).toEqual([
      [0, 0],
      [1, 10],
      [1, null],
      [2, 100],
      [4, 120],
    ]);
  });

  it("no fit, no line", () => {
    expect(estimate_line([{ x: 0, y: 1 }], WEEK_MS)).toEqual([]);
  });

  it("manual: from now's value at the typed rate", () => {
    expect(manual_line(50, 7, 0, 2 * WEEK_MS)).toEqual([
      { x: 0, y: 50 },
      { x: 2 * WEEK_MS, y: 64 },
    ]);
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
