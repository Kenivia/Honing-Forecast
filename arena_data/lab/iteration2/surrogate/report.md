# `surrogate` investigation

**Verdict: take one piece, not the algorithm.**

- **The piece:** a re-plan between a kick and `solve`'s polish (`sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach`, optionally `sur_screen=true`).
- **The gain:** real_test +0.0064 (hit 96.6 to 97.9), quick_dev +0.0018 (95.5 to 97.3), speed_test +0.0016, `large` +0.0066, the regime sets to 100, income +0.0002. No payload is lost.
- **The cost:** x2.0–3.3 the CPU, or 13–24% less with `sur_screen` at nearly the same gain (10-seed gap sum 0.288 against 0.276).
- **Why it works:** after the kick, every normal upgrade re-plans and the free tap order moves with them, in rounds, before `order_polish` can put the order back. The descent then lands on the new basin itself, and the true polish only confirms it.
  - The order moves among the plans are the larger part: without them the sum is 0.399 against 0.276, and order basins such as `chain_0017:2` are lost.
  - The plans are the rest: without them 0.656.
- **The model:** it adds only a little over true responses (0.276 against 0.303).
  - That little is mostly its error acting as a kick. True responses and true order moves on band prices perturbed at random reach L (0.264).
  - The one exception is `single_3116`, which only the surrogate's plan steps hold on every seed.
  - A version with no surrogate costs +87% on `large` and about +30% elsewhere, or +45% and +5% with the screen. The cost is the true order and advanced evaluations inside the descent.
  - Edgeworth and the correction to shadow prices are worth nothing; a plain normal model does as well.
- **What doesn't:** no re-plan, a model-free price kick, skipping the polish of deep rounds (`sur_skip`), and the prototype's own start and budget.

Scripts are in `scripts/`. All numbers are % of flow.

**Old free tap cut in sections 1–5.** The re-run (`lab_r1`), the smoke of section 3 and batches `a` and `b` (`sur_a`, `sur_b`) ran before the cut of 2026-10-31. Comparisons inside them stand; their gains on order-basin payloads run high. Batches `c`–`g` are at the new cut and carry the verdict.

## 0. State of the work

| Round | Snapshot | Batch | Tests | Status |
|---|---|---|---|---|
| a | `sur_a` | `iteration2/surrogate/a` | every main piece ablated at effort 1, `fromsolve`, `truedescent`, `points128`, `top8` | read, section 4 |
| b | `sur_b` | `iteration2/surrogate/b`, `b_income` | see section 5 | read, section 5 |
| c | `sur_c` (HEAD, new cut) | `iteration2/surrogate/c`, `c_income` | see section 6 | read, section 6 |
| d | `sur_d` (HEAD) | `iteration2/surrogate/d`, `d_income` | see section 7 | read, section 7 |
| e | `sur_e` | `iteration2/surrogate/e`, `e_regime`, `e_income`, `e_seeds` | see section 8 | read, section 8 |
| f | `sur_f` | `iteration2/surrogate/f` | model against true responses at each level of the descent, which half carries `large`, price noise as a kick; 10 seeds on `large` and the payloads that move | read, section 9 |
| g | `sur_g` | `iteration2/surrogate/g` | true order moves in the re-plan (no surrogate in the descent), and cheaper polishes after the kick (`sur_skip`, `sur_screen`); 10 seeds on f's payloads | read, section 10 |

`sur_a` and `sur_b` used the old free tap cut. Comparisons inside either stand. Anything from `sur_c` on repeats `cut_a`.

Base for ablations in batch a: `sur_a:algo=surrogate,lab_effort=1`. In batch b: `sur_b:algo=surrogate,lab_effort=1,lab_on=points128`.

Switches in `surrogate.rs`:

- `lab_off`: `skew`, `calib`, `descent`, `kickrepair`, `pricekick`, `pempty`, `prand`, `ppiece`, `last`, `flip`, `widest`, `blend`, `surpolish`, `truepolish`, `dorder`, `dchest`, `dadv`, `dplan`, `porder`, `pchest`, `padv`, `greedy`.
- `lab_on`: `points128`, `fromsolve`, `truedescent`, `trueorder`, `keepchest`, `keepplans`, `randorder`, `randplan`, `randreach`, `noisy`, `noisy3`.
  - `trueorder`: in the descent, order, adv and chest moves take the best candidate by the true metric.
  - `noisy` / `noisy3`: during the descent, each non-linear row's band prices are multiplied by a random factor of log spread 0.1 or 0.3, drawn once per kick.
- Removed as failed: `points512`, `top8`, `anystack`, `calibwide`, `advtrue`, `randboth`, `checked`, `d1`, `skipchest`, `skipw`, the kind `lprice`; and in `solve`, `sur_start` and `sur_skip`.

Knobs in `solve`, off by default: `sur_stall` (0), `sur_kinds` (`price+rand+chest`), `sur_screen` (false: the restart is polished one round first, `polish_rounds` 1 and `response_polish` 1, and in full only where that beats the best).

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
| `single_3300` | not open: 9 of 10 seeds hit | | 0.213 on all 10 seeds, stored `sur_a` 0.142 on 3 | 0.213 on 10 | 0.071 |
| `b10_single_3300` | 0.250 | 0.039 | 0.213 | 0.213 | 0.150 |

`single_3300` is a surrogate weakness, not a shared hard payload; section 5 has the cause.

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

## 5. Batch `b`, on `sur_b`, old cut

Plain `sur_b` repeats `lab_r1`, and the surrogate variant repeats `lab_r1` surrogate 1. All gains against `lab_r1` here are provisional: the new cut alone lifts `solve` on order basins, and batch c re-measures them at HEAD.

**Each piece, against `points128` (`sur_b`):**

| change | real_test | quick_dev | speed_test | large | cpu |
|---|---|---|---|---|---|
| `points512` | 0.0000 | 0.0000 | +0.0001 | 0.0000 | 1.00 / 1.01 / 1.00 / 1.00 |
| `keepchest` | **-0.0544** | 0.0000 | **-0.0261** | 0.0000 | 0.68 / 0.96 / 1.01 / 1.00 |
| `keepplans` | -0.0024 | -0.0016 | 0.0000 | **-0.0171** | 0.98 / 0.95 / 0.98 / 0.86 |
| `randorder` | -0.0009 | -0.0006 | +0.0008 | +0.0057 | 1.01 / 1.01 / 1.00 / 0.97 |
| `randplan` | 0.0000 | **-0.0128** | -0.0008 | -0.0025 | 0.99 / 0.91 / 0.90 / 0.88 |
| `advtrue` | +0.0001 | -0.0002 | 0.0000 | +0.0010 | 1.01 / 1.46 / 1.01 / 1.49 |
| `anystack` | -0.0016 | 0.0000 | -0.0009 | 0.0000 | 0.96 / 1.02 / 0.98 / 1.00 |
| `calibwide` | +0.0006 | -0.0001 | +0.0002 | +0.0028 | 1.01 / 0.98 / 1.00 / 1.00 |
| `flip` off | -0.0001 | 0.0000 | 0.0000 | 0.0000 | 0.99 / 1.00 / 0.99 / 1.00 |
| `widest` off | -0.0001 | 0.0000 | 0.0000 | 0.0000 | 0.99 / 1.00 / 0.99 / 1.00 |
| lean, effort 0.5 | -0.0019 | -0.0048 | -0.0131 | -0.0045 | 0.46 / 0.48 / 0.38 / 0.58 |
| lean, effort 1 | +0.0006 | -0.0022 | -0.0021 | +0.0006 | 0.89 / 0.88 / 0.76 / 1.12 |
| lean, effort 2 | +0.0013 | +0.0012 | +0.0012 | +0.0055 | 1.74 / 1.71 / 1.48 / 2.16 |
| leaner, effort 1 | -0.0005 | -0.0022 | -0.0021 | +0.0037 | 0.77 / 0.83 / 0.75 / 1.01 |

- **128 points is enough.** 512 changes nothing.
- **The empty restart's value is its random chests, on `real_test` and `speed_test`.** `keepchest` loses what `pempty` off lost in round a (real_test -0.0544 against -0.0533). `keepplans` keeps nearly all of it. So it is a chest kick followed by a descent, which re-plans before any greedy repair. That is the kick loop's own lesson. Emptying the plans adds something only on `large`, where `keepplans` loses 0.0171.
- **The random restart's value is its order moves.** `randplan` keeps only random plans and loses on quick_dev (-0.0128) and `free_taps` (-0.0569). `randorder` keeps only the order moves and loses nothing that matters.
- **A stronger advanced polish does nothing.** `advtrue` takes x1.0–2.2 the CPU and moves nothing, and it leaves `chain_0001` where it was. So does an exhaustive true order polish (`ordertrue`, smoke only: all 24 runs on `chain_0001` end on the same metric at x4.4 the evaluations). Surrogate's end state there is locally optimal in order and in advanced strategies apart. The miss is the two together.
- **`anystack`, `calibwide`, `flip` and `widest` are null**, which closes those items. On speed_test `flip` and `widest` move evaluations and nothing else, as round a implied.
- **Checked moves in the polish carry no weight.** Leaner matches lean on quick_dev and speed_test, and is close on real_test and `large`.
- **Lean is not a clean win.** It is level on real_test at x0.89 and slightly behind on quick_dev and speed_test. Its losses are real on a few payloads: `chain_0001` (-0.22, `kickrepair` helps there), `realinc_8f8ac564` steps 0–1 (about -0.09), income overall -0.0010. Each piece was null on its own in round a, though the mix is not. They seem to cover different payloads. Lean at effort 2 is ahead of the base at about x1.7 the CPU.
- **Income (`points128`, against `lab_r1`):** +0.0002, hit 97.9, x0.85. Lean: -0.0010, hit 98.0, x0.79.

**Swap-in: surrogate restarts as a loop after `solve` (`sur_stall`), with `solve`'s own polish.** Against `lab_r1`, with the kick loop for comparison:

| set | `sur_stall=8` | cpu | `sur_stall=32` | hit | cpu | kick loop | hit | cpu |
|---|---|---|---|---|---|---|---|---|
| real_test | +0.0034 | x1.24 | +0.0039 | 98.2 | x1.98 | +0.0056 | 97.9 | x1.51 |
| quick_dev | +0.0012 | x1.21 | +0.0021 | 97.1 | x1.84 | +0.0023 | 97.1 | x1.84 |
| speed_test | +0.0006 | | +0.0013 | 98.3 | x2.09 | +0.0007 | 97.2 | x1.39 |
| large | +0.0079 | x1.38 | +0.0087 | 60.0 | x2.45 | +0.0121 | 63.3 | x2.56 |
| advanced | 0.0000 | x1.42 | +0.0031 | 100 | x2.62 | 0.0000 | 94.4 | x1.60 |
| free_taps | +0.0001 | x1.56 | +0.0005 | 100 | x3.09 | +0.0001 | 97.2 | x1.50 |
| juice | +0.0003 | x1.39 | +0.0034 | 100 | x2.61 | +0.0003 | 94.4 | x1.30 |
| income | | | +0.0003 | 98.7 | x1.83 | 0.0000 | 98.1 | x1.47 |

The CPU column for the kick loop is against `sur_b`/`lab_r1` in matched runs. Kick-loop ratios cross sessions, so they are rough.

- **`sur_stall=32` loses no payload on any set**: every worst row is -0.000 to the fourth place, and income shows 0 losses. `sur_stall=8` takes 14 wins and 0 losses. Against `kick_stall=8` (price report 9) it gets more for similar CPU on real_test and `large`.
- **By kind, `sur_stall=32`, as with the kick loop each set has its own:**

| kind | real_test | quick_dev | speed_test | large |
|---|---|---|---|---|
| `price` | +0.0021 | +0.0011 | +0.0007 | **+0.0100** |
| `rand` | +0.0016 | **+0.0022** | +0.0008 | +0.0066 |
| `chest` | **+0.0070** | +0.0001 | +0.0009 | +0.0004 |
| `empty` | +0.0055 | +0.0011 | **+0.0014** | +0.0016 |

- **It is complementary to the kick loop.** Taking the better of the two per seed (both start from the same `solve` endpoint):

| set | loop 32 | kick loop | better of the two |
|---|---|---|---|
| quick_dev hit | 97.1 | 97.1 | **98.6** (+0.0029) |
| large hit | 60.0 | 63.3 | **70.0** (+0.0146) |
| real_test hit | 98.2 | 97.9 | **98.7** (+0.0066) |
| speed_test hit | 98.3 | 97.2 | 98.4 (+0.0016) |

This is a bound by construction, not a measured run of both. The kick loop's edge: `b10_single_3300`, `real_27a2bc40`, `chain_1051:0`, `chain_0017:2`, `single_0159`, `chain_0001`. The surrogate loop's edge: `chain_1051:1` (+0.123), `b10_week_inc_0004:3` (+0.051), `single_0085`, `single_3397`, `single_3375`.

- **Evals rise much less than CPU** (x1.04–2.0 against x1.7–3.1). Section 8 measures where it goes: kick and descent are 30–55% of the loop, and the true polish is the rest.
- **`fromsolve` with `points128` and the loop in `solve` agree closely.** quick_dev hit is 97.1 for both and real_test 98.7 against 98.2. So the prototype's own polish adds little over `solve`'s. Income: `fromsolve` +0.0004, hit 99.1, x1.68.
- **Open payloads under `sur_stall=32`:** `single_3397` 0.003, `b10_week_inc_0004:3` 0.007 (0 with `sur_kinds=price` or `rand`), `single_3375` 0.019 (0.002 with `empty`), `b10_single_3300` 0.110. In the `solve` loop, `single_3375` is not fully closed by the mix.
- **`sur_start` failed and is removed.** A descent as a chain start: quick_dev +0.0001 (6 wins, 9 losses), real_test +0.0004, `large` +0.0015, at x1.0 the CPU. It adds only what any restart adds on the regime sets.
- **`real_08927991_4514f9` and `chain_0001` are the prototype's start, not its restarts.** The loop holds `solve`'s 0.002 and 0.010 there, where surrogate alone stays at 0.101 and 0.181.

## 6. Batch `c`, on `sur_c`, new cut

Plain `sur_c` repeats `cut_a` with 0 wins, 0 losses, evals x1.00 on all 9 sets, and `points128` does on the 8 cold sets.

**Timing:** the machine changed during this batch and ran hotter after a restart. CPU here compares only variants of the same scenarios in the same timeframe, against `sur_c`. Never compare these with `cut_a`, `lab_r1`, batches a or b. `c_real_test` is before the restart, and speed_test onward is after it. `c_quick_dev` spans the stop, so its CPU uses the 461 of 528 scenario+seed pairs that started after it (ratios move by at most 0.03). Evals and metrics are unaffected.

**The loop after `solve`, at HEAD, against plain `sur_c`:**

| set | `sur_c` hit | `max_iters=96000` | `sur_stall=8` | `sur_stall=16` | `sur_stall=32` |
|---|---|---|---|---|---|
| real_test | 96.6 | +0.0040, 97.6, x1.74 | +0.0034, 97.6, x1.29 | +0.0038, 98.2, x1.56 | +0.0039, 98.2, x2.10 |
| quick_dev | 95.5 | +0.0008, 96.1, x1.62 | +0.0007, 96.2, x1.23 | +0.0013, 96.7, x1.46 | **+0.0018, 97.4**, x1.92 |
| speed_test | 96.3 | +0.0002, 97.4, x1.59 | +0.0006, 97.5, x1.26 | +0.0007, 97.7, x1.54 | **+0.0013, 98.0**, x2.20 |
| large | 53.3 | +0.0057, 56.7, x1.83 | +0.0054, 60.0, x1.43 | +0.0065, 63.3, x1.93 | **+0.0081, 66.7**, x2.87 |
| advanced | 94.4 | 0, 94.4, x1.88 | 0, 94.4, x1.39 | +0.0023, 97.2, x1.74 | **+0.0031, 100**, x2.51 |
| free_taps | 97.2 | +0.0001, 97.2, x1.90 | 0, 97.2, x1.48 | 0, 97.2, x1.78 | +0.0004, 100, x2.75 |
| juice | 94.4 | 0, 94.4, x1.88 | +0.0004, 94.4, x1.35 | +0.0027, 97.2, x1.68 | **+0.0034, 100**, x2.33 |

- **The gain survives the new cut.** quick_dev goes from +0.0021 to +0.0018 and `large` from +0.0087 to +0.0081; the others hold.
- **The loop loses no payload at any stall.** The worst payload on the cold sets is -0.000.
- **`sur_stall=16` beats 96k at about the same CPU** on quick_dev, speed_test, `large`, advanced and juice, and ties it on real_test and free_taps.
- **Evals rise much less than CPU**, x1.05–2.2. The extra time is in the surrogate.

Income, with `randreach`: `sur_stall=16` +0.0003, hit 98.4, x1.47; `sur_stall=32` +0.0003, hit 98.5, x1.88; `sur_c` hit 98.0; worst payload -0.001.

**`fromsolve` at HEAD** matches the loop, so the prototype's polish adds little:

| set | `S+fromsolve` | `S+randreach+fromsolve` |
|---|---|---|
| real_test | +0.0043, 98.7, x2.19 | +0.0042, 98.7 |
| quick_dev | +0.0012, 97.2, x1.67 | +0.0019, 97.6, x1.63 |
| speed_test | +0.0008, 98.1, x2.28 | +0.0008, 98.0 |
| large | +0.0100, 70.0, x1.97 | +0.0082, 66.7 |

Advanced, free_taps and juice all reach 100, and no payload is lost.

**The restart's reach (`randreach`) is a trade.**

- In the prototype: `single_3300` +0.105, `b10_single_3300` +0.071, `single_0159` +0.038. Against that `chain_0017` steps 1–3 lose -0.06 to -0.24 and `single_0041_serca_n19` -0.20. Overall quick_dev -0.0025, free_taps -0.0153.
- So `single_3300`'s gap was mostly this bug, but the first-four rule was also what reached `chain_0017`: those basins are at the front of the order, and insertions across the whole reach seldom land there.
- In `solve`'s loop at stall 32 the fix costs nothing: 2 wins, 0 losses of 632. `b10_single_3300` goes 0.071 to 0 on all 3 seeds, and `single_0044` +0.014. At stall 16 it costs `chain_0017` one seed each on steps 2–3.

**`skipw`, your point 5: the mismatch does not matter.** `sur_stall=32` with and without it: 0 wins, 0 losses of 632. In the prototype: quick_dev -0.0002, real_test 0, speed_test +0.0001, `large` +0.0050 on a bimodal set. Removed.

**Gaps at HEAD, 3 seeds:**

| payload | `sur_c` | 96k | `S` | loop 32 | loop 32, reach | `price+rand+empty`, reach |
|---|---|---|---|---|---|---|
| `single_3300` | 0 | 0 | 0.213 | 0 | 0 | 0 |
| `b10_single_3300` | 0.162 | 0.162 | 0.213 | 0.071 | **0** | **0** |
| `single_3375` | 0.047 | 0.033 | **0** | 0.019 | 0.019 | **0** |
| `single_3397_serca_n4` | 0.062 | 0.062 | 0.002 | 0.003 | 0.002 | 0.002 |
| `b10_week_inc_0004:3` | 0.064 | 0.035 | 0.021 | 0.014 | 0.014 | 0.007 |
| `chain_0017:2` | 0.072 | 0.072 | 0 | 0 | 0 | 0 |
| `chain_0017:3` | 0.175 | 0.116 | 0 | 0 | 0 | 0 |
| `chain_0001:0` | 0.010 | 0.001 | 0.181 | 0.010 | 0.010 | 0.010 |

With `price+rand+empty` and reach, all the open payloads are within 0.007, in `solve`, on 3 seeds. This mix is +0.0034 at hit 97.6 on real_test, slightly under `chest`'s mix, since `chest` carries `real_27a2bc40` there.

**The neutral-drift lead:** nothing to add. `skipw` left the search almost unchanged, so the surrogate can't separate that effect.

## 7. Batch `d`, on `sur_d`, new cut

Plain `sur_d` repeats `cut_a`: 0 wins, 0 losses, evals x1.00 on all 8 sets. `L` is `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach`.

**Timing.** Every CPU ratio is against plain `sur_d` in the same set. quick_dev uses the 763 runs of scenario+seed pairs that started after the stop (167 pairs are excluded). Cells are diff, hit, CPU / evals.

| set (`sur_d` hit) | L | true-response descent | no descent | normal model | four kinds | price+rand+chest, `randboth` | four kinds, stall 16 |
|---|---|---|---|---|---|---|---|
| real_test (96.6) | +0.0064, 97.9, x2.23/1.40 | +0.0063, 97.9, x2.39/1.40 | +0.0056, 97.1, x2.47/1.77 | +0.0060, 97.4, x2.14/1.38 | +0.0054, 98.4, x2.18/1.37 | +0.0038, 97.9, x2.03/1.34 | +0.0051, 97.9, x1.59/1.19 |
| quick_dev (95.5) | +0.0018, 97.3, x2.03/1.28 | +0.0018, 97.0, x2.04/1.28 | **+0.0008**, 96.3, x2.22/1.55 | +0.0018, 97.1, x1.94/1.29 | +0.0020, 97.3, x2.02/1.29 | +0.0017, 97.3, x1.94/1.28 | +0.0015, 96.8, x1.51/1.14 |
| speed_test (96.3) | +0.0016, 98.3, x1.96/1.23 | +0.0016, 98.1, x2.08/1.22 | +0.0015, 98.0, x1.97/1.39 | +0.0016, 98.4, x2.04/1.26 | +0.0012, 98.0, x1.94/1.23 | +0.0014, 98.3, x1.95/1.22 | +0.0009, 97.7, x1.45/1.11 |
| large (53.3) | +0.0066, 66.7, x3.08/2.35 | **+0.0035**, 63.3, x2.90/2.22 | **+0.0030**, 56.7, x3.16/2.89 | +0.0055, 63.3, x2.99/2.35 | +0.0072, 63.3, x3.12/2.30 | +0.0094, 66.7, x2.77/2.15 | +0.0070, 63.3, x2.04/1.66 |
| advanced (94.4) | +0.0031, 100, x2.44 | +0.0031, 100, x2.55 | +0.0031, 100, x2.40 | +0.0031, 100, x2.96 | +0.0031, 100, x2.47 | +0.0031, 100, x2.40 | +0.0031, 100, x1.75 |
| free_taps (97.2) | +0.0004, 100, x2.70 | +0.0004, 100, x2.48 | +0.0004, 100, x2.06 | +0.0004, 100, x2.73 | +0.0004, 100, x2.64 | +0.0004, 100, x2.70 | +0.0004, 100, x1.73 |
| juice (94.4) | +0.0034, 100, x2.23 | +0.0033, 100, x2.27 | +0.0031, 100, x1.92 | +0.0033, 100, x2.63 | +0.0034, 100, x2.26 | +0.0034, 100, x2.23 | +0.0034, 100, x1.64 |

Income: L +0.0002, hit 98.4, x1.97. True-response descent +0.0002, hit 98.1, x2.15. `sur_d` hit is 98.0. Worst payload -0.001.

No variant loses a payload: the worst row anywhere is -0.002.

**All of it is about 20 payloads.** Only 1 of 38 on real_test, 9 of 349 on quick_dev, 7 of 215 on speed_test and 2 of 10 on `large` differ between any two variants by over 0.01. Gaps, mean of 3 seeds:

| payload | `sur_d` | L | true | none | normal | four | `randboth` | stall 16 |
|---|---|---|---|---|---|---|---|---|
| `chain_0017:3` | 0.175 | 0 | 0 | **0.175** | 0 | 0 | 0 | 0.058 |
| `b10_chain_0017:3` | 0.092 | 0 | 0 | 0.052 | 0 | 0 | 0 | 0.052 |
| `chain_0017:2` | 0.072 | 0 | 0 | 0.055 | 0 | 0 | 0 | 0 |
| `b10_single_3300` | 0.162 | 0.039 | 0.039 | 0.071 | 0.039 | 0 | 0.110 | 0.039 |
| `b10_week_inc_0004:3` | 0.064 | 0.014 | 0.014 | **0.007** | 0.014 | 0.014 | 0.008 | 0.014 |
| `single_3116` | 0.020 | 0.002 | 0.017 | 0.020 | 0.010 | 0.004 | 0.001 | 0.004 |
| `single_3375` | 0.047 | 0.005 | 0.005 | 0.024 | 0 | 0.006 | 0 | 0.013 |
| `single_3397_serca_n4` | 0.062 | 0.003 | 0 | 0.005 | 0.005 | 0.003 | 0.002 | 0.003 |
| `single_3337` | 0.012 | 0.005 | 0.009 | 0.012 | 0.003 | 0.006 | 0.001 | 0.007 |
| `single_0044` | 0.079 | 0.044 | 0.071 | 0.060 | 0.053 | 0.042 | 0.020 | 0.042 |
| `single_0067` | 0.015 | 0.001 | 0.001 | 0.012 | 0.002 | 0.002 | 0.001 | 0.002 |
| `single_0085` | 0.037 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `real_27a2bc40_2991b9` (10 seeds) | 0.274 | 0.044 | 0.050 | 0.071 | 0.059 | 0.084 | 0.145 | 0.094 |

`chain_1051` steps 0-2 move the speed_test row between mixes, but they are listed as bimodal, and seed by seed they are one or two seeds switching between 0 and about 0.19.

**A descent is needed on part of it, and the part is order basins.** With no descent `chain_0017:3` stays at 0.175 on all three seeds, `single_3116` and `single_3337` do not move, and `single_0067`, `chain_0017:2` and `single_3375` lose one or two seeds. So quick_dev halves and `large` falls to +0.0030. It is not needed for `real_27a2bc40`, `chain_1051`, `single_0085`, `week_hone_0009:6`, or for `b10_week_inc_0004:3`, which goes further without it. No descent is not no re-plan: `empty` with no descent is a full gated rebuild, so those payloads say less than they seem to.

**The model was not shown to matter here. Section 8 shows it does.** At 3 seeds a true-response descent ties on quick_dev, speed_test, real_test and income, and the plain normal model ties everywhere. At 10 seeds (section 8) true responses fall behind and the normal model does not.

**Four kinds do not beat three.** Every difference between L and four kinds is `chain_1051`, `real_27a2bc40` and `b10_single_3300`, the listed noisy payloads, one seed each. Price matters on `large` only: price+rand+chest with reach (batch c) +0.0095, price+rand+empty +0.0099, `randboth` +0.0094, against L +0.0066 and four kinds +0.0072. That is `single_0044` alone, on 3 seeds. On a payload with no stack the chest kick moves nothing, so it spends a third or a quarter of the stall on nothing but the descent, which is why four kinds can be behind three on `large`.

**Stall 16 is enough for most, not all.** It costs `chain_0017:3`, `b10_chain_0017:3` and part of `single_3375`. It beats 96k at the same CPU or less on every set (batch c: real_test +0.0040, quick_dev +0.0008, speed_test +0.0002, `large` +0.0057, regime sets near 0, at x1.59-1.90, against +0.0051, +0.0015, +0.0009, +0.0070, +0.0031, +0.0004, +0.0034 at x1.45-2.04).

**`randboth` does nothing the reach does not**, confirmed at 10 seeds in section 8, and it is removed. With its default price+rand+chest, against the same kinds with `randreach` (`sur_c`): real_test +0.0038 and +0.0039, quick_dev +0.0017 and +0.0020, speed_test +0.0014 and +0.0013, `large` +0.0094 and +0.0095.

**CPU: no descent is no cheaper.** Without one, evals rise (x1.77 against x1.40 on real_test, x2.89 against x2.35 on `large`) and CPU is equal or higher on the main sets, since the polish has more to redo. Only free_taps and juice are cheaper. The line in section 5 that the CPU goes to the surrogate is too strong (see section 8).

**Every win is a joint move, which bears on the open question.** Of 45 seed wins of L over `sur_d` by more than 0.01, on 25 payloads (`scripts/joint.py`), not one is a single plan. The median is 7 normal plans moved. 31 of 45 also move the free tap order, 11 chests and 7 advanced plans. `single_3397` moves 3-4 of 4 plans plus the order, `single_3375` 12-14 of 21, `chain_0017:3` 7-10 of 12 plus an order swap at place 3, `single_0044` 17-36 of 40 plus the order. So the open payloads are plans and order moving together. Plans alone is only 9 of 45.

## 8. Batch `e`, on `sur_e`, new cut

Plain `sur_e` repeats `cut_a` on every session (metric and evals). Everything ran in one stretch, and plain CPU is x0.99–1.02 against `cut_a`. CPU is against plain `sur_e` in the same session. `e_seeds` re-ran seeds 0–2, which repeat exactly, so read `e` by session (121–124); the 10-seed figures are sessions 130–144 alone (`scripts/e_read.py`).

Variants:

- `L` = `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach`.
- `T` = L with true responses only in the descent (`truedescent`, `dorder+dadv+dchest` off).
- `C32` = `lprice+rand+chest+empty`, T, one round of responses (`d1`), `skipchest`.

**Batch e, 3 seeds (10 on real_test).** Each cell is diff, hit, CPU.

| variant | real_test | quick_dev | speed_test | large |
|---|---|---|---|---|
| L | +0.0064, 97.9, x2.32 | +0.0018, 97.3, x2.10 | +0.0016, 98.3, x2.02 | +0.0066, 66.7, x3.31 |
| no descent | +0.0056, 97.1, x2.51 | +0.0008, 96.3, x2.29 | +0.0015, 98.0, x2.02 | +0.0030, 56.7, x3.41 |
| descent without plans | +0.0046, 97.6, x2.76 | +0.0010, 96.5, x2.42 | +0.0018, 98.1, x2.21 | +0.0046, 60.0, x3.40 |
| descent of plans only | +0.0059, 97.9, x2.17 | +0.0018, 97.3, x2.20 | +0.0016, 98.3, x1.78 | +0.0042, 60.0, x3.06 |
| T | +0.0057, 97.6, x2.32 | +0.0017, 96.8, x2.13 | +0.0016, 98.1, x1.93 | +0.0017, 60.0, x2.88 |
| T, each response checked | same as T to the 4th place | same | same | +0.0016 |
| T, one round | +0.0055, 97.4, x2.23 | +0.0017, 97.0, x2.15 | +0.0016, 98.4, x1.85 | +0.0038, 60.0, x3.25 |
| T, order-only kick | +0.0057, 97.1, x2.20 | +0.0013, 96.4, x2.13 | +0.0013, 97.7, x1.85 | +0.0035, 60.0, x2.90 |
| T, `skipchest` | +0.0057, 97.6, x2.34 | +0.0016, 96.8, x2.52 | +0.0016, 98.3, x2.00 | +0.0016, 60.0, x3.14 |
| `lprice` mix, T | +0.0047, 97.4, x2.10 | +0.0012, 96.5, x1.99 | +0.0013, 98.1, x1.82 | +0.0020, 60.0, x2.76 |
| four kinds, surrogate | +0.0054, 98.4, x2.26 | +0.0020, 97.3, x2.12 | +0.0012, 98.0, x1.99 | +0.0072, 63.3, x3.23 |
| C32 | +0.0057, 97.6, x2.11 | +0.0015, 96.8, x2.24 | +0.0013, 98.1, x1.81 | +0.0031, 60.0, x3.03 |
| C16 | +0.0051, 97.6, x1.57 | +0.0013, 96.5, x1.62 | +0.0010, 98.0, x1.39 | +0.0020, 56.7, x2.02 |

- Regime sets (`e_regime`): L, T, C32 and C16 all hit 100 on advanced, free_taps and juice, and nothing moves on `small`. CPU: L x2.16–2.50, T x2.31–2.96, C16 x1.59–1.89.
- Income: L +0.0002, hit 98.4, x2.05; T +0.0001, hit 98.0, x2.36; C32 +0.0001, hit 98.1, x2.20.
- No variant loses a payload anywhere; the worst is -0.001.

**10 seeds on the payloads where the variants differ** (`e_seeds`). Mean gap:

| payload | `sur_e` | L | none | normal | T | T 1 round | P | P 16 | PRC | PRE | both | `lprice` | C32 | C16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `chain_0017:2` | 0.092 | **0** | 0.087 | 0 | 0.054 | 0.033 | 0.033 | 0.049 | 0.033 | 0.033 | 0 | 0.049 | 0.033 | 0.033 |
| `chain_0017:3` | 0.142 | 0.017 | 0.126 | 0.017 | 0.017 | 0.017 | 0 | 0.017 | 0.021 | 0 | 0 | 0.065 | 0.017 | 0.035 |
| `b10_chain_0017:2` | 0.060 | **0** | 0.049 | 0 | 0.015 | 0.015 | 0.030 | 0.030 | 0.015 | 0.015 | 0.030 | 0.030 | 0.015 | 0.015 |
| `b10_chain_0017:3` | 0.087 | **0** | 0.065 | 0 | 0.016 | 0.016 | 0.010 | 0.057 | 0 | 0 | 0 | 0.055 | 0.016 | 0.031 |
| `b10_single_3300` | 0.095 | 0.031 | 0.021 | 0.031 | 0.043 | 0.043 | 0.008 | 0.027 | 0.008 | 0.008 | 0.045 | 0 | 0.021 | 0.021 |
| `b10_week_inc_0004:3` | 0.039 | 0.008 | 0.002 | 0.006 | 0.006 | 0.006 | 0.004 | 0.004 | 0.008 | 0.006 | 0.005 | 0.006 | 0.006 | 0.008 |
| `single_3116` | 0.018 | **0.003** | 0.016 | 0.011 | 0.014 | 0.015 | 0.003 | 0.003 | 0.002 | 0.002 | 0.002 | 0.013 | 0.014 | 0.016 |
| `single_3337` | 0.014 | 0.008 | 0.012 | 0.004 | 0.010 | 0.011 | 0.006 | 0.008 | 0.007 | 0.007 | 0.004 | 0.013 | 0.011 | 0.012 |
| `single_3375` | 0.045 | 0.005 | 0.025 | 0.005 | 0.005 | 0 | 0.003 | 0.009 | 0.007 | 0 | 0.005 | 0 | 0 | 0.003 |
| `single_3397_serca_n4` | 0.042 | 0.002 | 0.006 | 0.002 | 0 | 0 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 | 0 | 0.001 | 0.001 |
| `single_0044` | 0.062 | 0.040 | 0.051 | 0.047 | 0.062 | 0.048 | 0.040 | 0.041 | **0.030** | 0.032 | 0.030 | 0.060 | 0.051 | 0.058 |
| `single_0067` | 0.013 | 0.001 | 0.006 | 0.005 | 0.005 | 0.005 | 0.001 | 0.001 | 0.001 | 0.001 | 0.003 | 0.005 | 0.005 | 0.008 |
| `single_0161` | 0.035 | 0.020 | 0.035 | 0.018 | 0.032 | 0.032 | 0.019 | 0.020 | 0.019 | 0.019 | 0.019 | 0.032 | 0.035 | 0.035 |
| `chain_1051` steps 0–2, summed | 0.304 | 0.134 | 0.134 | 0.134 | 0.134 | 0.134 | 0.217 | 0.292 | 0.229 | 0.229 | 0.229 | 0.208 | 0.208 | 0.284 |
| sum of 22 payloads | 1.140 | **0.289** | 0.673 | 0.300 | 0.431 | 0.390 | 0.393 | 0.583 | 0.406 | 0.370 | 0.396 | 0.559 | 0.452 | 0.587 |

The rest of the 22 payloads sum to 0.05 or less for every variant, but L leads there too.

P is all four kinds; P 16 is the same at stall 16; PRC is price+rand+chest with reach, PRE price+rand+empty with reach; both is `randboth`. `chain_1051` is bimodal (each seed ends near 0 or 0.19), so it is the noise in every column. Leaving it out, the sums are: L 0.155, PRE 0.141, normal 0.166, `randboth` 0.167, P 0.176, PRC 0.177, C32 0.244, T 1 round 0.256, P 16 0.291, T 0.297, `lprice` 0.351.

**What the loop's trace says** (`scripts/loop_read.py`, batch e). The rows are `sur_win` and `sur_kind`, from code since removed.

| set | loop share of CPU | kick and descent share of loop, L | the same, T | L: state below best after descent (median win) | T: the same | order already in place in order wins, L | the same, T |
|---|---|---|---|---|---|---|---|
| real_test | 59% | 55% | 20% | -0.003 | +0.041 | 25 of 36 | 6 of 30 |
| quick_dev | 51% | 37% | 16% | -0.002 | +0.110 | 66 of 94 | 6 of 76 |
| speed_test | 52% | 52% | 32% | -0.006 | 0.000 | 33 of 44 | 3 of 56 |
| large | 73% | 30% | 8% | +0.001 | +0.020 | 3 of 57 | 0 of 20 |

On `large`, a `chest` round on a payload with no stack is a surrogate descent from the best and nothing else. It wins 0.47 times a run under L and 0.50 with plans only, against 0.08 under T. `rand` carries every set's order basins: under L it gains 0.0048 a run on quick_dev and 0.0092 on `large`.

**The pieces:**

- **A re-plan between kick and polish is needed.** Without it: quick_dev +0.0008, `large` +0.0030, and a 10-seed sum of 0.673 against 0.289. `order_polish` puts a kicked order straight back unless the plans have moved first.
- **The re-plan must be on the model; true responses are not enough.**
  - At 3 seeds T ties L but for `large` (+0.0017 against +0.0066).
  - At 10 seeds T is 0.431 against 0.289, and one round of true responses 0.390.
  - It loses `chain_0017:2` (0.054 against 0), `b10_chain_0017` steps 2–3, `single_3116`, `single_0044` and `single_0161`.
  - The batch d smoke (3 seeds, `chain_0017` only) said otherwise.
- **Why the model works.** Its descent finds the new basin itself:
  - The winning state is at or above the best before any true move, and the new order is already in place in most order wins.
  - Under T the polish has to find the order, and it mostly does not.
  - A surrogate descent with no kick at all still wins on `large`, and a true one does not. So the model's error is a kick that moves many upgrades at once.
  - Every win is a joint move (section 7: median 7 plans, none single), which is what true responses taken one at a time rarely reach.
- **"Unchecked" is not the point.** Checking each true response by a full evaluation changed nothing on any set. A true response is its own check; what matters is the order of steps, and the model.
- **Both halves of the descent count.** Without its plans it loses quick_dev (+0.0010) and real_test (+0.0046). Without its order, advanced and chest moves it ties everywhere but `large` (+0.0042 against +0.0066, 3 seeds).
- **The model needs no calibration.** The normal model is 0.300 against 0.289 at 10 seeds, and level on every set in batch d.
- **The kinds trade payloads; no mix beats another.** Price carries `single_0044` (0.030 with price+rand+chest against 0.040). Rand+chest+empty carries `chain_0017:2` and `b10_chain_0017` steps 2–3. Four kinds dilute both (0.176 without `chain_1051`, against 0.155 and 0.141).
- **Stall 32, not 16.** At 16, all four kinds sum to 0.291 against 0.176 without `chain_1051`, losing order-basin seeds. It still beats `max_iters` 96k at about the same CPU (section 7).
- **Failed and removed:**
  - `lprice`: a price kick by linear responses under kicked shadow prices, no model. Sum 0.559; `single_0044` 0.060.
  - The model-free candidate: C32 0.452, C16 0.587.
  - `skipchest`: same gain, more CPU, since the stall then runs more real rounds.
  - `checked`: identical to T.
  - `d1`: only matters with T.
  - `randboth`.
- **CPU.** The loop is half to three quarters of a run. Most of it is the true polish of a kicked state, 50–3000 ms a round against 2–8 ms to build the surrogate. True responses make the descent cheaper but the polish dearer, so totals are level.

**Confound, settled in section 9.** `L` against `T` changes two parts at once: true responses in place of the surrogate's, and the descent's order, advanced and chest moves taken out. Batch f separates them.

- Most of the 0.289 against 0.431 above is the order moves.
- The order-in-place statistic belongs to the order moves, not the model.
- The model's own share is small, and price noise recovers it.

The bullets "The re-plan must be on the model" and "Why the model works" above are superseded by section 9.

## 9. Batch `f`, on `sur_f`, new cut

Plain `sur_f` repeats `cut_a` on the 144 runs that pair. Everything ran in one stretch. All figures are inside the batch, at 10 seeds:

- all of `large`;
- the quick_dev families `chain_0017`, `b10_single_3300`, `b10_week_inc_0004`, `single_3116`, `single_0085`, `single_3328`;
- the speed_test families `chain_1051`, `single_3397`, `single_3375`, `single_3337`, `week_hone_0009`, `single_1010`.

Variants: `R` = `sur_stall=32,sur_kinds=rand+chest+empty`, all with `randreach`.

| name | plans in the descent | order, adv and chest moves in the descent |
|---|---|---|
| L | surrogate | surrogate |
| SP | surrogate | none |
| TO | true | surrogate |
| T | true | none |
| NP | none | surrogate |
| Tn, Tn3 | true, on prices perturbed at 0.1 and 0.3 of log spread | none |
| TOn | true, on prices perturbed at 0.1 | surrogate, also on the perturbed prices |

The noise level: the surrogate's marginal price error against true shadow prices, on non-linear rows, has a median absolute log of 0.021, p75 0.068 and mean 0.121 (5,987 calibrations in a smoke).

**Gaps** (mean of 10 seeds, the payloads that move):

| payload | plain | L | SP | TO | T | NP | Tn | Tn3 | TOn |
|---|---|---|---|---|---|---|---|---|---|
| `chain_0017:2` | 0.092 | 0 | 0.054 | 0 | 0.054 | 0.087 | 0.054 | 0.054 | 0 |
| `chain_0017:3` | 0.142 | 0.017 | 0.017 | 0.017 | 0.017 | 0.126 | 0.017 | 0.017 | 0.017 |
| `b10_chain_0017:2` | 0.060 | 0 | 0.026 | 0 | 0.015 | 0.060 | 0.015 | 0.026 | 0 |
| `b10_chain_0017:3` | 0.087 | 0 | 0.016 | 0 | 0.016 | 0.065 | 0.016 | 0.016 | 0 |
| `chain_0017:1` | 0.028 | 0.012 | 0.010 | 0.008 | 0.011 | 0.021 | 0.005 | 0.003 | 0.007 |
| `b10_single_3300` | 0.095 | 0.031 | 0.043 | 0.031 | 0.043 | 0 | 0.043 | 0.064 | 0.031 |
| `b10_week_inc_0004:3` | 0.039 | 0.008 | 0.008 | 0.006 | 0.006 | 0.002 | 0.004 | 0 | 0.004 |
| `single_3116` | 0.018 | 0.003 | 0.003 | 0.013 | 0.014 | 0.016 | 0.011 | 0.001 | 0.011 |
| `single_3337` | 0.014 | 0.008 | 0.005 | 0.011 | 0.010 | 0.012 | 0.006 | 0.003 | 0.007 |
| `single_3375` | 0.045 | 0.005 | 0.005 | 0.005 | 0.005 | 0.025 | 0.004 | 0 | 0 |
| `single_3397_serca_n4` | 0.042 | 0.002 | 0.002 | 0.001 | 0 | 0.006 | 0.001 | 0.002 | 0.002 |
| `single_0044` | 0.062 | 0.040 | 0.046 | 0.051 | 0.062 | 0.057 | 0.050 | 0.032 | 0.047 |
| `single_0067` | 0.013 | 0.001 | 0.004 | 0.004 | 0.005 | 0.006 | 0.004 | 0.005 | 0.006 |
| `single_0161` | 0.035 | 0.020 | 0.032 | 0.021 | 0.032 | 0.027 | 0.035 | 0.035 | 0.020 |
| `single_0085` | 0.030 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `chain_1051` steps 0–2, summed | 0.304 | 0.134 | 0.134 | 0.134 | 0.134 | 0.002 | 0.134 | 0.134 | 0.038 |
| sum, 25 payloads | 1.287 | 0.411 | 0.534 | 0.437 | 0.558 | 0.658 | 0.522 | 0.505 | 0.312 |
| without `chain_1051` | 0.984 | **0.276** | 0.399 | 0.303 | 0.424 | 0.656 | 0.388 | 0.371 | **0.274** |
| `large` diff, 10 payloads | | +0.0057 | +0.0035 | +0.0039 | +0.0018 | +0.0031 | +0.0032 | +0.0054 | +0.0050 |
| `large` hit | | 59.0 | 56.0 | 56.0 | 55.0 | 57.0 | 59.0 | 61.0 | 59.0 |
| CPU, `large` / rest | 1 | x3.09 / 1.70 | x2.86 / 1.75 | x3.05 / 1.64 | x2.83 / 1.74 | x3.34 / 1.64 | x3.11 / 2.04 | x3.55 / 2.08 | x3.90 / 1.96 |

The other 23 payloads in the batch sum under 0.01 for every variant. `chain_1051` is bimodal and changes with any change of stream. No variant loses a seed to plain by over 0.0015.

**Model against true.**

- With no order moves the model is slightly ahead: SP 0.399, T 0.424, `large` +0.0035 against +0.0018.
- With order moves in both it is also slightly ahead: L 0.276, TO 0.303, `large` +0.0057 against +0.0039.
- At both levels the difference is the same payloads: `single_3116`, `single_3337`, `single_0044`, and `single_0161` without order moves. Elsewhere they tie.

**Order moves in the descent are the main piece.**

- L against SP and TO against T: 0.276 against 0.399, and 0.303 against 0.424.
- They alone take `chain_0017:2` and `b10_chain_0017` steps 2–3.
- They are why the new order is already in place in a win:

| | L | TO | SP | T |
|---|---|---|---|---|
| quick_dev, order in place / order wins | 47 of 63 | 36 of 51 | 2 of 51 | 3 of 50 |
| speed_test, the same | 22 of 30 | 23 of 25 | 1 of 44 | 1 of 35 |

- They are also why the state is at or above the best when the descent ends, which it is for L and TO only.

**Plans in the descent are needed too.** Without them (NP, 0.656) `chain_0017:3` is 0.126, `single_3375` 0.025 and `single_3116` 0.016. NP's wins (`b10_single_3300`, `b10_week_inc_0004:3`, `chain_1051`) are bimodal payloads.

**`large`'s two halves:** plans only +0.0035, order only +0.0031, both +0.0057. Each is about half, and they add up.

**Price noise recovers the model's edge, so the claim holds.**

- Without order moves, Tn3 is 0.371 against SP's 0.399, and `large` +0.0054 against +0.0035, level with L. It takes SP's payloads exactly: `single_3116` 0.001, `single_3337` 0.003, `single_0044` 0.032.
- With order moves, TOn 0.274 ties L 0.276.
- A chest round on a payload with no stack, a descent from the best and nothing else, wins on `large`:

| | T | Tn | Tn3 | TO | TOn | SP | L |
|---|---|---|---|---|---|---|---|
| wins a run | 0.17 | 1.22 | 1.76 | 0.31 | 1.81 | 0.42 | 0.42 |
| gain a run | 0.0001 | 0.0012 | 0.0036 | 0.0001 | 0.0018 | 0.0014 | 0.0009 |

- So where the model beats true responses, its error is the kick, and random noise of the same size or 3x does as well.
- Noise costs CPU (x3.55–3.90 on `large` against x3.09), because the extra wins reset the stall.
- This tests noise in marginal prices only, and that is enough to recover the effect.

## 10. Batch `g`, on `sur_g`, new cut

Plain `sur_g` repeats `cut_a` on all 144 runs that pair. L, TO and TOn repeat their `sur_f` runs exactly (480 each). Everything ran in one stretch, at 10 seeds, on f's payloads: all of `large`, six quick_dev families and six speed_test families. CPU is against plain inside the batch.

Variants: `R` = `sur_stall=32,sur_kinds=rand+chest+empty`, all with `randreach`.

| name | variant |
|---|---|
| L | surrogate descent |
| TO | true plans, surrogate order/adv/chest moves |
| TOn | TO with `noisy` |
| TT | true plans, true order/adv/chest moves (`trueorder`): nothing from the surrogate decides anything in the descent |
| TTn | TT with `noisy` |
| TTo | TT with `lab_off=dadv+dchest` |
| sk.1, sk.05 | L with `sur_skip` 0.1 and 0.05: no polish where the descent ends more than that % of flow below the best |
| scr | L with `sur_screen` |
| TTnS | TTn with `sur_screen` |

| | plain | L | TO | TOn | TT | TTn | TTo | sk.1 | sk.05 | scr | TTnS |
|---|---|---|---|---|---|---|---|---|---|---|---|
| sum of gaps, 25 payloads | 1.287 | 0.411 | 0.437 | 0.312 | 0.448 | 0.406 | 0.464 | 0.429 | 0.468 | 0.422 | 0.466 |
| without `chain_1051` | 0.984 | 0.276 | 0.303 | 0.274 | 0.313 | **0.264** | 0.330 | 0.294 | 0.334 | 0.288 | 0.275 |
| `large` diff | | +0.0057 | +0.0039 | +0.0050 | +0.0044 | +0.0059 | +0.0027 | +0.0057 | +0.0051 | +0.0051 | +0.0050 |
| CPU `large` | 1 | x3.11 | x3.08 | x3.94 | x4.38 | x5.83 | x3.24 | x2.19 | x2.08 | x2.35 | x4.50 |
| CPU quick_dev payloads | 1 | x1.59 | x1.56 | x1.90 | x1.71 | x2.07 | x1.70 | x1.53 | x1.41 | x1.38 | x1.68 |
| CPU speed_test payloads | 1 | x1.96 | x1.83 | x2.23 | x2.07 | x2.50 | x2.13 | x1.93 | x1.91 | x1.50 | x2.06 |
| evals `large` | 1 | x2.27 | x2.17 | x2.74 | x6.08 | x8.69 | x5.36 | x1.73 | x1.59 | x1.67 | x7.03 |

No run of any variant is more than 0.01% of flow below plain on any seed.

**Payloads that differ, seed by seed:**

- **`single_3116`** ends each seed near 0 or near 0.023. Seeds near 0.023, of 10: L 0 (worst 0.006), TO, TT and TTo 5, sk.1 and sk.05 5, TTn 4–5, TTnS 4, scr 2 (0.020, 0.022). It recurs, so the surrogate's plan steps really do carry this payload; price noise does not give it back here.
- **`single_0161`:** TTo is 0.037–0.048 on 5 seeds where TT is 0.018 or less. True adv moves carry it, and that is the whole of TTo's `large` loss.
- **`chain_0017:2`:** sk.1 and sk.05 each lose 1 seed (0.164); everything else is 0 on all seeds.
- **`b10_chain_0017:2`:** TT, TTo and sk.05 lose 1 seed (0.148); TTn and TTnS none.
- **`single_3375`:** scr has 0.016 on 3 seeds where L has it on 1, and 0.033 on one seed for both. The mean is 0.008 against 0.005, under the noise line.
- **`b10_single_3300`:** sk and scr variants 0.024, all others 0.031.

**A re-plan with no surrogate.**

- True order moves are worth what the surrogate's are (TT 0.313, TO 0.303). With price noise the surrogate-free re-plan reaches L (TTn 0.264, `large` +0.0059).
- True adv moves matter: without them `single_0161` is lost on 5 seeds and `large` falls to +0.0027. The round g smoke (2 seeds) had said they add nothing.
- `single_3116` is the one payload only L holds on every seed.
- **The cost is the descent.**
  - Evals on `large` are x8.69 against x2.27.
  - On `large`, L's kick and descent take 1,921 s and its polish 5,443 s. TT's take 7,864 s (about 4,070 s for the true adv and chest moves, about 3,600 s for the true order moves) and its polish 3,940 s.
  - Overall TTn is +87% of L's CPU on `large` and about +30% elsewhere. With the screen (TTnS 0.275) it is +45% on `large` and +5–6% elsewhere.

**A cheaper polish.**

- **`sur_screen` keeps L's gain on every payload within 0.01, at 13–24% less CPU.**
  - Sum 0.288 against 0.276.
  - `large` +0.0051 at x2.35 against +0.0057 at x3.11; quick_dev payloads x1.38 against x1.59; speed_test payloads x1.50 against x1.96.
  - The polish time on `large` halves (2,723 s against 5,443 s).
  - It shows a little on `single_3116` (2 seeds), `single_3375` (3 seeds of 0.016), and on `large`. So it is a CPU option on top of L, not a free win.
- **`sur_skip` saves only on `large`** (x2.19 at the same +0.0057) and 2–4% elsewhere. It loses `single_3116` on 5 seeds and one seed of `chain_0017:2`. At 0.05 it is worse (0.334). Removed.

## 11. Pieces

| Piece | Verdict | Where | Evidence | Cost | Why |
|---|---|---|---|---|---|
| Loop after `solve` (`sur_stall=32`, rand+chest+empty, `randreach`) | **Helps, candidate** | every set | real_test +0.0064, quick_dev +0.0018, speed_test +0.0016, `large` +0.0066, regime sets to 100, income +0.0002; no payload lost (sections 5–8) | x2.0–3.3 CPU | reaches order basins and joint moves that `solve`'s chains miss; every win moves several plans together (median 7, none single) |
| `sur_screen` | **Helps CPU, part of the candidate as an option** | every payload set | gap sum 0.288 against 0.276, every payload within 0.01 | 13–24% less than L | most kicked states are not new bests, and one polish round says so |
| Re-plan between kick and polish | **Needed** | order basins, `large` | without it quick_dev +0.0008 against +0.0018, sum 0.673 against 0.289 | | `order_polish` puts a kicked order back unless the plans have moved first |
| Order moves inside the re-plan | **The main piece** | `chain_0017`, `b10_chain_0017`, half of `large` | 0.276 against 0.399 (surrogate plans), 0.303 against 0.424 (true plans) | | order and plans settle together, so the descent ends on the new basin |
| Plans in the re-plan | **Needed** | `chain_0017:3`, `single_3375`, `single_3116`, half of `large` | without them 0.656 | | the kicked order needs plans fitted to it |
| Advanced moves in the re-plan | **Needed on advanced-heavy payloads** | `single_0161`; `chain_0001` in the prototype | true form without them: `single_0161` lost on 5 seeds, `large` +0.0027 against +0.0044 | | strategies move with the plans |
| Surrogate in place of true responses | **Small help** | `single_3116` (only L holds it on all seeds), `single_3337`, `single_0044` | 0.276 against 0.303 with order moves | about none | mostly its error acting as a kick: price noise on true moves reaches 0.264 |
| Re-plan with no surrogate (`truedescent+trueorder+noisy`) | **Matches L, dearer** | every payload set but `single_3116` | 0.264 against 0.276; `large` +0.0059 | +87% of L's CPU on `large`, about +30% elsewhere; with the screen +45% and +5% | true order and adv candidates are each a full evaluation |
| Price noise in the re-plan | **Helps true moves, not the surrogate** | `single_3375`, `single_0044`, `large` | TT 0.313 → TTn 0.264; TO 0.303 → TOn 0.274; L-level already with the surrogate | +25–35% CPU | a kick that moves many upgrades at once |
| Edgeworth terms, correction to shadow prices | **Nothing** | | normal model 0.300 against 0.289 at 10 seeds | | the error that helps is the kick, not accuracy |
| `points128` in the prototype | **Helps** | `single_3375`, `chain_0017:3`, `real_08927991` | section 4 | none | the `response_points` error again |
| Kick kinds | **Each its own payloads** | price `single_0044`; rand the order basins; chest real_test; empty real_test, `speed_test` | mixes 0.141–0.177 at 10 seeds without `chain_1051` | | |
| Stall 32 against 16 | **32** | order basins | 0.176 against 0.291 | 16 is x1.4–2.0 | |
| `randreach` | **Helps at stall 32** | `b10_single_3300`, `single_0044` | section 6 | none | |
| `sur_skip` | **Fails** | | saves 30% only on `large`; loses `single_3116` (5 of 10 seeds) | | deep rounds are cheap, and skipping them changes the stream |
| `randboth`, `lprice`, `skipchest`, `checked`, `d1`, a model-free candidate without true order moves | **Fail** | | `randboth` 0.396 against 0.406; `lprice` 0.559; `skipchest` same gain at more CPU; `checked` identical to T; C32 0.452 | | |
| The prototype as a whole | **Behind `solve`** | | quick_dev -0.0028, speed_test -0.0058 at effort 1 | | weak start (`real_08927991`, `chain_0001`); its budget doesn't bind |
| `sur_start`, `kickrepair`, `surpolish`, `blend`, `top8`, `pchest`, `last`, `flip`, `widest`, `advtrue`, `anystack`, `calibwide`, `skipw` | **Nothing** | | sections 4–6 | | |

## Not yet answered

- **The kick loop and `sur_stall` run together.** The better-of-two table is a bound by construction; a real test needs `price/final.patch` with `--3way`. Deferred to the final stage.

Nothing else the review asked is open. Whether a production version needs `surrogate.rs` is now a trade: L, or `truedescent+trueorder+noisy` at the CPU above. Building that version is design for the final stage.
