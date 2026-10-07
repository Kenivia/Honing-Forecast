export const DEFAULT_ARTISAN_MULTIPLIER = 10000.0 / 21500.0;
export const FLOAT_TOL = 1e-9;

export const WORKER_URL = import.meta.env.VITE_WORKER_URL;

// export const BUCKET_COUNT = 50 // number of x values to evaluate when drawing the graphs
export const ANNOTATION_COLORS = [
  "--average",
  "--bound",
  "--roster",
  "--tradable",
];
export const ANNOTATION_POSITIONS: ("top" | "middle" | "bottom" | "graph")[] = [
  "graph",
  "bottom",
  "middle",
  "top",
];
export const SYNCED_LABELS = [
  "Shards",
  "Gold",
  "Silver",
  "Lava's Breath",
  "Glacier's Breath",
];

export const ANNOTATION_LABELS = [
  "Average",
  "Bound",
  "+Roster-Bound",
  "+Tradable",
];
export const BUTTON_LABELS = [
  "Show Average cost",
  "Show Bound owned",
  "Show Bound+Roster bound",
  "Show Bound+Roster+Trade",
];
export const CSS_NAMES = ["avg", "bound", "roster-bound", "tradable"];

// These must be the same as the rust side (advanced_honing/utils), will need to manually update if these change
export const GRACE_FIRST_N = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 255];
export const NON_GRACE_FIRST_N = [5, 10, 20, 30, 255];
export const JOINED_ADV_JUICE = GRACE_FIRST_N.map((x) => [x, 0]).concat(
  NON_GRACE_FIRST_N.map((x) => [255, x]),
);

export const NARROW_WIDTH = 900;
export const BUDGET_NARROW_WIDTH = 1300;
export const PLUS_TIER_CONVERSION = [
  // index corresponds to the old tier
  // this only really works for 2 tiers
  {
    // note this is the +n number not the upgrade index (= upgrade_index + 1)
    "20": 11,
    "21": 12,
    "22": 13,
    "23": 14,
    "24": 16,
    "25": 18,
  },
  {
    "11": 20,
    "12": 21,
    "13": 22,
    "14": 23,
    "16": 24,
    "18": 25,
  },
];
export const TIER_LABELS = ["Tier 4", "T4.5 Serca"];
export const TIER_OPTIONS = TIER_LABELS.map((label, index) => ({
  label,
  value: index,
}));
export const DEFAULT_TIER = 0;

// ============================================================================
// Materials
//
// MATERIAL_TABLE is the single source of truth for everything about a material
// except its row order. Keyed by label so a row can never be paired with the
// wrong colour, bundle size, price or icon.
// ============================================================================

// [graph colour, market bundle size, fallback price per bundle, icon file]
const MATERIAL_TABLE: Record<string, [string, number, number, string]> = {
  Red: ["red", 100, 647, "Red.webp"],
  Blue: ["blue", 100, 10, "Blue.webp"],
  Leaps: ["leaps", 1, 20, "Leapstone.webp"],
  Shards: ["shards", 1000, 999999999, "Shard.webp"],
  Fusion: ["fusion", 1, 180, "Fusion.webp"],
  Gold: ["gold", 1, 1, "Gold.webp"],
  Silver: ["silver", 1000000, 0, "Silver.webp"],
  "Serca Red": ["red", 100, 3494, "Serca unique/Serca Red.png"],
  "Serca Blue": ["blue", 100, 196, "Serca unique/Serca Blue.png"],
  "Serca Leaps": ["leaps", 1, 156, "Serca unique/Serca Leapstone.png"],
  "Serca Fusion": ["fusion", 1, 226, "Serca unique/Serca Fusion.png"],
  "Glacier's Breath": ["blue", 1, 260, "Glacier's Breath.webp"],
  "Lava's Breath": ["red", 1, 430, "Lava's Breath.webp"],
  "11-14 Armor": ["books", 1, 298, "Armor Book.webp"],
  "11-14 Weapon": ["books", 1, 737, "Weapon Book.webp"],
  "15-18 Armor": ["books", 1, 19, "Armor Book.webp"],
  "15-18 Weapon": ["books", 1, 119, "Weapon Book.webp"],
  "19-20 Armor": ["books", 1, 2748, "Armor Book.webp"],
  "19-20 Weapon": ["books", 1, 3890, "Weapon Book.webp"],
  "Enhanced 19-20 Armor": ["books", 1, 15000, "Enhanced Armor Book.png"],
  "Enhanced 19-20 Weapon": ["books", 1, 15000, "Enhanced Weapon Book.png"],
  "Scroll 1 Armor": ["blue", 1, 150, "Scroll 1 Armor.png"],
  "Scroll 1 Weapon": ["red", 1, 496, "Scroll 1 Weapon.png"],
  "Scroll 2 Armor": ["blue", 1, 70, "Scroll 2 Armor.png"],
  "Scroll 2 Weapon": ["red", 1, 50, "Scroll 2 Weapon.png"],
  "Scroll 3 Armor": ["blue", 1, 1800, "Scroll 3 Armor.png"],
  "Scroll 3 Weapon": ["red", 1, 1933, "Scroll 3 Weapon.png"],
  "Scroll 4 Armor": ["blue", 1, 3187, "Scroll 4 Armor.png"],
  "Scroll 4 Weapon": ["red", 1, 2369, "Scroll 4 Weapon.png"],
};

export interface Material {
  label: string;
  color: string;
  bundle_size: number;
  fallback_price: number;
  icon: string;
}

export const MATERIALS: Record<string, Material> = Object.fromEntries(
  Object.entries(MATERIAL_TABLE).map(
    ([label, [color, bundle_size, fallback_price, icon]]) => [
      label,
      {
        label,
        color: `--series-${color}`,
        bundle_size,
        fallback_price,
        icon: `/Icons/Materials/${icon}`,
      },
    ],
  ),
);

// Row order per tier. Rust indexes material arrays by this, so it must match the
// tier's constants JSON on the Rust side. Base materials first, then juices.
export const NUM_BASE_MATS = 7;
export const SPECIAL_LEAP_LABEL = "Special Leap";

const T4_JUICE_LABELS = [
  ["Glacier's Breath", "Lava's Breath"],
  ["11-14 Armor", "11-14 Weapon"],
  ["15-18 Armor", "15-18 Weapon"],
  ["19-20 Armor", "19-20 Weapon"],
  ["Scroll 1 Armor", "Scroll 1 Weapon"],
  ["Scroll 2 Armor", "Scroll 2 Weapon"],
  ["Scroll 3 Armor", "Scroll 3 Weapon"],
  ["Scroll 4 Armor", "Scroll 4 Weapon"],
  ["Enhanced 19-20 Armor", "Enhanced 19-20 Weapon"],
];
const SERCA_JUICE_LABELS = [["Glacier's Breath", "Lava's Breath"]];

export const ALL_LABELS = [
  ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver"].concat(
    T4_JUICE_LABELS.flat(),
  ),
  [
    "Serca Red",
    "Serca Blue",
    "Serca Leaps",
    "Shards",
    "Serca Fusion",
    "Gold",
    "Silver",
  ].concat(SERCA_JUICE_LABELS.flat()),
];

// The same rows, carrying their attributes, for anything that iterates a tier.
export const TIER_MATERIALS: Material[][] = ALL_LABELS.map((labels) =>
  labels.map((label) => MATERIALS[label]),
);

export const FALLBACK_PRICES: number[][] = TIER_MATERIALS.map((mats) =>
  mats.map((mat) => mat.fallback_price),
);

// A juice row only shows when the character wants an upgrade in its range:
// [low, high, is_weapon, is_adv], inclusive.
export const JUICE_RANGES: Record<string, [number, number, boolean, boolean]> =
  {
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

// Serca rows that share their value with a T4 row, so the two stay in sync.
export const SERCA_SYNC_MAP: { serca_index: number; T4_index: number }[] =
  ALL_LABELS[1]
    .map((label, serca_index) => {
      const T4_index = ALL_LABELS[0].indexOf(label);
      return SYNCED_LABELS.includes(label) && T4_index !== -1
        ? { serca_index, T4_index }
        : null;
    })
    .filter((x) => x !== null);
export const SERCA_TO_T4_INDICES: Record<number, number> = Object.fromEntries(
  SERCA_SYNC_MAP.map(({ serca_index, T4_index }) => [serca_index, T4_index]),
);

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

// Icons for everything that is not a material row.
const OTHER_ICONS: Record<string, string> = {
  Helmet: "/Icons/Equipments/Helmet.webp",
  Shoulder: "/Icons/Equipments/Shoulder.webp",
  Chest: "/Icons/Equipments/Chest.webp",
  Pants: "/Icons/Equipments/Pants.webp",
  Glove: "/Icons/Equipments/Gloves.webp",
  Weapon: "/Icons/Equipments/Weapon.webp",
  Vambrace: "/Icons/Equipments/Vambrace.png",
  "Special Leap": "/Icons/Materials/Special Leapstone.webp",
  "Serca Special Leap":
    "/Icons/Materials/Serca unique/Serca Special Leapstone.png",
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
