# memetic

`crates/core/src/optimizer/lab/memetic.rs`. Archetype: population, structured recombination, local improvement.

## Algorithm as built

- **Individual**: every upgrade's payload, the free tap order, the chest states, its metric. One working `StateBundle`; an individual is loaded by swapping only the payloads that differ, so the evaluator's per-upgrade caches and gold memo carry across individuals.
- **Seeds**: the given state and every carried plan, normal plans put on the block grid (kept as is if already on it, else `snap_to_block`, else `Model::filled`), each checked with `check::invalid`. All-empty if none is valid. The best seed is never lost (replacement only by better).
- **Population** (`4 + 4 * effort`, 4..16): the seeds, then price seeds: every normal upgrade's `Response::linear` best at prices mixed per row between the shadow prices at the best seed and the market (last band) price, times lognormal noise; half of them get an order sorted by `base_chance * cost / special_cost`; then 1-3 upgrades toggled in or out of the free taps; chest stacks put whole on a random option; advanced slots set at random.
- **Anchor**: the best member gets a full polish (below), and a second copy is polished from the leapstone-saving order.
- **Children** (steady state): two tournament parents. Groups at a random grain (piece type, piece, or single upgrade): each group's plans come from A or B. The free tap order: 70% a place crossover (each upgrade keeps its share-of-the-run place in the free taps of the parent its group came from, or is past where free taps stop), 30% classic OX. Chest stacks whole from A or B. Mutation (50%, always if the child duplicates a member): one block of one upgrade's plan to a random option (to the block end or to the end), an advanced index, an order swap, or a chest move.
- **Child local search**: true best responses (`Response::of` + `better`) on 3 upgrades of the piece types that mixed (random ones otherwise), kept if the metric agrees; one advanced, one order and one chest move.
- **Replacement**: crowding: the child replaces the nearest member (distance = upgrades whose plans differ + order + chest stacks) if better.
- **Restarts**: after `140 * effort` children with no new best: polish the best, replace every other member by a price seed from it plus one pass of responses. Stop after `ceil(3 * effort)` restarts in a row without a new best, or at 80% of the budget; then a final polish.
- **Polish** (until a round gains nothing, at most 4): responses on every normal upgrade in chunks of 4; every advanced slot scanned over all 19 indices; order: each upgrade moved near itself, to the front, beside where free taps stop, to the end, and beside the other levels of its piece; one in and one out of the free taps; runs of 2-3 to the front or the end. A failed order move is retried with the moved upgrades' responses (plans answering the order). Chests: per stack, moves between options halving from count/2 to 1, then single-chest moves in two places at once.
- **Budget**: counted in evaluations plus `response.work`, `effort * (20000 + 3000 * upgrades)`; it is a cap, the stall rule is what usually stops it. Deterministic, all draws from `rng`.

## What mattered (pilot, found on the way)

- The order must not be forced to rising level per piece: the best orders free tap a higher level and leave lower ones past the stop (pilot hit 22% -> 78% when dropped).
- Free taps can come from chests only, so the order is searched whenever a chest offers the special leap.
- Order moves need the plans to answer them; polishing the best at each restart, not only at the end, took pilot from 81% to 97% hit.

## Dev numbers, `quick_dev`, 1 seed, session 16 of `dev_memetic.sqlite` (cold / chained)

| variant | diff | hit % | x | worse payloads |
| --- | --- | --- | --- | --- |
| effort 0.5 | -0.0087 / -0.0077 | 92.0 / 92.0 | 0.55 / 0.52 | 28 |
| effort 1 | -0.0023 / -0.0007 | 92.8 / 94.6 | 0.85 / 0.82 | 23 |
| effort 2 | +0.0021 / +0.0028 | 95.1 / 96.0 | 1.73 / 1.67 | 17 |
| effort 1, crossover off | -0.0091 / -0.0094 | 87.7 / 90.0 | 0.87 / 0.81 | 39 |
| `current` | 0 / -0.0003 | 95.4 / 94.8 | 1 / 0.85 | |

No invalid states, no panics (0 errors in the 2792 memetic runs). Pilot: session 11/12 (effort 1, cold -0.0004, 97% hit, x 0.77).

**Recombination earns**: with crossover off (child = mutated copy of one tournament parent, same local search, same budget) the mean is 4x worse, p10 -0.017 vs -0.001, hit 88% vs 93%; on pilot 88% vs 97%.

## Weak spots

- `chain_0017_t4_n14` (one piece, 14 levels, big free tap budget) stays 0.4-0.6% behind at efforts 0.5 and 1: which levels get free tapped and the plans of those are coupled several at a time.
- Multi-pool upgrades (weapon with two juice pools and books, `single_0002_t4_n3`, -0.07%): `better` falls back to `improve`, which stops short of the annealer's plans.
- CPU does not halve at 0.5 (0.55 vs 0.85): the polish is a fixed cost.

## Next

- A joint response of the upgrades next to each other in the free taps, or a small enumeration of which levels of a piece are free tapped, in place of the single and paired toggles.
- Polish only the members that recombination will use, and let the child local search take an order move with responses.
- Tune the stall/restart rule so that 0.5 spends half.
