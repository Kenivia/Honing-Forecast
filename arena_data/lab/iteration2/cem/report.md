# `cem` investigation

**Verdict: take no piece of its model. Its strength is its restarts, and its large losses were mostly one cap in its order polish.**

- **What cem really is:** many restarts from one cheap start. Each restart is a short descent by partial redraws, kept only if better, followed by a polish.
- **The model carries nothing.** With both the refit and the piece-type factor off, it is level overall: quick_dev -0.0071 against -0.0078, real_test -0.0054 against -0.0077. Each moves single payloads both ways.
- **The restarts carry it.** With every cycle about the best (`fresh` off), it is worse on every set: quick_dev -0.0223 with 59 losses, against -0.0121.
- **The order polish was capped at 10 places, and that was most of its loss on `large`.** Uncapped (`fullreach`):
  - `large` goes from -0.130 to -0.029 (hit 33 → 47), quick_dev from -0.0121 to -0.0078, `free_taps` from -0.0097 to -0.0029, at less CPU;
  - `single_0031_serca_n36` goes from 0.368 on every seed to 0, and `single_0161` from 0.60 to 0.07.
- **Even uncapped, it is behind `solve`:** at effort 2 it is still quick_dev -0.0042 and `large` -0.018, at x1.73 / 1.09.
  - Its remaining losses are `chain_0017` steps 0–2, `real_08927991` (advanced with normal plans), `single_0044` and `chain_1051`.
  - `solve`'s exchanges help a little (real_test -0.0042; `real_08927991` 0.196 → 0.067).
- **Its samples cannot improve a polished state.** From `solve`'s endpoint it gains quick_dev +0.0001, at x2.5–3.75 CPU.
- **What it reaches that `solve` misses, from its own start:** `b10_single_3300`, `b10_week_inc_0004:3`, `single_3397`, `single_3375` and `week_hone_0009:6`, at effort 1. The restarts carry these; the model, the order-prefix rebuild and the start do not.
- **`solve`, then cem's restarts from its own start (`fromsolve+ownstart`), loses nothing.**
  - At effort 0.5: quick_dev +0.0008 (8 wins, 0 losses), real_test +0.0033, x1.21–1.54.
  - Alone it is behind the kick loop with `mem_polish` and behind L.
  - It takes `b10_week_inc_0004:3` and `single_3375`, which the kick loop with `mem_polish` misses.
  - **Offline arithmetic, not a run** (each run's better result of the two): the kick loop with `mem_polish`, plus these restarts at effort 0.5, would give quick_dev +0.0021 (hit 98.1) at about x2.34. The kick loop with `mem_polish` plus L would give +0.0022 at x2.96.

Scripts are in `scripts/`:
- `dump.py`/`an.py` for the stored runs; `ra.py`, `curve.py`, `seeds.py`, `moving.py` and `state.py` (end states part by part) read them;
- `sgap.py` for a scratch db;
- `attr.py` for the trace;
- `rep.py` for repeat checks.

`an.py` needs `runs.pkl` from `dump.py`, which is not kept. All numbers are % of flow.

## 0. State of the work

| Round | Snapshot | Batch | Tests | Status |
|---|---|---|---|---|
| a | `cem_a` (cec43f6e + switches) | `iteration2/cem/a`, `a_income` | start (`fromsolve`, `shadow`), restarts (`fresh`), model (`refit`, `group`), polish (`solvepolish`, `advfull`, `fullreach`), the snap on chained runs (`keepgiven`) | read, section 3 |
| b | `cem_b` | `iteration2/cem/b` | on R = cem 1 with `fullreach`: effort 2, `oprefix` off, `refit+group` off, `exchange`, `shadow`, `fromsolve+ownstart` at efforts 1 and 0.5; kick loop + `mem_polish` and L in the same session | read, section 4 |

- **Repeat checks:**
  - Plain `cem_a` and `cem_b`, and cem 1, repeat `cut_a` on all 3,128 runs of the 9 sets.
  - R in `cem_b` repeats `cem_a`'s `fullreach` on all 3,128 runs.
  - The final code (smoke, scratch label `cem_c`): plain and cem 1 repeat `cut_a` (66 and 141 runs, `advanced` and chained included); `fullreach`, `exchange`, `fromsolve+ownstart` and `oprefix` repeat `cem_b`; `solvepolish` repeats `cem_a`.
  - `lab_algos_repeat_and_are_valid` passes.
- **Switches in `cem.rs`:**
  - `lab_off`:
    - `refit`: the model is never refit.
    - `fresh`: every cycle about the best, at the narrow spread.
    - `group`: no piece-type redraws and no shared front shift.
    - `oprefix`: no greedy rebuild of the order's front in the polish.
  - `lab_on`:
    - `fullreach`: no cap of 10 places on the order polish.
    - `fromsolve`: `solve`'s result as a candidate, raw and snapped; its cost is outside the budget.
    - `ownstart`: with `fromsolve`, `solve`'s result is the best to beat, and every cycle restarts from cem's own start.
    - `solvepolish`: `solve`'s polish in place of cem's.
    - `exchange`: `solve`'s `refine` after each polish.
  - Removed as failed: `evalfirst` (section 2), `keepgiven` and `advfull` (section 3), `shadow` (section 4).
- **Lab helpers in `simulated_annealing.rs`** (`solve` itself is unchanged):
  - `polish_counted`: `solve`'s polish with its evaluation count, for `solvepolish`.
  - `refine_counted`: `refine` with the evaluations the probe counted, response work left out, for `exchange`.
- **Trace,** at iter = phase (`scripts/attr.py`):
  - `cem_gain`, `cem_late` (gain after the first polish), `cem_wins` and `cem_cost`;
  - the phases are cand, shadow, fresh, narrow, p.resp, p.adv, p.oprefix, p.oinsert, p.chest, solvepolish and exchange;
  - plus `cem_budget`, `cem_cycles` and `cem_polishes`.

## 1. Where it stands (stored effort curve, `pr_a` batch `ra`)

Against `solve` 48k in the same batch, as diff (wins, losses over 0.01), hit %, and x CPU. `solve`'s own hit % is in brackets after the set. Price report 11 has the whole curve.

| set | effort 1 | effort 2 |
|---|---|---|
| real_test (96.6) | -0.0078 (1, 8), 80.5, x0.95 | -0.0019 (1, 3), 86.6, x1.90 |
| quick_dev (95.5) | -0.0121 (4, 39), 88.2, x0.92 | -0.0087 (4, 25), 90.3, x1.81 |
| speed_test (96.3) | -0.0097 (3, 26), 86.4, x0.78 | -0.0044 (4, 24), 88.2, x1.54 |
| large (53.3) | -0.1301 (0, 6), 33.3, x0.66 | -0.0997 (0, 6), 33.3, x1.28 |
| income (98.0) | -0.0017 (0, 14), 94.3, x1.30 | -0.0010 (1, 8), 96.2, x2.58 |
| advanced (94.4) | -0.0011 (0, 0), 88.9, x0.76 | -0.0003, 88.9, x1.49 |
| free_taps (97.2) | -0.0097 (0, 1), 83.3, x0.44 | -0.0069, 86.1, x0.84 |
| juice (94.4) | -0.0034 (0, 1), 86.1, x0.50 | -0.0001, 91.7, x0.97 |
| small (100) | 0, 100, x0.61 | 0, 100, x1.21 |

- **Its evaluations are close to `solve`'s** (x0.84–0.94 at effort 1, x1.36 on income), but its CPU is lower where responses are cheap (`free_taps`, `juice`, `large`).
- **`solve` 24k (x0.64) beats cem 2 on every main set but real_test.**

**Its wins are open joint-move payloads, reached from its own start** (mean gap over 3 seeds):

| payload | `solve` 48k | 96k | kick loop | cem 0.5 | cem 1 | cem 2 |
|---|---|---|---|---|---|---|
| `b10_single_3300` | 0.162 | 0.162 | 0.071 | 0.039 | 0.039 | 0 |
| `b10_week_inc_0004:3` | 0.064 | 0.035 | 0.064 | 0.007 | 0 | 0 |
| `single_3397_serca_n4` | 0.062 | 0.062 | 0.025 | 0.008 | 0.006 | 0.006 |
| `single_3375` | 0.047 | 0.033 | 0.047 | 0.020 | 0.017 | 0.006 |
| `week_hone_0009_serca_n21:6` | 0.023 | 0.023 | 0 | 0 | 0 | 0 |
| `b10_chain_0017:3` | 0.090 | 0.039 | 0 | 0.093 | 0.061 | 0.040 |
| `chain_0017:3` | 0.175 | 0.116 | 0 | 0.128 | 0.125 | 0.123 |
| `chain_1051:1` | 0.123 | 0.123 | 0.123 | 0.185 | 0.185 | 0.185 |

**Its losses:**

| payload | `solve` 48k | cem 0.5 | cem 1 | cem 2 |
|---|---|---|---|---|
| `chain_0017:0` | 0.020 | 0.729 | 0.562 | 0.476 |
| `chain_0017:1` | 0.030 | 0.467 | 0.331 | 0.227 |
| `b10_chain_0017:1` | 0 | 0.802 | 0.624 | 0.560 |
| `single_0161` (large) | 0.032 | 0.975 | 0.602 | 0.400 |
| `single_0031_serca_n36` (large) | 0 | 0.368 | 0.368 | 0.368 |
| `single_0044` (large) | 0.079 | 0.309 | 0.303 | 0.220 |
| `real_08927991_4514f9` | 0.002 | 0.277 | 0.196 | 0.143 |
| `single_1056_serca_n3` | 0 | 0.936 | 0.624 | 0 |
| `single_1208` | 0.015 | 1.856 | 0.335 | 0.142 |
| `single_3097` | 0 | 0.148 | 0.114 | 0.044 |
| `free_taps` `single_0071_serca_n19` | 0.001 | 0.614 | 0.107 | 0.079 |
| `juice` `single_0034` | 0 | 0.077 | 0.034 | 0.005 |

- **Most losses shrink with effort.** `single_0031_serca_n36` does not: it ends at 0.368 on every seed at every effort, a fixed point.
  - Its end states: two piece types. Armour juice is owned; weapon juice is bought at market.
  - cem's seeds split the armour juice differently and end on the same metric.
  - In every seed, every weapon upgrade whose plan differs takes 1–2 fewer taps of juice than in `solve`'s plan (`N1.5` 0 against 2, `N1.6` 3 against 4, `N1.7`–`N1.8` 6 against 7, `N1.11`–`N1.12` 15 against 17).
  - That is a joint shift of one piece type's fronts. cem's group factor exists for exactly this, yet does not find it.
- **`chain_0017:0/1`:** cem's free tap order differs from `solve`'s (seed 0 `[0,1,2,4,6,…]` against `[0,1,2,6,7,…]`), and its plans are plainer: fronts only, little juice on the first two upgrades, few back streaks or mixed ids.

## 2. Reading the code

1. **Given and carried plans are snapped at any streak length** (a flaw; plain behaviour kept).
   - `snap_to_block` rebuilds a plan from its streaks, so at streak length 1 too a plan loses its mixed ids and middle runs. `solve` snaps only when the streak length is above 1.
   - Every chained run starts from a streak copy of the run before.
   - In the first `fromsolve` smoke, the snapped endpoint lost about 2% of flow on `b10_chain_0017:2/3` seed 2. `fromsolve` now always keeps `solve`'s raw result.
   - Keeping every given plan unsnapped (`keepgiven`, removed) did not help on income (section 3).
2. **The polish's responses read a state that was applied but not evaluated.** `Response::of` refreshes the distributions and free tap probabilities itself, and evaluating first changed nothing (identical on 24 runs). The `evalfirst` switch is removed.
3. **The order polish is capped at 10 places** (`reach + 2`).
   - On `free_taps` `single_0071` the smoke with `fullreach` takes 0.107 to 0.026.
4. **The order polish moves the raw permutation, and the evaluator cleans it.** Advanced upgrades, and any upgrade placed ahead of a lower level of its own piece, go to the back.
   - Such moves cost an evaluation and change nothing.
   - Raw positions run ahead of cleaned ones, so the window is smaller than it looks.
   - The Plackett-Luce draws make many such orders. This wastes evaluations; it is not a wrong result.
5. **The advanced polish moves each slot by ±1 and ±2.** `solve`'s `adv_polish` tries every value, for 3 rounds. Switch: `advfull`.
6. **The start** is one round of linear responses at the shadow prices of the best given plan (on a cold run, the empty plan), in the given order. There is no greedy order and no averaging of price rounds, as `solve`'s `shadow_start` does. Half the cycles restart from this start, at spreads 1, 2 and 0.5. Switches: `shadow`, `fresh`.
7. **A cycle's result is polished only if its metric is new.**
   - A cycle that finds nothing leaves `cur` at its start, which was polished already.
   - From `solve`'s endpoint the samples never beat it, so nothing is ever polished (smoke: 0.5 polishes a run, sample gain 0.0000).
   - The samples are never polished themselves; only a sample that beats the incumbent moves the search.
8. **No exchange** of `solve`'s `refine`.
9. **Budget:** effort × (10000 + 1300 n), with responses charged their work. The stall ends cycles, not the run, so the budget always binds (spent/budget 1.00 in the smoke).
10. **What earns, from the trace** (smoke, `b10_single_3300`):
    - After the first polish, the gains are the order prefix rebuild (0.29 a run), order insertions (0.20) and responses (0.08). The samples earn nothing there.
    - Before it, a fresh cycle's samples lift the start (9.4 a run on top of 64 from the shadow response). This is a cheap descent, not a learned distribution.

## 3. Batch `a`: every piece at effort 1, and from `solve`'s endpoint

One session per set. Each cell is diff vs `cem_a` (`solve` 48k), with (wins, losses) where useful. CPU is against `solve` 48k in the same batch. M is cem at effort 1. Cold runs use 3 seeds (real_test 10); income is chained, 3 seeds.

| variant | real_test | quick_dev | speed_test | large | income | advanced | free_taps | juice | CPU quick_dev / large |
|---|---|---|---|---|---|---|---|---|---|
| M | -0.0078 (1, 8) | -0.0121 (4, 39) | -0.0097 (3, 26) | -0.1301 (0, 6), hit 33 | -0.0017 (0, 14) | -0.0011 | -0.0097 | -0.0034 | x0.93 / 0.67 |
| `fullreach` | -0.0077 (1, 8) | **-0.0078** (4, 36) | -0.0092 | **-0.0286** (0, 5), hit 47 | -0.0015 | -0.0011 | **-0.0029** | -0.0034 | x0.88 / 0.57 |
| `solvepolish` | -0.0084 | **-0.0060** (5, 30) | -0.0179 | **-0.0159** (1, 4), hit 53 | -0.0013 | -0.0024 | -0.0006 | -0.0032 | x1.25 / 1.08 |
| `shadow` | -0.0188 | -0.0103 | -0.0262 | -0.0799 | -0.0016 | -0.0010 | -0.0063 | -0.0021 | x1.01 / 0.75 |
| `fresh` off | -0.0112 | **-0.0223** (1, 59) | -0.0283 | -0.1443 | -0.0018 | -0.0037 | **-0.0717** | -0.0073 | x1.46 / 1.19 |
| `refit` off | -0.0082 | -0.0138 | -0.0065 | -0.1193 | -0.0016 | -0.0009 | -0.0368 | -0.0019 | x1.11 / 0.72 |
| `group` off | -0.0076 | -0.0131 | -0.0105 | -0.1617 | -0.0016 | +0.0010 | -0.0046 | -0.0016 | x0.90 / 0.67 |
| `advfull` | -0.0080 | -0.0129 | -0.0097 | -0.1254 | -0.0017 | -0.0025 | -0.0090 | -0.0034 | x1.01 / 0.75 |
| `fromsolve` | +0.0024 (1, 0) | +0.0001 (0, 0) | +0.0007 (5, 0) | +0.0019 (1, 0) | +0.0001 | 0 | 0 | 0 | x2.87 / 2.46 |

- **`keepgiven`** (`a_income`, against plain in its own session): income -0.0021 (0 wins, 15 losses) at x1.58, against M's -0.0017 at x1.35. Keeping the carried plan unsnapped does not help, and costs. The snap is a real flaw, but it is not why cem loses on income. Removed.
- **`small`** is 100 for every variant.
- **CPU per evaluation moves with the variant.** `refit` off costs x1.11 at x0.88 the evaluations, and `fresh` off x1.46 at x1.02, both with M's number of polishes (11–12 a run on quick_dev). Read CPU, not evaluations.

**Mean gaps on the payloads that move** (3 seeds; real_test 10):

| payload | `solve` | M | `fullreach` | `solvepolish` | `shadow` | `fresh` off | `refit` off | `group` off | `fromsolve` |
|---|---|---|---|---|---|---|---|---|---|
| `b10_single_3300` | 0.162 | 0.039 | **0** | 0.025 | 0.039 | 0.187 | 0.071 | 0.039 | 0.162 |
| `b10_week_inc_0004:3` | 0.064 | 0 | 0 | 0 | 0 | 0.029 | 0 | 0 | 0.064 |
| `single_3397_serca_n4` | 0.062 | 0.006 | 0.006 | 0.001 | 0.005 | 0.041 | 0.018 | 0.001 | 0.037 |
| `single_3375` | 0.047 | 0.017 | 0.017 | 0.049 | 0 | 0.045 | 0.022 | 0.016 | 0.047 |
| `week_hone_0009_serca_n21:6` | 0.023 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `chain_0017:3` | 0.175 | 0.125 | 0.125 | 0.116 | 0.245 | 0.226 | 0.139 | 0.152 | 0.175 |
| `chain_0017:0` | 0.020 | 0.562 | 0.646 | 0.205 | 0.646 | 0.521 | 0.532 | 0.229 | 0.012 |
| `chain_0017:1` | 0.030 | 0.331 | 0.331 | 0.217 | 0.331 | 0.248 | 0.376 | 0.365 | 0.030 |
| `b10_chain_0017:1` | 0 | 0.624 | 0.624 | 0.484 | 0.624 | 0.481 | 0.251 | 0.457 | 0 |
| `single_0031_serca_n36` | 0 | 0.368 | **0** | 0 | 0.368 | 0.368 | 0.368 | 0.368 | 0 |
| `single_0161` | 0.032 | 0.602 | **0.066** | 0.039 | **0.063** | 0.829 | 0.506 | 0.954 | 0.032 |
| `single_0044` | 0.079 | 0.303 | 0.198 | 0.141 | 0.371 | 0.278 | 0.255 | 0.287 | 0.079 |
| `single_3097` | 0 | 0.114 | 0 | 0 | 0.190 | 0.021 | 0.089 | 0.138 | 0 |
| `real_08927991_4514f9` | 0.002 | 0.196 | 0.196 | 0.194 | 0.126 | 0.080 | 0.247 | 0.157 | 0.002 |
| `chain_1051:0` | 0.063 | 0.190 | 0.190 | 0.190 | 0.190 | 0.190 | 0.127 | 0.190 | 0 |
| `single_1208` | 0.015 | 0.335 | 0.335 | 2.435 | 3.131 | 3.131 | 0.483 | 0.526 | 0.010 |
| `single_1056_serca_n3` | 0 | 0.624 | 0.624 | 0.312 | 0.624 | 1.240 | 0 | 0.312 | 0 |
| `free_taps` `single_0071_serca_n19` | 0.001 | 0.107 | 0.026 | 0.002 | 0.078 | 0.115 | 0.053 | 0.059 | 0.001 |
| `juice` `single_0034` | 0 | 0.034 | 0.034 | 0.021 | 0.028 | 0.006 | 0.026 | 0.041 | 0 |

Noise: `single_1208` and `single_1056` are bimodal under every variant. `free_taps` `single_0087` is too: 0 for M, 0.38 with `refit` off, 0.58 with `fresh` off. Those rows say nothing about the piece.

**The cap on the order polish.**
- **Most of cem's `large` loss was this cap,** not its plans.
  - The order polish tried each place up to `reach + 2`, but never past 10.
  - Uncapped, `single_0031` goes to 0 on every seed, `single_0161` from 0.602 to 0.066 and `single_3097` from 0.114 to 0. `b10_single_0031` and `b10_single_0161` follow, on quick_dev.
  - On `single_0031` (section 1), the weapon plans that took 1–2 fewer taps of juice were a consequence of an order wrong beyond place 10.
- **`solve`'s polish** (`solvepolish`) has no cap, and gets the same three plus a little more on `large` (-0.016) and quick_dev (-0.0060). It costs more (x1.25 / x1.08), and is worse on `speed_test` (bimodal `single_1208`) and `advanced`.
- **`solve`'s start** (`shadow`) fixes `single_0161` (0.063) the other way: its greedy order puts the right upgrades inside the reach from the start. It is mixed elsewhere: `large` -0.080, real_test -0.019 (`real_27a2bc40` 0.759), `speed_test` -0.026 (`single_1208`).

**The restarts carry cem; the model does not.**
- **With every cycle about the best (`fresh` off), cem is worse on every set.** It loses `b10_single_3300` (0.187), `single_3397` (0.041) and `single_3375` (0.045), and on `free_taps` `single_0087` and `single_0057`. It also costs more CPU (x1.46).
- **Refit off and group off are level overall,** moving single payloads both ways:
  - Group off: `chain_0017:0` 0.229, `advanced` `single_0085` +0.027 (`advanced` +0.0010), `large` `single_0161` worse.
  - Refit off: `speed_test` and `large` slightly better, `free_taps` `single_0087` worse.
  - Iteration 1 found the same for the refit at the old cut.
- **The trace says the same** (`fullreach`, all sets):
  - Gains after the first polish are small. On quick_dev, per run: responses 0.012, order prefix 0.010, the samples of later fresh cycles 0.003, narrow cycles 0.008, order insertions 0.004.
  - Each fresh cycle's descent from the start lands in another basin, and its polish settles it.

**Its samples cannot improve a polished state** (`fromsolve`).
- From `solve`'s endpoint, cem's cycles find almost nothing. Gains (no payload lost):
  - `chain_1051:0` 0.063, `single_3397` 0.025, `week_hone_0009:6` 0.023, `week_inc_0009:7/8`;
  - `real_27a2bc40` 0.080 (noisy) and `single_0159` 0.015.
- It costs x2.5–3.75 the CPU (`solve` included); sampling about a polished state is dear per evaluation.
- In the smoke trace, its samples found 0.29 new bests a run worth 0.0000, and so its polish never ran (0.5 polishes a run).
- So, unlike memetic, cem has no machinery that improves on `solve`'s endpoint. Its value is its own restarts, and round b's `fromsolve+ownstart` tests those after `solve`.

**The open joint-move payloads come from the restarts, with any polish.**
- **Uncapped, cem reaches** `b10_single_3300` (0 on every seed), `b10_week_inc_0004:3` (0), `single_3397` (0.006), `single_3375` (0.017) and `week_hone_0009:6` (0).
- **Under `solve`'s polish** in place of its own, it still reaches four of them (`single_3375` is lost, 0.049). Under `solve`'s start it reaches all five.
- **What they need is the restarts:** without them, three are lost.
- **Smoke on these payloads** (round b build, 3 seeds): with the order-prefix rebuild off, or with refit and group both off, all five are still reached.
- **What it does not reach:** `chain_0017:3` (0.125, kick loop 0) and `chain_1051` (0.190, worse than `solve`).

**What advanced moves do not explain.** A full scan of each advanced slot (`advfull`) is level or worse everywhere: `advanced` -0.0025, `real_08927991` unchanged at 0.196. The ±2 step is not why cem trails on `advanced` or `real_08927991`. Removed.

**Its remaining losses, uncapped:**
- `chain_0017` steps 0–2 and `b10_chain_0017` (0.15–0.65, every seed): `solve`'s polish halves them (0.205 on step 0) and group off nearly does (0.229);
- `real_08927991` (0.196): advanced with normal plans, lost by `price`, `memetic` and `surrogate` too;
- `single_0044` (0.198), `chain_1051` (0.190), `juice` `single_0034` (0.034);
- the bimodal `single_1056` and `single_1208`.

`fromsolve` has none of these, so they are cem's search, not the payloads. Round b tests `solve`'s exchanges and the order-prefix rebuild against them.

## 4. Batch `b`: on cem with the cap removed, and `solve` followed by cem's restarts

One session per set. Each cell is diff vs `cem_b` (`solve` 48k), with (wins, losses) where useful. CPU is against `solve` 48k in the same batch.
- R = cem 1 with `fullreach`.
- FSo = R with `fromsolve+ownstart`: `solve`, then cem's cycles, each a restart from cem's own start. Its cost includes `solve`'s.
- KM = `kick_stall=32,mem_polish=true`; L = `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach`.

| variant | real_test | quick_dev | speed_test | large | income | advanced | free_taps | juice | CPU quick_dev / large |
|---|---|---|---|---|---|---|---|---|---|
| R | -0.0077 (1, 8) | -0.0078 (4, 36) | -0.0092 | -0.0286, hit 47 | -0.0015 | -0.0011 | -0.0029 | -0.0034 | x0.88 / 0.56 |
| R, effort 2 | -0.0018 | -0.0042 (4, 24) | -0.0041 | -0.0180 | -0.0009 | -0.0003 | -0.0007 | -0.0001 | x1.73 / 1.09 |
| R, `oprefix` off | -0.0095 | -0.0056 (4, 33) | -0.0169 | -0.0282 | -0.0016 | -0.0005 | -0.0021 | -0.0032 | x0.91 / 0.60 |
| R, `refit+group` off | -0.0054 | -0.0071 (3, 36) | -0.0039 | -0.0436 | -0.0014 | -0.0006 | -0.0182 | -0.0004 | x1.00 / 0.54 |
| R + `exchange` | **-0.0042** | -0.0066 (4, 23) | -0.0094 | -0.0222, hit 53 | -0.0016 | -0.0032 | -0.0026 | -0.0019 | x0.95 / 0.68 |
| R + `shadow` | -0.0207 | -0.0077 | -0.0249 | -0.0233 | -0.0015 | -0.0010 | -0.0011 | -0.0021 | x0.94 / 0.61 |
| FSo, effort 1 | +0.0034, hit 97.4 | +0.0012 (9, 0), 96.5 | +0.0007 | +0.0007 | +0.0002 | 0 | +0.0004, 100 | +0.0003 | x1.81 / 1.49 |
| FSo, effort 0.5 | +0.0033 | +0.0008 (8, 0), 96.2 | +0.0006 | +0.0007 | +0.0001 | 0 | 0 | +0.0003 | x1.42 / 1.26 |
| KM | +0.0058 | +0.0019 (10, 0), 97.6 | +0.0007 | +0.0093, hit 73 | +0.0001 | 0 | 0 | +0.0003 | x1.92 / 2.57 |
| L | +0.0064 | +0.0018 (8, 0), 97.3 | +0.0016 | +0.0066 | +0.0002 | +0.0031, 100 | +0.0004, 100 | +0.0034, 100 | x2.04 / 2.96 |

- `small` is 100 for every variant.
- FSo, KM and L lose no payload anywhere (worst -0.002).
- FSo's CPU on real_test, speed_test and income is x1.84, x1.71 and x2.06 at effort 1, and x1.44, x1.37 and x1.54 at effort 0.5.

**Mean gaps on the payloads that move** (3 seeds; real_test 10):

| payload | `solve` | R | R 2 | `oprefix` off | model off | `exchange` | FSo 1 | FSo 0.5 | KM | L |
|---|---|---|---|---|---|---|---|---|---|---|
| `b10_single_3300` | 0.162 | 0 | 0 | 0 | 0 | 0 | 0.025 | 0.025 | 0 | 0.039 |
| `b10_week_inc_0004:3` | 0.064 | 0 | 0 | 0.007 | 0.014 | 0 | 0 | 0 | 0.064 | 0.014 |
| `single_3375` | 0.047 | 0.017 | 0.006 | 0.017 | 0.012 | 0.023 | 0.009 | 0.009 | 0.047 | 0.005 |
| `single_3397_serca_n4` | 0.062 | 0.006 | 0.006 | 0.006 | 0.002 | 0.007 | 0.009 | 0.009 | 0.014 | 0.004 |
| `week_hone_0009_serca_n21:6` | 0.023 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `chain_0017:2` | 0.072 | 0.173 | 0.168 | 0.113 | 0.122 | 0.173 | 0 | 0.060 | 0 | 0 |
| `chain_0017:3` | 0.175 | 0.125 | 0.123 | 0.125 | 0.198 | 0.128 | 0.116 | 0.175 | 0 | 0 |
| `chain_1051:0` / `:1` | 0.063 / 0.123 | 0.190 / 0.185 | same | same | 0.127 / 0.185 | 0.190 / 0.185 | 0.063 / 0.123 | 0.063 / 0.123 | 0 / 0.123 | 0 / 0 |
| `chain_0017:0` | 0.020 | 0.646 | 0.319 | 0.280 | 0.449 | 0.749 | 0.020 | 0.020 | 0.020 | 0.012 |
| `chain_0017:1` | 0.030 | 0.331 | 0.227 | 0.355 | 0.438 | 0.253 | 0.030 | 0.030 | 0.015 | 0.008 |
| `b10_chain_0017:0` | 0 | 0.208 | 0.075 | 0.389 | 0.197 | 0.485 | 0 | 0 | 0 | 0 |
| `b10_chain_0017:1` | 0 | 0.624 | 0.560 | 0.351 | 0.413 | 0.173 | 0 | 0 | 0 | 0 |
| `real_08927991_4514f9` | 0.002 | 0.196 | 0.143 | 0.196 | 0.219 | **0.067** | 0.002 | 0.002 | 0.002 | 0.002 |
| `single_0044` | 0.079 | 0.198 | 0.171 | 0.164 | 0.271 | 0.195 | 0.079 | 0.079 | 0.059 | 0.044 |
| `single_0159` | 0.056 | 0.139 | 0.091 | 0.140 | 0.177 | 0.150 | 0.056 | 0.056 | 0.008 | 0.053 |
| `advanced` `single_0085` | 0.037 | 0.046 | 0.036 | 0.036 | 0.018 | 0.055 | 0.037 | 0.037 | 0.037 | 0 |
| `real_27a2bc40` (10 seeds) | 0.274 | 0.212 | 0.130 | 0.233 | 0.143 | 0.200 | 0.155 | 0.155 | 0.065 | 0.044 |

Noise: `single_1208` (R 0.335, `oprefix` off 2.018, `exchange` 1.049), `single_1056` (0.624 or 0) and `free_taps` `single_0087` (0.192 with the model off) end each seed in one of two places. They make the `speed_test` and `free_taps` rows of R's ablations unreliable.

**The model is worth nothing.**
- With the refit and the piece-type factor both off, cem is level:
  - better on real_test (-0.0054) and `speed_test` (-0.0039);
  - worse on `large` (-0.044, through `single_0044`/`single_0159`, which swing) and `free_taps` (one bimodal payload).
- It costs a little more CPU (x1.00 against x0.88), as wider samples cost more per evaluation.
- Every open payload is still reached.

**The order-prefix rebuild is neither good nor bad.**
- Without it, quick_dev is better (-0.0056) and `chain_0017:0` falls from 0.646 to 0.280, but `b10_chain_0017:0` rises from 0.208 to 0.389.
- Elsewhere it is level, but for the bimodal `single_1208`.
- The open payloads do not need it.

**`solve`'s start does not help once the cap is gone.** `shadow` is real_test -0.021 (`real_27a2bc40`) and `speed_test` -0.025 (`single_1208` 3.1), and level elsewhere. Removed.

**`solve`'s exchanges help a little.**
- They take real_test from -0.0077 to -0.0042, and quick_dev losses from 36 to 23. `real_08927991` falls from 0.196 to 0.067.
- That is the advanced-with-normal-plans payload that `price`, `memetic` and `surrogate` lose too. Moving juice uses between upgrades is part of what `solve` has there.
- They do nothing for `chain_0017`.

**Effort 2 halves the loss on quick_dev (-0.0042) and `large` (-0.018), at x1.7 / 1.1, still behind `solve` 48k.**
- `solve` 24k is -0.0027 on quick_dev at x0.64 (price report 11), so cem is behind `solve` at matched CPU even uncapped.

**`chain_0017` steps 0–1 stay lost under every cem variant** (0.2–0.75), while `solve` and FSo keep 0.02–0.03.
- In the end states (section 1), cem's free tap order differs from `solve`'s. Its plans are plainer: fronts only, few back streaks or mixed ids, little juice on the first two upgrades.
- Its restarts all end in that basin, and neither the uncapped polish nor the exchanges leave it. What `solve`'s annealing does there that cem's descent does not is not settled (see "Not yet answered").

**`solve`, then cem's restarts (FSo).**
- **It loses nothing**, because `solve`'s endpoint is the best to beat.
- **It gains a little:** quick_dev +0.0012 (9 wins) at effort 1, +0.0008 (8 wins) at 0.5; real_test +0.0033 (`real_27a2bc40`); income +0.0001–0.0002; `free_taps` 100 at effort 1.
- **Effort 0.5 is the better trade:** level with effort 1 but for `chain_0017:2`, at x1.21–1.54.
- **Alone it is behind KM and L** on quick_dev, `large` and the regime sets.
- **It takes two payloads that KM misses:** `b10_week_inc_0004:3` (0 against 0.064) and `single_3375` (0.009 against 0.047).
- **KM takes what FSo misses:** `chain_0017:2/3`, `b10_chain_0017:3`, `chain_1051:0`, `single_0159`, `single_0044`.
- **KM and FSo together (offline arithmetic, not a run):** each run's better result of the two, with CPU KM + FSo - `solve`. Against KM and L together, worked out the same way:

| set | KM ∪ FSo 0.5 | KM ∪ L |
|---|---|---|
| real_test | +0.0061, hit 97.9, x2.07 | +0.0070, 98.7, x2.84 |
| quick_dev | +0.0021, 98.1, x2.34 | +0.0022, 98.3, x2.96 |
| speed_test | +0.0010, 97.8, x1.85 | +0.0016, 98.3, x2.44 |
| `large` | +0.0096, 73.3, x2.83 | +0.0120, 76.7, x4.53 |
| income | +0.0002, 98.4, x2.17 | +0.0002, 98.4, x2.61 |
| `advanced` / `juice` | 0 / +0.0003 | +0.0031 / +0.0034, both 100 |

  On quick_dev and income, cem's restarts add to the kick loop what L adds, at about 0.8 of the CPU. L does more on `speed_test`, `large` and the regime sets.

## 5. Pieces

| Piece | Verdict | Where | Evidence | Cost | Why |
|---|---|---|---|---|---|
| The model (refit, piece-type factor) | **Nothing** | | both off: level (quick_dev -0.0071 against -0.0078); each alone level | | partial redraws act as a random perturbation; the learning adds nothing |
| Restarts from cem's start (`fresh`) | **Its strength** | `b10_single_3300`, `b10_week_inc_0004:3`, `single_3397`, `single_3375`, `week_hone_0009:6` | without them quick_dev -0.0223 (59 losses), `free_taps` -0.072, three open payloads lost | | each descent from the start lands in another basin, and its polish settles it |
| Order-polish cap of 10 places (`fullreach` removes it) | **The main fixable loss** | `large`, `single_0031`, `single_0161`, `single_3097`, `single_0071` | `large` -0.130 → -0.029, quick_dev -0.0121 → -0.0078 | less CPU | on long orders the free taps reach past place 10 |
| Greedy order-prefix rebuild (`oprefix`) | **Nothing either way** | moves `chain_0017` both ways | quick_dev -0.0056 without it; open payloads kept | | |
| `solve`'s polish in its place (`solvepolish`) | **Helps, mostly as the uncapped order** | `large`, quick_dev | -0.016 / -0.0060 | x1.08–1.25 | |
| `solve`'s exchanges (`exchange`) | **Small help** | real_test, `real_08927991` (0.196 → 0.067) | real_test -0.0042 against -0.0077 | about +0.1 CPU | joint juice moves between upgrades |
| `solve`'s start (`shadow`) | **Nothing; removed** | fixes `single_0161` through the order | uncapped: real_test -0.021, `speed_test` -0.025 | | |
| Full advanced scan (`advfull`) | **Nothing; removed** | | `advanced` -0.0025 against -0.0011 | | |
| Unsnapped given plans (`keepgiven`) | **Nothing; removed** | income | -0.0021 at x1.58 against -0.0017 at x1.35 | | the snap is a flaw, but not the income loss |
| From `solve`'s endpoint (`fromsolve`) | **Nothing** | | quick_dev +0.0001 | x2.5–3.75 | its samples never beat a polished state |
| `solve`, then cem's restarts (`fromsolve+ownstart`) | **Small help; a complement to the kick loop** | `b10_week_inc_0004:3`, `single_3375` | effort 0.5: quick_dev +0.0008 (8, 0), real_test +0.0033; with KM (offline arithmetic) quick_dev +0.0021 against KM ∪ L +0.0022 | x1.21–1.54; with KM x2.34 against x2.96 | restarts from a cheap start reach basins that kicks from the best do not |
| Effort, uncapped | **Behind at matched CPU** | | effort 2: quick_dev -0.0042, `large` -0.018 | x1.73 / 1.09 | |

## Not yet answered

- **Why cem's restarts all end in a worse basin on `chain_0017` steps 0–1** (0.2–0.75), while `solve` and every other route keep 0.02–0.03. The end states differ in the free tap order and in plainer plans. Neither the uncapped polish nor the exchanges leave that basin.
- **cem's restarts after the kick loop and `mem_polish`, run as one search.** The numbers above are the offline better-of-two arithmetic, not a run. Also open is a production form: restarts from the shadow start, each a partial-redraw descent and a polish, after `refine`.
- **Why `refit` off and `fresh` off cost more CPU per evaluation** (x1.11 at x0.88 the evaluations, x1.46 at x1.02, with the same number of polishes). Seen, not explained.
