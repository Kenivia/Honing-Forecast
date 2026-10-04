# Domain model

How `crates/core` represents the problem. Field names on these structs are effectively API: the frontend reads the serialised structs directly.

## Structs

- **`Payload`** (`payload.rs`): what JS sends. Hand-mirrored in `frontend/WasmInterface/PayloadBuilder.ts`; there is no generated binding.
- **`PreparationOutput`** (`parser.rs`): everything derived once from the payload: material budgets and prices, the treatment plan, and the event-adjusted `JuiceInfo`.
- **`Upgrade`** (`upgrade.rs`): one +N to +N+1 step on one piece. Holds its probability distribution, its per-material cost supports and its `State`.
- **`State`** (`state.rs`): the decision for one upgrade, and the thing the optimizer mutates.
  - Normal honing: for each tap, the set of juice/book ids used on that tap.
  - Advanced honing: two strategy indices (juice, scroll), each meaning "use it on the first N grace or non-grace taps".
- **`Support`** (`support.rs`): for one upgrade and one material, the amount consumed at each outcome, paired with the outcome probabilities.
- **`StateBundle`** (`state_bundle.rs`): one complete candidate solution: all upgrades, the free-tap order (`special_state`), the prepared inputs, caches and the last metric. `StateEssence` is the hashable subset (states plus free-tap order).

## Index conventions

These orderings are shared with the frontend and with the constants JSON. Nothing enforces them at compile time.

- **Pieces**: indices 0 to 4 are armour pieces, 5 is the weapon, 6 is the vambrace (Serca only). Piece *type* is a separate three-value index: armour, weapon, vambrace.
- **Upgrade index**: the target level minus one for normal honing; the block number for advanced honing.
- **Materials** (`support_index`): seven base materials in a fixed order (Red, Blue, Leapstones, Shards, Fusion, Gold, Silver), then one row per juice id. So juice id `k` lives at row `7 + k`.
- **Juice ids**: defined by the order of rows in the tier's constants JSON. The first two ids are the juices; higher ids are books and scrolls. Several places rely on that split.
- **Tier**: index into the embedded data tables. 0 is T4, 1 is Serca.

## Materials and prices

Each material carries a list of **breakpoints**, each an `(owned, price)` pair. The frontend always sends four: a zero placeholder, character-bound, roster-owned, and tradable. Price is what a unit is worth at that level: nothing or a leftover value for bound stock, the taxed sell price for roster stock, the market price beyond that.

A **treatment plan** says which breakpoints to merge when valuing materials, for example whether unused tradable stock counts as sellable. `distribute_budgets` turns breakpoints plus plan into cumulative `(threshold, marginal price)` pairs, which is the piecewise-linear gold function the evaluator integrates against.

## Probability model

- **Normal honing**: the distribution is over "succeeds on exactly tap i", up to the pity tap. Slot 0 is reserved for the free-tap outcome and is otherwise zero.
- **Advanced honing**: computed by a memoised recursion over the XP / grace state, giving three separate distributions: paid taps, juices used, scrolls used. The memo (`adv_cache`) is expensive, so it round-trips through JS between calls.
- **Free taps**: `special_state` is a permutation of upgrade indices. Free taps are attempted on one upgrade until it succeeds or the budget runs out, then on the next. This restriction is deliberate; the paper explains why.

## Game data

Costs and chances come from JSON embedded in the binary, under `constants/`, generated from a Google Sheet (see `docs/Constants.md`). Only two of the JSON files are wired in, one per tier. Vambrace support is partial: it exists for normal honing and is absent from the advanced tables.
