import { ALL_LABELS, NUM_BANDS, SPECIAL_LEAP_LABELS } from "@/Utils/Constants";
import { merge_lower_tier } from "@/WasmInterface/PayloadBuilder";

type ByLabel = Record<string, number>;

// Per label and band, what a Serca character's T4 adds, as Rust counts it (merge_lower_tier).
// The special leap has one band, from the free-tap budget.
export function t4_gains(
  tier: number,
  owned: [ByLabel, ByLabel, ByLabel],
  enabled: Record<string, boolean>,
  special: ByLabel,
): Record<string, number[]> {
  const merged = merge_lower_tier(tier, ...owned, enabled);
  const out: Record<string, number[]> = {};
  for (const label of ALL_LABELS[tier]) {
    out[label] = [merged.bound, merged.roster, merged.tradable].map(
      (m, band) => m[label] - owned[band][label],
    );
  }
  const leap = SPECIAL_LEAP_LABELS[tier];
  out[leap] = [
    merge_lower_tier(tier, special, {}, {}).bound[leap] - special[leap],
  ];
  return out;
}

// Which graph lines show: the average, and the owned line of the chance band.
export const annotations_shown = (chance_band: number): boolean[] => [
  true,
  ...Array.from({ length: NUM_BANDS }, (_, band) => band === chance_band),
];

// The graph's lines for one label: the average, then the owned amount up to each band
// (stock, chest gains and T4 added). A band the user does not trade in, or one under it,
// has no line: its stock is effectively infinite.
export function owned_lines(
  average: number,
  owned: number[],
  added: number[],
  traded: boolean[],
): (number | null)[] {
  let total = 0;
  let shown = true;
  return [
    average,
    ...owned.map((amount, band) => {
      shown &&= traded[band];
      total += amount + added[band];
      return shown ? total : null;
    }),
  ];
}

// The free-tap chances per number of pieces tapped, one per normal upgrade, padded with 0.
export function special_graph_data(state_bundle: {
  latest_special_probs: number[];
  upgrade_arr: { is_normal_honing: boolean }[];
}): [number, number][] {
  const count = state_bundle.upgrade_arr.filter(
    (x) => x.is_normal_honing,
  ).length;
  return Array.from({ length: count }, (_, index) => [
    index,
    state_bundle.latest_special_probs[index] ?? 0,
  ]);
}
