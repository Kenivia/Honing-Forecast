import { describe, expect, it } from "vitest";
import {
  add_stack,
  amount_text,
  chest_gains,
  merge,
  process_chests,
  to_serca,
  type Bag,
  type Chest,
  type ManifestChest,
} from "@/Utils/Chest";

const chest = (
  title: string,
  band: number,
  count: number,
  options: Bag[],
  icons?: string[],
): ManifestChest => ({
  key: title,
  title,
  band,
  count,
  options,
  obtain_all: options.length === 1,
  ...(icons ? { icons } : {}),
});
const brief = (chests: Chest[]) =>
  chests.map((x) => [x.title, x.band, x.count, x.options]);

describe("bags", () => {
  it("merge sums by label", () => {
    expect(merge([{ Red: 1, Blue: 2 }, { Red: 3 }, {}])).toEqual({
      Red: 4,
      Blue: 2,
    });
  });

  it("to_serca converts T4 at the ratio, unrounded, into what is there", () => {
    expect(
      to_serca({
        Red: 7,
        "Serca Red": 1,
        "Special Leap": 2,
        Shards: 100,
        Gold: 5,
      }),
    ).toEqual({
      "Serca Red": 2.4,
      "Serca Special Leap": 0.4,
      Shards: 100,
      Gold: 5,
    });
  });

  it("amount_text shows at most one decimal", () => {
    expect(amount_text(1234.56)).toBe("1,234.6");
    expect(amount_text(2)).toBe("2");
  });
});

describe("process_chests", () => {
  it("a Serca chest with nothing converted keeps its band", () => {
    const out = process_chests(
      [
        chest("Trade", 2, 1, [{ "Serca Red": 5 }]),
        chest("Bound", 0, 1, [{ Red: 50 }]),
      ],
      1,
    );
    expect(brief(out)).toEqual([
      ["Trade", 2, 1, [{ "Serca Red": 5 }]],
      ["Bound", 0, 1, [{ "Serca Red": 10 }]],
    ]);
  });

  it("special leaps follow the tier", () => {
    const c = [
      chest("Leaps", 0, 1, [
        { "Special Leap": 10 },
        { "Serca Special Leap": 3 },
      ]),
    ];
    expect(brief(process_chests(c, 0))).toEqual([
      ["Leaps", 0, 1, [{ "Special Leap": 10 }]],
    ]);
    expect(brief(process_chests(c, 1))).toEqual([
      ["Leaps", 0, 1, [{ "Serca Special Leap": 3 }]],
    ]);
  });

  it("unknown labels and tradable gold go; a chest left with nothing is gone", () => {
    const out = process_chests(
      [
        chest("Odd", 1, 1, [{ "Some Accessory": 1, Gold: 10 }]),
        chest("Rich", 2, 1, [{ Gold: 1000 }]),
        chest("Serca only", 0, 1, [{ "Serca Blue": 3 }]),
      ],
      0,
    );
    expect(brief(out)).toEqual([["Odd", 1, 1, [{ Gold: 10 }]]]);
  });

  it("tradable gold is dropped by the scanned band, not the converted one", () => {
    expect(
      brief(process_chests([chest("Mix", 2, 1, [{ Red: 5, Gold: 9 }])], 1)),
    ).toEqual([["Mix", 1, 1, [{ "Serca Red": 1 }]]]);
  });

  it("dominated options go, of two alike the first stays", () => {
    const out = process_chests(
      [
        chest("Pick", 0, 1, [
          { Red: 10 },
          { Red: 10, Blue: 1 },
          { Blue: 50 },
          { Blue: 50 },
        ]),
      ],
      0,
    );
    expect(out[0].options).toEqual([{ Blue: 1, Red: 10 }, { Blue: 50 }]);
  });

  it("alike stacks are one whatever their order, titles and icons joined once", () => {
    const out = process_chests(
      [
        chest("A", 1, 2, [{ Red: 1, Blue: 2 }, { Leaps: 3 }], ["a.png"]),
        chest(
          "B",
          1,
          3,
          [{ Leaps: 3 }, { Blue: 2, Red: 1 }],
          ["b.png", "a.png"],
        ),
        chest("A", 1, 1, [{ Leaps: 3 }, { Red: 1, Blue: 2 }]),
        chest("C", 0, 1, [{ Leaps: 3 }, { Red: 1, Blue: 2 }]),
      ],
      0,
    );
    expect(out.map((x) => [x.title, x.band, x.count, x.icons])).toEqual([
      ["A / B", 1, 6, ["a.png", "b.png"]],
      ["C", 0, 1, undefined],
    ]);
    expect(out[0].key).not.toBe(out[1].key);
    expect(out[0].key).toBe(
      process_chests(
        [chest("Z", 1, 1, [{ Leaps: 3 }, { Blue: 2, Red: 1 }])],
        0,
      )[0].key,
    );
  });

  it("does not touch its input", () => {
    const input = [
      chest("A", 2, 2, [{ Red: 1 }]),
      chest("B", 2, 3, [{ Red: 1 }]),
    ];
    const copy = structuredClone(input);
    process_chests(input, 1);
    expect(input).toEqual(copy);
  });

  it("an option that costs gold does not dominate the same one for free", () => {
    const out = process_chests(
      [chest("Bonus", 0, 1, [{ Gold: -100, Red: 5 }, { Red: 5 }])],
      0,
    );
    expect(out[0].options).toEqual([{ Red: 5 }]);
  });
});

describe("add_stack", () => {
  it("kind keeps apart stacks that give the same", () => {
    const stacks = new Map<string, ManifestChest>();
    const base = { title: "T", band: 0, count: 1, options: [{ Red: 1 }] };
    add_stack(stacks, { ...base, obtain_all: true }, "true|");
    add_stack(stacks, { ...base, obtain_all: false }, "false|");
    add_stack(stacks, { ...base, obtain_all: true }, "true|");
    expect([...stacks.values()].map((s) => [s.key, s.count])).toEqual([
      ['true|0|[{"Red":1}]', 2],
      ['false|0|[{"Red":1}]', 1],
    ]);
  });
});

describe("chest_gains", () => {
  const stacks = process_chests(
    [
      chest("Pick", 1, 5, [{ "Serca Red": 1 }, { "Serca Blue": 2 }]),
      chest("Bound", 0, 3, [{ Red: 3 }]),
    ],
    1,
  );
  const [pick, bound] = stacks;

  it("no state puts everything on the first option, floored per band", () => {
    // options are sorted into the key, so "first" is not the scanned order
    expect(pick.options).toEqual([{ "Serca Blue": 2 }, { "Serca Red": 1 }]);
    expect(bound.options).toEqual([{ "Serca Red": 0.6 }]);
    expect(chest_gains(stacks)).toEqual({
      "Serca Red": [1, 0, 0],
      "Serca Blue": [0, 10, 0],
    });
  });

  it("follows a fitting state, ignores an unfitting one", () => {
    const states = (s: number[]) => new Map([[pick.key, s]]);
    expect(chest_gains(stacks, states([2, 3]))).toEqual({
      "Serca Red": [1, 3, 0],
      "Serca Blue": [0, 4, 0],
    });
    expect(chest_gains(stacks, states([2, 2]))).toEqual(chest_gains(stacks));
    expect(chest_gains(stacks, states([5]))).toEqual(chest_gains(stacks));
  });

  it("a cost floors down", () => {
    const cost = process_chests([chest("Cost", 0, 1, [{ Gold: -10.5 }])], 0);
    expect(chest_gains(cost).Gold).toEqual([-11, 0, 0]);
  });
});
