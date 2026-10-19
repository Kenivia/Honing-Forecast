import {
  ALL_BUNDLE_KEYS,
  bundle_key,
  cheapest_bundles,
  FALLBACK_PRICES,
  MATERIALS,
  WORKER_URL,
} from "./Constants";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { InputColumn } from "./InputColumn";
import { useRuntimeStore } from "@/Stores/RuntimeState";

const FETCH_MARKET_COOLDOWN_MS = 60 * 60 * 1000;
export type MarketRegions = "nae" | "euc" | "Custom";
// Custom is user-entered and is never fetched.
export const FETCHABLE_REGIONS: MarketRegions[] = ["nae", "euc"];
const BODY = {
  region_slug: "nae",
  item_slugs: [
    "superior-abidos-fusion-material",
    "destiny-crystallized-destruction-stone",
    "destiny-crystallized-guardian-stone",
    "great-destiny-leapstone",
    "destiny-guardian-stone",
    "destiny-destruction-stone",
    "destiny-shard-pouch-s",
    "destiny-shard-pouch-m",
    "destiny-shard-pouch-l",
    "destiny-leapstone",
    "abidos-fusion-material",
    "glaciers-breath",
    "lavas-breath",
    "artisans-metallurgy-level-1",
    "artisans-tailoring-level-1",
    "artisans-metallurgy-level-2",
    "artisans-tailoring-level-2",

    "artisans-metallurgy-level-3",
    "artisans-tailoring-level-3",
    "artisans-metallurgy-level-4",
    "artisans-tailoring-level-4",

    "metallurgy-hellfire-11-14",
    "tailoring-hellfire-11-14",
    "metallurgy-hellfire-15-18",
    "tailoring-hellfire-15-18",
    "metallurgy-hellfire-19-20",
    "tailoring-hellfire-19-20",
  ],
};

export async function fetch_market_data(
  region: MarketRegions,
): Promise<string> {
  let body = structuredClone(BODY);
  body["region_slug"] = region.toLowerCase();
  // console.log(body)
  const response = await fetch(WORKER_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = await response.json();

  // console.log("Market Data Payload:", data);

  return data;
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

// Returns the price of every bundle, and which bundle is cheapest per unit.
export function parse_response(
  response: any,
): [Record<string, number>, Record<string, number>] {
  // a fresh copy: a fetch must not mutate FALLBACK_PRICES
  const prices: Record<string, number> = { ...FALLBACK_PRICES };
  for (const { item_slug, price } of response) {
    const entry = ITEM_SLUG_TO_LABEL[item_slug];
    if (entry === undefined) {
      continue;
    }
    const key = slug_bundle_key(entry);
    if (Object.hasOwn(prices, key)) {
      prices[key] = parseInt(price);
    }
  }
  return [prices, cheapest_bundles(prices)];
}

const ITEM_SLUG_TO_LABEL = {
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

function is_data_stale(region: MarketRegions, cooldown: number): boolean {
  const roster_store = useRosterStore();
  const { roster_config } = storeToRefs(roster_store);
  const cached = roster_config.value.latest_market_data[region];
  if (cached === undefined) return true;
  const [timestamp, _] = cached;
  return Date.now() - timestamp >= cooldown;
}

export async function start_fetch(
  region: MarketRegions,
  force?: boolean,
  ignore_fetch_cooldown?: boolean,
) {
  const runtime = useRuntimeStore();
  const roster_store = useRosterStore();
  const { roster_config } = storeToRefs(roster_store);
  if (runtime.is_fetching && !ignore_fetch_cooldown) return;
  if (region === "Custom" || (!roster_config.value.auto_fetch && !force)) {
    return;
  }
  const cached = roster_config.value.latest_market_data[region];
  if (
    cached !== undefined &&
    !is_data_stale(region, force === true ? 1000 : FETCH_MARKET_COOLDOWN_MS) &&
    !runtime.market_fetch_failed
  ) {
    runtime.is_fetching = true;
    await new Promise((r) => setTimeout(r, 200));
    runtime.is_fetching = false;
    const [_, result] = cached;
    fetch_callback(...parse_response(result), region);
    return;
  }

  runtime.is_fetching = true;

  // Fetch new data
  const result = await (async () => {
    try {
      const out = await fetch_market_data(region);

      runtime.market_fetch_failed = false;
      return out;
    } catch {
      runtime.market_fetch_failed = true;
      return cached !== undefined ? cached : [];
    }
  })();

  const parsed = parse_response(result);

  // Store the raw response data with timestamp
  if (!runtime.market_fetch_failed) {
    roster_config.value.latest_market_data[region] = [Date.now(), result];
  }

  runtime.is_fetching = false;

  fetch_callback(...parsed, region);
}
function fetch_callback(
  prices: Record<string, number>,
  cheapest: Record<string, number>,
  region: MarketRegions,
) {
  const { roster_config } = storeToRefs(useRosterStore());
  const column = roster_config.value.mats_prices[region];
  for (const key of ALL_BUNDLE_KEYS) {
    if (prices[key] !== undefined) {
      column.values[key] = prices[key].toLocaleString();
    }
  }
  // a fetch also picks the best-value bundle for anything sold in several sizes
  roster_config.value.selected_bundles[region] = cheapest;
}
