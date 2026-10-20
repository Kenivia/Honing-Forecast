import {
  bundle_key,
  cheapest_bundles,
  MATERIALS,
  WORKER_URL,
} from "./Constants";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { input_column_to_num } from "./InputColumn";
import { useRuntimeStore } from "@/Stores/RuntimeState";

const FETCH_MARKET_COOLDOWN_MS = 60 * 60 * 1000;
export type MarketRegions = "nae" | "euc" | "Custom";
// Custom is user-entered and is never fetched.
export const FETCHABLE_REGIONS: MarketRegions[] = ["nae", "euc"];
export async function fetch_market_data(region: MarketRegions): Promise<any> {
  const body = {
    region_slug: region.toLowerCase(),
    item_slugs: Object.keys(ITEM_SLUG_TO_LABEL),
  };
  const response = await fetch(WORKER_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return await response.json();
}

// A material with several bundle sizes has one market slug per bundle, so the entry
// carries the size. Single-bundle materials just name the material.
function slug_bundle_key(entry: string | [string, number]): string {
  if (Array.isArray(entry)) {
    return bundle_key(entry[0], entry[1]);
  }
  const mat = MATERIALS[entry];
  return mat === undefined ? "" : bundle_key(entry, mat.bundle_sizes[0]);
}

// The price of every bundle the response lists. Throws on a body that is not a list.
export function parse_response(response: any): Record<string, number> {
  const prices: Record<string, number> = {};
  for (const { item_slug, price } of response) {
    const entry = ITEM_SLUG_TO_LABEL[item_slug];
    const value = parseInt(price);
    if (entry !== undefined && Number.isFinite(value)) {
      prices[slug_bundle_key(entry)] = value;
    }
  }
  return prices;
}

const ITEM_SLUG_TO_LABEL: Record<string, string | [string, number]> = {
  "superior-abidos-fusion-material": "Serca Fusion",
  "destiny-crystallized-destruction-stone": "Serca Red",
  "destiny-crystallized-guardian-stone": "Serca Blue",
  "great-destiny-leapstone": "Serca Leaps",

  "destiny-guardian-stone": "Blue",
  "destiny-destruction-stone": "Red",
  "destiny-shard-pouch-s": ["Shards", 1000],
  "destiny-shard-pouch-m": ["Shards", 2000],
  "destiny-shard-pouch-l": ["Shards", 3000],
  "destiny-leapstone": "Leaps",
  "abidos-fusion-material": "Fusion",
  "glaciers-breath": "Glacier's Breath",
  "lavas-breath": "Lava's Breath",
  "artisans-metallurgy-level-1": "Scroll 1 Weapon",
  "artisans-tailoring-level-1": "Scroll 1 Armor",
  "artisans-metallurgy-level-2": "Scroll 2 Weapon",
  "artisans-tailoring-level-2": "Scroll 2 Armor",

  "artisans-metallurgy-level-3": "Scroll 3 Weapon",
  "artisans-tailoring-level-3": "Scroll 3 Armor",
  "artisans-metallurgy-level-4": "Scroll 4 Weapon",
  "artisans-tailoring-level-4": "Scroll 4 Armor",

  "metallurgy-hellfire-11-14": "11-14 Weapon",
  "tailoring-hellfire-11-14": "11-14 Armor",
  "metallurgy-hellfire-15-18": "15-18 Weapon",
  "tailoring-hellfire-15-18": "15-18 Armor",
  "metallurgy-hellfire-19-20": "19-20 Weapon",
  "tailoring-hellfire-19-20": "19-20 Armor",
};

// Fetches one region, or reuses its saved response within the cooldown. On failure the
// saved response, however old, is applied instead.
export async function start_fetch(region: MarketRegions, force?: boolean) {
  const runtime = useRuntimeStore();
  const { roster_config } = storeToRefs(useRosterStore());
  if (region === "Custom" || runtime.is_fetching[region]) return;
  if (!roster_config.value.auto_fetch && !force) return;
  const cached = roster_config.value.latest_market_data[region];
  const cooldown = force ? 1000 : FETCH_MARKET_COOLDOWN_MS;
  runtime.is_fetching[region] = true;
  if (cached !== undefined && Date.now() - cached[0] < cooldown) {
    await new Promise((r) => setTimeout(r, 200));
    apply_prices(parse_response(cached[1]), region);
  } else {
    try {
      const response = await fetch_market_data(region);
      apply_prices(parse_response(response), region);
      roster_config.value.latest_market_data[region] = [Date.now(), response];
      runtime.market_fetch_failed[region] = false;
    } catch {
      runtime.market_fetch_failed[region] = true;
      if (cached !== undefined) apply_prices(parse_response(cached[1]), region);
    }
  }
  runtime.is_fetching[region] = false;
}

// Writes only the listed bundles, then picks the best-value bundle of each material.
function apply_prices(prices: Record<string, number>, region: MarketRegions) {
  const { roster_config } = storeToRefs(useRosterStore());
  const column = roster_config.value.mats_prices[region];
  for (const [key, price] of Object.entries(prices)) {
    if (key in column.values) column.values[key] = price.toLocaleString();
  }
  roster_config.value.selected_bundles[region] = cheapest_bundles(
    input_column_to_num(column, true),
  );
}
