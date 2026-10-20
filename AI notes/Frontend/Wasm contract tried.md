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
