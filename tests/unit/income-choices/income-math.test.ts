import { describe, expect, it } from "vitest";
import { fit, owned_at, resets, value_of, WEEK_MS } from "@/Utils/Income";
import { snap } from "./helpers";

// Edge cases of the estimate maths not in tests/income.test.ts.
describe("fit", () => {
  it("no points, or one time only, is no line", () => {
    expect(fit([])).toBeNull();
    expect(
      fit([
        { x: 5, y: 1 },
        { x: 5, y: 9 },
      ]),
    ).toBeNull();
  });

  it("decreases give a negative slope", () => {
    const slope = fit([
      { x: 0, y: 100 },
      { x: WEEK_MS, y: 70 },
    ]);
    expect(slope * WEEK_MS).toBeCloseTo(-30);
  });

  it("a gap is just a longer step", () => {
    const slope = fit([
      { x: 0, y: 0 },
      { x: 10 * WEEK_MS, y: 50 },
    ]);
    expect(slope * WEEK_MS).toBeCloseTo(5);
  });

  it("a run of equal times adds no slope", () => {
    const slope = fit([
      { x: 0, y: 0 },
      { x: WEEK_MS, y: 10 },
      { x: 2 * WEEK_MS, y: 500, breaks: true },
      { x: 2 * WEEK_MS, y: 700 },
    ]);
    expect(slope * WEEK_MS).toBeCloseTo(10);
  });

  it("a break on the first point starts no extra run", () => {
    const slope = fit([
      { x: 0, y: 0, breaks: true },
      { x: WEEK_MS, y: 10 },
    ]);
    expect(slope * WEEK_MS).toBeCloseTo(10);
  });

  it("every run a single point is no line", () => {
    expect(
      fit([
        { x: 0, y: 0 },
        { x: WEEK_MS, y: 10, breaks: true },
      ]),
    ).toBeNull();
  });

  it("points out of time order fit the same", () => {
    const slope = fit([
      { x: WEEK_MS, y: 10 },
      { x: 0, y: 0 },
    ]);
    expect(slope * WEEK_MS).toBeCloseTo(10);
  });
});

describe("resets", () => {
  const reset = Date.UTC(2026, 9, 22, 10);
  it("includes a reset on either end", () => {
    expect(resets(reset, reset)).toEqual([reset]);
    expect(resets(reset - 1, reset + WEEK_MS)).toEqual([
      reset,
      reset + WEEK_MS,
    ]);
  });
  it("an empty or reversed range has none", () => {
    expect(resets(reset + 1, reset + WEEK_MS - 1)).toEqual([]);
    expect(resets(reset + WEEK_MS, reset)).toEqual([]);
  });
});

describe("owned_at", () => {
  it("a character not in the snapshot owns only the roster bands", () => {
    const snapshot = snap(0, "other", { Red: 5 }, { tradable: { Red: 7 } });
    const owned = owned_at(snapshot, "me", 0);
    expect(owned.Red).toEqual([0, 0, 7]);
    expect(owned["Special Leap"]).toEqual([0, 0, 0]);
  });

  it("without a run, a chest gives its first option, options sorted", () => {
    const snapshot = snap(0, "me", {});
    snapshot.chars.me.chests = [
      {
        key: "k",
        title: "C",
        band: 0,
        count: 2,
        options: [{ Red: 10 }, { Blue: 5 }],
        obtain_all: false,
      },
    ];
    const owned = owned_at(snapshot, "me", 0);
    expect(owned.Blue[0]).toBe(10);
    expect(owned.Red[0]).toBe(0);
  });

  it("a run's gains replace the chests, free taps summed into char bound", () => {
    const snapshot = snap(0, "me", { Blue: 1 });
    snapshot.chars.me.chests = [
      {
        key: "k",
        title: "C",
        band: 0,
        count: 2,
        options: [{ Red: 10 }, { Blue: 5 }],
        obtain_all: false,
      },
    ];
    const owned = owned_at(snapshot, "me", 0, {
      gains: { Blue: [10, 0, 0], "Special Leap": [1, 2, 3] },
      gold: 0,
      leftover: 0,
    });
    expect(owned.Red[0]).toBe(0);
    expect(owned.Blue[0]).toBe(11);
    expect(owned["Special Leap"]).toEqual([6, 0, 0]);
  });
});

describe("value_of", () => {
  it("one band, an unknown label 0", () => {
    expect(value_of({ Red: [1, 2, 3] }, "Red", 1)).toBe(2);
    expect(value_of({}, "Red", 1)).toBe(0);
  });
});
