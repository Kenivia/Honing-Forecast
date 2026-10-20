import { vi } from "vitest";
import { ALL_LABELS, SPECIAL_LEAP_LABELS } from "@/Utils/Constants";

// jsdom has neither
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;
window.matchMedia ??= ((query: string) => ({
  matches: false,
  media: query,
  addEventListener: () => {},
  removeEventListener: () => {},
})) as unknown as typeof window.matchMedia;

// A histogram result with every label of the tier, as Rust returns it.
export function fake_histogram(tier: number, over: Record<string, any> = {}) {
  const labels = ALL_LABELS[tier];
  const per = <T>(f: (label: string, i: number) => T) =>
    Object.fromEntries(labels.map((label, i) => [label, f(label, i)]));
  return {
    avg_used: per((_, i) => (i < 2 ? 100 * (i + 1) : 0)),
    gold_per_plan: [per((_, i) => (i === 0 ? -500 : 20)), per(() => 0)],
    chance_within: per(() => [0.1, 0.5, 0.9]),
    cum_percentiles: per(() => [
      [0, 0.2],
      [10, 0.6],
      [20, 1],
    ]),
    metric_per_plan: [1000, 700],
    state_bundle: {
      chest_states: new Map<string, number[]>(),
      latest_special_probs: [0.5, 0.25],
      upgrade_arr: [],
    },
    ...over,
  };
}

export const leap = (tier: number) => SPECIAL_LEAP_LABELS[tier];
export const flush = () => new Promise((r) => setTimeout(r, 0));
export { vi };
