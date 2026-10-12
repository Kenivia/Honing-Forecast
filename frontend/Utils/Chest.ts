import { CONVERTIBLE_MATERIALS } from "./Constants";

export type Bag = Record<string, number>; // label -> amount

// One stack of identical chests. An option is what one chest gives when it is picked; an
// obtain-all chest is a chest with one option.
export interface ManifestChest {
  key: string;
  title: string;
  band: number;
  count: number;
  options: Bag[];
}

export function merge(bags: Bag[]): Bag {
  const out: Bag = {};
  for (const bag of bags) {
    for (const [label, amount] of Object.entries(bag)) {
      out[label] = (out[label] ?? 0) + amount;
    }
  }
  return out;
}

const SERCA_OF = Object.fromEntries(
  CONVERTIBLE_MATERIALS.map((mat) => [mat.from.label, mat]),
);

// T4 materials as the Serca ones they convert to. Not rounded: a count multiplies it later.
export const to_serca = (option: Bag): Bag =>
  merge(
    Object.entries(option).map(([label, amount]) => {
      const mat = SERCA_OF[label];
      return mat
        ? { [mat.label]: amount / mat.from.ratio }
        : { [label]: amount };
    }),
  );
