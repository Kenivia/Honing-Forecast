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

