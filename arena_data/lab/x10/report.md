# Forced x10 streak: can `solve` be faster there?

**Verdict: (pending batch `a`).**

**The question.** At a streak length of 10 (the "Force x10 streak length" checkbox, on by default for new profiles), the state space is far smaller. Does `solve` almost always reach the best result there, how long does it take, and can it take less time while still reaching it?

All numbers are in % of flow. **Hit** is a run within 0.01 of the best known at streak length 10; **exact** is within 1e-4.

## 1. What stored runs already say

From runs already in `runs.sqlite`, before any new batch: the 387 payload steps whose `min_resolution` is 10. These are 137 `b10_` generated steps, 75 choices stars, 19 real singles and 156 real income steps. They come from `pr_a`'s `ra` batch (`solve` 6k–96k) and the 384k references.

| Streak-10 kind | Budget | Hit | Exact | Worst | Evaluations | CPU s a run |
|---|---|---|---|---|---|---|
| choices (real stars, cold) | 12k | 96.9 | 92.9 | 0.040 | 12,834 | 0.83 |
| | 24k | 100 | 98.7 | 0.007 | 18,081 | 1.22 |
| | 48k | 100 | 100 | 0 | 27,367 | 1.98 |
| real income (chained) | 12k | 98.1 | 90.0 | 0.040 | 13,429 | 3.43 |
| | 24k | 99.8 | 94.0 | 0.012 | 20,112 | 5.06 |
| | 48k | 99.8 | 96.4 | 0.012 | 25,952 | 7.17 |
| `b10` (generated, cold) | 24k | 91.7 | 85.9 | 0.372 | 16,940 | 2.20 |
| | 48k | 95.1 | 88.1 | 0.367 | 24,042 | 3.32 |
| | 96k | 96.1 | 90.3 | 0.367 | 35,202 | 5.43 |
| real singles (cold, 10 seeds) | 24k | 91.6 | 82.6 | 1.084 | 21,966 | 2.86 |
| | 48k | 94.2 | 81.6 | 0.646 | 32,691 | 4.31 |
| | 96k | 95.8 | 85.8 | 0.517 | 51,308 | 7.83 |

- **On the real pages' payloads, yes:** `solve` 48k hits every choices star and 99.8% of real income steps, and 24k already does as well.
- **On generated and real singles, not quite:** about 95% at 48k. The misses are a few known hard payloads, not a spread:
  - `b10_single_3300` (one seed in three at 0.367);
  - `b10_chain_0017:3`, `b10_week_inc_0004:3` (0.09–0.15 on a seed);
  - `b10_chain_0001:1–3` (0.01–0.06 on most seeds);
  - `real_27a2bc40` (0.08–0.65 on 6 seeds of 10).

  These are the same free tap order and joint re-split basins `solve` misses at streak length 1 (`iteration2/report/report.md` 4). A streak of 10 does not make them easy.
- **Time follows payload size:** `b10_single_0161` (39 upgrades) takes 77 s a run, `b10_single_0141` (34) 23 s, a 10-upgrade single 2–5 s. Runs end early: 24–33k of the 48k evaluations.

## 2. Batch `a`

Snapshot `x10_a` (the `fin_a` build; it repeats `fin_a` on `small`). Every variant at `streak_length=10`, so that payloads at streak length 1 run at 10 too. The best known for these is taken over streak-10 runs only.

| Set | Mode | Seeds |
|---|---|---|
| choices_test | cold | 3 |
| real_test (all 38 at 10) | cold | 10 |
| speed_test10 | cold | 3 |
| `large` (at 10) | cold | 3 |
| real_income_test (all at 10) | chained | 3 |

| Variant | What it asks |
|---|---|
| base | `solve` 48k |
| 192k | the reference: `solve` longer |
| KMe | the reference: kick loop + `mem_polish` + `rebuildempty` |
| 24k, 12k | a lower cap on evaluations |
| c125, c60, c30 | shorter chains: `chain_iters_per_upgrade` 125/60/30, `chain_iters_min` 2000/1000/500 |
| a3 | leave sooner: `agree_exit=3,unanimous=2` |

(Results pending.)
