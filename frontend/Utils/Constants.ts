export const FETCH_MARKET_COOLDOWN_MS = 60 * 60 * 1000
export const DEFAULT_ARTISAN_MULTIPLIER = 0.4651
export const FLOAT_TOL = 1e-9

// export const BUCKET_COUNT = 50 // number of x values to evaluate when drawing the graphs
export const ANNOTATION_COLORS = ["--hf-graph-average-color", "--hf-graph-bound-color", "--hf-graph-roster-color", "--hf-graph-tradable-color"]
export const ANNOTATION_POSITIONS: ("top" | "middle" | "bottom" | "graph")[] = ["graph", "bottom", "middle", "top"]
export const ANNOTATION_LABELS = ["Avg", "Bound", "+Roster-Bound", "+Tradable"] // these names are tied with their css class names
export const SYNCED_LABELS = ["Shards", "Gold", "Silver", "Lava's Breath", "Glacier's Breath"]

// These must be the same as the rust side (advanced_honing/utils), will need to manually update if these change
export const GRACE_FIRST_N = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 255]
export const NON_GRACE_FIRST_N = [5, 10, 20, 40, 255]
export const JOINED_ADV_JUICE = GRACE_FIRST_N.map((x) => [x, 0]).concat(NON_GRACE_FIRST_N.map((x) => [255, x]))

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
export const T4_MATS_LABELS = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver"]
export const SERCA_MATS_LABELS = ["Serca Red", "Serca Blue", "Serca Leaps", "Shards", "Serca Fusion", "Gold", "Silver"]
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
    T4_MATS_LABELS.concat(T4_JUICE_LABELS.map((x) => x[0])).concat(T4_JUICE_LABELS.map((x) => x[1])),
    SERCA_MATS_LABELS.concat(Serca_JUICE_LABELS.map((x) => x[0])).concat(Serca_JUICE_LABELS.map((x) => x[1])),
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
    "Serca Special Leap": "/Icons/Materials/Serca unique/Serca Special Leap.png",

    "Forecast Icon": "/Icons/Forecast Icon.webp",
}
let temp = {}
for (const [label, path] of Object.entries(base_icon_map)) {
    if (!base_icon_map.hasOwnProperty("Serca " + label)) {
        temp["Serca " + label] = path
    }
}
export const IconMap = { ...temp, ...base_icon_map } as Record<string, string>

export const PIECE_NAMES = ["Helmet", "Shoulder", "Chest", "Pants", "Glove", "Weapon"]
