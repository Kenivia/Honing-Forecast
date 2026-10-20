import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import {
  fetch_market_data,
  parse_response,
  start_fetch,
} from "@/Utils/MarketDataFetcher";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { FALLBACK_PRICES } from "@/Utils/Constants";
import { input_column_to_num } from "@/Utils/InputColumn";

const HOUR = 60 * 60 * 1000;

// a market response, as the Worker forwards it
const RESPONSE = [
  { item_slug: "destiny-destruction-stone", price: "700" },
  { item_slug: "destiny-crystallized-destruction-stone", price: 3000 },
  { item_slug: "great-destiny-leapstone", price: "150.9" },
  { item_slug: "metallurgy-hellfire-19-20", price: "4000" },
  { item_slug: "destiny-shard-pouch-s", price: "100" },
  { item_slug: "destiny-shard-pouch-m", price: "180" },
  { item_slug: "destiny-shard-pouch-l", price: "260" },
  { item_slug: "some-new-item", price: "5" },
  { item_slug: "destiny-leapstone", price: "n/a" },
];

function mock_fetch(impl?: () => Promise<any>) {
  const fn = vi.fn(
    impl ?? (() => Promise.resolve({ json: () => Promise.resolve(RESPONSE) })),
  );
  vi.stubGlobal("fetch", fn);
  return fn;
}

const nae_prices = () =>
  input_column_to_num(useRosterStore().roster_config.mats_prices.nae);

beforeEach(() => setActivePinia(createPinia()));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("parse_response", () => {
  it("maps listed slugs onto bundle keys and nothing else", () => {
    expect(parse_response(RESPONSE)).toEqual({
      "Red x100": 700,
      "Serca Red x100": 3000,
      "Serca Leaps x1": 150, // parseInt truncates
      "19-20 Weapon x1": 4000,
      "Shards x1000": 100,
      "Shards x2000": 180,
      "Shards x3000": 260,
    });
  });

  it("throws on a body that is not a list", () => {
    expect(() => parse_response({ error: "x" })).toThrow();
  });
});

describe("fetch_market_data", () => {
  it("posts the lower-cased region and every slug", async () => {
    const fn = mock_fetch();
    await fetch_market_data("nae");
    const [, init] = fn.mock.calls[0] as any;
    const body = JSON.parse(init.body);
    expect(init.method).toBe("POST");
    expect(body.region_slug).toBe("nae");
    expect(body.item_slugs).toContain("destiny-shard-pouch-l");
    expect(body.item_slugs).toHaveLength(27);
  });
});

describe("start_fetch", () => {
  it("never fetches the custom region", async () => {
    const fn = mock_fetch();
    await start_fetch("Custom", true);
    expect(fn).not.toHaveBeenCalled();
  });

  it("respects auto fetch unless forced", async () => {
    const fn = mock_fetch();
    useRosterStore().roster_config.auto_fetch = false;
    await start_fetch("nae");
    expect(fn).not.toHaveBeenCalled();
    await start_fetch("nae", true);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("writes the listed prices, cheapest bundles and the raw response", async () => {
    mock_fetch();
    const store = useRosterStore();
    await start_fetch("nae");
    expect(nae_prices()["Red x100"]).toBe(700);
    expect(nae_prices()["Shards x2000"]).toBe(180);
    expect(nae_prices()["Blue x100"]).toBe(FALLBACK_PRICES["Blue x100"]);
    expect(store.roster_config.selected_bundles.nae.Shards).toBe(3000);
    expect(store.roster_config.latest_market_data.nae[1]).toEqual(RESPONSE);
    expect(useRuntimeStore().market_fetch_failed.nae).toBe(false);
    expect(useRuntimeStore().is_fetching.nae).toBe(false);
    // other regions untouched
    expect(
      input_column_to_num(store.roster_config.mats_prices.euc)["Red x100"],
    ).toBe(FALLBACK_PRICES["Red x100"]);
  });

  it("picks the cheapest shard bag per shard", async () => {
    mock_fetch(() =>
      Promise.resolve({
        json: () =>
          Promise.resolve([
            { item_slug: "destiny-shard-pouch-s", price: "50" },
          ]),
      }),
    );
    const store = useRosterStore();
    store.roster_config.mats_prices.nae.values["Shards x2000"] = "180";
    store.roster_config.mats_prices.nae.values["Shards x3000"] = "260";
    await start_fetch("nae");
    expect(store.roster_config.selected_bundles.nae.Shards).toBe(1000);
  });

  it("reuses the cached response within the cooldown", async () => {
    vi.useFakeTimers();
    const fn = mock_fetch();
    await start_fetch("nae");
    useRosterStore().roster_config.mats_prices.nae.values["Red x100"] = "1";
    const second = start_fetch("nae");
    expect(useRuntimeStore().is_fetching.nae).toBe(true);
    await vi.advanceTimersByTimeAsync(200);
    await second;
    expect(fn).toHaveBeenCalledTimes(1);
    expect(nae_prices()["Red x100"]).toBe(700); // re-applied from cache
  });

  it("refetches a stale cache, and a forced fetch only waits a second", async () => {
    vi.useFakeTimers();
    const fn = mock_fetch();
    const store = useRosterStore();
    store.roster_config.latest_market_data.nae = [Date.now() - HOUR, []];
    await start_fetch("nae");
    expect(fn).toHaveBeenCalledTimes(1);

    store.roster_config.latest_market_data.nae = [Date.now() - 500, RESPONSE];
    const p = start_fetch("nae", true);
    await vi.advanceTimersByTimeAsync(200);
    await p;
    expect(fn).toHaveBeenCalledTimes(1);

    store.roster_config.latest_market_data.nae = [Date.now() - 1500, RESPONSE];
    await start_fetch("nae", true);
    expect(fn).toHaveBeenCalledTimes(2);
  });

  it("skips a region already fetching, but regions fetch in parallel", async () => {
    const fn = mock_fetch();
    useRuntimeStore().is_fetching.nae = true;
    await start_fetch("nae");
    expect(fn).not.toHaveBeenCalled();
    await start_fetch("euc");
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("keeps existing prices on failure without a cache", async () => {
    mock_fetch(() => Promise.reject(new Error("offline")));
    const store = useRosterStore();
    store.roster_config.mats_prices.nae.values["Red x100"] = "1";
    await start_fetch("nae");
    expect(useRuntimeStore().market_fetch_failed.nae).toBe(true);
    expect(useRuntimeStore().is_fetching.nae).toBe(false);
    expect(store.roster_config.latest_market_data.nae).toBeUndefined();
    expect(nae_prices()["Red x100"]).toBe(1);
  });

  it("a bad body counts as a failure and does not leave the region fetching", async () => {
    mock_fetch(() =>
      Promise.resolve({ json: () => Promise.resolve({ error: "x" }) }),
    );
    await start_fetch("nae");
    expect(useRuntimeStore().market_fetch_failed.nae).toBe(true);
    expect(useRuntimeStore().is_fetching.nae).toBe(false);
    expect(
      useRosterStore().roster_config.latest_market_data.nae,
    ).toBeUndefined();
  });

  it("falls back to the cached response when a fetch fails", async () => {
    mock_fetch(() => Promise.reject(new Error("offline")));
    useRosterStore().roster_config.latest_market_data.nae = [
      Date.now() - 2 * HOUR,
      RESPONSE,
    ];
    await start_fetch("nae");
    expect(nae_prices()["Red x100"]).toBe(700);
  });

  it("does not refetch on app load while the saved cache is fresh", async () => {
    vi.useFakeTimers();
    const fn = mock_fetch();
    useRosterStore().roster_config.latest_market_data.nae = [
      Date.now() - 60_000,
      RESPONSE,
    ];
    const p = start_fetch("nae");
    await vi.advanceTimersByTimeAsync(200);
    await p;
    expect(fn).not.toHaveBeenCalled();
    expect(nae_prices()["Red x100"]).toBe(700);
  });

  it("keeps a typed price for a material the market does not list", async () => {
    mock_fetch();
    const store = useRosterStore();
    store.roster_config.mats_prices.nae.values["Silver x1000000"] = "50";
    await start_fetch("nae");
    expect(nae_prices()["Silver x1000000"]).toBe(50);
  });
});
