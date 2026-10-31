# `surrogate` investigation

**Verdict: not reached yet.** Scripts are in `scripts/` (`overview.py`, `gaps.py`, `trace.py`, `states.py`, `attr.py`). All numbers are % of flow.

## 0. State of the work

| Round | Snapshot | Batch | Tests | Status |
|---|---|---|---|---|
| a | `sur_a` | `iteration2/surrogate/a` | every main piece ablated at effort 1, `fromsolve`, `truedescent`, `points128`, `top8` | read, section 4 |
| b | `sur_b` | `iteration2/surrogate/b`, `b_income` | see section 5 | queued |

Base for ablations in batch a: `sur_a:algo=surrogate,lab_effort=1`. In batch b: `sur_b:algo=surrogate,lab_effort=1,lab_on=points128`.

Switches in `surrogate.rs`:

- `lab_off`: `skew`, `calib`, `descent`, `kickrepair`, `pricekick`, `pempty`, `prand`, `ppiece`, `last`, `flip`, `widest`, `blend`, `surpolish`, `truepolish`, `dorder`, `dchest`, `dadv`, `porder`, `pchest`, `padv`, `greedy`.
- `lab_on`: `points128`, `points512`, `top8`, `fromsolve`, `truedescent`, `anystack`, `keepchest`, `keepplans`, `randorder`, `randplan`, `calibwide`, `advtrue`.

Knobs in `solve`, off by default: `sur_start`, `sur_stall`, `sur_kinds`.

## 1. Where it stands on the re-run

| set | surrogate 1 | surrogate 2 | `solve` 48k hit |
|---|---|---|---|
| real_test | +0.0016, hit 92.4, x1.17 | +0.0032, hit 93.7, x2.30 | 96.8 |
| real_income_test | +0.0001, hit 97.9, x0.91 | +0.0003, hit 98.4, x1.69 | 98.1 |
| quick_dev | -0.0028, hit 92.2, x0.73 | -0.0018, hit 93.7, x1.44 | 94.3 |
| speed_test | -0.0058, hit 92.9, x1.33 | -0.0020, hit 94.1, x2.58 | 96.4 |
| large | -0.0017, hit 63.3, x0.97 | +0.0035, hit 63.3, x1.83 | 56.7 |
| advanced | -0.0027, x0.69 | -0.0025, x1.42 | 94.4 |
| free_taps | -0.0055, x0.73 | -0.0038, x1.32 | 97.2 |
| juice | -0.0037, x0.70 | -0.0012, x1.43 | 94.4 |

Its wins and losses are a few payloads.

- **Wins at effort 1:**
  - `chain_0017` step 2 (+0.109 cold, +0.181 chained) and step 3 (+0.017 cold, +0.156 chained)
  - `real_27a2bc40` (+0.25), which is a listed noisy payload
  - `single_3397_serca_n4` (+0.06)
  - `b10_week_inc_0004:3` (+0.04)
  - `single_0044_t4_n40` (+0.05)
  - `single_0085_t4_n17` (+0.04)
- **Losses at effort 1:**
  - `single_1056_serca_n3` (-0.62, gone at effort 2)
  - `chain_0001` (-0.17)
  - `real_08927991_4514f9` (-0.14)
  - `week_hone_0004` steps 1–3 (-0.08 to -0.12)
  - `chain_1051`, `single_1208`, `single_1000`
  - `single_3300` (-0.07)

**The open payloads, by gap** (the listed `week_inc_0004` step 3 is `b10_week_inc_0004:3`; the plain payload is solved by `solve`, gap 0.001):

| payload | `solve` | kick loop | surrogate 1 | `points128` | `fromsolve` |
|---|---|---|---|---|---|
| `single_3375` | 0.038 | 0.038 | 0.039 | **0** (all 3 seeds) | 0.028 |
| `single_3397_serca_n4` | 0.062 | 0.025 | **0.002** | **0.002** | **0.002** |
| `b10_week_inc_0004:3` | 0.058 | 0.058 | 0.014 | 0.021 | 0.007 |
| `single_3300` | 0.071 | 0.071 | 0.142 | 0.142 | 0.071 |
| `b10_single_3300` | 0.250 | 0.039 | 0.213 | 0.213 | 0.150 |

## 2. Reading the code

**Repeat.** The switch build repeats `lab_r1:algo=surrogate,lab_effort=1` with 0 wins, 0 losses and evaluations x1.00:

- `sur_s1`: on `advanced` (12 payloads, 3 seeds) and `quick_dev` (349 payloads, 1 seed).
- `sur_s3`, whose source is the same as `sur_a`: only on `chain_0017` (8 payloads, 1 seed).

**Smoke.** All switches but `truedescent` ran on `chain_0017`, 1 seed, on `sur_s2`. That exe has the same code minus `truedescent`, because a failed build left it in place. `truedescent` itself ran on `sur_s3`.

**Checked and found right:** moments and sums over the free tap outcomes, the skipped prefix, the band sum in `gold_of`, the correction's sign, the tables' outcomes, order cleaning, restoring after a rejected move.

**What is off:**

1. **The budget does not bind, so effort is not what it says.** Median spent/budget is 0.01–0.20 on every set but `large` (0.76). Runs nearly always end on the stall of `40 x effort` restarts. From reading the code, `refresh_dist`, `Mom::of`, `sums()` and free tap orders go uncounted; I have not measured their share. Effort is really stall length plus kick trials.
2. **`POINTS` is 24.** `response_points` at 16 picked wrong plans by up to 0.96 (price report 1). Tested by `points128`.
3. **`perturb`'s piece kick always opens the first stack** (`.nth(0)`). `anystack` fixes it; not queued yet.
4. **The correction's slope and the shadow price use different widths** (about 0.01% against 1% of use). Not tested on its own. `lab_off=calib` shows whether the correction matters at all.

## 3. Where its results come from (attribution, smoke, effort 1)

Each new best is credited to the restart kind and the first move that reached it, past the first restart. So a descent that lands just above the best takes that credit, and gains in the first restart are not in the move columns. These are small samples: `real_test` 3 seeds, `quick_dev`, `speed_test` and `large` 1 seed, `advanced`, `free_taps` and `juice` 3 seeds. Mean gain per run:

| set | rounds med | best round med | descent / repair / polish time | best found at (share of time) | gain by descent | polish moves together | last phase |
|---|---|---|---|---|---|---|---|
| real_test | 42 | 12 | .46 / .25 / .28 | .58 | .0754 | .0044 | .0003 |
| quick_dev | 41 | 0 | .52 / .13 / .34 | .46 | .0657 | .0056 | .0000 |
| speed_test | 41 | 4 | .40 / .27 / .32 | .52 | .1342 | .0013 | .0001 |
| large | 41 | 25 | .62 / .07 / .30 | .73 | .1061 | .0155 | .0001 |
| advanced | 41 | 4 | .38 / .23 / .38 | .42 | .0077 | 0 | 0 |
| free_taps | 41 | 8 | .48 / .12 / .39 | .55 | .0738 | .0049 | 0 |
| juice | 41 | 2 | .30 / .20 / .50 | .55 | .0126 | .0015 | 0 |

- **Past the first restart, most gains are a surrogate descent landing above the best before the true metric has checked it.** Polish moves add 1–15% of what descents do. This is only attribution; the `descent` ablation has to confirm it.
- **The last phase gains next to nothing** (0.0003 at most).
- **On the median payload the first descent is already the end** (gap 0 on `quick_dev`, 0.001 on `real_test`). The time after the last new best is 27–58% of the run.
- **By restart kind:** price kicks lead on `large` and `quick_dev`, empty restarts on `real_test` and `speed_test`, random restarts on `free_taps`.
- **`chain_0017`:** `lab_off=prand` loses 0.68 across all 8 steps (1 seed), so one kind of restart seems to carry that payload.
- **`truedescent`:** on `chain_0017` it took x0.85 the CPU and ended worse (gap 0.58 against 0.29, 1 seed), so a true response was no dearer there.

**The losses, end state against `solve`:**

- `single_1056_serca_n3` (n=3, seed 0, gap 0.94): at least one chest stack differs (Lava's Breath where the best takes Serca Special Leap), and upgrade 1 has a shorter plan. A chest and a plan must move together.
- `chain_0001` step 0 (n=18): no juice on the normal upgrades. The free tap order and seven advanced strategies differ together.

**Checked against batch a.** The attribution overstated the descent as a whole. Taking it out costs about what taking out its advanced moves costs (section 4). Plan responses in the descent do not carry the result by themselves.

## 4. Batch `a`: every piece at effort 1

Plain `sur_a` repeats `lab_r1` surrogate 1 (0 wins, 0 losses, evaluations x1.00). Diff is against plain `sur_a`, seed for seed; cpu is real_test / quick_dev / speed_test / large.

| change | real_test | quick_dev | speed_test | large | cpu |
|---|---|---|---|---|---|
| `skew` off | -0.0024 | -0.0003 | +0.0027 | -0.0005 | 1.08 / 1.05 / 1.03 / 0.99 |
| `calib` off | -0.0003 | +0.0004 | +0.0002 | +0.0019 | 0.99 / 0.99 / 0.99 / 0.96 |
| `skew+calib` off | +0.0017 | +0.0001 | +0.0025 | +0.0020 | 1.05 / 1.02 / 1.02 / 0.95 |
| `descent` off | -0.0037 | **-0.0174** | +0.0025 | **-0.0451** | 1.14 / 1.23 / 0.84 / 1.15 |
| `truedescent` on | -0.0022 | -0.0020 | -0.0037 | -0.0039 | 1.22 / 1.18 / 0.97 / 1.11 |
| `kickrepair` off | +0.0014 | -0.0003 | -0.0019 | +0.0039 | 0.97 / 0.95 / 0.82 / 1.10 |
| `pricekick` off | -0.0009 | **-0.0120** | +0.0009 | **-0.0398** | 1.12 / 1.15 / 1.09 / 1.12 |
| `pempty` off | **-0.0533** | 0.0000 | **-0.0898** | -0.0095 | 0.70 / 0.94 / 1.02 / 0.96 |
| `prand` off | -0.0014 | **-0.0249** | **-0.0102** | -0.0021 | 1.05 / 0.90 / 0.88 / 0.92 |
| `ppiece` off | -0.0002 | -0.0001 | -0.0003 | +0.0073 | 0.99 / 1.05 / 1.03 / 1.00 |
| all state kicks off | -0.0546 | -0.0296 | -0.1027 | -0.0059 | |
| all kicks off | -0.0747 | -0.0697 | -0.1348 | -0.1126 | |
| `dadv` off | -0.0017 | **-0.0180** | -0.0002 | **-0.0403** | 0.97 / 0.90 / 0.99 / 1.00 |
| `dchest` off | **-0.0051** | +0.0001 | **-0.0046** | 0.0000 | 1.14 / 1.04 / 1.01 / 1.00 |
| `dorder` off | -0.0009 | -0.0003 | -0.0016 | **-0.0124** | 0.85 / 0.91 / 0.86 / 0.77 |
| `padv` off | -0.0005 | +0.0001 | -0.0001 | +0.0031 | 0.98 / 0.93 / 0.97 / 0.98 |
| `pchest` off | -0.0005 | 0.0000 | 0.0000 | 0.0000 | 0.88 / 1.00 / 0.97 / 1.00 |
| `porder` off | 0.0000 | -0.0016 | -0.0002 | +0.0030 | 0.99 / 0.95 / 0.96 / 0.96 |
| `surpolish` off | +0.0003 | +0.0001 | +0.0001 | +0.0005 | 0.98 / 0.95 / 0.93 / 0.97 |
| `truepolish` off | -0.0008 | -0.0009 | -0.0019 | -0.0022 | 0.87 / 0.89 / 0.93 / 0.95 |
| `blend` off | 0.0000 | 0.0000 | -0.0002 | +0.0008 | 1.00 / 1.00 / 0.99 / 1.01 |
| `last` off | -0.0004 | 0.0000 | 0.0000 | -0.0001 | 0.98 / 0.99 / 0.98 / 0.99 |
| `greedy` off | -0.0004 | -0.0006 | 0.0000 | +0.0018 | 0.98 / 1.05 / 0.97 / 1.06 |
| `top8` on | -0.0002 | 0.0000 | 0.0000 | 0.0000 | 0.98 / 1.00 / 0.97 / 1.01 |
| `points128` on | **+0.0015** | **+0.0008** | 0.0000 | **+0.0043** | 1.01 / 1.01 / 0.98 / 0.99 |

**Noise.** `single_1056_serca_n3` is bimodal: base has seeds at 0 and 0.936, and many unrelated variants move it by +0.624. That one payload is worth +0.0029 on speed_test, which covers the +0.0025 to +0.0027 rows there. Add it to the noisy list. Most of the kept pieces' wins and losses are a few payloads, and many small rows are noise.

**The model carries no measured weight.** With neither Edgeworth nor correction, a plain normal surrogate holds level on every main set. The two pieces that made it a calibrated surrogate are worth nothing measurable. The one sign of a cost is on `small` (one payload, -0.014, hit 100 to 95). The correction's width mismatch can matter little if the whole correction is null; `calibwide` is queued to confirm.

**True responses do not beat surrogate ones in a descent.** `truedescent` is slightly behind at x1.0–1.2 the CPU. Price kicks do not reach normal plans there, so this is not a clean swap. It is still enough to say that a cheap coarse table is no handicap. The surrogate saves no CPU over true responses either, which contradicts the reason it was built.

**`descent` off loses through advanced moves.** `descent` off and `dadv` off lose nearly the same on quick_dev (-0.0174, -0.0180) and `large` (-0.0451, -0.0403). On quick_dev that loss is `chain_0001`: -1.00 at step 0, -0.65 to -0.67 at steps 1–3. Checked advanced moves in the polish (`padv`) add nothing. So what works is advanced strategies moving unchecked, one per upgrade per round, several upgrades together, with other moves between. `advtrue` tests whether the polish's single ranked move is simply too weak.

**Three restarts do the work, each on its own payloads.**

- `pempty`: `single_1208` -3.0, `single_1056` -2.2, `real_27a2bc40` -1.25, `real_fbaad097` -0.81, `single_1104_serca_n1` -0.39. Several of these are lone or chest-driven. The empty restart also opens every stack at random, so my hypothesis is that the chests do the work; `keepplans`/`keepchest` test it. `single_1235_t4_n1` at -13.5 is a 24-point artefact, not a restart effect: at `points128`, `lab_off=pempty` leaves it at gap 0. That payload alone is about -0.063 of the -0.0898 speed_test row, so the row overstates the restart.
- `prand`: `chain_0017` steps 0–2 -0.6 to -0.8, `single_0087` -0.58. These are the order basins; `randorder`/`randplan` split them.
- `pricekick`: `chain_0001` and `single_0159` (-0.40), so again advanced-heavy payloads. It is the same family as `dadv`.
- `ppiece`: nothing.

One descent and one polish with no restarts hit 72–77% on the main sets (30% on `large`) at x0.11–0.21 the evaluations.

**Pieces that do nothing:**

- `kickrepair`, the prototype's "joint move" (10 wins, 7 losses on quick_dev, spread both ways).
- `surpolish` (x0.49–0.64 the evaluations, x0.93–0.98 the CPU).
- `blend`, `top8`, `pchest`.
- `last`. Flips and wide responses exist only in that phase, so together they cannot be worth more than its 0.0004. `flip` and `widest` are queued to close this directly.

**`truepolish` does matter a little:** hit falls 92.3 to 90.2 on quick_dev and 92.9 to 90.2 on speed_test, and `small` loses one payload. The surrogate's plans still need true responses to finish them.

**`points128` is a real fix at no CPU cost.** It wins `single_3375` (gap 0.039 to 0, all 3 seeds), `chain_0017:3` (+0.128) and `real_08927991_4514f9` (+0.045). This is the `response_points` error again. It also bears on the open question: `single_3375` was at least partly a table-resolution problem in this prototype. `truedescent` gets close too (0.011). In both cases the moves are responses taken in turn with no metric check, using exact enough tables. A true fixed point stays put under gated responses, so an unchecked walk needs tables or prices that differ from the truth. My working explanation is that a surrogate descent kicks the objective, and the upgrades re-settle together; it is not tested directly yet.

**`fromsolve`:** against `solve` 48k, with the kick loop beside it.

| set | `fromsolve` | hit | cpu | kick loop | hit | cpu |
|---|---|---|---|---|---|---|
| real_test | +0.0057 | 98.9 | x2.00 | +0.0056 | 97.9 | x1.56 |
| quick_dev | +0.0019 | 96.8 | x1.47 | +0.0023 | 97.1 | x1.69 |
| speed_test | +0.0007 | 98.0 | x2.08 | +0.0007 | 97.2 | x1.29 |
| large | +0.0101 | 60.0 | x1.67 | +0.0121 | 63.3 | x2.36 |
| advanced | +0.0031 | 100 | x1.53 | 0.0000 | 94.4 | x1.59 |
| free_taps | +0.0004 | 100 | x1.50 | +0.0001 | 97.2 | x1.51 |
| juice | +0.0034 | 100 | x1.58 | +0.0003 | 94.4 | x1.38 |

CPU ratios are across sessions, so treat them as rough. It loses no payload to `solve` by more than 0.01% on any of these sets, and it takes the regime sets to 100.

The two are complementary:

- `fromsolve` wins `single_3397` (+0.023), `single_0085` (+0.037), `single_0044` (+0.028) and `b10_week_inc_0004:3` (+0.051).
- The kick loop wins `b10_single_3300` (-0.110), `chain_1051:0` (-0.063), `chain_0017:2` (-0.055) and `single_0159` (-0.049).

On `real_08927991_4514f9`, `fromsolve` holds `solve`'s 0.002 where the plain surrogate is at 0.146, so that loss is its start, not its restarts. Almost all its gains over `solve` are again descents landing above the best.

## 5. Batch `b`

To be read when it has run. It tests: the points limit, what the empty and random restarts really are, a stronger advanced polish, a lean surrogate at matched CPU, and surrogate restarts as a stall-bounded loop in `solve` under `solve`'s own polish (`sur_stall`, `sur_kinds`), by kind.
