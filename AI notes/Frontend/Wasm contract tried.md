# Wasm contract: don't try

Traps in the payload and worker protocol. The current state is in `Wasm contract.md`.

## Payload

- Don't tax the per-unit price (`floor(0.95 * unit)`, at least `min(1, price)`): a unit under one gold (Blue at 0.1) is not taxed at all and Red (6.47) loses 7.3%. Tax the bundle price, then divide by the bundle size.
- Don't leave the taxed sell price above the buy price: with stub prices Serca Blue sold at 3.73 a unit but cost 3.40 made from T4, so a tradable leftover was worth more under "taxed sell" than under "market" (3,311,723 against 3,018,729 for 1M on a fixed state). Cap it at the buy price.
- Don't build the payload in two places: `build_payload` and `payload_for` drifted. The fed-back states need the optimizer result's own `juice_info`, not the histogram's.
- Don't write `double_balls` or clear reset markers on the stored upgrades while building a payload: Income's runs did it too. Only `build_payload` clears reset markers.
- Don't append the ids of a "Full" juice or book override to the tap's: replace them with every usable one, or an id appears twice.
- Don't ask for the gold distribution on every histogram: it made each run five to six times as long (`Calculator core/Evaluation pipeline tried.md`). Don't give `start_eval_hist` a parameter: `@change="start_eval_hist"` passes the event in.

## Workers

- Don't let an op throw unhandled inside the worker's async handler: a wasm panic then never reaches `onerror`, the histogram bundle stays "busy" for the rest of the page and an `OptimizerJobs` slot stays taken. Catch, post `{type: "error"}`, terminate the worker and call back with `null`.
- Don't queue a hash twice (Income's selected snapshot is also one the estimate uses): both ran at once and the second never freed its slot.
- Don't store a finished run's result into only the job that ran: another job of the same hash stays hollow and its re-queue runs the optimization again.
- Don't let an interim state replace `runtime.adv_cache`: Rust sends it empty, so advanced distributions were recomputed during every run.
- Don't post progress only together with a state, or from a timer per chain: short runs went 0.01% then 100% and a heavy Serca run sat at 24% for 50 s.
- Don't show 100% when the optimizer's result arrives: the histogram's gold lands 150 to 330 ms later.
- Don't leave prices out of `Calc.vue`'s watch: a market fetch landing after mount left the page and `optimizer_cache` on the old prices without a run, and the next reload missed the hash.
