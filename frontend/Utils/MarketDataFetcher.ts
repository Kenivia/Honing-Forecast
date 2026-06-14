import {
  ALL_LABELS,
  FALLBACK_PRICES,
  WORKER_URL,
  SERCA_TO_T4_INDICES,
} from "./Constants";
import { storeToRefs } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  create_input_column,
  InputColumn,
  InputType,
  parse_locale_int,
} from "./InputColumn";

export interface ShardInfo {
  selected: number;
  prices: Record<number, InputColumn>; // x1000, price in a single-celled column
}
export const DEFAULT_SHARD_INFO: ShardInfo = {
  prices: {
    3000: create_input_column(InputType.Int, ["Shard"], ["0"]),
    2000: create_input_column(InputType.Int, ["Shard"], ["0"]),
    1000: create_input_column(InputType.Int, ["Shard"], ["0"]), // the labels kinda not needed so not gonna bother
  },
  selected: 3000,
};
const FETCH_MARKET_COOLDOWN_MS = 60 * 60 * 1000;
export type MarketRegions = "nae" | "euc" | "Custom";
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

const default_prices: number[][] = FALLBACK_PRICES;

export function parse_response(
  response: any,
): [number[][], number, Record<number, InputColumn>] {
  let out = default_prices;
  // for (let tier = 0; tier < ALL_LABELS.length; tier++) {
  //     for (let index = 0; index < ALL_LABELS[tier].length; index++) {
  //         let label = ALL_LABELS[tier][index]
  //         // if (OVERRIDE_DEFAULT.hasOwnProperty(label)) {
  //         //     out[tier][index] = OVERRIDE_DEFAULT[label]
  //         // }
  //         out[tier][index] = FALLBACK_PRICES[tier][index]
  //     }
  // }

  // Track shard pouch prices: { 1000: price, 2000: price, 3000: price }
  const shard_prices: Record<number, InputColumn> = {};
  // console.log(response)
  for (let index = 0; index < response.length; index++) {
    const { item_slug, price } = response[index];
    // console.log(ITEM_SLUG_TO_LABEL, item_slug)
    if (Object.hasOwn(ITEM_SLUG_TO_LABEL, item_slug)) {
      let label: string = ITEM_SLUG_TO_LABEL[item_slug];
      for (let tier = 0; tier < ALL_LABELS.length; tier++) {
        let index_in_labels = ALL_LABELS[tier].findIndex((x) => x == label);

        const string_price = parseInt(price).toLocaleString();
        const this_column = create_input_column(
          InputType.Int,
          ["Shard"],
          [string_price],
        );
        if (index_in_labels >= 0) {
          out[tier][index_in_labels] = price;
        } else if (label === "Shards small") {
          shard_prices[1000] = this_column;
        } else if (label === "Shards medium") {
          shard_prices[2000] = this_column;
        } else if (label === "Shards large") {
          shard_prices[3000] = this_column;
        }
      }
    }
  }

  // Calculate which shard bag size is most efficient (lowest price per shard)
  let selected_shard = 1000;
  if (Object.keys(shard_prices).length > 0) {
    let best_value = Infinity;
    for (const [shard_count, this_column] of Object.entries(shard_prices)) {
      const value_per_shard =
        parse_locale_int(this_column.data[0]) / parseInt(shard_count);
      if (value_per_shard < best_value) {
        best_value = value_per_shard;
        selected_shard = parseInt(shard_count);
      }
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
  const roster_store = useRosterStore();
  const { roster_config } = storeToRefs(roster_store);
  if (roster_config.value.is_fetching && !ignore_fetch_cooldown) return;
  if (region === "Custom" || (!roster_config.value.auto_fetch && !force)) {
    return;
  }
  const cached = roster_config.value.latest_market_data[region];
  if (
    cached !== undefined &&
    !is_data_stale(region, force === true ? 1000 : FETCH_MARKET_COOLDOWN_MS) &&
    !roster_config.value.market_fetch_failed
  ) {
    roster_config.value.is_fetching = true;
    await new Promise((r) => setTimeout(r, 200));
    roster_config.value.is_fetching = false;
    const [_, result] = cached;
    const [parsed, selectedShardSize, shard_prices] = parse_response(result);
    fetch_callback(parsed, selectedShardSize, shard_prices, region);
    return;
  }

  roster_config.value.is_fetching = true;

  // Fetch new data
  const result = await (async () => {
    try {
      const out = await fetch_market_data(region);

      roster_config.value.market_fetch_failed = false;
      return out;
    } catch {
      roster_config.value.market_fetch_failed = true;
      return cached !== undefined ? cached : FALLBACK_PRICES;
    }
  })();

  const [parsed, selectedShardSize, shard_prices] = parse_response(result);

  // Store the raw response data with timestamp
  if (!roster_config.value.market_fetch_failed) {
    roster_config.value.latest_market_data[region] = [Date.now(), result];
  }

  roster_config.value.is_fetching = false;

  fetch_callback(parsed, selectedShardSize, shard_prices, region);
}
function fetch_callback(
  result: number[][],
  selected_shard_size: number,
  shard_prices: Record<number, InputColumn>,
  region: MarketRegions,
) {
  const roster_store = useRosterStore();
  const { roster_config } = storeToRefs(roster_store);
  // console.log(result)
  // console.log("fetch callback");
  // console.log(roster_store.active_mats_prices);
  // if (region === "Custom") {
  //   // shouldn't happen anyway but just in case
  //   return;
  // }
  roster_config.value.shard_infos[region].selected = selected_shard_size;
  roster_config.value.shard_infos[region].prices = shard_prices;
  for (let tier = 0; tier < ALL_LABELS.length; tier++) {
    for (let index = 0; index < ALL_LABELS[tier].length; index++) {
      const syncing = index in SERCA_TO_T4_INDICES && tier == 1;
      const actual_tier = syncing ? 0 : tier;
      const actual_index = syncing ? SERCA_TO_T4_INDICES[index] : index;

      roster_config.value.mats_prices[region][actual_tier].data[actual_index] =
        result[actual_tier][actual_index].toLocaleString();
      if (ALL_LABELS[actual_tier][actual_index] == "Shards") {
        roster_config.value.mats_prices[region][actual_tier].data[
          actual_index
        ] = "0"; // this shoulnd never actually be read
      }
    }
  }
}
