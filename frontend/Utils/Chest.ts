import { CONVERTIBLE_MATERIALS, MATERIALS } from "./Constants";

export type Bag = Record<string, number>; // label -> amount

// One stack of identical chests. An option is what one chest gives when it is picked; an
// obtain-all chest is a chest with one option.
export interface Chest {
  key: string;
  title: string;
  band: number;
  count: number;
  options: Bag[];
}

// A stack as the scanner found it, with both tiers in it.
export interface ManifestChest extends Chest {
  obtain_all: boolean;
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

// the same options whatever order they, or the things in them, were listed in
const sorted = (options: Bag[]): Bag[] =>
  options
    .map((bag) =>
      Object.fromEntries(
        Object.entries(bag).sort(([a], [b]) => (a < b ? -1 : 1)),
      ),
    )
    .sort((a, b) => (JSON.stringify(a) < JSON.stringify(b) ? -1 : 1));

// Adds a stack to the one that gives the same, if there is one: counts summed, titles joined.
// `kind` keeps apart stacks that are not to be one although they give the same.
export function add_stack<T extends Chest>(
  stacks: Map<string, T>,
  chest: Omit<T, "key">,
  kind = "",
) {
  const options = sorted(chest.options);
  const key = `${kind}${chest.band}|${JSON.stringify(options)}`;
  const old = stacks.get(key);
  if (!old) return void stacks.set(key, { ...chest, key, options } as T);
  old.count += chest.count;
  if (!old.title.split(" / ").includes(chest.title))
    old.title += ` / ${chest.title}`;
}

const covers = (a: Bag, b: Bag) =>
  Object.entries(b).every(([label, amount]) => (a[label] ?? 0) >= amount);

// The stacks as a character of this tier takes them: T4 materials as Serca ones for a Serca
// character, nothing of the other tier, and no option that another is at least as good as.
// Stacks that come out alike are one, keyed by what they give.
export function process_chests(chests: ManifestChest[], tier: number): Chest[] {
  const out = new Map<string, Chest>();
  for (const chest of chests) {
    const converted =
      tier === 1 &&
      chest.options.some((option) =>
        Object.keys(option).some((label) => SERCA_OF[label]),
      );
    // a converted material is roster bound at best, and a stack has one band
    const band = converted ? Math.min(chest.band, 1) : chest.band;
    const bags = chest.options
      .map((option) =>
        Object.fromEntries(
          Object.entries(tier === 1 ? to_serca(option) : option).filter(
            ([label]) =>
              MATERIALS[label]?.tiers.includes(tier) &&
              // tradable gold is never counted
              !(label === "Gold" && chest.band === 2),
          ),
        ),
      )
      .filter((bag) => Object.keys(bag).length);
    // of two alike the first stays
    const options = bags.filter(
      (bag, at) =>
        !bags.some(
          (other, index) =>
            index !== at &&
            covers(other, bag) &&
            (index < at || !covers(bag, other)),
        ),
    );
    if (options.length)
      add_stack(out, { title: chest.title, band, count: chest.count, options });
  }
  return [...out.values()];
}

// Per label and band, what the chests give opened as the states say, floored as Rust does. A
// missing or unfitting state puts every chest on the first option, also as Rust does.
export function chest_gains(
  chests: Chest[],
  states?: Map<string, number[]>,
): Record<string, number[]> {
  const out: Record<string, number[]> = {};
  for (const chest of chests) {
    let state = states?.get(chest.key);
    if (
      state?.length !== chest.options.length ||
      state.reduce((a, b) => a + b, 0) !== chest.count
    )
      state = chest.options.map((_, index) => (index ? 0 : chest.count));
    chest.options.forEach((option, index) => {
      for (const [label, amount] of Object.entries(option)) {
        out[label] ??= [0, 0, 0];
        out[label][chest.band] += amount * state[index];
      }
    });
  }
  for (const gains of Object.values(out)) {
    gains.forEach((x, band) => (gains[band] = Math.floor(x + 1e-6)));
  }
  return out;
}
