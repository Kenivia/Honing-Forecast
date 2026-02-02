export const MATS_LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver", "Special Leap"]

export const JUICE_LABELS = [
    ["Lava's Breath", "Glacier's Breath"],
    ["11-14 Weapon", "11-14 Armor"],
    ["15-18 Weapon", "15-18 Armor"],
    ["19-20 Weapon", "19-20 Armor"],
]
export const GRAPH_COLORS = [
    "--series-red",
    "--series-blue",
    "--series-leaps",
    "--series-shards",
    "--series-oreha",
    "--series-gold",
    "--series-silver",
    "--series-red",
    "--series-books",
    "--series-books",
    "--series-books",
    "--series-blue",
    "--series-books",
    "--series-books",
    "--series-books",
]
export const OUTPUT_LABELS = ["Red", "Blue", "Leaps", "Shards", "Oreha", "Gold", "Silver", "Red juice", "Blue juice"]

export const STORAGE_KEY = "HF_UI_STATE_V1"

export const TOP_ROWS = 6
export const TOP_COLS = 25
export const BOTTOM_ROWS = 6
export const BOTTOM_COLS = 4

export const CELL_W = 28
export const CELL_H = 28
export const DEFAULT_JUICE_PRICES = [
    [400, 300],
    [20, 10],
    [200, 100],
    [4000, 2000],
]

export const DEFAULT_JUICE_LEFTOVER = [
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
]

export const IconMap: Record<string, string> = {
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
    Oreha: "/Icons/Materials/Oreha.webp",
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

    "Forecast Icon": "/forecast icon.webp",
}

export const DEFAULT_MATS_PRICES = ["1.67", "0.04", "15.0", "0.5", "100", "1.0", "0.0"]

export const DEFAULT_MATS_LEFTOVER = ["1.23", "0.03", "13.0", "0", "95", "1.0", "0.0"]

export const PIECE_NAMES = ["Helmet", "Shoulder", "Chest", "Pants", "Glove", "Weapon"]

const DEFAULT_STATE_PAIR: [boolean, number] = [false, 0]

export const RESET_UI_DEFAULTS = {
    topGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)),
    bottomGrid: Array.from({ length: BOTTOM_ROWS }, () => Array.from({ length: BOTTOM_COLS }, () => false)),
    prev_checked_arr: Array.from({ length: TOP_COLS }, () => false),
    prev_checked_arr_bottom: Array.from({ length: BOTTOM_COLS }, () => false),
    userMatsOwned: Object.fromEntries(MATS_LABELS.map((label) => [label, "0"])),
    userMatsPrices: Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, index) => [label, DEFAULT_MATS_PRICES[index]])),
    userMatsLeftover: Object.fromEntries(MATS_LABELS.map((label, index) => [label, DEFAULT_MATS_LEFTOVER[index]])),
    userWeaponJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], "0"])),
    userArmorJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], "0"])),
    userWeaponJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[0], DEFAULT_JUICE_PRICES[index]?.[0] ?? 0])),
    userArmorJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[1], DEFAULT_JUICE_PRICES[index]?.[1] ?? 0])),
    userWeaponJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[0], DEFAULT_JUICE_LEFTOVER[index]?.[0] ?? 0])),
    userArmorJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[1], DEFAULT_JUICE_LEFTOVER[index]?.[1] ?? 0])),
    desired_chance: "50",
    adv_hone_strategy: "x2 balls",
    express_event: true,
    bucketCount: "100",
    cumulativeGraph: false,
    dataSize: "100000",
    lockXAxis: false,
    lockedMins: null as number[] | null,
    lockedMaxs: null as number[] | null,
    incomeArr: Array.from({ length: 6 }, () => Array.from({ length: 7 }, () => 0)),
    minResolution: 10,
    specialState: [] as number[],
    succeededGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)),
    unlockGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)),
    stateBundleGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => [])),
    progressGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => 0)),
    evaluateAverageResult: null as any,
}

export const DEMO_UI_VALUES = {
    topGrid: Array.from({ length: TOP_ROWS }, (_, rowIndex) =>
        Array.from({ length: TOP_COLS }, (_, colIndex) => colIndex === 19 || colIndex === 20 || colIndex === 21 || (colIndex > 21 && rowIndex === 5))
    ),
    bottomGrid: Array.from({ length: BOTTOM_ROWS }, (_, piece) => Array.from({ length: BOTTOM_COLS }, (_, colIndex) => colIndex === 3 && piece < 3)),
    prev_checked_arr: Array.from({ length: TOP_COLS }, (_, colIndex) => colIndex === 19 || colIndex === 20 || colIndex === 21),
    prev_checked_arr_bottom: Array.from({ length: BOTTOM_COLS }, () => false),

    userMatsOwned: {
        Red: "631777",
        Blue: "1064398",
        Leaps: "33748",
        Shards: "12010948",
        Oreha: "25125",
        Gold: "3803792",
        Silver: "999999999",
        "Red juice": "1420",
        "Blue juice": "690",
        "Special Leap": "6767",
    },
    userMatsPrices: Object.fromEntries(MATS_LABELS.slice(0, 7).map((label, index) => [label, DEFAULT_MATS_PRICES[index]])),
    userMatsLeftover: Object.fromEntries(MATS_LABELS.map((label) => [label, "0"])),
    userWeaponJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], "0"])),
    userArmorJuiceOwned: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], "0"])),
    userWeaponJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[0], DEFAULT_JUICE_PRICES[index]?.[0] ?? 0])),
    userArmorJuicePrices: Object.fromEntries(JUICE_LABELS.map((labels, index) => [labels[1], DEFAULT_JUICE_PRICES[index]?.[1] ?? 0])),
    userWeaponJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[0], 0])),
    userArmorJuiceLeftover: Object.fromEntries(JUICE_LABELS.map((labels) => [labels[1], 0])),
    desired_chance: "50",
    minResolution: 0,
    specialState: [] as number[],
    succeededGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)),
    unlockGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => false)),
    stateBundleGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => [DEFAULT_STATE_PAIR])),
    progressGrid: Array.from({ length: TOP_ROWS }, () => Array.from({ length: TOP_COLS }, () => 0)),
}

export const DEMO_INCOME_1680_ROSTER_BOUND = [2606, 7751, 133, 0, 0, 90000, 69420]
export const DEMO_INCOME_1720_CHAR_BOUND = [13600, 28160, 594, 360279, 1500, 120000, 69420]
