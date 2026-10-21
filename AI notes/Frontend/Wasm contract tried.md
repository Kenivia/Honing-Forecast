# Wasm contract: tried and changed

Before/after of changes to the payload and worker protocol. The current state is in `Wasm contract.md`.

## October 2026 audit

- **Sell tax.** Before: `apply_tax` floored 95% of the *per-unit* price, at least `min(1, price)`. A unit under one gold (Blue at 0.1) was not taxed at all, and Red (6.47) lost 7.3%. After: the fee is taken on the bundle price (`sell_tax`), then divided by the bundle size (`unit_prices(..., sell_tax)`). Every "taxed sell" metric moved slightly.
- **Worker errors.** Before: an op that threw inside the async message handler, a wasm panic included, was an unhandled rejection in the worker. `onerror` never fired, so the histogram bundle stayed "busy" and stopped rerunning until the page was left. `onerror` itself set the bundle idle without terminating the worker or calling back, so an `OptimizerJobs` slot stayed taken forever. After: the worker posts `{type: "error"}`; the bundle logs it, terminates the worker and calls back with `null`.
- **Job queue.** Before: a hash queued twice (Income's selected snapshot is also one the estimate uses) ran on both slots at once, and the second to finish never freed its slot. After: `queue_jobs` keeps each hash once and skips running ones.
- **Interim states and `adv_cache`.** Before: every interim optimizer state set `runtime.adv_cache` to the empty map Rust sends with it, so advanced distributions were recomputed during every run. After: only a non-empty cache replaces it.
- **One payload assembly.** Before: `build_payload` and `payload_for` each built the whole object. The calculator's fed-back states needed the *histogram's* `juice_info`. `keyed_to_array` wrote `double_balls` and cleared reset markers on the stored upgrades, from Income's runs too. After: `build_payload` = `payload_for` + `states_payload`, states fed back with the optimizer result's own `juice_info`, and only `build_payload` clears reset markers. A "Full" juice or book override now replaces the tap's ids of that kind with every usable one instead of appending them, so no id appears twice (Rust counted a repeat once anyway).

## One run per hash serves every job of it (Oct 2026)

Before: once the queue kept one job per hash, two jobs with the same payload (Before and After an export that did not touch the character, or now and a snapshot) ran once, but only the job that ran stored its result; the other stayed hollow, and the re-queue after the store ran the same optimization again. After: `queue_jobs` keeps its jobs by hash and a finished run calls every one's `store`. Covered by `tests/unit/wasm/optimizer_jobs.test.ts` and `tests/unit/income-choices/jobs.test.ts`.

## The gold distribution (October 2026)

- Measured on every histogram, the distribution made each run five to six times as long (`Calculator core/Evaluation pipeline tried.md`). It is asked for only while the optimizer is idle now. `start_eval_hist` takes no argument, because `@change="start_eval_hist"` passes the event in; the flag is in `eval_hist`.

## Taxed sell above market (Oct 2026)

Before: `taxed_price` was a material's own bundle after the fee, `market_price` the cheaper of buying and converting up (`effective_unit_prices`). With the stub prices Serca Blue sold at 3.73 a unit but cost 3.40 made from T4, so a tradable leftover was worth more under "taxed sell" than under "market" (1M tradable Serca Blue: 3,311,723 against 3,018,729 on a fixed state). After: the taxed price is capped at the buy price.

## Progress (Oct 2026)

Before: Rust posted progress only with a state, at most once a second per chain (each chain restarted its timer), and a later chain posted nothing until it beat the chains before. Short runs went 0.01% then 100%; a heavy Serca run sat at 24% for 50 s. The panel showed 100% as soon as the optimizer's result arrived, 150 to 330 ms before the histogram's gold. After: progress every 0.1 s, the state still at most once a second and only past the floor; the panel shows 99.99% until the histogram is done. A heavy run now climbs smoothly to about 99.9% and then waits on the merge, exchange and polish after the chains (about 25 s of a 55 s run in the fixture's all-wanted Serca case), which report nothing.
