# Domain model

How `crates/core` represents the problem. Layout of `crates/core/src`: `payload.rs` and `parser.rs` (the input), `upgrade.rs`, `tuning.rs`, `probe.rs`, `helpers.rs` (`my_dbg!`); `honing_utils/` (materials, chests, supports, `StateBundle`, an upgrade's distributions), `advanced_honing/`, `constants/`, `eval/` (the evaluator), `optimizer/` (with `state.rs`, the lab in `lab/`), `ui/` (what only the page asks for: histogram, gold distribution, marginal prices). Field names on these structs are effectively API: the frontend reads the serialised structs directly.

## Structs

- **`Payload`** (`payload.rs`): what JS sends. Hand-mirrored in `frontend/WasmInterface/PayloadBuilder.ts`; there is no generated binding.
- **`PreparationOutput`** (`parser.rs`): everything derived once from the payload: the `MaterialTable`, the materials in row order, the valuation plans and their precomputed bands, the chests, the free-tap budget and the event-adjusted `JuiceInfo`.
- **`honing_utils/materials.rs`**: the material vocabulary. `MaterialTable` holds the labels and the label to row map; `OneMaterial` is one material's owned amounts and prices; `ValuationPlan` is what a leftover unit of each band is worth.
- **`Upgrade`** (`upgrade.rs`): one +N to +N+1 step on one piece. Holds its probability distribution, its per-material cost supports (`cost_dist`) and its `State`, plus a `Spare` of the previous state's distributions (`Evaluation pipeline.md`).
- **`State`** (`optimizer/state.rs`): the decision for one upgrade, and the thing the optimizer mutates.
  - Normal honing: for each tap, the set of juice/book ids used on that tap.
  - Advanced honing: two strategy indices (juice, scroll), each meaning "use it on the first N grace or non-grace taps".
- **`Support`** (`honing_utils/support.rs`): for one upgrade and one material, the amount consumed at each outcome, paired with the outcome probabilities.
- **`Tuning`** (`tuning.rs`): every evaluator and optimizer knob, overridable by name from the arena. The default is production; `Tuning::reference()` is the accurate evaluator used for reported numbers.
- **`StateBundle`** (`honing_utils/state_bundle.rs`): one complete candidate solution: all upgrades, the free-tap order (`special_state`), how the chests are opened (`chest_states`), the prepared inputs, caches (`special_cache`, `adv_cache`, `gold_memo`) and the last metric. `StateEssence` is the hashable subset (states, free-tap order, chest states).
- **`Chest`** (`honing_utils/chest.rs`): one stack of identical chests still to be opened. See "Chests" below.

## Index conventions

These orderings are shared with the frontend and with the constants JSON. Nothing enforces them at compile time.

- **Pieces**: indices 0 to 4 are armour pieces, 5 is the weapon, 6 is the vambrace. Both tiers have it: the T4 JSON is "Vambrace padded", with vambrace costs but a free-tap cost of 9999999. Piece *type* is a separate three-value index: armour, weapon, vambrace.
- **Upgrade index**: the target level minus one for normal honing; the block number for advanced honing.
- **Materials** (`support_index`): `NUM_BASE_MATS` base materials in a fixed order (Red, Blue, Leapstones, Shards, Fusion, Gold, Silver), then one row per juice id, so juice id `k` lives at `table.juice_row(k)`. The order comes from the payload's `material_labels` and the base prefix must match the cost tables, which carry no labels. Ask `MaterialTable` rather than comparing an index against a literal.
- **Juice ids**: defined by the order of rows in the tier's constants JSON. The first two ids are the juices; higher ids are books and scrolls. Several places rely on that split.
- **Tier**: index into the embedded data tables. 0 is T4, 1 is Serca.

## Materials and prices

A material is owned in three **bands**: character-bound, roster-bound and tradable. A `ValuationPlan` says what one *leftover* unit of each band is worth, out of three levels — nothing, the taxed sell price, or the buy price — and the three must not decrease across the bands, which `bands()` asserts.

`ValuationPlan::bands` turns a plan plus a material into the cumulative `(threshold, marginal value above it)` list the evaluator integrates against: entry `i`'s price applies to the interval `(threshold_i, threshold_{i+1}]`, and the **last** entry's price is what a unit costs to buy above the total owned.

**The list length is the cost.** `one_dimension_average_gold` spends a leftover evaluation per entry past the first and short-circuits to no work at all at one entry, and it does not skip an entry whose price gap is zero — it evaluates and then multiplies by zero. So every redundant entry is paid for. Three collapses keep the list minimal:

- a zero-width band's entry drops out;
- consecutive equal prices are one band;
- an all-zero list becomes the single entry `(0, 0)`, which is what a disabled material produces.

Neither of the first two may remove the final entry: it is a buy price, not a credit, so dropping it moves the whole function by a constant: it would silently erase a band's credit whenever its sell price equals its buy price, which is any material priced under 1 gold per unit, since `apply_tax` floors. `every_plan_produces_a_minimal_list` locks all of this in.

That final entry is still allowed to repeat the price under it, and when it does, **it costs nothing**: its coefficient is `prev_price - price`, so `one_dimension_average_gold` skips the two evaluations outright. It has to stay for the base term, which carries the constant, but the probabilities at it are never needed. A two-entry list always has value zero at its kink, which is why the entry cannot simply be merged away instead. This is what makes a fully market-valued plan free to evaluate: every price in its list is the same, so nothing is computed.

**Gold's tradable band is worth its face value in every plan** (`all_bands` forces it to `Market`, gold's price being 1). Gold is never sold, so owned tradable gold (stock and chests alike) is simply gold: every metric is gold spent past char-bound and roster-bound gold *minus* the tradable gold owned, and can be positive. That is what lets the optimizer value a chest option of tradable gold. The UI adds it back to show a gross spend (histogram output `tradable_gold`: Gold's settled tradable band, floored at 0). `prep_output.materials` keeps the real stock, so Gold's graph and `chance_within` count it.

A payload carries several plans. The optimizer uses `plans[optimizer_plan]`; the histogram evaluates every one and reports a metric per plan, so the UI can show the gross spend and the credit side by side without a second run.

## Chests

A `Chest` is a count, the band it is owned in and its options, each a `label -> amount` map of what **one** chest gives. An obtain-all chest is a chest with one option; nothing tells the two apart, and no chest holds a chest (the frontend flattens them, `Screen scanner/Manifest.md`). `Chests` is a map of them by key. The key is the frontend's, built from the stack's band and options, and Rust never reads into it.

- A **`ChestState`** is how one stack is opened: one number per option, summing to the count. `ChestStates` (`chest_states`) has one per chest under the chest's key, so a state from an earlier run still finds its chest when others were added or removed. A given state that does not fit its chest (its count changed), or a chest with none given, falls back to everything on the first option.
- **`resolve_chests`** turns the states into amounts gained per row and band, plus special leaps. Amounts are fractional (a T4 material converted to Serca is a fifth of a unit) and are summed unrounded. Every label must be a row of the tier or `special_label`, the label that is the free-tap budget: the frontend converts tiers and drops the rest (`process_chests`), and anything else panics.
- **`apply_chest_state(update_materials)`** floors what was gained, adds it to `base_materials` and `base_special_budget`, and rebuilds `bands` and `special_budget`. **`prep_output.bands` and `special_budget` are therefore always those of the current chest states; `prep_output.materials` is not**: only the histogram reads it, so only the histogram passes `true`. Anything that changes `chest_states` must call it; `set_chest_state` does, and skips the work when nothing changed.
- **An amount may be negative**: an option that costs gold (the Choices page's bonus chests, `Screen scanner/Choices.md`). `OneMaterial::settled` takes a negative band from the looser bands above it (char bound, then roster bound, then tradable), and `apply_chest_state` settles what it builds. Only the tradable band can stay negative, which is owing: `bands` makes a negative total the single entry `(total, buy price)`, everything bought from below zero, which is exact because the metric is linear in that threshold. A cost past char-bound and roster-bound gold comes out of tradable gold (see below), and past that is owed.
- `special_cache` is keyed by the free-tap budget and then the order, so a chest state that moves the budget does not read another budget's probabilities.

## Probability model

- **Normal honing**: the distribution is over "succeeds on exactly tap i", up to the pity tap. Slot 0 is reserved for the free-tap outcome and is otherwise zero.
- **Advanced honing**: computed by a memoised recursion over the XP / grace state, giving three separate distributions: paid taps, juices used, scrolls used. The memo (`adv_cache`) is expensive, so it round-trips through JS between calls.
- **Free taps**: `special_state` is a permutation of upgrade indices. Free taps are attempted on one upgrade until it succeeds or the budget runs out, then on the next. This restriction is deliberate: it keeps the free-tap outcomes to one per prefix of the order, so step 3 of an evaluation (`Evaluation pipeline.md`) stays linear in the number of upgrades.

## Game data

Costs and chances come from JSON embedded in the binary, in `crates/core/src/constants/` (`constants.rs` picks the file per tier). Only two of the JSON files in that folder are wired in, one per tier: `T4 Nov 2026.json` (rates from the game, the rest from a Google Sheet, `T4 July 2026 (Vambrace padded).json`; T4 has no vambrace, so its rows are padding) and `Serca Oct 2026.json`, whose normal honing is the game's (below) and the rest the sheet's. Vambrace support is partial: it exists for normal honing and is absent from the advanced tables. Serca's juice ids are Glacier's 0, Lava's 1, then the Tremor books 2 to 5 (12-15 armour, 12-15 weapon, 16-19 armour, 16-19 weapon), the order of their labels in the frontend's `MATERIAL_TABLE` (`Utils/Constants.ts`).

### Normal taps are the game's integers

Every normal honing chance and the artisan are integers out of `RATE_SCALE` (100,000, the juice's scale). `Tap` (`honing_utils/update_normal.rs`, built by `get_tap`) holds an upgrade's rule: the rate after n failed taps is `min(base + min(n·bonus, bonus_max) + event + juice, RATE_SCALE)`, each failed tap adds `rate · artisan_percent / 100` to the artisan, and a tap is certain once the artisan reaches `threshold` (215,000). The JSON keeps the game's own units: `NORMAL_CHANCES`, `NORMAL_FAIL_BONUS`, `NORMAL_FAIL_BONUS_MAX`, `ARTISAN_THRESHOLD` and `EVENT_NORMAL_EXTRA_CHANCE` out of 10,000 (scaled by 10 on load), `EVENT_ARTISAN_MULTIPLIER` in percent, and the normal juice column of `JUICE_BOOKS_AVAIL` out of 100,000 (books, out of 10,000 in the game, are written times 10).

- 100,000 is finer than anything modelled needs: every juice at its full count, and every book, is a whole /10,000. Only one unit of a Serca vambrace juice (375 or 25 per unit) is not, and the model never uses part of a stack.
- The bonus is not always a tenth of the base: T4 +1 to +3 are 10,000 with a bonus of 0.
- Floats appear only as `rate / RATE_SCALE` where a probability is needed (`chance`). Two orders of the same taps give exactly the same artisan, and an artisan landing exactly on the threshold is certain, which the old floats sometimes missed by one tap (from 0 artisan too).
- maxroll.gg's calculator (its decompiled wasm and JS fallback) does the same: integers out of 10,000, certain at `artisan >= threshold`, the fail bonus and its cap from the tables.
- The payload's `starting_artisan` stays a fraction of the threshold; `Upgrade::start_artisan` is it rounded to the integer the taps count from.
- For the UI, a normal `Upgrade` carries `tap_artisan` (the artisan before each tap of its state, a fraction, worked out with `normal_dist`) and `fail_chances` (the chance with nothing used, by failed taps up to where the bonus stops). The frontend only indexes them (`HoningUtil.ts`); the Introduction demo (`demo.ts`) is the one JS copy of the rule.

### Honing data in the game files

The same numbers are in the installed game, in the SQLite tables of `EFGame/data2.lpk` (`Screen scanner/Game files.md` has how they are read). **`python scripts/game_files/honing.py`** writes both tiers. For Serca it rebuilds normal honing from the tables (rates, costs, unlocks, free-tap leapstones, juice and book rows), keeping the advanced and `EVENT_*` fields of the JSON it starts from (`Serca Aug 2026 (Vambrace).json`), and writes `Serca Oct 2026.json`. For T4 it takes only the four rate fields from the game and keeps the rest of `T4 July 2026 (Vambrace padded).json`, writing `T4 Nov 2026.json` (the discount layer below). Float fields of the bases are converted to the integers above. It prints which fields changed.

- **Normal honing, all there.** `ItemEnhanceCommon`, one group per gear set and piece type (411100 Serca weapon, 411101 Serca armour, 411102 Serca vambrace, 410500 T4 weapon and 410501 T4 armour; T4 has no vambrace), `SecondaryKey` 100 plus the level:
  - `LevelUpSuccessRate`, `LevelUpFailureBonusRate`, `LevelUpFailureBonusMaxRate` and `LevelUpGuaranteeThreshold` out of 10,000; the juice (`LevelUpAdditiveId0k`, `Rate0k` a unit, `Max0k` units, `MaxRate` the cap over all) out of 100,000. The vambrace takes both juices, up to its max each;
  - shards, silver and gold in `LevelUpCostType/Amount` (type 18 shards, 1 silver, 2 gold);
  - unlock shards as the difference of `AccExp` between levels, unlock silver as that times `FeedSubCost`;
  - free-tap leapstones in `SpecialEnhanceMaterialAmount`.
- Stones, leapstones and fusion are in `ItemQualityOption`, the rows whose `EnhanceCommonId` is the group (many items per group, all alike). Books are in `ItemEnhanceMaterial` under the level's `SpecialMaterialGroupId`, one row per book item (event, bound and tradable copies), `EffectType00` 1 and the chance in `EffectArg00` out of 10,000.
- The client only holds what the installed version has.
- **Advanced honing** is in `ItemAmplificationBase` (the unlocks per ten levels) and `ItemAmplificationMaterial` (costs, success and great-success rates, juice, scrolls). Only armour was looked at.

**The open part is the discount layer.** The base tables are the undiscounted costs, and the JSON has the discounted ones:

- T4 normal honing gold in the base table is higher than the JSON's for the low levels (and equal from +19). The JSON's number is in `WelcomeBoostEnhance`, which `ItemQualityOption.WelcomeBoostId` points at, and which repeats every cost of the level.
- **The rates are not discounted.** `WelcomeBoostEnhance` also has `LevelUpSuccessRateBonus` and `LevelUpFailureBonusMultiply`, 0 and 1 at every T4 level, so the base table's T4 rates are the live ones and the generator takes them. The sheet's T4 rates were Serca's.
- T4 advanced honing stages 1 and 2 are undiscounted in the base table (e.g. Guardian Stones, shards). `ItemAmplificationBoost` has discount rates per material and `ItemAmplificationBase` names boost ids, but which row gives the JSON's numbers was not traced.
- Rows are gated by `ClassifyType`/`ClassifyIndex`, and the client tables alone do not say which gates are open. With no event running there is nothing live to compare an event discount against, so the `EVENT_*` rows cannot be checked now.

Until that is understood, generating T4's costs from the game would get the permanent discounts wrong. Serca has no `WelcomeBoostId` and the game's numbers are the JSON's.
