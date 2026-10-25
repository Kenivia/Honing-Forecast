# Arena: what not to try

Warnings about running the arena and reading its results, behind `Arena.md`. The optimizer's own are in `Optimizer tried.md`, the evaluator's in `Evaluation pipeline tried.md`.

## Running it

- Don't run a long job with the default `--jobs` or time anything at more than one job: running many workers at once inflated CPU time about threefold. Counters are the only thing comparable across sessions at 16 jobs.
- Don't iterate on a set that takes too long: a full `opt_dev` pass was too slow to iterate on, hence `quick_dev` (about 2 minutes a variant and seed at 16 jobs) and `pilot` for a first look. A full `explore` pass at 8 seeds and 16 jobs takes hours; benchmarks run beside it slow both.
- Don't resume a run with different arguments: `--resume` only reuses the latest session started with exactly the same ones.
- Don't judge a snapshot's speed against a build with a different advanced honing table: a snapshot from before the table works out the distributions inside its time (21.0 CPU seconds a seed on `pilot` against 13.0 with the table; payloads that are mostly advanced honing were three quarters distributions or more).
- Don't leave `arena_data/adv_cache.sqlite` after a change to `compute_adv_dist`: nothing notices a stale one.
- Don't copy a new driver over the one a run is using; run the driver from a copy (`target/arena/driver.exe`) while building.

## Reading results

- Don't read a hit rate across days: best known moves whenever a variant finds something new. Compare with a frozen snapshot (`diff_report.py`).
- Don't trust a mean gap over flow: a few payloads with almost no flow rule it (millions of percent). Read the median, 90th percentile and hit rate.
- Don't believe gaps where everything is owned: the true metric is zero there and the optimizer chases approximation error (best states with a few thousand gold of negative cost on jobs moving millions).
- Don't count a run with an invalid state as a best: a state that puts the same id twice on a tap is scored well above any real plan, so such runs are marked as errors by hand (`error` starts with `invalid state`).
- Don't trust a run recorded before the reference scoring (before 16 October 2026): run `pnpm arena rescore` first.
- Don't keep results across an evaluator change for generated payloads: the quantiles that size owned amounts are read off the evaluator, so `explore` regenerated after the evaluator fix moved the owned amounts of two thirds of the singles and sweeps by a unit or so; it was all run again.
- Don't expect a payload edited in place to keep its history: best known is per payload hash.

## Checking the evaluator

- Don't chase a failing `verify` row as a simulation bug when the material is a saddlepoint one: `verify` on the `juice` set failed 13 rows (up to 66 standard errors, Glacier's Breath) because the material had 3 or 4 upgrades just over both `max_brute_size` and `max_lattice_work` and got the saddlepoint formula, 0.01 to 0.06 off. `no_root_week_inc_0022` was off by 1% likewise, and every row passed with `max_lattice_work=2000000000`. Measured while the saddlepoint was in the metric; not run again since.
- Don't use the exact evaluator (`max_lattice_work=2000000000`) for broad checks: 790 states took 40 minutes on 8 threads, 1585 weekly ones 87 minutes on 12.
- Don't assume the old default evaluator was close: against the exact one, 12 to 20% of states had an error over 1e-4 of flow (worst 0.49%), mostly optimistic, growing along a weekly chain, and tied to how the variance is spread (few effective upgrades) rather than the count of upgrades. That is why the metric dropped the saddlepoint (`Evaluation pipeline tried.md`).

## Scenarios

- Don't load a Serca payload without the four Serca books: `MaterialTable::new` asserts one label per juice. `scripts/arena/add_serca_books.py` adds them (none owned, market 15,000, taxed 14,250), which makes every such payload a new case with no best known.

## Re-planning

- Don't expect re-planning to be worth much: on 36 singles of 5 to 14 upgrades the gain averaged 0.12% of flow (median 0.02%), 4% of what the optimizer gains over the default state (median 0.3%). The control was zero everywhere and the exact gain was 0.99 of the approximate one. A third of the scenarios had no gain; it was near zero where most is owned and did not go to zero where little is.
