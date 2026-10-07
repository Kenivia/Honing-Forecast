// <define:import.meta.env>
var define_import_meta_env_default = {};

// frontend/Utils/Constants.ts
var DEFAULT_ARTISAN_MULTIPLIER = 1e4 / 21500;
var WORKER_URL = define_import_meta_env_default.VITE_WORKER_URL;
var GRACE_FIRST_N = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 255];
var NON_GRACE_FIRST_N = [5, 10, 20, 30, 255];
var JOINED_ADV_JUICE = GRACE_FIRST_N.map((x) => [x, 0]).concat(
  NON_GRACE_FIRST_N.map((x) => [255, x])
);
var TIER_LABELS = ["Tier 4", "T4.5 Serca"];
var TIER_OPTIONS = TIER_LABELS.map((label, index) => ({
  label,
  value: index
}));
var MATERIAL_TABLE = {
  Red: [[0], "red", 100, 647, "Red.webp"],
  "Serca Red": [[1], "red", 100, 3494, "Serca unique/Serca Red.png"],
  Blue: [[0], "blue", 100, 10, "Blue.webp"],
  "Serca Blue": [[1], "blue", 100, 196, "Serca unique/Serca Blue.png"],
  Leaps: [[0], "leaps", 1, 20, "Leapstone.webp"],
  "Serca Leaps": [[1], "leaps", 1, 156, "Serca unique/Serca Leapstone.png"],
  Shards: [[0, 1], "shards", 1e3, 999999999, "Shard.webp"],
  Fusion: [[0], "fusion", 1, 180, "Fusion.webp"],
  "Serca Fusion": [[1], "fusion", 1, 226, "Serca unique/Serca Fusion.png"],
  Gold: [[0, 1], "gold", 1, 1, "Gold.webp"],
  Silver: [[0, 1], "silver", 1e6, 0, "Silver.webp"],
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
  "Enhanced 19-20 Armor": [[0], "books", 1, 15e3, "Enhanced Armor Book.png"],
  "Enhanced 19-20 Weapon": [[0], "books", 1, 15e3, "Enhanced Weapon Book.png"]
};
var SPECIAL_LEAP_LABELS = ["Special Leap", "Serca Special Leap"];
var CONVERTS_FROM = {
  "Serca Red": ["Red", 5],
  "Serca Blue": ["Blue", 5],
  "Serca Leaps": ["Leaps", 5],
  "Serca Fusion": ["Fusion", 5],
  "Serca Special Leap": ["Special Leap", 5]
};
var NUM_TIERS = TIER_LABELS.length;
function build_materials() {
  const out = {};
  const add = (label, tiers, color, bundle_size, fallback_price, icon) => {
    const from = CONVERTS_FROM[label];
    out[label] = {
      label,
      tiers,
      color: `--series-${color}`,
      bundle_size,
      fallback_price,
      icon: `/Icons/Materials/${icon}`,
      ...from ? { from: { label: from[0], ratio: from[1] } } : {}
    };
  };
  for (const [label, spec] of Object.entries(MATERIAL_TABLE)) {
    add(label, ...spec);
  }
  SPECIAL_LEAP_LABELS.forEach(
    (label, tier) => add(
      label,
      [tier],
      "leaps",
      1,
      0,
      tier === 0 ? "Special Leapstone.webp" : "Serca unique/Serca Special Leapstone.png"
    )
  );
  return out;
}
var MATERIALS = build_materials();
var ALL_LABELS = Array.from(
  { length: NUM_TIERS },
  (_, tier) => Object.keys(MATERIAL_TABLE).filter(
    (label) => MATERIALS[label].tiers.includes(tier)
  )
);
var TIER_MATERIALS = ALL_LABELS.map(
  (labels) => labels.map((label) => MATERIALS[label])
);
var ALL_MATERIAL_LABELS = Object.keys(MATERIALS);
var SHARED_LABELS = ALL_MATERIAL_LABELS.filter(
  (label) => MATERIALS[label].tiers.length > 1
);
var CONVERTIBLE_MATERIALS = ALL_MATERIAL_LABELS.map(
  (label) => MATERIALS[label]
).filter((mat) => mat.from !== void 0);
var FALLBACK_PRICES = Object.fromEntries(
  ALL_MATERIAL_LABELS.map((label) => [label, MATERIALS[label].fallback_price])
);
function by_label(values, tier) {
  const out = {};
  ALL_LABELS[tier].forEach((label, row) => {
    out[label] = values[row];
  });
  return out;
}
function effective_prices(prices2) {
  const out = { ...prices2 };
  for (const mat of CONVERTIBLE_MATERIALS) {
    const source = prices2[mat.from.label];
    if (source !== void 0) {
      out[mat.label] = Math.min(source * mat.from.ratio, prices2[mat.label]);
    }
  }
  return out;
}
var OTHER_ICONS = {
  Helmet: "/Icons/Equipments/Helmet.webp",
  Shoulder: "/Icons/Equipments/Shoulder.webp",
  Chest: "/Icons/Equipments/Chest.webp",
  Pants: "/Icons/Equipments/Pants.webp",
  Glove: "/Icons/Equipments/Gloves.webp",
  Weapon: "/Icons/Equipments/Weapon.webp",
  Vambrace: "/Icons/Equipments/Vambrace.png",
  "Forecast Icon": "/Icons/Forecast Icon.webp",
  Pity: "/Icons/Artist Caught.png",
  Warning: "/Icons/Warning.png"
};
var IconMap = {
  ...OTHER_ICONS,
  ...Object.fromEntries(
    Object.values(MATERIALS).map((mat) => [mat.label, mat.icon])
  )
};

// .wip/check2.ts
var OLD = [
  ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver", "Glacier's Breath", "Lava's Breath", "11-14 Armor", "11-14 Weapon", "15-18 Armor", "15-18 Weapon", "19-20 Armor", "19-20 Weapon", "Scroll 1 Armor", "Scroll 1 Weapon", "Scroll 2 Armor", "Scroll 2 Weapon", "Scroll 3 Armor", "Scroll 3 Weapon", "Scroll 4 Armor", "Scroll 4 Weapon", "Enhanced 19-20 Armor", "Enhanced 19-20 Weapon"],
  ["Serca Red", "Serca Blue", "Serca Leaps", "Shards", "Serca Fusion", "Gold", "Silver", "Glacier's Breath", "Lava's Breath"]
];
var OLD_COLORS = [
  ["red", "blue", "leaps", "shards", "fusion", "gold", "silver", "blue", "red", "books", "books", "books", "books", "books", "books", "blue", "red", "blue", "red", "blue", "red", "blue", "red", "books", "books"],
  ["red", "blue", "leaps", "shards", "fusion", "gold", "silver", "blue", "red"]
];
var OLD_BUNDLE = [100, 100, 1, 1e3, 1, 1, 1e6];
var OLD_PRICES = [
  [647, 10, 20, 999999999, 180, 1, 0, 260, 430, 298, 737, 19, 119, 2748, 3890, 150, 496, 70, 50, 1800, 1933, 3187, 2369, 15e3, 15e3],
  [3494, 196, 156, 999999999, 226, 1, 0, 260, 430]
];
var f = 0;
var eq = (n, a, b) => {
  if (JSON.stringify(a) !== JSON.stringify(b)) {
    f++;
    console.log("FAIL", n, "\n new:", JSON.stringify(a), "\n old:", JSON.stringify(b));
  } else console.log("ok  ", n);
};
eq("ALL_LABELS matches Rust row order", ALL_LABELS, OLD);
ALL_LABELS.forEach((labels, t) => {
  eq(`colors[${t}]`, TIER_MATERIALS[t].map((m) => m.color.replace("--series-", "")), OLD_COLORS[t]);
  eq(`bundle[${t}]`, TIER_MATERIALS[t].map((m) => m.bundle_size), labels.map((_, i) => OLD_BUNDLE[i] ?? 1));
  eq(`prices[${t}]`, labels.map((l) => FALLBACK_PRICES[l]), OLD_PRICES[t]);
});
eq("SHARED_LABELS", [...SHARED_LABELS].sort(), ["Glacier's Breath", "Gold", "Lava's Breath", "Shards", "Silver"].sort());
eq(
  "convertible",
  CONVERTIBLE_MATERIALS.map((m) => [m.label, m.from.label, m.from.ratio]),
  [["Serca Red", "Red", 5], ["Serca Blue", "Blue", 5], ["Serca Leaps", "Leaps", 5], ["Serca Fusion", "Fusion", 5], ["Serca Special Leap", "Special Leap", 5]]
);
eq(
  "special leap icons",
  [IconMap["Special Leap"], IconMap["Serca Special Leap"]],
  ["/Icons/Materials/Special Leapstone.webp", "/Icons/Materials/Serca unique/Serca Special Leapstone.png"]
);
eq("every tier label has a material", ALL_LABELS.flat().filter((l) => !MATERIALS[l]), []);
eq("special leaps not in tier arrays", ALL_LABELS.flat().filter((l) => l.includes("Special Leap")), []);
eq("label count", ALL_MATERIAL_LABELS.length, 31);
var prices = {};
ALL_MATERIAL_LABELS.forEach((l) => prices[l] = FALLBACK_PRICES[l]);
var eff = effective_prices(prices);
eq("Serca Red effective = min(647*5, 3494)", eff["Serca Red"], Math.min(647 * 5, 3494));
eq("Serca Blue effective = min(10*5, 196)", eff["Serca Blue"], Math.min(10 * 5, 196));
eq("shared label untouched", eff["Shards"], prices["Shards"]);
eq("tier-0 only untouched", eff["Red"], prices["Red"]);
var rows = ALL_LABELS[0].map((_, i) => i * 10);
var keyed = by_label(rows, 0);
eq("by_label round trip", ALL_LABELS[0].map((l) => keyed[l]), rows);
console.log(f === 0 ? "\nALL PASS" : `
${f} FAILURES`);
