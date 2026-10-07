import { ALL_MATERIAL_LABELS, FALLBACK_PRICES, WORKER_URL } from "./Constants";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  create_input_column,
  InputColumn,
  InputType,
  parse_locale_int,
} from "./InputColumn";
import { useRuntimeStore } from "@/Stores/RuntimeState";

export interface ShardInfo {
  selected: number;
  prices: Record<number, InputColumn>; // x1000, price in a single-celled column
}
// Shard bags are priced per bag, in their own single-cell columns.
export const SHARD_LABEL = "Shard";
export const DEFAULT_SHARD_INFO: ShardInfo = {
  prices: {
    3000: create_input_column(InputType.Int, [SHARD_LABEL]),
    2000: create_input_column(InputType.Int, [SHARD_LABEL]),
    1000: create_input_column(InputType.Int, [SHARD_LABEL]),
  },
  selected: 3000,
};
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

const SHARD_BAG_LABELS: Record<string, number> = {
  "Shards small": 1000,
  "Shards medium": 2000,
  "Shards large": 3000,
};

export function parse_response(
  response: any,
): [Record<string, number>, number, Record<number, InputColumn>] {
  // a fresh copy: a fetch must not mutate FALLBACK_PRICES
  const out: Record<string, number> = { ...FALLBACK_PRICES };
  const shard_prices: Record<number, InputColumn> = {};

  for (const { item_slug, price } of response) {
    const label: string = ITEM_SLUG_TO_LABEL[item_slug];
    if (label === undefined) {
      continue;
    }
    const bag_size = SHARD_BAG_LABELS[label];
    if (bag_size !== undefined) {
      shard_prices[bag_size] = create_input_column(
        InputType.Int,
        [SHARD_LABEL],
        { value: () => parseInt(price).toLocaleString() },
      );
    } else if (Object.hasOwn(out, label)) {
      out[label] = price;
    }
  }

  // Calculate which shard bag size is most efficient (lowest price per shard)
  let selected_shard = 1000;
  let best_value = Infinity;
  for (const [shard_count, column] of Object.entries(shard_prices)) {
    const value_per_shard =
      parse_locale_int(column.values[SHARD_LABEL]) / parseInt(shard_count);
    if (value_per_shard < best_value) {
      best_value = value_per_shard;
      selected_shard = parseInt(shard_count);
    }
  }

  return [out, selected_shard, shard_prices];
}

const ITEM_SLUG_TO_LABEL = {
  "superior-abidos-fusion-material": "Serca Fusion",
  "destiny-crystallized-destruction-stone": "Serca Red",
  "destiny-crystallized-guardian-stone": "Serca Blue",
  "great-destiny-leapstone": "Serca Leaps",

  "destiny-guardian-stone": "Blue",
  "destiny-destruction-stone": "Red",
  "destiny-shard-pouch-s": "Shards small",
  "destiny-shard-pouch-m": "Shards medium",
  "destiny-shard-pouch-l": "Shards large",
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
    const [parsed, selectedShardSize, shard_prices] = parse_response(result);
    fetch_callback(parsed, selectedShardSize, shard_prices, region);
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

  const [parsed, selectedShardSize, shard_prices] = parse_response(result);

  // Store the raw response data with timestamp
  if (!runtime.market_fetch_failed) {
    roster_config.value.latest_market_data[region] = [Date.now(), result];
  }

  runtime.is_fetching = false;

  fetch_callback(parsed, selectedShardSize, shard_prices, region);
}
function fetch_callback(
  prices: Record<string, number>,
  selected_shard_size: number,
  shard_prices: Record<number, InputColumn>,
  region: MarketRegions,
) {
  const roster_store = useRosterStore();
  const { roster_config } = storeToRefs(roster_store);
  roster_config.value.shard_infos[region].selected = selected_shard_size;
  roster_config.value.shard_infos[region].prices = shard_prices;
  const column = roster_config.value.mats_prices[region];
  for (const label of ALL_MATERIAL_LABELS) {
    if (prices[label] === undefined) {
      continue;
    }
    // shards are priced per bag in shard_infos, so this row is never read
    column.values[label] =
      label === "Shards" ? "0" : prices[label].toLocaleString();
  }
}
