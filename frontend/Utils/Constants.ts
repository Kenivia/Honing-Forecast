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
export const T4_MATS_LABELS = [
  "Red",
  "Blue",
  "Leaps",
  "Shards",
  "Fusion",
  "Gold",
  "Silver",
];
export const SERCA_MATS_LABELS = [
  "Serca Red",
  "Serca Blue",
  "Serca Leaps",
  "Shards",
  "Serca Fusion",
  "Gold",
  "Silver",
];
export const SPECIAL_LEAP_LABEL = "Special Leap";
export const T4_JUICE_LABELS = [
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

export const Serca_JUICE_LABELS = [["Glacier's Breath", "Lava's Breath"]];
export const ALL_LABELS = [
  T4_MATS_LABELS.concat(T4_JUICE_LABELS.flatMap((x) => [x[0], x[1]])),
  SERCA_MATS_LABELS.concat(Serca_JUICE_LABELS.flatMap((x) => [x[0], x[1]])),
];

// const old = [
//   "Red",
//   "Blue",
//   "Leaps",
//   "Shards",
//   "Fusion",
//   "Gold",
//   "Silver",
//   "Lava's Breath",
//   "11-14 Weapon",
//   "15-18 Weapon",
//   "19-20 Weapon",
//   "Scroll 1 Weapon",
//   "Scroll 2 Weapon",
//   "Scroll 3 Weapon",
//   "Scroll 4 Weapon",
//   "Glacier's Breath",
//   "11-14 Armor",
//   "15-18 Armor",
//   "19-20 Armor",
//   "Scroll 1 Armor",
//   "Scroll 2 Armor",
//   "Scroll 3 Armor",
//   "Scroll 4 Armor",
// ];
// function getIndices(arr_1: string[], arr_2: string[]): number[] {
//   const indexMap = new Map<string, number>();
//   for (let i = 0; i < arr_1.length; i++) {
//     indexMap.set(arr_1[i], i);
//   }
//   return arr_2.map((val) => indexMap.get(val)!);
// }
// console.log(getIndices(old, ALL_LABELS[0]));
export const GRAPH_COLORS = [
  [
    "--series-red",
    "--series-blue",
    "--series-leaps",
    "--series-shards",
    "--series-fusion",
    "--series-gold",
    "--series-silver",
    "--series-blue",
    "--series-red",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-blue",
    "--series-red",
    "--series-blue",
    "--series-red",
    "--series-blue",
    "--series-red",
    "--series-blue",
    "--series-red",
    "--series-books",
    "--series-books",
  ],
  [
    "--series-red",
    "--series-blue",
    "--series-leaps",
    "--series-shards",
    "--series-fusion",
    "--series-gold",
    "--series-silver",
    "--series-blue",
    "--series-red",
  ],
];
export const GRAPH_FONT_SIZE = 10;
export const GRAPH_HEIGHT = 40;
export const BUNDLE_SIZE = [100, 100, 1, 1000, 1, 1, 1000000].concat(
  new Array(
    ALL_LABELS.map((labels) => labels.length).reduce((prev, next) =>
      Math.max(prev, next),
    ) - 7,
  ).fill(1),
); // this is like really hacked together rn, but i doubt it'll need to be fixed


export const NUM_PIECES = 7;
export const NUM_ADV_PIECES = 6;
export const NORMAL_COLS = 25;
export const ADV_COLS = 4;

export const base_icon_map: Record<string, string> = {
  Helmet: "/Icons/Equipments/Helmet.webp",
  Shoulder: "/Icons/Equipments/Shoulder.webp",
  Chest: "/Icons/Equipments/Chest.webp",
  Pants: "/Icons/Equipments/Pants.webp",
  Glove: "/Icons/Equipments/Gloves.webp",
  Weapon: "/Icons/Equipments/Weapon.webp",
  Vambrace: "/Icons/Equipments/Vambrace.png",

  Red: "/Icons/Materials/Red.webp",
  Blue: "/Icons/Materials/Blue.webp",
  Leaps: "/Icons/Materials/Leapstone.webp",
  Shards: "/Icons/Materials/Shard.webp",
  Fusion: "/Icons/Materials/Fusion.webp",
  Gold: "/Icons/Materials/Gold.webp",
  Silver: "/Icons/Materials/Silver.webp",
  "Lava's Breath": "/Icons/Materials/Lava's Breath.webp",
  "Glacier's Breath": "/Icons/Materials/Glacier's Breath.webp",
  "Special Leap": "/Icons/Materials/Special Leapstone.webp",
  "11-14 Armor": "/Icons/Materials/Armor Book.webp",
  "11-14 Weapon": "/Icons/Materials/Weapon Book.webp",
  "15-18 Armor": "/Icons/Materials/Armor Book.webp",
  "15-18 Weapon": "/Icons/Materials/Weapon Book.webp",
  "19-20 Armor": "/Icons/Materials/Armor Book.webp",
  "19-20 Weapon": "/Icons/Materials/Weapon Book.webp",
  "Enhanced 19-20 Armor": "/Icons/Materials/Enhanced Armor Book.png",
  "Enhanced 19-20 Weapon": "/Icons/Materials/Enhanced Weapon Book.png",

  "Scroll 1 Weapon": "/Icons/Materials/Scroll 1 Weapon.png",
  "Scroll 1 Armor": "/Icons/Materials/Scroll 1 Armor.png",
  "Scroll 2 Weapon": "/Icons/Materials/Scroll 2 Weapon.png",
  "Scroll 2 Armor": "/Icons/Materials/Scroll 2 Armor.png",
  "Scroll 3 Weapon": "/Icons/Materials/Scroll 3 Weapon.png",
  "Scroll 3 Armor": "/Icons/Materials/Scroll 3 Armor.png",
  "Scroll 4 Weapon": "/Icons/Materials/Scroll 4 Weapon.png",
  "Scroll 4 Armor": "/Icons/Materials/Scroll 4 Armor.png",

  "Serca Red": "/Icons/Materials/Serca unique/Serca Red.png",
  "Serca Blue": "/Icons/Materials/Serca unique/Serca Blue.png",
  "Serca Leaps": "/Icons/Materials/Serca unique/Serca Leapstone.png",
  "Serca Fusion": "/Icons/Materials/Serca unique/Serca Fusion.png",
  "Serca Special Leap":
    "/Icons/Materials/Serca unique/Serca Special Leapstone.png",

  "Forecast Icon": "/Icons/Forecast Icon.webp",
  Pity: "/Icons/Artist Caught.png",

  Warning: "/Icons/Warning.png",
};
let temp = {};
for (const [label, path] of Object.entries(base_icon_map)) {
  if (!Object.hasOwn(base_icon_map, "Serca " + label)) {
    temp["Serca " + label] = path;
  }
}
export const IconMap = { ...temp, ...base_icon_map } as Record<string, string>;

export const PIECE_NAMES = [
  "Helmet",
  "Shoulder",
  "Chest",
  "Pants",
  "Glove",
  "Weapon",
  "Vambrace",
];

export const SERCA_SYNC_MAP: { serca_index: number; T4_index: number }[] =
  ALL_LABELS[1]
    .map((label, serca_index) => {
      if (!SYNCED_LABELS.includes(label)) return null;
      const T4_index = ALL_LABELS[0].findIndex(
        (x) => x === label.replace("Serca ", ""),
      );
      return T4_index === -1 ? null : { serca_index, T4_index };
    })
    .filter((x) => x !== null);
export const SERCA_TO_T4_INDICES: Record<number, number> = Object.fromEntries(
  SERCA_SYNC_MAP.map(({ serca_index, T4_index }) => [serca_index, T4_index]),
);

export const FALLBACK_PRICES = [
  [
    647, 10, 20, 999999999, 180, 1, 0, 260, 430, 298, 737, 19, 119, 2748, 3890,
    150, 496, 70, 50, 1800, 1933, 3187, 2369, 15000, 15000,
  ],
  [3494, 196, 156, 999999999, 226, 1, 0, 260, 430],
];
