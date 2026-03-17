export const FLOAT_TOL = 1e-9

export const BUCKET_COUNT = 50
export const ANNOTATION_COLORS = ["--hf-graph-average-color", "--hf-graph-bound-color", "--hf-graph-roster-color", "--hf-graph-tradable-color"]
export const ANNOTATION_POSITIONS: ("top" | "middle" | "bottom" | "graph")[] = ["graph", "bottom", "middle", "top"]
export const ANNOTATION_LABELS = ["Avg", "Bound", "Roster-Bound", "Tradable"] // these names are tied with their css class names
export const SYNCED_LABELS = ["Shards", "Gold", "Silver", "Lava's Breath", "Glacier's Breath"]

export const NARROW_WIDTH = 800
export const BUDGET_NARROW_WIDTH = 1300
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
]
export const TIER_LABELS = ["Tier 4", "T4.5 Serca"]
export const TIER_OPTIONS = TIER_LABELS.map((label, index) => ({
    label,
    value: index,
}))
export const DEFAULT_TIER = 0
export const MATS_LABELS = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver"]
export const SPECIAL_LEAP_LABEL = "Special Leap"
export const T4_JUICE_LABELS = [
    ["Lava's Breath", "Glacier's Breath"],
    ["11-14 Weapon", "11-14 Armor"],
    ["15-18 Weapon", "15-18 Armor"],
    ["19-20 Weapon", "19-20 Armor"],
    ["Scroll 1 Weapon", "Scroll 1 Armor"],
    ["Scroll 2 Weapon", "Scroll 2 Armor"],
    ["Scroll 3 Weapon", "Scroll 3 Armor"],
    ["Scroll 4 Weapon", "Scroll 4 Armor"],
]

export const Serca_JUICE_LABELS = [["Lava's Breath", "Glacier's Breath"]]
export const ALL_LABELS = [
    MATS_LABELS.concat(T4_JUICE_LABELS.map((x) => x[0])).concat(T4_JUICE_LABELS.map((x) => x[1])),
    MATS_LABELS.concat(Serca_JUICE_LABELS.map((x) => x[0])).concat(Serca_JUICE_LABELS.map((x) => x[1])),
]

export const GRAPH_COLORS = [
    "--series-red",
    "--series-blue",
    "--series-leaps",
    "--series-shards",
    "--series-fusion",
    "--series-gold",
    "--series-silver",
    "--series-red",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-red",
    "--series-red",
    "--series-red",
    "--series-red",
    "--series-blue",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-blue",
    "--series-blue",
    "--series-blue",
    "--series-blue",
]
export const GRAPH_FONT_SIZE = 10
export const GRAPH_HEIGHT = 40
export const BUNDLE_SIZE = [100, 100, 1, 1000, 1, 1, 1000000].concat(
    new Array(ALL_LABELS.map((labels) => labels.length).reduce((prev, next) => Math.max(prev, next)) - 7).fill(1),
) // this is like really hacked together rn, but i doubt it'll need to be fixed

export const OUTPUT_LABELS = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver", "Red juice", "Blue juice"]

export const STORAGE_KEY = "HF_UI_STATE_V3"

export const NUM_PIECES = 6
export const NORMAL_COLS = 25
export const ADV_COLS = 4

export const CELL_W = 28
export const CELL_H = 28
export const DEFAULT_JUICE_PRICES = [
    [400, 250],
    [200, 100],
    [1200, 600],
    [7000, 4000],
    [400, 250],
    [200, 100],
    [1200, 600],
    [7000, 4000],
]

export const DEFAULT_JUICE_LEFTOVER = [
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
]

export const base_icon_map: Record<string, string> = {
    Helmet: "/Icons/Equipments/Helmet.webp",
    Shoulder: "/Icons/Equipments/Shoulder.webp",
    Chest: "/Icons/Equipments/Chest.webp",
    Pants: "/Icons/Equipments/Pants.webp",
    Glove: "/Icons/Equipments/Gloves.webp",
    Weapon: "/Icons/Equipments/Weapon.webp",
    Red: "/Icons/Materials/Red.webp",
    Blue: "/Icons/Materials/Blue.webp",
    Leaps: "/Icons/Materials/Leap.webp",
    Shards: "/Icons/Materials/Shard.webp",
    Fusion: "/Icons/Materials/Fusion.webp",
    Gold: "/Icons/Materials/Gold.webp",
    Silver: "/Icons/Materials/Silver.webp",
    "Lava's Breath": "/Icons/Materials/Lava's Breath.webp",
    "Glacier's Breath": "/Icons/Materials/Glacier's Breath.webp",
    "Special Leap": "/Icons/Materials/Special Leap.webp",
    "11-14 Armor": "/Icons/Materials/Armor Book.webp",
    "11-14 Weapon": "/Icons/Materials/Weapon Book.webp",
    "15-18 Armor": "/Icons/Materials/Armor Book.webp",
    "15-18 Weapon": "/Icons/Materials/Weapon Book.webp",
    "19-20 Armor": "/Icons/Materials/Armor Book.webp",
    "19-20 Weapon": "/Icons/Materials/Weapon Book.webp",

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
    "Serca Leaps": "/Icons/Materials/Serca unique/Serca Leaps.png",
    "Serca Fusion": "/Icons/Materials/Serca unique/Serca Fusion.png",
    "Serca Special Leap": "/Icons/Materials/Serca unique/Serca Special.png",

    "Forecast Icon": "/Icons/Forecast Icon.webp",
}
let temp = {}
for (const [label, path] of Object.entries(base_icon_map)) {
    if (!base_icon_map.hasOwnProperty("Serca " + label)) {
        temp["Serca " + label] = path
    }
}
export const IconMap = { ...temp, ...base_icon_map } as Record<string, string>

export const DEFAULT_MATS_PRICES = ["1.84", "0.04", "20", "0.6", "128", "1.0", "0.0"]

export const DEFAULT_MATS_LEFTOVER = ["0", "0", "0", "0", "0", "0", "0"]

export const DEFAULT_TOGGLES = {
    mats: Array.from({ length: MATS_LABELS.length - 1 }, () => true),
    weapon: Array.from({ length: T4_JUICE_LABELS.length }, () => true),
    juice: Array.from({ length: T4_JUICE_LABELS.length }, () => true),
}

export const PIECE_NAMES = ["Helmet", "Shoulder", "Chest", "Pants", "Glove", "Weapon"]

const DEFAULT_STATE_PAIR: [boolean, number] = [false, 0]

// export const RESET_UI_DEFAULTS = {
//     topGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => false)),
//     bottomGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: ADV_COLS }, () => false)),
//     prev_checked_arr: Array.from({ length: NORMAL_COLS }, () => false),
//     prev_checked_arr_bottom: Array.from({ length: ADV_COLS }, () => false),
//     userMatsOwned: Object.fromEntries(MATS_LABELS.map((label) => [label, "0"])),
//     userMatsPrices: Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, index) => [label, DEFAULT_MATS_PRICES[index]])),
//     userMatsLeftover: Object.fromEntries(MATS_LABELS.map((label, index) => [label, DEFAULT_MATS_LEFTOVER[index]])),
//     userWeaponJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], "0"])),
//     userArmorJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], "0"])),
//     userWeaponJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[0], DEFAULT_JUICE_PRICES[index]?.[0] ?? 0])),
//     userArmorJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[1], DEFAULT_JUICE_PRICES[index]?.[1] ?? 0])),
//     userWeaponJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[0], DEFAULT_JUICE_LEFTOVER[index]?.[0] ?? 0])),
//     userArmorJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[1], DEFAULT_JUICE_LEFTOVER[index]?.[1] ?? 0])),
//     desired_chance: "50",
//     adv_hone_strategy: "x2 grace",
//     express_event: true,
//     bucketCount: "100",
//     cumulativeGraph: true,
//     dataSize: "100000",
//     lockXAxis: false,
//     lockedMins: null as number[] | null,
//     lockedMaxs: null as number[] | null,
//     incomeArr: Array.from({ length: 6 }, () => Array.from({ length: 7 }, () => 0)),
//     minResolution: 10,
//     specialState: [] as number[],
//     succeededGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => false)),
//     unlockGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => false)),
//     stateBundleGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => [])),
//     progressGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => 0)),
//     evaluateAverageResult: null as any,
// }

// export const DEMO_UI_VALUES = {
//     topGrid: Array.from({ length: NUM_PIECES }, (_, rowIndex) =>
//         Array.from({ length: NORMAL_COLS }, (_, colIndex) => colIndex === 19 || colIndex === 20 || colIndex === 21 || (colIndex > 21 && rowIndex === 5)),
//     ),
//     bottomGrid: Array.from({ length: NUM_PIECES }, (_, piece) => Array.from({ length: ADV_COLS }, (_, colIndex) => colIndex === 3 && piece < 3)),
//     prev_checked_arr: Array.from({ length: NORMAL_COLS }, (_, colIndex) => colIndex === 19 || colIndex === 20 || colIndex === 21),
//     prev_checked_arr_bottom: Array.from({ length: ADV_COLS }, () => false),

//     userMatsOwned: {
//         Red: "631777",
//         Blue: "1064398",
//         Leaps: "33748",
//         Shards: "12010948",
//         Fusion: "25125",
//         Gold: "3803792",
//         Silver: "999999999",
//         "Red juice": "1420",
//         "Blue juice": "690",
//         "Special Leap": "6767",
//     },
//     userMatsPrices: Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, index) => [label, DEFAULT_MATS_PRICES[index]])),
//     userMatsLeftover: Object.fromEntries(MATS_LABELS.map((label) => [label, "0"])),
//     userWeaponJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], "0"])),
//     userArmorJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], "0"])),
//     userWeaponJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[0], DEFAULT_JUICE_PRICES[index]?.[0] ?? 0])),
//     userArmorJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[1], DEFAULT_JUICE_PRICES[index]?.[1] ?? 0])),
//     userWeaponJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], 0])),
//     userArmorJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], 0])),
//     desired_chance: "50",
//     minResolution: 0,
//     specialState: [] as number[],
//     succeededGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => false)),
//     unlockGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => false)),
//     stateBundleGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => [DEFAULT_STATE_PAIR])),
//     progressGrid: Array.from({ length: NUM_PIECES }, () => Array.from({ length: NORMAL_COLS }, () => 0)),
// }

// export const DEMO_INCOME_1680_ROSTER_BOUND = [2606, 7751, 133, 0, 0, 90000, 69420]
// export const DEMO_INCOME_1720_CHAR_BOUND = [13600, 28160, 594, 360279, 1500, 120000, 69420]
