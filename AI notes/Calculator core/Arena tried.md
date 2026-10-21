# Arena: what was measured

What past arena sessions measured, and the snapshots they used. Only read this when trying something new, or to find which session of `arena_data/arena.sqlite` holds a result. All of it is October 2026. The optimizer's own results are in `Optimizer tried.md`, the evaluator's in `Evaluation pipeline tried.md`.

## Snapshots

- **`baseline`** is the code at `69fef473`, before the evaluator's speed-up.
- **`v0`** is the code at `2dacbf9a`, the optimizer before the round of October 2026: one chain of 24,000 evaluations and a pass of responses at the end. It is the frozen baseline `diff_report.py` compares against.
- `v1` and `v2` are snapshots of the defaults at the end of that round; `v2` differs only in `carry_stall`. `v6` is the defaults as handed over, `v7` that with an id counted once a tap and the crossover kept to plans of one length, `v8` that with the cumulants fixed for hard tilts (sessions 97 and 98 have `v7`). The `k*` labels in those sessions are the steps on the way, with their diffs in the `snapshot` table.
- `chest_base` is the random chest state, `chest_moves` the first chest moves, `adv_table` the faster advanced distributions and their table.

## Sessions

| session | what |
| --- | --- |
| 14 | `explore`, 8 seeds, `baseline` |
| 20 | `explore`, 8 seeds, the evaluator sped up |
| 18 | `pilot`, 3 seeds, `--jobs 1`: `baseline`, `current`, and `current` with the memo off and with the old root finding too |
| 16, 19 | `stress`, 1 seed, `--jobs 2`: `baseline` and `current` |
| 21 | `pilot`, 3 seeds, `--jobs 1`, with the advanced honing table: 13.0 CPU seconds a seed against 21.0 |
| 23 | `explore`, 8 seeds, `adv_table`. Every run has the state and metric of session 20; 13468 CPU seconds became 5867, in 59 minutes |
| 25 | `pilot`, 3 seeds, `--jobs 1`, after the optimizer's own work was cut: 11.0 CPU seconds a seed |
| 28 | the `chests` folder, 1 seed: `chest_base` against chest moves and polishes |
| 29, 31, 32 | the `chests` set, 3 seeds: variants of the chest moves. In 31 the compound variants died, a bug of theirs |
| 30 | the `chests` set, 1 seed, 96,000 evaluations: the best known to measure against |
| 33 | the new `pilot`, 3 seeds, `--jobs 1`: `chest_base` 7.6 CPU seconds a seed, `chest_moves` 7.9 |
| 42 to 46 | responses |
| 47, 48 | streak lengths |

The optimizer round, all cold at 16 jobs unless said:

| session | what |
| --- | --- |
| 50 | `nh_dev`, 8 seeds: `v0` at 24,000 and at 48,000 evaluations |
| 51, 52, 54 | `nh_dev`, 4 seeds: single knobs, the free tap order's fixes, chains |
| 55, 57, 61 to 64, 67, 68 | `nh_dev`, 4 seeds: mixed responses, leaving early, merge, exchange, chains by size, the streak fix |
| 70 | `nh_dev`, 4 seeds: `v1`, the new defaults, and other chain lengths |
| 73 to 76 | `v0` against `v1`: `nh_test` (8 seeds), `explore` singles 100 to 199 (3), the `chests` set (1), `nh_quick` at a streak length of 10 (4) |
| 59, 60, 65, 66, 71, 72, 77 | `weekly_dev`, 2 seeds, cold and chained: what a carried plan is worth and when to stop |
| 79, 81, 95 | `weekly_test`, chained against cold |
| 91 | the `seeded` folder, 4 seeds |
| 92 | `weekly_test`, 1 seed: leaving at 3 that agree |
| 94 | `nh_test`, 8 seeds, the defaults |

Sessions 3 to 5 are of the payloads before the folder was generated again and match nothing in it now. Session 17 is session 20 with a root finder bug that killed 3 of its jobs; 15 is 18 before that fix. The re-planning run is session 8 of the `replan` table.

Runs recorded before 16 October 2026 were scored at the default of the time until `rescore` (`ref_reference` null); the database had some 196,000.

**`explore` was generated again after the evaluator fix**, which moved the owned amounts of two thirds of the singles and sweeps by a unit or so, which is why the whole folder was run again and not only its chains.

## The pilot set

Two scenarios without chests are from the pilot of before (a chain cut to four steps and a single, picked from session 14), and their runs match sessions 18 to 25 seed for seed. Seven have chests (picked from session 29): optional materials only, almost no base materials over 8 stacks, mostly base materials, Serca fifths with a free tap choice and without, and two chains cut to three steps, one with 8 stacks and one where a chest step adds a stack. The three that were dropped (`chain_0057_serca_n8`, `single_0466_t4_n10`, `single_0085_t4_n17`) are still in `explore`.

## The advanced honing table

`explore` has 121 start positions, 3 of them fresh. The `pilot` runs of session 21 have the metrics of session 18, all 33. Before the table, on payloads that are mostly advanced honing the distributions were three quarters of a solve or more.

## The evaluator before and after its speed-up

Against `baseline`:

- **CPU time with one job at a time.** `pilot`: 58.0 seconds to 21.0 (29.6 with `max_memo_size=0`, 33.8 with `taylor_skip=0` and the biased root unseeded as well). `stress`: 1517 to 274. Payloads that are mostly advanced honing gain least (`single_0085_t4_n17`: 5.8 to 4.6), because half of what was left of a solve was `compute_adv_dist_wrapper` on a cache miss.
- **The same search.** With the memo off, most `pilot` payloads find their best at the same iteration with the same gap as `baseline`, seed for seed. The memo changes the 9th digit and the search with it.
- **The metric state by state** (`bench --out` and `bench_diff.py`), over the given state, two scrambles and the best and worst found of every `explore` payload, 11465 states: 99.6% within 1e-6 of the baseline's metric, 10 further than 1e-5, none further than 1e-4. The 3 furthest move as far with the old root finding, on rounding alone: they are thresholds deep in a tail, where the root finder's tolerance on the derivative leaves the root loose. On all of them the saddlepoint's own error against `max_lattice_work` at 2e9 is 10 to 1000 times the movement.
- **`stress`** on 100 singles: the table of calls by error bucket is the same but for a handful of calls on two rows. **`verify --set juice`**: the same 13 failing rows, largest 66 standard errors, no row changing side.
- **A full pass.** Over all of `explore`, 8 seeds at 16 jobs (sessions 14 and 20): hit rate 77.5% and 77.5% (the difference by payload is +0.04 points, within 0.5 either way), the new best better by more than 1e-4 on 86 payloads and worse on 75, no job died in either, and 39147 CPU seconds became 13468.

`decide_s` was 29% of `pilot` at one job and 19% of `explore` in session 23, and 12% of `pilot` after the optimizer's clean-up.

## How far the default evaluator is from the exact one

With `max_lattice_work=2000000000` as the reference, which agreed with 100k-sample Monte Carlo on every total of `nh_dev` and `weekly_dev` but for rare events under one gold. Error as a share of the step's best known `flow`. This was measured while the metric still had the saddlepoint in it:

| states | over 1e-4 | over 1e-3 | worst | optimistic, of those over 1e-4 |
| --- | --- | --- | --- | --- |
| `nh_dev`, given and 3 scrambles (632) | 18.5% | 2.5% | 0.49% | 82% |
| `nh_dev`, best known plan (158) | 12.0% | 1.3% | 0.27% | 95% |
| `weekly_dev`, given and 1 scramble (634) | 14.4% | 1.1% | 0.16% | 100% |
| `weekly_dev`, best known plan (317) | 20.2% | 3.2% | 0.22% | 97% |

- On `nh_dev` optimized plans are no worse than random ones; on `weekly_dev` somewhat worse, so the optimizer leant a little towards the error there. Where best known and v0's seed 0 differ (54 and 159 steps), the exact evaluator orders them the other way on 3 and 4, each by under 2e-5 of flow.
- In a weekly chain it grows as the chain goes on: 9% of states over 1e-4 in weeks 0 to 4, 27% in weeks 25 to 29.
- Not a matter of the number of upgrades: 13% over 1e-4 at 4 to 7 upgrades, 23% at 25 and up. Red, Fusion and Serca Red carry it.
- **Per material it is how the variance is spread.** All of it is on saddlepoint materials; lattice and brute force are exact. Taking (sum of variances)^2 / sum of squared variances as the effective number of upgrades: 9 to 11% of materials over 1e-4 at 2 to 3, 5 to 7% at 3 to 5, under 0.5% at 5 to 8, none past 8. The raw count of upgrades in the material hardly matters.
- **The lattice work of those materials** (over 1e-4) is a median of 1e7 (weekly) to 3e7 (`nh_dev`), 50 to 150 times `max_lattice_work`, and up to 2.7e9. A limit of 3e7 would take 52 to 82% of them and 1e9 all but a few, but those limits also take 67% and 95% of every saddlepoint material.
- The exact evaluator is slow: 790 states took 40 minutes on 8 threads and 1585 weekly ones 87 minutes on 12, against seconds.
- Scripts were in the session scratchpad only; `verify` with the two variants and `bench --states --out` under each tuning reproduce it.

That is what led to the metric dropping the saddlepoint (`Evaluation pipeline tried.md`).

## verify failures that were the formula

- **`verify` on the `juice` set fails 13 rows, and they are the saddlepoint formula, not the simulation.** The worst are juice rows (`Glacier's Breath`, up to 66 standard errors). On those states the simulation agrees with an exact sum over the evaluator's own supports, and the evaluator does not: the material has 3 or 4 upgrades with more than one outcome, just over both `max_brute_size` (65,400 against 50,000) and `max_lattice_work` (234,000 against 200,000), so it gets the saddlepoint formula, which is 0.01 to 0.06 off there. That is multiplied by the threshold and the price.
- **`verify` on the weekly chain `no_root_week_inc_0022` is off by 1%**, on its first week too: Red, Fusion, Shards and the total of the given state and of scrambles, the evaluator cheaper than the simulation by 5 to 6 standard errors. It is the saddlepoint formula: with the free taps taken out it is still there (2% on one scramble), and with `max_lattice_work=2000000000` every row passes within 0.4 standard errors. The payload is one weapon from +8 to +24, so a few high levels make most of the spread of the sum. As a share of flow it is 0.12 to 0.28%; the 1 to 2% is of the total, which credit makes small.
- Both were measured with the saddlepoint in the metric; not run again since.

## Regressions

Seed 2 of the first payload of `arena_data/scenarios/regressions` used to panic, and every seed of `no_root_week_inc_0022` (the second week of a weekly chain) did until the root finder's failure stopped being one.

## Errors marked by hand

An experiment that could put the same id twice on a tap was scored well above any real plan; its 6 runs were marked as errors by hand.

## What re-planning is worth

Session 8: the `replan` set, 36 singles of 5 to 14 upgrades, 30 paths, every upgrade, 24000 iterations. The gain averaged 0.12% of flow over scenarios, median 0.02%, the largest scenario 0.64% and the largest single path 3.7%. As a share of what the optimizer gains over the default state: mean 4%, median 0.3%. The control was zero everywhere and the exact gain was 0.99 of the approximate one. A third of the scenarios had no gain at all; the gain was near zero where most is owned (wealth above 0.75) and did not go to zero where little is.

## Serca books in the scenarios (2026-10-21)

The October 2026 update brought Serca books, juice ids 2 to 5 of `Serca Oct 2026.json`, and `MaterialTable::new` asserts one label per juice, so the 755 Serca scenarios (1,857 payloads) no longer loaded. `scripts/arena/add_serca_books.py` added the four labels to each, none owned, market 15,000 and taxed 14,250. Every Serca payload is therefore a new case with no best known; their history before that date is of payloads without books.
