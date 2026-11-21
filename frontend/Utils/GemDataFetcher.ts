import { ref } from "vue";
import { WORKER_URL } from "./Constants";
import { MarketRegions } from "./MarketDataFetcher";

// Gem searches from the region's latest scanner pass (`sections.gem`), kept for a minute.
export type GemListing = { buy_now: number | null };
export type GemSearch = {
  at: string;
  item: string;
  gem_level: number;
  rows: GemListing[];
};
export type GemData = { fetched: number; searches: GemSearch[] };

const COOLDOWN_MS = 60 * 1000;
export const gem_data = ref<Partial<Record<MarketRegions, GemData>>>({});
export const gem_fetching = ref(false);
export const gem_failed = ref(false);

export async function fetch_gems(region: MarketRegions) {
  if (region === "Custom" || gem_fetching.value) return;
  const cached = gem_data.value[region];
  if (cached && Date.now() - cached.fetched < COOLDOWN_MS) return;
  gem_fetching.value = true;
  gem_failed.value = false;
  try {
    const res = await fetch(`${WORKER_URL}/scans/latest/${region}`);
    const body = await res.json();
    gem_data.value[region] = {
      fetched: Date.now(),
      searches: body.scan.sections.gem ?? [],
    };
  } catch {
    gem_failed.value = true;
  }
  gem_fetching.value = false;
}
