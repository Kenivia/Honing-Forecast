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

## After adoption: what it did to the lab's data

Checked at a2d2c0f2 with snapshot `cut_a`. Tables are in `results/`: `rescore_head.txt`, `movers_a.txt`, `cut_cause.txt`, `reach_only.txt`.

**Stored metrics stay valid.** Every stored state (414,624 over 918 steps) was rescored at HEAD's `Tuning::reference`, on a copy of `runs.sqlite`, and compared with its stored `metric_ref`.

| Check | Result |
|---|---|
| Largest move | 2.23e-6 of flow (`single_0031_serca_n36`) |
| p99 | 5.2e-8 |
| Over 1e-5 | 0 |
| Hit (1e-4 of flow) changing either way | 0 of 414,624 |
| Largest move of a step's best known | 3.9e-7 |

`gross` moves up to 3.4e-4 of gross, and no lab tool reads it.

**Search on average does not move.** Batch `a` (`cut_a`), each variant against its stored twin, in mean % of flow. Wins and losses are over 0.01%:

| Variant | real_test | income | quick_dev | speed_test | large | wins / losses, all 918 steps |
|---|---|---|---|---|---|---|
| `solve` 48k | -0.0000 | +0.0000 | +0.0007 | +0.0000 | +0.0033 | 8 / 3 |
| `solve` 96k | +0.0000 | -0.0000 | +0.0003 | +0.0000 | +0.0041 | 8 / 2 |
| `price` 1 | -0.0000 | -0.0000 | +0.0002 | -0.0006 | +0.0061 | 6 / 4 |
| `surrogate` 1 | -0.0000 | -0.0000 | -0.0002 | -0.0000 | -0.0020 | 1 / 4 |
| `memetic` 1 | -0.0000 | -0.0001 | +0.0004 | +0.0000 | -0.0030 | 6 / 9 |
| `cem` 1 | +0.0002 | -0.0001 | +0.0008 | +0.0040 | -0.0146 | 18 / 13 |
| `lns` 1 | +0.0000 | +0.0001 | +0.0021 | +0.0012 | -0.0270 | 14 / 5 |

No sign test passes 5% (the lowest is `lns`, p=0.06, leaning to the new cut). `real_test` has 0 wins and 0 losses on every variant. The large rows on `large` are 10 payloads with bimodal seeds.

CPU against the twins is x0.75–1.07, mostly x0.85–0.95, lowest on `large`. That crosses sessions, so it is only a rough guide.

**But the cut does steer the search on a few payloads, and upwards.** Same build, 10 seeds, production cut against `special_tol=1e-7`:

| Payload | Algo | Hits, production | Hits, 1e-7 | Mean gap, production | Mean gap, 1e-7 |
|---|---|---|---|---|---|
| `single_1033_serca_n21` | `lns` | 10/10 | 0/10 | 0 | 0.098 |
| `b10_single_1033_serca_n21` | `lns` | 10/10 | 2/10 | 0 | 0.083 |
| `b10_chain_0017:2` | `solve` | 6/10 | 0/10 | 0.060 | 0.158 |
| `b10_single_3300` | `solve` | 4/10 | 1/10 | 0.095 | 0.198 |
| `b10_chain_0017:0` | `solve` | 8/10 | 6/10 | 0.005 | 0.053 |
| `single_3300` | `solve` | 9/10 | 9/10 | 0.011 | 0.011 |
| `single_0071_serca_n19` | `price` | 3/10 stuck at 1.2 | 1/10 stuck at 1.2 | 0.385 | 0.143 |

- These gaps of 0.1% are far over the cut's own error of 1.8e-5, so it is the path that changes, not the score.
- The `single_0071` trap is at both cuts, so it is not caused by this.
- `single_1033` at `lab_r1` is 0.098 on all 3 seeds, so 1e-7 does stand in for the old cut.

**It is not the reach of order moves.** A build with the production cut and the old reach (`cut_x`) gives the same result on all 12 payloads. A probe found the two reaches differ on 0 of 73,747 moves (`b10_chain_0017`) and 1 of 29,669 (`single_1033`). The high end of the outcomes stops at the budget before it gets small; what gets cut is the low end, the outcomes where almost nothing is skipped. So the effect is through the weights or the responses, and that part is not pinned down. Not tested: with the low end cut, an upgrade near the front of the order may cost nothing to the optimizer, so moves there are neutral and the search can drift across them.

**What this does to earlier claims.** Gains measured against `lab_r1` on order-basin payloads are somewhat high, since `solve` now reaches part of them itself. The stored kick loop against each `solve`:

| Set | vs `lab_r1` | vs `cut_a` |
|---|---|---|
| quick_dev | +0.0023 (14 wins, 0 losses) | +0.0016 (12 wins, 1 loss) |
| large | +0.0121 | +0.0089 |
| real_test | +0.0056 | +0.0056 |
| speed_test | +0.0007 | +0.0007 |

The kick loop itself has not been run at the new cut, since `price/final.patch` no longer applies at HEAD.

## Files

`final.patch` is the experiment as it ended against HEAD (the knobs, `special_dump`, the DP timer), not a proposed change. `scripts/` has the dump, the analysis and the batches, with paths to the scratchpad of the session. `results/` has the printed tables.
