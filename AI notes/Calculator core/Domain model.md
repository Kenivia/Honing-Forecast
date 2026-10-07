# Domain model

How `crates/core` represents the problem. Field names on these structs are effectively API: the frontend reads the serialised structs directly.

## Structs

- **`Payload`** (`payload.rs`): what JS sends. Hand-mirrored in `frontend/WasmInterface/PayloadBuilder.ts`; there is no generated binding.
- **`PreparationOutput`** (`parser.rs`): everything derived once from the payload: the `MaterialTable`, the materials in row order, the valuation plans and their precomputed bands, and the event-adjusted `JuiceInfo`.
- **`materials.rs`**: the material vocabulary. `MaterialTable` holds the labels and the label to row map; `OneMaterial` is one material's owned amounts and prices; `ValuationPlan` is what a leftover unit of each band is worth.
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
- **Materials** (`support_index`): `NUM_BASE_MATS` base materials in a fixed order (Red, Blue, Leapstones, Shards, Fusion, Gold, Silver), then one row per juice id, so juice id `k` lives at `table.juice_row(k)`. The order comes from the payload's `material_labels` and the base prefix must match the cost tables, which carry no labels. Ask `MaterialTable` rather than comparing an index against a literal.
- **Juice ids**: defined by the order of rows in the tier's constants JSON. The first two ids are the juices; higher ids are books and scrolls. Several places rely on that split.
- **Tier**: index into the embedded data tables. 0 is T4, 1 is Serca.

## Materials and prices

A material is owned in three **bands**: character-bound, roster-bound and tradable. A `ValuationPlan` says what one *leftover* unit of each band is worth, out of three levels — nothing, the taxed sell price, or the buy price — and the three must not decrease across the bands, which `bands()` asserts.

`ValuationPlan::bands` turns a plan plus a material into the cumulative `(threshold, marginal value above it)` list the evaluator integrates against: entry `i`'s price applies to the interval `(threshold_i, threshold_{i+1}]`, and the **last** entry's price is what a unit costs to buy above the total owned.

**The list length is the cost.** `one_dimension_average_gold` spends two saddlepoint evaluations per entry past the first and short-circuits to no work at all at one entry, and it does not skip an entry whose price gap is zero — it evaluates and then multiplies by zero. So every redundant entry is paid for. Three collapses keep the list minimal:

- a zero-width band's entry drops out;
- consecutive equal prices are one band;
- an all-zero list becomes the single entry `(0, 0)`, which is what a disabled material produces.

Neither of the first two may remove the final entry: it is a buy price, not a credit, so dropping it moves the whole function by a constant. The old `distribute_budgets` did drop it, which silently erased a band's credit whenever its sell price equalled its buy price — any material priced under 1 gold per unit, since `apply_tax` floors. `every_plan_produces_a_minimal_list` locks all of this in.

That final entry is still allowed to repeat the price under it, and when it does, **it costs nothing**: its coefficient is `prev_price - price`, so `one_dimension_average_gold` skips the two evaluations outright. It has to stay for the base term, which carries the constant, but the probabilities at it are never needed. A two-entry list always has value zero at its kink, which is why the entry cannot simply be merged away instead. This is what makes a fully market-valued plan free to evaluate: every price in its list is the same, so nothing is computed.

A payload carries several plans. The optimizer uses `plans[optimizer_plan]`; the histogram evaluates every one and reports a metric per plan, so the UI can show the gross spend and the credit side by side without a second run.

## Probability model

- **Normal honing**: the distribution is over "succeeds on exactly tap i", up to the pity tap. Slot 0 is reserved for the free-tap outcome and is otherwise zero.
- **Advanced honing**: computed by a memoised recursion over the XP / grace state, giving three separate distributions: paid taps, juices used, scrolls used. The memo (`adv_cache`) is expensive, so it round-trips through JS between calls.
- **Free taps**: `special_state` is a permutation of upgrade indices. Free taps are attempted on one upgrade until it succeeds or the budget runs out, then on the next. This restriction is deliberate; the paper explains why.

## Game data

Costs and chances come from JSON embedded in the binary, under `constants/`, generated from a Google Sheet (see `docs/Constants.md`). Only two of the JSON files are wired in, one per tier. Vambrace support is partial: it exists for normal honing and is absent from the advanced tables.
