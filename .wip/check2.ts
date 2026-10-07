import { ALL_LABELS, TIER_MATERIALS, FALLBACK_PRICES, SHARED_LABELS, CONVERTIBLE_MATERIALS, IconMap, MATERIALS, ALL_MATERIAL_LABELS, effective_prices, by_label } from "@/Utils/Constants";

const OLD = [
  ["Red","Blue","Leaps","Shards","Fusion","Gold","Silver","Glacier's Breath","Lava's Breath","11-14 Armor","11-14 Weapon","15-18 Armor","15-18 Weapon","19-20 Armor","19-20 Weapon","Scroll 1 Armor","Scroll 1 Weapon","Scroll 2 Armor","Scroll 2 Weapon","Scroll 3 Armor","Scroll 3 Weapon","Scroll 4 Armor","Scroll 4 Weapon","Enhanced 19-20 Armor","Enhanced 19-20 Weapon"],
  ["Serca Red","Serca Blue","Serca Leaps","Shards","Serca Fusion","Gold","Silver","Glacier's Breath","Lava's Breath"],
];
const OLD_COLORS = [
  ["red","blue","leaps","shards","fusion","gold","silver","blue","red","books","books","books","books","books","books","blue","red","blue","red","blue","red","blue","red","books","books"],
  ["red","blue","leaps","shards","fusion","gold","silver","blue","red"],
];
const OLD_BUNDLE = [100,100,1,1000,1,1,1000000];
const OLD_PRICES = [
  [647,10,20,999999999,180,1,0,260,430,298,737,19,119,2748,3890,150,496,70,50,1800,1933,3187,2369,15000,15000],
  [3494,196,156,999999999,226,1,0,260,430],
];
let f = 0;
const eq = (n: string, a: any, b: any) => { if (JSON.stringify(a) !== JSON.stringify(b)) { f++; console.log("FAIL", n, "\n new:", JSON.stringify(a), "\n old:", JSON.stringify(b)); } else console.log("ok  ", n); };

eq("ALL_LABELS matches Rust row order", ALL_LABELS, OLD);
ALL_LABELS.forEach((labels, t) => {
  eq(`colors[${t}]`, TIER_MATERIALS[t].map((m) => m.color.replace("--series-", "")), OLD_COLORS[t]);
  eq(`bundle[${t}]`, TIER_MATERIALS[t].map((m) => m.bundle_size), labels.map((_, i) => OLD_BUNDLE[i] ?? 1));
  eq(`prices[${t}]`, labels.map((l) => FALLBACK_PRICES[l]), OLD_PRICES[t]);
});
eq("SHARED_LABELS", [...SHARED_LABELS].sort(), ["Glacier's Breath","Gold","Lava's Breath","Shards","Silver"].sort());
eq("convertible", CONVERTIBLE_MATERIALS.map((m) => [m.label, m.from.label, m.from.ratio]),
  [["Serca Red","Red",5],["Serca Blue","Blue",5],["Serca Leaps","Leaps",5],["Serca Fusion","Fusion",5],["Serca Special Leap","Special Leap",5]]);
eq("special leap icons", [IconMap["Special Leap"], IconMap["Serca Special Leap"]],
  ["/Icons/Materials/Special Leapstone.webp","/Icons/Materials/Serca unique/Serca Special Leapstone.png"]);
eq("every tier label has a material", ALL_LABELS.flat().filter((l) => !MATERIALS[l]), []);
eq("special leaps not in tier arrays", ALL_LABELS.flat().filter((l) => l.includes("Special Leap")), []);
eq("label count", ALL_MATERIAL_LABELS.length, 31);

// effective price generalises min(t4*5, serca)
const prices: Record<string, number> = {};
ALL_MATERIAL_LABELS.forEach((l) => (prices[l] = FALLBACK_PRICES[l]));
const eff = effective_prices(prices);
eq("Serca Red effective = min(647*5, 3494)", eff["Serca Red"], Math.min(647 * 5, 3494));
eq("Serca Blue effective = min(10*5, 196)", eff["Serca Blue"], Math.min(10 * 5, 196));
eq("shared label untouched", eff["Shards"], prices["Shards"]);
eq("tier-0 only untouched", eff["Red"], prices["Red"]);

// by_label is the inverse of the row order
const rows = ALL_LABELS[0].map((_, i) => i * 10);
const keyed = by_label(rows, 0);
eq("by_label round trip", ALL_LABELS[0].map((l) => keyed[l]), rows);
console.log(f === 0 ? "\nALL PASS" : `\n${f} FAILURES`);
