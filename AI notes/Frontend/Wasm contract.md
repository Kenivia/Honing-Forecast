# Wasm contract

How the frontend talks to Rust for the calculator. Scanner operations share the same worker file and are covered under `Screen scanner/`.

## The boundary

- The frontend imports the wasm-pack output in `crates/wasm/pkg`. Rust changes need `pnpm run wasm`.
- There are no generated types. `PayloadBuilder.ts` hand-mirrors Rust's `Payload`, and result types are hand-written or untyped. A field rename on either side is a silent break.
- `WasmTypes.ts` holds `WasmOp` and the result types, and `Overrides.ts` the strategy override enums. Both are free of side effects: the main thread imports these, never `WasmWorker.ts` (that would pull the wasm glue into the main bundle and put the worker's `message` listener on `window`), and the profile imports the overrides without a cycle through `PayloadBuilder`.
- Results are the serialised Rust structs. Rust hash maps arrive as JS `Map`s, not plain objects.
- Extra frontend-only fields on payload objects are tolerated because Rust ignores unknown fields.

## Building the payload

There is one assembly. `payload_for` builds a payload from a profile and given budgets, chests and prices (per bundle key, with the bundle bought), with no optimizer state, plain JSON so it can be hashed. `states_payload` adds an optimizer result's states to it, with the user's overrides if given. `build_payload`, the calculator's, is `payload_for` over the store's columns and processed chests, plus `states_payload` of the last optimizer result and `runtime.adv_cache`. Income and Choices build theirs with `snapshot_payload` (`payload_for` of a snapshot's budgets and prices) and run the histogram on `states_payload` of their optimizer run. The payload holds:

- the upgrade list, from keyed upgrades, without the page-only fields (`expanded`, `taps_since_last_input`, `used_materials`). An upgrade gets the state of the fed-back optimizer result, from the taps since the last input on, only when that result has a state for it and carries its own `prep_output.juice_info`. An upgrade whose stored state is `[]` (reset by the user) is sent empty; `build_payload` then clears that marker to `null`, so the next payload takes the optimizer's state again. Building a payload writes nothing else to the stored upgrades;
- `material_labels`, the row order, and `materials`, a label-keyed map of the three owned amounts plus the taxed sell price and the buy price. For a Serca character each Serca material with a `CONVERTS_FROM` source has the source's stock added at the ratio (`merge_lower_tier`, pure, in `PayloadBuilder.ts`): T4 bound into bound, T4 roster-bound and tradable pooled into roster-bound (converting binds it), each floored like the market page's convert button. A material disabled on either side is left out. The free-tap budget adds the T4 special leaps the same way. Rust sees one stock per band and knows nothing of T4. Prices are sent per unit, of the bundle the user buys, the buy price the cheaper of buying and converting up from a lower tier (`effective_unit_prices`). The taxed price is the bundle's after the market's 5% fee (`sell_tax`, floored), divided by its size, and never above the buy price: a Serca material made more cheaply from T4 would otherwise sell for more than it costs, and plans assume taxed sell is worth no more than market. A material the user disabled is sent with zero prices and no tradable stock, because it is not traded at all and its bound stock must not be valued. The calculator's bound stock of it is `parse_input`'s 999999999 sentinel; a snapshot's payload has the snapshot's own amount;
- `plans`, every valuation plan to evaluate, and `optimizer_plan`, an index into it. Index 0 is always the all-worthless plan, so the histogram always reports the gross spend alongside the user's plan. The user's plan is clamped here, because the control panel's dropdowns only *disable* the values that would make it decrease, and Rust asserts it does not;
- tier, event flags and free-tap budget;
- `chests`, the profile's chest stacks after `process_chests` (`Screen scanner/Manifest.md`, "On the calculator page"), as a record of count, band and options by the stack's key, and `special_label`, the tier's special leap label. Amounts are unrounded and Rust floors the total. Every label is of the tier: Rust panics on any other;
- `chest_states`, the last optimizer result's, by the same keys, fed back like the free-tap order so the histogram evaluates what the optimizer chose. It arrives from Rust as a JS Map and goes back as one;
- the previous free-tap order and the advanced-honing cache, fed back in to avoid recomputation;
- `dist_plan`, the plan the histogram draws the gold distribution of (`Calculator core/Evaluation pipeline.md`), or none. Only the calculator sets it, to 0, and only in `start_eval_hist` (`CharWorkerUtils.ts`), never in `build_payload`, so Income's and Choices' hashes do not see it.

**The wasm carries the advanced-honing distributions of an upgrade not yet started.** `crates/wasm/build.rs` works out every strategy of the two fresh starts with double balls (+10/+20 and +30/+40) with the core's own code at build time, so the table cannot go stale, and `add_fresh_adv` puts it into the optimizer's payload when an upgrade is at its start. It is 0.6 MB of the wasm. An upgrade in progress, or one without double balls, is still worked out in the browser.

**Every advanced upgrade is sent with `double_balls`**, as the game now has it (`ADV_ALWAYS_DOUBLE_BALLS` in `Constants.ts`). Set to false, the rule of before is back: double only with the express event, or for +30/+40 with the "pretend" checkbox, which is hidden while the constant is true. Rust still takes either. The arena's generated scenarios are of before: double only under the express event. The whole table then comes back in the result's `adv_cache` like anything else in it.

## What must stay aligned with Rust

- **Material row order.** `ALL_LABELS` in `Constants.ts` defines row order per tier: seven base materials (`NUM_BASE_MATS`), then juices. It must match the tier's constants JSON on the Rust side, because the JSON carries no labels of its own. It is **derived** by filtering `MATERIAL_TABLE` by tier, so that table's order is the thing to get right. It is sent as `material_labels` and is the **only** place the order is declared: Rust builds a label to row map from it and keys every result by label, so a reorder cannot silently put a value on the wrong material, only mismatch the cost tables.
- **Piece order and counts**, including which index is the weapon and which the vambrace.
- **The advanced-honing strategy tables**, duplicated in `Constants.ts` and in Rust.
- **Band order**: char-bound, roster-bound, tradable, shared by `ValuationPlan`, `chance_within` and the graph annotations.
- **Juice ids**: the frontend also assumes the first two ids are juices and the rest books or scrolls.

## Workers

Each character owns three worker bundles (`WorkerBundle.ts`), held in `useRuntimeStore` and keyed by profile id, one per operation:

- **Optimizer**: debounced. Starting a run terminates any run in progress, and the worker is terminated again when the result arrives, so every run uses a fresh worker.
- **Histogram**: throttled. The latest payload wins, runs are never cancelled midway, and the worker is reused.
- **Marginal** (shelved: `start_marginal` and the column are commented out, see `Calculator core/Optimizer.md`): `WasmOp.MarginalValue` (`Calculator core/Optimizer.md`, "Marginal value"), started only by a final optimizer result (`start_marginal` in `CharWorkerUtils.ts`) with `{payload: build_payload(), labels}`. The labels are every enabled row but Gold. A new optimizer run cancels it. Its result, a label-keyed record, stays until the next one and is shown faded while either worker is busy. Rust returns, for juices and books only, `[price per unit, found]`, so base rows show nothing. The "Est. gold value" column shows the price per market bundle (the currently cheapest per unit, `cheapest_bundles`), to at least 2 significant figures, as "~n" when found, "<n" when not, "-" when not even at 0, with "(xN)" under it when the bundle is over 1.

Messages: the page sends an operation and payload; the worker replies with a result. During optimization, Rust also posts intermediate messages directly, bypassing the TypeScript worker code: the progress estimate every tenth of a second, with the best state at most once a second (`maybe_send_wasm_progress`). Their `adv_cache` is emptied on purpose, so the page keeps the cache it has until a final result brings one.

The histogram's result carries the `state_bundle` it evaluated (with `chest_states`), which the chest panel, the instructions and the graphs read.

**Errors**: the worker catches anything an op throws, a wasm panic included, and posts `{type: "error"}`. The bundle treats that like the worker's `onerror`: it logs it, sets `error`, terminates the worker (a panicked wasm instance is not reused) and calls back with `null`. Every callback returns on `null`; the throttled histogram goes on with the next pending payload.

**Off the calculator page**, `OptimizerJobs.ts` runs Income's and Choices' jobs two at a time on two bundles of its own. A hash is queued once, and a job already running is not queued again. A result depends only on the hash, so a finished run is stored into every job of the last `queue_jobs` with that hash, not just the one that ran: two snapshots, or now and a snapshot, with the same payload for the character (an export that did not change it, another character's mats moving) cost one run, not two. A run that fails is dropped and frees its slot.

## Flow on an input change

1. An input callback starts both workers (the optimizer only if auto-start is on).
2. The histogram runs immediately against the current state so the graphs update.
3. Each optimizer message, intermediate or final, stores the new state and triggers another histogram run.
4. **The gold distribution is only asked for while the optimizer is idle**: the histogram of the final optimizer result, and any histogram started while nothing runs (auto start off, an override). Not for intermediate results, nor for the histogram `start_all_workers` starts when the optimizer is about to run. The graph (`MaterialDist/GoldDist.vue`, the total gold row of the material table, on `MaterialGraph`) keeps the last one, faded, until the next comes.
5. When the user overrides part of the plan ("compare with simple strategies"), only the histogram reruns, and the instructions display the histogram's state instead of the optimizer's (`get_relevant_result`; the in-game order is `sort_upgrades` in `InstructionUtils.ts`). Whether anything is overridden is read off the override controls alone (`get_any_overwritten`).

**The progress shown waits for that histogram**: the optimizer panel caps it at 99.99% while the histogram is busy or has a throttled run waiting (`WorkerBundle.queued`), so 100% means the final gold is on the page.

**Every gold number shown is the histogram's** (`metric_per_plan`), scored with `Tuning::reference()`, not the optimizer's `metric`, which is of a cheaper evaluator and never displayed (`Calculator core/Evaluation pipeline.md`). The two differ by about 1e-5 of the metric, so they cannot be compared to tell anything.

Leaving the calculator page or switching character cancels every worker.
