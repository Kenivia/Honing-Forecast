import { ref, computed } from "vue"
import { ALL_LABELS, FETCH_MARKET_COOLDOWN_MS, SYNCED_LABELS } from "./Constants"
import { storeToRefs } from "pinia"
import { useRosterStore } from "@/Stores/RosterConfig"
const OVERRIDE_DEFAULT = {
    Gold: 1,
    Silver: 0,
    "Serca Fusion": 200,
}
const BODY = {
    region_slug: "nae",
    item_slugs: [
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
        "metallurgy-hellfire-11-14",
        "tailoring-hellfire-11-14",
        "metallurgy-hellfire-15-18",
        "tailoring-hellfire-15-18",
        "metallurgy-hellfire-19-20",
        "tailoring-hellfire-19-20",
    ],
}
const MY_WORKER_URL = "https://snowy-base-1817.kenivia-fan.workers.dev/"

export async function fetchMarketData(region: string) {
    let body = structuredClone(BODY)
    body["region_slug"] = region.toLowerCase()
    console.log(body)
    const response = await fetch(MY_WORKER_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
    })

    // Await the parsing of the stream to get the actual payload
    const data = await response.json()

    // Log the actual payload, not the Response object
    console.log("Market Data Payload:", data)

    return data
}

const default_prices: number[][] = ALL_LABELS.map((x) => new Array(x.length).fill(999999999))

export function parse_response(response: any): [number[][], number, number] {
    let out = default_prices
    for (let tier = 0; tier < ALL_LABELS.length; tier++) {
        for (let index = 0; index < ALL_LABELS[tier].length; index++) {
            let label = ALL_LABELS[tier][index]
            if (OVERRIDE_DEFAULT.hasOwnProperty(label)) {
                out[tier][index] = OVERRIDE_DEFAULT[label]
            }
        }
    }

    // Track shard pouch prices: { 1000: price, 2000: price, 3000: price }
    const shard_prices: { [key: number]: number } = {}

    for (let index = 0; index < response.length; index++) {
        const { item_slug, price } = response[index]
        if (ITEM_SLUG_TO_LABEL.hasOwnProperty(item_slug)) {
            let label: string = ITEM_SLUG_TO_LABEL[item_slug]
            for (let tier = 0; tier < ALL_LABELS.length; tier++) {
                let index_in_labels = ALL_LABELS[tier].findIndex((x) => x == label)
                if (index_in_labels >= 0) {
                    out[tier][index_in_labels] = price
                } else if (label === "Shards small") {
                    shard_prices[1000] = price
                } else if (label === "Shards medium") {
                    shard_prices[2000] = price
                } else if (label === "Shards large") {
                    shard_prices[3000] = price
                }
            }
        }
    }

    // Calculate which shard bag size is most efficient (lowest price per shard)
    let selected_shard = 1000
    let shard_price = 0
    if (Object.keys(shard_prices).length > 0) {
        let best_value = Infinity
        for (const [shard_count, price] of Object.entries(shard_prices)) {
            const value_per_shard = price / parseInt(shard_count)
            if (value_per_shard < best_value) {
                best_value = value_per_shard
                selected_shard = parseInt(shard_count)
                shard_price = price
            }
        }
    }

    return [out, selected_shard, shard_price]
}

const ITEM_SLUG_TO_LABEL = {
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
    "metallurgy-hellfire-11-14": "11-14 Weapon",
    "tailoring-hellfire-11-14": "11-14 Armor",
    "metallurgy-hellfire-15-18": "15-18 Weapon",
    "tailoring-hellfire-15-18": "15-18 Armor",
    "metallurgy-hellfire-19-20": "19-20 Weapon",
    "tailoring-hellfire-19-20": "19-20 Armor",
}
export function useTimedFetch(callback: (data: number[][], selectedShardSize: number, shard_price: number) => void) {
    const roster_store = useRosterStore()
    const { roster_config } = storeToRefs(roster_store)

    const isFetching = ref(false)

    function isDataStale(region: string): boolean {
        const cached = roster_config.value.latest_market_data[region]
        if (cached === undefined) return true
        const [timestamp, _] = cached
        return Date.now() - timestamp >= FETCH_MARKET_COOLDOWN_MS
    }

    const disabled = computed(() => {
        // Only disabled if there's an actual pending fetch
        return isFetching.value
    })

    async function start_fetch(region: string) {
        if (isFetching.value) return

        // Check if we have fresh cached data
        const cached = roster_config.value.latest_market_data[region]
        if (cached !== undefined && !isDataStale(region)) {
            const [_, result] = cached
            const [parsed, selectedShardSize, shard_price] = parse_response(result)
            callback(parsed, selectedShardSize, shard_price)
            return
        }

        isFetching.value = true

        // Fetch new data
        const result = await fetchMarketData(region)
        // console.log(result)
        const [parsed, selectedShardSize, shard_price] = parse_response(result)

        // Store the raw response data with timestamp
        roster_config.value.latest_market_data[region] = [Date.now(), result]

        isFetching.value = false

        callback(parsed, selectedShardSize, shard_price)
    }

    return { disabled, start_fetch }
}
export function fetch_callback(result: number[][], selectedShardSize: number, shard_price: number) {
    const { roster_config } = storeToRefs(useRosterStore())

    roster_config.value.selected_shard_bag_size = selectedShardSize
    for (let tier = 0; tier < ALL_LABELS.length; tier++) {
        for (let index = 0; index < ALL_LABELS[tier].length; index++) {
            let actual_tier_to_modify = tier == 0 ? tier : SYNCED_LABELS.includes(ALL_LABELS[1][index]) ? 0 : 1
            roster_config.value.mats_prices[actual_tier_to_modify].data[index] = result[tier][index].toLocaleString()
            if (ALL_LABELS[tier][index] == "Shards") {
                roster_config.value.mats_prices[actual_tier_to_modify].data[index] = shard_price.toLocaleString()
            }
        }
    }
}
