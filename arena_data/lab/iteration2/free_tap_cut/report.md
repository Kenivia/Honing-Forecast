# The free tap cut (`SPECIAL_TOL`)

**Adopted 2026-10-31:** the scaled cut at 1e-4 is now production (`skip_weights`), with `Tuning::reference` at the floor of 1e-11. The rest of this report is the investigation as it stood before that, so "production" below means the old cut.

**Verdict.** 1e-7 does not mislead the optimizer: its error is about 500 times under the evaluator's at p99, and no order move worth over 2e-8 of flow changes sign. It is finer than it needs to be, but the saving from raising it is small (about 5 to 10% of CPU), because a state has about 6 live outcomes and only 1 or 2 are cut. The best cut found folds what it drops onto the nearest outcome kept, with the threshold scaled by the mass outside the most likely outcome: `tol * (1 - p_max)`, tol 1e-4. A flat 1e-4 is not safe. No finer pass at the end is needed. Nothing was changed in production.

All errors are shares of flow. Runs were cold unless noted, in a copy of HEAD 8444a44a, scored by one reference (64 cells, cut 1e-11 with fold), so the variants share a scorer. The data is in a scratch database that was not kept; `results/` has the printed tables.

## What the code does

- `compute_special_probs` is an exact DP over order and budget. `live_skips` then keeps outcomes with `p >= SPECIAL_TOL`, which is a drop, not a renormalisation: the weights no longer sum to 1, so the metric loses `tail mass * G`, not `tail mass * (G(k) - G(edge))`.
- `eliminate_tail(actual_out.len()..0, ...)` is an empty range, so the high end is never merged.
- The low end's merge leaves its mass at most `SPECIAL_TOL` in one entry, which `live_skips` then filters unless the sum passed the cut, so it does almost nothing.
- The cut reaches the optimizer in three more ways: the `reach` of order moves (`neighbour.rs`, `order_polish`), `shadow_prices` and `Response::of`.
- `G(k)` does not depend on the weights, so a cut applied where the weights are read needs no new memo key or cache key.

## Offline: every cut over 22,088 states

From `p_k` and `G(k)` of 1,023 payloads with more than one live outcome (best known, given, scrambles, order moves), in `real_test`, `real_income_test`, `quick_dev`, `speed_test`, `free_taps`, `large`, `advanced`, `juice`, `small`, `chests`. Order moves: 12,053 pairs.

The evaluator, 8 cells against 64, for scale: p99 4.0e-5, worst 1.5e-4. On order moves: p99 1.9e-5, worst 2.9e-4, 12 sign flips, the largest on a true delta of 7.5e-6.

| Cut | p99 | Worst | Worst, real payloads | Outcomes, all / real | Flips on deltas over 1e-5 | Largest flipped delta |
|---|---|---|---|---|---|---|
| now, 1e-7 | 7.6e-8 | 2.3e-6 | 9.7e-8 | 1.00 / 1.00 | 0 | 1.9e-8 |
| now, 1e-5 | 1.1e-5 | 3.1e-3 | 1.1e-5 | 0.93 / 0.90 | 0 | 6.6e-6 |
| fold 1e-5 | 1.7e-6 | 8.1e-4 | 1.7e-6 | 0.93 / 0.90 | 0 | 7.1e-7 |
| fold 1e-4 | 2.1e-5 | 5.5e-3 | 1.7e-5 | 0.87 / 0.82 | 3 | 7.1e-4 |
| **scaled 1e-4** | 3.4e-6 | **1.8e-5** | 4.0e-6 | 0.91 / 0.87 | 0 | 5.0e-6 |
| scaled 3e-4 | 1.4e-5 | 2.4e-4 | 2.0e-5 | 0.88 / 0.82 | 1 | 1.1e-5 |

- Folding beats renormalising and losing the mass, at every threshold.
- Keeping the mean of k as well (fold plus a linear correction) gains little at 1e-4, and on real payloads is worse at 1e-3. Dropped.
- A cumulative tail cut is about the same as a per-entry cut. Dropped.

**Why a flat cut fails.** Where free taps cover nearly everything, flow is small and the rare outcomes are the cost. `single_1007_serca_n5` has flow 54k and metric +17M, with p of 1.0 on the top outcome and 1.6e-4 below it. Cutting that tail removes much of the flow. `1 - p_max` is tiny exactly there, so the scaled cut keeps those outcomes.

## Solves

CPU is a ratio to the 1e-7 baseline in the same session at 16 jobs (except `--jobs 1` and `real_income_test`, see below). W/L counts payload means that moved over 0.01% of flow. Gap is the worst |optimizer's metric - reference| of a returned state.

| Set (seeds) | Baseline gap | fold 1e-5: CPU, W/L, gap | fold 1e-4: CPU, W/L, gap | scaled 1e-4: CPU, W/L, gap | scaled 3e-4: CPU, W/L, gap |
|---|---|---|---|---|---|
| `ft_high` (5) | 1.1e-4 | 0.96, 0/0, 7.9e-5 | 0.82, 0/5, **1.2e-2** | 0.99, 0/0, 1.5e-4 | 0.95, 0/0, 6.2e-5 |
| `quick_dev` (3) | 1.0e-4 | 0.95, 4/8, 1.0e-4 | 0.90, 2/8, 1.4e-4 | 0.95, 5/4, 1.0e-4 | 0.91, 1/9, 1.0e-4 |
| `speed_test` (3) | 8.8e-5 | 0.96, 0/0, 8.8e-5 | 0.88, 1/1, **9.5e-4** | 0.94, 0/0, 8.8e-5 | 0.94, 2/1, 8.4e-5 |
| `real_test` (5) | 3.3e-5 | 0.94, 0/0, 3.3e-5 | 0.93, 0/0, 3.3e-5 | 0.95, 0/0, 3.3e-5 | 0.94, 0/0, 3.3e-5 |
| `real_income_test` chained (3) | 4.9e-5 | 0.95, 0/0, 4.9e-5 | 0.90, 0/0, 4.9e-5 | 0.87, 0/0, 4.9e-5 | 0.82, 0/0, 4.9e-5 |
| `cpu_check` (2, `--jobs 1`) | 4.8e-5 | 0.91, 0/0, 4.8e-5 | not run | 0.94, 0/1, 4.8e-5 | not run |

`ft_high` is 50 singles of this investigation, where free taps skip 80% or more of the normal upgrades at the best state. Its `fold 1e-4` loses up to 0.65%.

- The scaled rows of `real_income_test` come from a later session than their baseline, so their CPU is less sure; their states and the metric are compared the same way.
- `--jobs 1` cannot separate 0.91 from 0.94. Ticks and random streams are that size.
- `quick_dev` wins and losses are mostly noise. All variants of a session are compared with one baseline run, so a lucky baseline seed on a noisy payload shows as a loss for all of them. `b10_single_3300` and `b10_chain_0017` swing by 0.1 to 0.2% under every cut, including `fold 1e-5`. This is why scaled 3e-4's 1/9 is inconclusive, not a proven loss, though it gives no reason to go past 1e-4.
- One `free_taps` loss at `fold 1e-4` (`single_0102_t4_n9`, -0.35%) was one seed of 5, on a payload where nothing is cut at the best state: a random stream miss.

Evaluation alone, memo off, one thread, flat fold only (`--jobs 1`): 1e-5 takes 0.89 to 0.93, 1e-4 takes 0.84 to 0.93, 1e-3 takes 0.77 to 0.83. The scaled cut was not timed this way.

## A finer pass at the end, and the UI

- **Final polish: not needed.** Under the scaled cut and `fold 1e-5` the returned gaps match the baseline's, and the runs show no quality loss to win back. Not tested as a variant.
- **UI: cheap and worth doing with the change.** `Tuning::reference` can carry its own threshold, since the histogram already swaps it in and the cut is on the read side. All scoring here ran that way, at 1e-11 with fold. 1e-8 costs 1.02 times the outcomes. The histogram path itself was not run.

## The DP

Its share of a solve, one thread, `bench --solve`: `real_test` 2.3% (worst 6.2%), `speed_test` 7.5%, `free_taps` 27.7%. Nearly all of the last is one payload: 48% of `single_0071_serca_n19` (budget 46,865, 923 us a call). The worst seen is 69%, and `single_1134_serca_n24` (budget 61,456) is at 62%. It matters only past a budget of about 25,000. No real payload in `real_test` is near that (the largest there is about 6,300). No speed-up was tried. The cut does not help, since it acts after the DP.

## The checks

- With the cut as production has it, the copy repeats HEAD: 36 of 36 states on `free_taps`, 60 of 60 on `small`. 3 states differed at first. HEAD's merge could make `reached` exactly 1.0, and `random_bool(1.0)` draws no random number, so the stream shifted; replaying the merge there gives 36 of 36. Anything that removes the merge shifts the stream on those payloads and changes nothing else.
- Later builds of the copy repeat each other: 1,942 of 1,942 states.
- 27,457 runs, 27 of them errors, all in the first `head` runs: the pristine copy had no advanced table. Those runs were done again.

## Files

`final.patch` is the experiment as it ended against HEAD (the knobs, `special_dump`, the DP timer), not a proposed change. `scripts/` has the dump, the analysis and the batches, with paths to the scratchpad of the session. `results/` has the printed tables.
