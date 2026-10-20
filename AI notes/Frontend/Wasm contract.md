# Wasm contract

How the frontend talks to Rust for the calculator. Scanner operations share the same worker file and are covered under `Screen scanner/`.

## The boundary

- The frontend imports the wasm-pack output in `crates/wasm/pkg`. Rust changes need `pnpm run wasm`.
- There are no generated types. `PayloadBuilder.ts` hand-mirrors Rust's `Payload`, and result types are hand-written or untyped. A field rename on either side is a silent break.
- Results are the serialised Rust structs. Rust hash maps arrive as JS `Map`s, not plain objects.
- Extra frontend-only fields on payload objects are tolerated because Rust ignores unknown fields.

## Building the payload

`build_payload` assembles, from the store (`payload_for` builds the same from a profile and given budgets, chests and prices, with no optimizer state, for the Income page's snapshot runs, and `states_payload` adds an optimizer result's states to it for the histogram):

- the upgrade list, from keyed upgrades, including each upgrade's state from the previous optimizer result so the next run starts from it;
- `material_labels`, the row order, and `materials`, a label-keyed map of the three owned amounts plus the taxed sell price and the buy price. For a Serca character each Serca material with a `CONVERTS_FROM` source has the source's stock added at the ratio (`merge_lower_tier`, pure, in `PayloadBuilder.ts`): T4 bound into bound, T4 roster-bound and tradable pooled into roster-bound (converting binds it), each floored like the market page's convert button. A material disabled on either side is left out. The free-tap budget adds the T4 special leaps the same way. Rust sees one stock per band and knows nothing of T4. Prices arrive already per-unit: the store resolves which bundle the user buys and whether converting up from a lower tier is cheaper. A material the user disabled is sent with zero prices, because it is not traded at all and its (effectively infinite) bound stock must not be valued;
- `plans`, every valuation plan to evaluate, and `optimizer_plan`, an index into it. Index 0 is always the all-worthless plan, so the histogram always reports the gross spend alongside the user's plan. The user's plan is clamped here, because the control panel's dropdowns only *disable* the values that would make it decrease, and Rust asserts it does not;
- tier, event flags and free-tap budget;
- `chests`, the profile's chest stacks after `process_chests` (`Screen scanner/Manifest.md`, "On the calculator page"), as a record of count, band and options by the stack's key, and `special_label`, the tier's special leap label. Amounts are unrounded and Rust floors the total. Every label is of the tier: Rust panics on any other;
- `chest_states`, the last optimizer result's, by the same keys, fed back like the free-tap order so the histogram evaluates what the optimizer chose. It arrives from Rust as a JS Map and goes back as one;
- the previous free-tap order and the advanced-honing cache, fed back in to avoid recomputation.

**The wasm carries the advanced-honing distributions of an upgrade not yet started.** `crates/wasm/build.rs` works out every strategy of the two fresh starts with double balls (+10/+20 and +30/+40) with the core's own code at build time, so the table cannot go stale, and `add_fresh_adv` puts it into the optimizer's payload when an upgrade is at its start. It is 0.6 MB of the wasm. An upgrade in progress, or one without double balls, is still worked out in the browser.

**Every advanced upgrade is sent with `double_balls`**, as the game now has it (`ADV_ALWAYS_DOUBLE_BALLS` in `Constants.ts`). Set to false, the rule of before is back: double only with the express event, or for +30/+40 with the "pretend" checkbox, which is hidden while the constant is true. Rust still takes either. The arena's generated scenarios are of before: double only under the express event. The whole table then comes back in the result's `adv_cache` like anything else in it.

## What must stay aligned with Rust

- **Material row order.** `ALL_LABELS` in `Constants.ts` defines row order per tier: seven base materials (`NUM_BASE_MATS`), then juices. It must match the tier's constants JSON on the Rust side, because the JSON carries no labels of its own. It is **derived** by filtering `MATERIAL_TABLE` by tier, so that table's order is the thing to get right. It is sent as `material_labels` and is the **only** place the order is declared: Rust builds a label to row map from it and keys every result by label, so a reorder cannot silently put a value on the wrong material, only mismatch the cost tables.
- **Piece order and counts**, including which index is the weapon and which the vambrace.
- **The advanced-honing strategy tables**, duplicated in `Constants.ts` and in Rust.
- **Band order**: char-bound, roster-bound, tradable, shared by `ValuationPlan`, `chance_within` and the graph annotations.
- **Juice ids**: the frontend also assumes the first two ids are juices and the rest books or scrolls.

## Workers

Each character owns two worker bundles (`WorkerBundle.ts`), held in `useRuntimeStore` and keyed by character name, one per operation:

- **Optimizer**: debounced. Starting a run terminates any run in progress, and the worker is terminated again when the result arrives, so every run uses a fresh worker.
- **Histogram**: throttled. The latest payload wins, runs are never cancelled midway, and the worker is reused.

Messages: the page sends an operation and payload; the worker replies with a result. During optimization, Rust also posts intermediate results with a progress estimate directly, bypassing the TypeScript worker code.

## Flow on an input change

1. An input callback starts both workers (the optimizer only if auto-start is on).
2. The histogram runs immediately against the current state so the graphs update.
3. Each optimizer message, intermediate or final, stores the new state and triggers another histogram run.
4. When the user overrides part of the plan ("compare with simple strategies"), only the histogram reruns, and the instructions display the histogram's state instead of the optimizer's. Whether anything is overridden is read off the override controls alone (`get_any_overwritten`).

**Every gold number shown is the histogram's** (`metric_per_plan`), scored with `Tuning::reference()`, not the optimizer's `metric`, which is of a cheaper evaluator and never displayed (`Calculator core/Evaluation pipeline.md`). The two differ by about 1e-5 of the metric, so they cannot be compared to tell anything.

Leaving the calculator page or switching character cancels both workers.
