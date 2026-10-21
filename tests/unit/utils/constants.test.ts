import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { resolve } from "path";
import {
  ALL_BUNDLE_KEYS,
  ALL_LABELS,
  ALL_MATERIAL_LABELS,
  ALL_WORTHLESS,
  band_value_allowed,
  bundle_key,
  clamp_band_plan,
  CONVERTIBLE_MATERIALS,
  convert_is_cheaper,
  default_selected_bundles,
  DEFAULT_ARTISAN_MULTIPLIER,
  effective_unit_prices,
  FALLBACK_PRICES,
  GRACE_FIRST_N,
  IconMap,
  JOINED_ADV_JUICE,
  JUICE_RANGES,
  linked_labels,
  MATERIALS,
  NON_GRACE_FIRST_N,
  NUM_BASE_MATS,
  NUM_TIERS,
  PIECE_NAMES,
  PLUS_TIER_CONVERSION,
  SHARED_LABELS,
  SPECIAL_LEAP_LABELS,
  TIER_MATERIALS,
  unit_prices,
} from "@/Utils/Constants";

// Rust sources read as text, so a drift on either side turns this red
const core = (p: string) =>
  readFileSync(resolve(__dirname, "../../../crates/core/src", p), "utf-8");
const rust_array = (src: string, name: string) =>
  src
    .match(new RegExp(`${name}: \\[usize; \\d+\\] = \\[([^\\]]*)\\]`))[1]
    .split(",")
    .map(Number);
// the JSON files constants.rs actually embeds, by tier
const wired = [
  ...core("constants/constants.rs").matchAll(/include_str!\("\.\/(.*?)"\)/g),
].map((m) => JSON.parse(core(`constants/${m[1]}`)));

const BASE = [
  ["Red", "Serca Red"],
  ["Blue", "Serca Blue"],
  ["Leaps", "Serca Leaps"],
  ["Shards", "Shards"],
  ["Fusion", "Serca Fusion"],
  ["Gold", "Gold"],
  ["Silver", "Silver"],
];

describe("MATERIAL_TABLE projections", () => {
  it("each tier starts with the base materials in Rust's order", () => {
    expect(NUM_TIERS).toBe(2);
    for (let tier = 0; tier < NUM_TIERS; tier++) {
      expect(ALL_LABELS[tier].slice(0, NUM_BASE_MATS)).toEqual(
        BASE.map((pair) => pair[tier]),
      );
    }
  });

  it("pins the full row order of both tiers", () => {
    expect(ALL_LABELS[0].slice(NUM_BASE_MATS)).toEqual([
      "Glacier's Breath",
      "Lava's Breath",
      "11-14 Armor",
      "11-14 Weapon",
      "15-18 Armor",
      "15-18 Weapon",
      "19-20 Armor",
      "19-20 Weapon",
      "Scroll 1 Armor",
      "Scroll 1 Weapon",
      "Scroll 2 Armor",
      "Scroll 2 Weapon",
      "Scroll 3 Armor",
      "Scroll 3 Weapon",
      "Scroll 4 Armor",
      "Scroll 4 Weapon",
      "Enhanced 19-20 Armor",
      "Enhanced 19-20 Weapon",
    ]);
    expect(ALL_LABELS[1].slice(NUM_BASE_MATS)).toEqual([
      "Glacier's Breath",
      "Lava's Breath",
      "12-15 Armor",
      "12-15 Weapon",
      "16-19 Armor",
      "16-19 Weapon",
    ]);
  });

  it("special leaps are materials but no cost row", () => {
    for (const [tier, label] of SPECIAL_LEAP_LABELS.entries()) {
      expect(MATERIALS[label].tiers).toEqual([tier]);
      expect(ALL_LABELS.flat()).not.toContain(label);
      expect(ALL_MATERIAL_LABELS).toContain(label);
    }
  });

  it("TIER_MATERIALS follows ALL_LABELS", () => {
    expect(TIER_MATERIALS.map((mats) => mats.map((m) => m.label))).toEqual(
      ALL_LABELS,
    );
  });

  it("shared labels are the multi-tier ones", () => {
    expect(SHARED_LABELS).toEqual([
      "Shards",
      "Gold",
      "Silver",
      "Glacier's Breath",
      "Lava's Breath",
    ]);
    expect(
      SHARED_LABELS.every((l) => ALL_LABELS.every((t) => t.includes(l))),
    ).toBe(true);
  });
});

describe("alignment with the wired Rust constants", () => {
  it("one cost row per base material, one juice id per juice row", () => {
    expect(wired).toHaveLength(NUM_TIERS);
    wired.forEach((data, tier) => {
      expect(data.NORMAL_WEAPON_COST).toHaveLength(NUM_BASE_MATS);
      expect(data.NORMAL_ARMOR_COST).toHaveLength(NUM_BASE_MATS);
      const ids = new Set(data.JUICE_BOOKS_AVAIL.map((row) => row[0]));
      expect(ids.size).toBe(ALL_LABELS[tier].length - NUM_BASE_MATS);
    });
  });

  it("weapons cost the red row, armour the blue row", () => {
    wired.forEach((data, tier) => {
      const row = (label: string) => ALL_LABELS[tier].indexOf(label);
      const red = row(BASE[0][tier]);
      const blue = row(BASE[1][tier]);
      expect(Math.max(...data.NORMAL_WEAPON_COST[blue])).toBe(0);
      expect(Math.max(...data.NORMAL_ARMOR_COST[red])).toBe(0);
      expect(Math.min(...data.NORMAL_WEAPON_COST[red])).toBeGreaterThan(0);
      expect(Math.min(...data.NORMAL_ARMOR_COST[blue])).toBeGreaterThan(0);
    });
  });

  it("JUICE_RANGES match each juice id's levels, piece type and kind", () => {
    for (const [label, [low, high, is_weapon, is_adv]] of Object.entries(
      JUICE_RANGES,
    )) {
      const tier = MATERIALS[label].tiers[0];
      const data = wired[tier];
      const id = ALL_LABELS[tier].indexOf(label) - NUM_BASE_MATS;
      expect(id, label).toBeGreaterThan(1);
      const rows = data.JUICE_BOOKS_AVAIL.filter((r) => r[0] === id);
      expect(new Set(rows.map((r) => r[1])), label).toEqual(
        new Set([is_adv ? 1 : 0]),
      );
      expect(new Set(rows.map((r) => r[2])), label).toEqual(
        new Set([is_weapon ? 1 : 0]),
      );
      expect(Math.min(...rows.map((r) => r[3])), label).toBe(low);
      expect(Math.max(...rows.map((r) => r[3])), label).toBe(high);
    }
    // every book and scroll has a range; the two juices always show
    expect(Object.keys(JUICE_RANGES).sort()).toEqual(
      ALL_LABELS.flatMap((labels) => labels.slice(NUM_BASE_MATS + 2)).sort(),
    );
  });

  it("juice id 0 is the armour juice and 1 the weapon juice, both tiers", () => {
    wired.forEach((data) => {
      const types = (id: number) =>
        new Set(
          data.JUICE_BOOKS_AVAIL.filter((r) => r[0] === id).map((r) => r[2]),
        );
      expect(types(0).has(0) && !types(0).has(1)).toBe(true);
      expect(types(1).has(1) && !types(1).has(0)).toBe(true);
    });
  });

  it("advanced strategy tables and constants match Rust", () => {
    const utils = core("advanced_honing/utils.rs");
    expect(GRACE_FIRST_N).toEqual(rust_array(utils, "GRACE_FIRST_N"));
    expect(NON_GRACE_FIRST_N).toEqual(rust_array(utils, "NON_GRACE_FIRST_N"));
    expect(JOINED_ADV_JUICE).toHaveLength(
      GRACE_FIRST_N.length + NON_GRACE_FIRST_N.length,
    );
    expect(JOINED_ADV_JUICE.at(-1)).toEqual([255, 255]);
    expect(core("constants/constants.rs")).toContain(
      "ARTISAN_MULTIPLIER: f64 = 10000.0 / 21500.0",
    );
    expect(DEFAULT_ARTISAN_MULTIPLIER).toBe(10000.0 / 21500.0);
    expect(core("materials.rs")).toContain(
      `NUM_BASE_MATS: usize = ${NUM_BASE_MATS};`,
    );
    expect(PIECE_NAMES[5]).toBe("Weapon");
    expect(PIECE_NAMES[6]).toBe("Vambrace");
  });
});

describe("cross-tier relations", () => {
  it("CONVERTS_FROM goes one tier up at 5", () => {
    expect(
      CONVERTIBLE_MATERIALS.map((m) => [m.label, m.from.label, m.from.ratio]),
    ).toEqual([
      ["Serca Red", "Red", 5],
      ["Serca Blue", "Blue", 5],
      ["Serca Leaps", "Leaps", 5],
      ["Serca Fusion", "Fusion", 5],
      ["Serca Special Leap", "Special Leap", 5],
    ]);
    for (const m of CONVERTIBLE_MATERIALS) {
      expect(MATERIALS[m.from.label].tiers).toEqual([0]);
      expect(m.tiers).toEqual([1]);
    }
  });

  it("linked_labels pairs both directions and nothing else", () => {
    expect(linked_labels("Red")).toEqual(["Red", "Serca Red"]);
    expect(linked_labels("Serca Red")).toEqual(["Serca Red", "Red"]);
    expect(linked_labels("Special Leap")).toEqual([
      "Special Leap",
      "Serca Special Leap",
    ]);
    expect(linked_labels("Gold")).toEqual(["Gold"]);
    expect(linked_labels("Not a material")).toEqual(["Not a material"]);
  });

  it("PLUS_TIER_CONVERSION is its own inverse", () => {
    const [up, down] = PLUS_TIER_CONVERSION;
    expect(Object.keys(up)).toHaveLength(Object.keys(down).length);
    for (const [from, to] of Object.entries(up)) {
      expect(down[String(to)]).toBe(Number(from));
    }
  });
});

describe("bundles and prices", () => {
  it("one key per bundle, shards in three bags", () => {
    expect(ALL_BUNDLE_KEYS.filter((k) => k.startsWith("Shards "))).toEqual([
      "Shards x1000",
      "Shards x2000",
      "Shards x3000",
    ]);
    expect(ALL_BUNDLE_KEYS).toHaveLength(ALL_MATERIAL_LABELS.length + 2);
    expect(new Set(ALL_BUNDLE_KEYS).size).toBe(ALL_BUNDLE_KEYS.length);
    expect(Object.keys(FALLBACK_PRICES)).toEqual(ALL_BUNDLE_KEYS);
    expect(bundle_key("Silver", 1000000)).toBe("Silver x1000000");
    expect(default_selected_bundles().Shards).toBe(3000);
  });

  it("unit prices divide the selected bundle by its size", () => {
    const prices = { ...FALLBACK_PRICES, "Red x100": 650, "Shards x1000": 50 };
    const unit = unit_prices(prices, { Shards: 1000 });
    expect(unit.Red).toBe(6.5);
    expect(unit.Shards).toBe(0.05);
    expect(unit.Silver).toBe(0);
  });

  it("a missing or stale selection prices the default bundle", () => {
    const prices = { ...FALLBACK_PRICES, "Shards x3000": 300 };
    expect(unit_prices(prices, { Shards: 1500 }).Shards).toBe(0.1);
    expect(unit_prices(prices, {}).Shards).toBe(0.1);
    expect(unit_prices(prices, {}).Red).toBe(6.47);
  });

  it("effective price takes converting when cheaper, and only then", () => {
    const unit = { Red: 1, "Serca Red": 4, Blue: 1, "Serca Blue": 6 };
    const eff = effective_unit_prices(unit);
    expect(eff["Serca Red"]).toBe(4);
    expect(eff["Serca Blue"]).toBe(5);
    expect(convert_is_cheaper(MATERIALS["Serca Red"], unit)).toBe(false);
    expect(convert_is_cheaper(MATERIALS["Serca Blue"], unit)).toBe(true);
    expect(convert_is_cheaper(MATERIALS.Red, unit)).toBe(false);
  });
});

describe("band plans", () => {
  it("allows only non-decreasing plans", () => {
    const plan = ["Worthless", "TaxedSell", "Market"] as const;
    expect(band_value_allowed([...plan], 1, "Market")).toBe(true);
    expect(band_value_allowed([...plan], 1, "Worthless")).toBe(true);
    expect(band_value_allowed([...plan], 0, "Market")).toBe(false);
    expect(band_value_allowed([...plan], 2, "Worthless")).toBe(false);
  });

  it("clamp raises each band to the one before", () => {
    expect(clamp_band_plan(["Market", "Worthless", "TaxedSell"])).toEqual([
      "Market",
      "Market",
      "Market",
    ]);
    expect(clamp_band_plan(ALL_WORTHLESS)).toEqual(ALL_WORTHLESS);
    expect(ALL_WORTHLESS).toEqual(["Worthless", "Worthless", "Worthless"]);
  });
});

it("every material has an icon", () => {
  for (const label of ALL_MATERIAL_LABELS) {
    expect(IconMap[label]).toMatch(/^\/Icons\/Materials\//);
  }
  expect(IconMap.Vambrace).toBe("/Icons/Equipments/Vambrace.png");
});
