// ============================================================================
// Materials
//
// MATERIAL_TABLE is the single source of truth for every material. Keyed by label, so
// an attribute can never be paired with the wrong material.
//
// THE ORDER OF THIS TABLE IS A CONTRACT. Filtering it by tier produces that tier's
// material row order, which Rust indexes its cost arrays by, so entries are interleaved
// such that each tier's projection matches its constants JSON on the Rust side.
//
// `tiers` is which tiers the material exists in. A label listed under several tiers is
// ONE material with ONE stored value, so nothing is ever synced between tiers.
// ============================================================================

// [tiers, graph colour, market bundle size, fallback price per bundle, icon file]
const MATERIAL_TABLE: Record<
  string,
  [number[], string, number, number, string]
> = {
  Red: [[0], "red", 100, 647, "Red.webp"],
  "Serca Red": [[1], "red", 100, 3494, "Serca unique/Serca Red.png"],
  Blue: [[0], "blue", 100, 10, "Blue.webp"],
  "Serca Blue": [[1], "blue", 100, 196, "Serca unique/Serca Blue.png"],
  Leaps: [[0], "leaps", 1, 20, "Leapstone.webp"],
  "Serca Leaps": [[1], "leaps", 1, 156, "Serca unique/Serca Leapstone.png"],
  Shards: [[0, 1], "shards", 1000, 999999999, "Shard.webp"],
  Fusion: [[0], "fusion", 1, 180, "Fusion.webp"],
  "Serca Fusion": [[1], "fusion", 1, 226, "Serca unique/Serca Fusion.png"],
  Gold: [[0, 1], "gold", 1, 1, "Gold.webp"],
  Silver: [[0, 1], "silver", 1000000, 0, "Silver.webp"],
  "Glacier's Breath": [[0, 1], "blue", 1, 260, "Glacier's Breath.webp"],
  "Lava's Breath": [[0, 1], "red", 1, 430, "Lava's Breath.webp"],
  "11-14 Armor": [[0], "books", 1, 298, "Armor Book.webp"],
  "11-14 Weapon": [[0], "books", 1, 737, "Weapon Book.webp"],
  "15-18 Armor": [[0], "books", 1, 19, "Armor Book.webp"],
  "15-18 Weapon": [[0], "books", 1, 119, "Weapon Book.webp"],
  "19-20 Armor": [[0], "books", 1, 2748, "Armor Book.webp"],
  "19-20 Weapon": [[0], "books", 1, 3890, "Weapon Book.webp"],
  "Scroll 1 Armor": [[0], "blue", 1, 150, "Scroll 1 Armor.png"],
  "Scroll 1 Weapon": [[0], "red", 1, 496, "Scroll 1 Weapon.png"],
  "Scroll 2 Armor": [[0], "blue", 1, 70, "Scroll 2 Armor.png"],
  "Scroll 2 Weapon": [[0], "red", 1, 50, "Scroll 2 Weapon.png"],
  "Scroll 3 Armor": [[0], "blue", 1, 1800, "Scroll 3 Armor.png"],
  "Scroll 3 Weapon": [[0], "red", 1, 1933, "Scroll 3 Weapon.png"],
  "Scroll 4 Armor": [[0], "blue", 1, 3187, "Scroll 4 Armor.png"],
  "Scroll 4 Weapon": [[0], "red", 1, 2369, "Scroll 4 Weapon.png"],
  "Enhanced 19-20 Armor": [[0], "books", 1, 15000, "Enhanced Armor Book.png"],
  "Enhanced 19-20 Weapon": [[0], "books", 1, 15000, "Enhanced Weapon Book.png"],
};

// Special leaps are budgeted through their own input rather than as a row in the cost
// array, so they are kept out of MATERIAL_TABLE. Indexed by tier.
export const SPECIAL_LEAP_LABELS = ["Special Leap", "Serca Special Leap"];

// [source label, source units per unit of this]. The only cross-tier relation besides a
// shared label: a material can be made from a lower tier's material at a fixed rate.
const CONVERTS_FROM: Record<string, [string, number]> = {
  "Serca Red": ["Red", 5],
  "Serca Blue": ["Blue", 5],
  "Serca Leaps": ["Leaps", 5],
  "Serca Fusion": ["Fusion", 5],
  "Serca Special Leap": ["Special Leap", 5],
};

export interface Material {
  label: string;
  tiers: number[];
  color: string;
  bundle_size: number;
  fallback_price: number;
  icon: string;
  // set when the material can be made from a lower tier's material
  from?: { label: string; ratio: number };
}

// Rust's cost arrays start with this many base materials, before the juices.
export const NUM_BASE_MATS = 7;
export const NUM_TIERS = TIER_LABELS.length;

function build_materials(): Record<string, Material> {
  const out: Record<string, Material> = {};
  const add = (
    label: string,
    tiers: number[],
    color: string,
    bundle_size: number,
    fallback_price: number,
    icon: string,
  ) => {
    const from = CONVERTS_FROM[label];
    out[label] = {
      label,
      tiers,
      color: `--series-${color}`,
      bundle_size,
      fallback_price,
      icon: `/Icons/Materials/${icon}`,
      ...(from ? { from: { label: from[0], ratio: from[1] } } : {}),
    };
  };
  for (const [label, spec] of Object.entries(MATERIAL_TABLE)) {
    add(label, ...spec);
  }
  // special leaps are materials too, they just live outside the tier cost arrays
  SPECIAL_LEAP_LABELS.forEach((label, tier) =>
    add(
      label,
      [tier],
      "leaps",
      1,
      0,
      tier === 0
        ? "Special Leapstone.webp"
        : "Serca unique/Serca Special Leapstone.png",
    ),
  );
  return out;
}

export const MATERIALS: Record<string, Material> = build_materials();

// Row order per tier, which Rust shares. Special leaps are excluded on purpose.
export const ALL_LABELS: string[][] = Array.from(
  { length: NUM_TIERS },
  (_, tier) =>
    Object.keys(MATERIAL_TABLE).filter((label) =>
      MATERIALS[label].tiers.includes(tier),
    ),
);

export const TIER_MATERIALS: Material[][] = ALL_LABELS.map((labels) =>
  labels.map((label) => MATERIALS[label]),
);

// Every label that holds a stored value, in table order. One column covers all of them,
// so a material shared between tiers has exactly one entry and needs no syncing.
export const ALL_MATERIAL_LABELS: string[] = Object.keys(MATERIALS);

// Materials that exist in more than one tier, so their stored value is shared.
export const SHARED_LABELS: string[] = ALL_MATERIAL_LABELS.filter(
  (label) => MATERIALS[label].tiers.length > 1,
);

// Materials that can be made from a lower tier.
export const CONVERTIBLE_MATERIALS: Material[] = ALL_MATERIAL_LABELS.map(
  (label) => MATERIALS[label],
).filter((mat) => mat.from !== undefined);

export const FALLBACK_PRICES: Record<string, number> = Object.fromEntries(
  ALL_MATERIAL_LABELS.map((label) => [label, MATERIALS[label].fallback_price]),
);

// Rust returns material arrays indexed by row. This is the only place a row becomes a
// label; everything downstream of it is keyed.
export function by_label<T>(values: T[], tier: number): Record<string, T> {
  const out: Record<string, T> = {};
  ALL_LABELS[tier].forEach((label, row) => {
    out[label] = values[row];
  });
  return out;
}

// A convertible material can be bought outright or made from its source, whichever is
// cheaper. Prices are per bundle, and one source bundle makes `ratio` of this material.
export function effective_prices(
  prices: Record<string, number>,
): Record<string, number> {
  const out = { ...prices };
  for (const mat of CONVERTIBLE_MATERIALS) {
    const source = prices[mat.from.label];
    if (source !== undefined) {
      out[mat.label] = Math.min(source * mat.from.ratio, prices[mat.label]);
    }
  }
  return out;
}

// True when making the material from its source costs less than buying it outright.
export function convert_is_cheaper(
  mat: Material,
  prices: Record<string, number>,
): boolean {
  return (
    mat.from !== undefined &&
    prices[mat.from.label] * mat.from.ratio < prices[mat.label]
  );
}

// A juice row only shows when the character wants an upgrade in its range:
// [low, high, is_weapon, is_adv], inclusive.
export const JUICE_RANGES: Record<
  string,
  [number, number, boolean, boolean]
> = {
  "11-14 Armor": [11, 14, false, false],
  "11-14 Weapon": [11, 14, true, false],
  "15-18 Armor": [15, 18, false, false],
  "15-18 Weapon": [15, 18, true, false],
  "19-20 Armor": [19, 20, false, false],
  "19-20 Weapon": [19, 20, true, false],
  "Enhanced 19-20 Armor": [19, 20, false, false],
  "Enhanced 19-20 Weapon": [19, 20, true, false],
  "Scroll 1 Armor": [1, 1, false, true],
  "Scroll 1 Weapon": [1, 1, true, true],
  "Scroll 2 Armor": [2, 2, false, true],
  "Scroll 2 Weapon": [2, 2, true, true],
  "Scroll 3 Armor": [3, 3, false, true],
  "Scroll 3 Weapon": [3, 3, true, true],
  "Scroll 4 Armor": [4, 4, false, true],
  "Scroll 4 Weapon": [4, 4, true, true],
};

export const GRAPH_FONT_SIZE = 10;
export const GRAPH_HEIGHT = 40;

export const NUM_PIECES = 7;
export const NUM_ADV_PIECES = 6;
export const NORMAL_COLS = 25;
export const ADV_COLS = 4;

export const PIECE_NAMES = [
  "Helmet",
  "Shoulder",
  "Chest",
  "Pants",
  "Glove",
  "Weapon",
  "Vambrace",
];

// Icons for everything that is not a material.
const OTHER_ICONS: Record<string, string> = {
  Helmet: "/Icons/Equipments/Helmet.webp",
  Shoulder: "/Icons/Equipments/Shoulder.webp",
  Chest: "/Icons/Equipments/Chest.webp",
  Pants: "/Icons/Equipments/Pants.webp",
  Glove: "/Icons/Equipments/Gloves.webp",
  Weapon: "/Icons/Equipments/Weapon.webp",
  Vambrace: "/Icons/Equipments/Vambrace.png",
  "Forecast Icon": "/Icons/Forecast Icon.webp",
  Pity: "/Icons/Artist Caught.png",
  Warning: "/Icons/Warning.png",
};

export const IconMap: Record<string, string> = {
  ...OTHER_ICONS,
  ...Object.fromEntries(
    Object.values(MATERIALS).map((mat) => [mat.label, mat.icon]),
  ),
};
