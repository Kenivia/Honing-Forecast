# Budget: how accuracy falls with CPU on real cases

**Verdict.**

- The CPU–accuracy curve of `solve` on real cases has the same shape for cold singles, income steps and family steps, with no knee. Each halving of CPU from 48k to about 6k costs about 0.05–0.1 in mlog, from about -2.8 to about -2.6.
- Later steps of a chain cost and score like cold runs under the current carry. `quick` and `carry_stall=1` are no better than a plain cut.
- Below about x0.3, the per-chain polishes, not the search, set the CPU.
- The cheap end's misses on real cases are mostly a deadlock between choice chests and the plans that use them, which generated cases rarely show.
- Suggested aim: about x0.2–0.25 of 48k at the accuracy of 12k–24k (§3).

**The question.** How does `solve`'s accuracy on real cases fall as its CPU is cut, far below 48k, separately for cold singles, the first and later steps of income chains, and chains of a character's own families, at streak length 1 and 10? Which part of that curve should a cheaper optimizer aim for?

All numbers are in % of flow against the best known for the same payload (per payload hash, over every run without a streak override).

- **mean**: the mean gap;
- **rms**: its root mean square, which weighs large misses;
- **mlog**: the mean of log10(gap + 0.001). -3 is every run exact; -2 is a typical miss of about 0.01;
- **hit**: within 0.01, kept for comparison with iterations 1–2;
- **cpu x**: CPU against `solve` 48k on the same runs of the same session.

The script is `scripts/curve.py`.

## 1. Batches

Snapshot `bud_a`, the code of 530bcd35. It repeats `cut_a` on `small` (60 of 60 runs, the same metrics and evaluation counts). 3 seeds; each set is one session.

| Batch | Sets | Mode | Variants |
|---|---|---|---|
| `a` | `real3_dev`, `real3_test` | cold | 48k; KMRe+Ls (`kick_stall=32,mem_polish=true,kick_types=window+chest+adv+piece+rebuildempty,sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach,sur_screen=true`) as the reference; `max_iters` 24k, 12k, 6k, 6k with 2 chains, 3k, 3k with 1 chain, 1.5k, 750, 0 |
| `a_inc` | `real3_income_dev`, `_test` | chained | 48k; KMRe+Ls; 24k, 12k, 6k, 3k, 0; `quick=true`; `carry_stall=1` |
| `a_seq` | `real3_seq_dev`, `_test` | chained | as `a_inc` without KMRe+Ls (its payloads are the singles', so `a`'s references cover them) |
| `a_gen` | `quick_dev` | cold | 48k; 3k, 1.5k, 750, 0 (generated, for contrast; `pr_a` holds 6k–96k) |

`max_iters=0` is the start and the polishes only: on a cold run, greedy order and shadow prices; on a chained one, the carried plan. Then the end polishes and the exchanges.

Smoke runs (1 seed, on slices) gave about 4.8 s a cold single at 48k, 0.5 s at 0 and 0.9 s at 3k; 8.5 s an income step and 5.5 s a family step at 48k.

## 2. Results

### 2.1 Cold singles (batch `a`)

831 payloads a streak length, 3 seeds. Below 16k, `solve` always runs 4 chains (`max_iters/4` caps the chain length), each ending in the full polishes; 24k runs about 6 chains and 48k about 12, with the agree exit.

Streak length 1:

| variant | cpu x | s/run | mean | rms | mlog | p90 | p99 | worst |
|---|---|---|---|---|---|---|---|---|
| KMRe+Ls | 2.59 | 10.8 | 0.0012 | 0.0059 | -2.88 | 0.0020 | 0.026 | 0.08 |
| 48k | 1.00 | 4.18 | 0.0026 | 0.0117 | -2.81 | 0.0044 | 0.063 | 0.19 |
| 24k | 0.60 | 2.49 | 0.0043 | 0.0173 | -2.74 | 0.0094 | 0.087 | 0.23 |
| 12k | 0.40 | 1.67 | 0.0072 | 0.0301 | -2.65 | 0.0168 | 0.109 | 0.58 |
| 6k | 0.32 | 1.32 | 0.0098 | 0.0400 | -2.59 | 0.0226 | 0.146 | 1.10 |
| 6k, 2 chains | 0.25 | 1.03 | 0.0126 | 0.0491 | -2.57 | 0.0258 | 0.247 | 0.69 |
| 3k | 0.27 | 1.15 | 0.0132 | 0.0459 | -2.53 | 0.0346 | 0.195 | 1.10 |
| 1.5k | 0.26 | 1.08 | 0.0188 | 0.0647 | -2.49 | 0.0460 | 0.354 | 1.10 |
| 750 | 0.25 | 1.05 | 0.0314 | 0.1375 | -2.44 | 0.0567 | 0.557 | 2.58 |
| 3k, 1 chain | 0.15 | 0.63 | 0.0235 | 0.0937 | -2.44 | 0.0484 | 0.418 | 1.81 |
| 0 | 0.13 | 0.54 | 0.1379 | 0.6330 | -2.20 | 0.2838 | 2.962 | 9.31 |

Streak length 10:

| variant | cpu x | s/run | mean | rms | mlog | p90 | p99 | worst |
|---|---|---|---|---|---|---|---|---|
| KMRe+Ls | 2.23 | 6.74 | 0.0011 | 0.0086 | -2.93 | 0.0001 | 0.033 | 0.17 |
| 48k | 1.00 | 3.02 | 0.0053 | 0.0374 | -2.85 | 0.0027 | 0.110 | 0.70 |
| 24k | 0.61 | 1.86 | 0.0083 | 0.0540 | -2.80 | 0.0072 | 0.194 | 1.08 |
| 12k | 0.39 | 1.19 | 0.0093 | 0.0529 | -2.76 | 0.0134 | 0.194 | 1.02 |
| 6k | 0.28 | 0.86 | 0.0132 | 0.0727 | -2.71 | 0.0205 | 0.270 | 1.12 |
| 6k, 2 chains | 0.22 | 0.67 | 0.0154 | 0.0756 | -2.68 | 0.0244 | 0.376 | 1.04 |
| 3k | 0.23 | 0.69 | 0.0149 | 0.0698 | -2.66 | 0.0308 | 0.270 | 1.29 |
| 1.5k | 0.20 | 0.60 | 0.0282 | 0.1742 | -2.60 | 0.0417 | 0.569 | 2.96 |
| 750 | 0.18 | 0.55 | 0.0487 | 0.2632 | -2.56 | 0.0524 | 1.326 | 3.00 |
| 3k, 1 chain | 0.11 | 0.35 | 0.0341 | 0.1787 | -2.57 | 0.0444 | 0.800 | 2.96 |
| 0 | 0.07 | 0.20 | 0.1465 | 0.6515 | -2.33 | 0.2893 | 2.962 | 9.31 |

**No knee above x0.25.** From 48k down to 6k, mlog falls steadily with log CPU: about 0.1 for every halving. Below x0.25, cutting `max_iters` saves almost nothing (750 costs as much as 3k), and only fewer chains or `max_iters=0` go lower.

**At low budgets, the polishes are the CPU, not the search.**

- Each chain ends in the full polishes, which cost about as much as a whole `max_iters=0` run. So 750, 1.5k and 3k all cost the same 4 polishes, and dropping to 1 chain halves the CPU of 3k.
- Annealing evaluations cost about 0.04 ms each, against about 0.27 ms for the start's and the polishes' evaluations: 3000 search iterations on 1 chain add only x0.02 to `max_iters=0`.
- The next lever down is therefore fewer or cheaper polishes, and longer chains, not fewer iterations.

**`max_iters=0` is deterministic and its misses are structural.** It misses 115 of 831 payloads by more than 0.1% at streak 1 (117 at streak 10), every seed alike. Its worst are tiny payloads: 1–3 upgrades, missed by 3–9%, which every search solves exactly.

The main cause is a deadlock between choice chests and the plan that uses what they give (`scripts/chest_split.py`).

- In the two worst tiny cases, `max_iters=0` opens every Glacier's/Lava's Breath choice chest for Glacier's Breath. Then neither the juice plan (a 1-upgrade case) nor the advanced honing plans (a 2-upgrade case) can justify the Lava's Breath the best plan uses.
- The polishes move one of the two at a time, so neither move pays alone.
- Of the 115 payloads it misses by more than 0.1% at streak 1, 84 have choice chests, and 74 of those open them for a different main option than the best plan does. Among the payloads it gets right, that is 37%: there the choice barely matters.
- At 3k with 1 chain, the share falls to 21 of 35, since the search makes joint moves.

A joint chest-and-plan move, or a start that tries each main option, is the first candidate for the cheap heuristics.

**One chain is a gamble on small payloads.** At 3k with 1 chain, 290 payloads are missed by more than 0.01% by some seed, but only 157 by every seed. One 4-upgrade payload is missed by 0, 2.9 and 2.9% across its seeds. Two chains (6k) bring those to 228 and 91, and the payloads missed by more than 0.1% by some seed from 67 to 44.

**By size, the CPU is in the big payloads.**

- CPU at 48k: 1–3 upgrades about 0.3 s, 4–10 about 1.2–1.8 s, 11–25 about 3.5–5 s, 26+ about 15–19 s.
- Payloads with 11+ upgrades are half the singles but 80–85% of the CPU (26+ alone, 7% of payloads, is about a third).
- Up to 10 upgrades, 3k (x0.15–0.23) is close to 48k except in the tail.
- 26+ is hard even for 48k: mlog about -2.6 at streak 1 and -2.5 at streak 10, with hit 86% and 78%. Its cheap variants still cost x0.3–0.45, because its polishes are expensive (`max_iters=0` takes 4 s there at streak 1).

| upgrades | payloads | 48k s/run (x1 / x10) | 48k mlog | 3k cpu x | 3k mlog | 0 mlog |
|---|---|---|---|---|---|---|
| 1–3 | 120 | 0.31 / 0.25 | -2.98 / -2.98 | 0.23 / 0.23 | -2.95 / -2.95 | -2.40 / -2.42 |
| 4–10 | 297 | 1.84 / 1.20 | -2.84 / -2.87 | 0.16 / 0.15 | -2.56 / -2.68 | -2.23 / -2.30 |
| 11–25 | 356 | 4.97 / 3.45 | -2.77 / -2.84 | 0.22 / 0.17 | -2.41 / -2.61 | -2.16 / -2.38 |
| 26+ | 58 | 19.3 / 15.4 | -2.61 / -2.53 | 0.42 / 0.34 | -2.22 / -2.27 | -1.88 / -1.97 |

**Streak 10:**

- Its runs cost about 30% less at 48k, and more than half less at `max_iters=0`. Its mlog is a little better than streak 1's, except on 26+.
- The hardest family is one roster's 10-upgrade character at streak 10: 48k misses it by 0.26–0.44% on average.

**Dev and test** show the same shape. Test has the heavier tail, including the 9% miss of `max_iters=0`.

**Weighting.**

- Weighting every roster equally changes the means by little and the order not at all.
- Weighting a single by its family's size makes every variant about 0.03–0.1 worse in mlog, so the cases users re-run are a little harder. The order holds.

### 2.2 Income chains (batch `a_inc`)

50 chains a streak length (one roster each), 13 steps, 3 seeds; each variant carries its own plan from step to step. Step 0 is a cold run, and `quick` and `carry_stall=1` are the same as 48k there.

All steps, CPU against 48k step by step:

| variant | x1 cpu x | x1 mean | x1 mlog | x1 p99 | x1 worst | x10 cpu x | x10 mean | x10 mlog | x10 p99 | x10 worst |
|---|---|---|---|---|---|---|---|---|---|---|
| KMRe+Ls | 2.98 | 0.0019 | -2.82 | 0.032 | 0.08 | 2.62 | 0.0016 | -2.87 | 0.031 | 0.09 |
| 48k | 1.00 | 0.0034 | -2.71 | 0.047 | 0.07 | 1.00 | 0.0038 | -2.77 | 0.059 | 0.13 |
| 24k | 0.72 | 0.0044 | -2.67 | 0.051 | 0.09 | 0.73 | 0.0047 | -2.73 | 0.072 | 0.21 |
| 12k | 0.50 | 0.0064 | -2.62 | 0.083 | 0.14 | 0.47 | 0.0057 | -2.68 | 0.078 | 0.27 |
| 6k | 0.39 | 0.0085 | -2.56 | 0.102 | 0.20 | 0.35 | 0.0070 | -2.64 | 0.083 | 0.14 |
| 3k | 0.33 | 0.0096 | -2.53 | 0.102 | 0.42 | 0.28 | 0.0080 | -2.62 | 0.093 | 0.25 |
| `quick=true` | 0.45 | 0.0102 | -2.51 | 0.123 | 0.19 | 0.48 | 0.0079 | -2.65 | 0.099 | 0.20 |
| `carry_stall=1` | 0.39 | 0.0176 | -2.43 | 0.182 | 0.26 | 0.40 | 0.0122 | -2.57 | 0.145 | 0.29 |
| 0 | 0.10 | 0.0427 | -2.19 | 0.529 | 0.86 | 0.06 | 0.0334 | -2.36 | 0.314 | 1.17 |

mlog by step (`scripts/by_step.py`), streak 1:

| variant | 0 | 1 | 2 | 3 | 4 | 6 | 8 | 10 | 12 |
|---|---|---|---|---|---|---|---|---|---|
| 48k | -2.79 | -2.72 | -2.72 | -2.68 | -2.64 | -2.68 | -2.67 | -2.74 | -2.76 |
| 3k | -2.46 | -2.51 | -2.47 | -2.50 | -2.46 | -2.48 | -2.55 | -2.60 | -2.65 |
| `quick=true` | -2.79 | -2.61 | -2.55 | -2.46 | -2.39 | -2.45 | -2.45 | -2.53 | -2.53 |
| `carry_stall=1` | -2.79 | -2.41 | -2.34 | -2.31 | -2.32 | -2.39 | -2.39 | -2.49 | -2.53 |
| 0 | -2.18 | -2.19 | -2.17 | -2.16 | -2.13 | -2.15 | -2.15 | -2.25 | -2.33 |

**Later steps are not easier under the current carry.**
- At every budget, steps 4+ score about the same mlog as step 0 and cost about the same CPU (48k: 8.0 s against 7.7 s at streak 1).
- A week's income moves the best plan enough that the carried plan plus the polishes (`max_iters=0`) misses by 0.04% of flow on average (p90 0.14%), about what `max_iters=0` misses on a cold step 0.

**Plain budget cuts beat the existing chained shortcuts.**
- `quick=true` and `carry_stall=1` lean on the carried plan and stop early. Neither beats a plain cut at the same CPU. At streak 1 both are worse than 3k, which costs less. At streak 10, `quick` (x0.48, -2.65) only matches 6k (x0.35, -2.64).
- They also get worse along the chain: `carry_stall=1` drops 0.4 in mlog at step 1. `quick` slides over the first 4–5 steps at streak 1, and over 10 at streak 10.
- The `max_iters` cuts, which still run fresh chains, hold steady from step to step, so their misses do not build up.

**The curve has the same shape as on singles, a little higher in CPU.**
- mlog falls by about 0.05–0.1 a halving from 48k to 6k.
- Below that, 3k (4 polished chains) costs x0.28–0.33, and only `max_iters=0` is cheaper.
- Income payloads are each roster's biggest character, so their polishes cost more than a typical single's.
- The tail is milder than on singles: at 3k the worst miss is 0.42% (0.25% at streak 10), against 1.1–1.3%.

### 2.3 Family chains (batch `a_seq`)

146 chains a streak length, each a character's families in order (3–10 steps, most 3–5), 3 seeds. A family differs from the one before in its grids or in at least 10% of the stock's gold value. Each step's payload is the same as that family's cold single in batch `a`.

All steps:

| variant | x1 cpu x | x1 mean | x1 mlog | x1 p99 | x1 worst | x10 cpu x | x10 mean | x10 mlog | x10 p99 | x10 worst |
|---|---|---|---|---|---|---|---|---|---|---|
| 48k | 1.00 | 0.0034 | -2.78 | 0.080 | 0.32 | 1.00 | 0.0050 | -2.83 | 0.130 | 0.40 |
| `quick=true` | 0.90 | 0.0037 | -2.77 | 0.080 | 0.32 | 0.90 | 0.0052 | -2.82 | 0.130 | 0.40 |
| 24k | 0.70 | 0.0043 | -2.74 | 0.087 | 0.32 | 0.70 | 0.0060 | -2.80 | 0.143 | 0.72 |
| `carry_stall=1` | 0.59 | 0.0104 | -2.66 | 0.207 | 0.90 | 0.56 | 0.0141 | -2.72 | 0.277 | 1.34 |
| 12k | 0.47 | 0.0065 | -2.67 | 0.115 | 0.48 | 0.45 | 0.0092 | -2.75 | 0.218 | 1.20 |
| 6k | 0.37 | 0.0086 | -2.60 | 0.137 | 0.58 | 0.32 | 0.0104 | -2.70 | 0.191 | 0.89 |
| 3k | 0.32 | 0.0129 | -2.55 | 0.202 | 1.10 | 0.25 | 0.0122 | -2.67 | 0.247 | 1.10 |
| 0 | 0.15 | 0.1063 | -2.21 | 2.463 | 3.80 | 0.07 | 0.1085 | -2.39 | 2.753 | 3.80 |

Steps 1+ against the cold single of the same payload, same variant and seed (`scripts/pair.py`; 1449 pairs a streak length; CPU in raw seconds, both batches at 16 jobs):

| variant | res | cold s | chained s | cold mean | chained mean | cold mlog | chained mlog | chained better / worse |
|---|---|---|---|---|---|---|---|---|
| 48k | 1 | 4.69 | 3.78 | 0.0028 | 0.0033 | -2.80 | -2.79 | 253 / 312 |
| 12k | 1 | 1.86 | 1.89 | 0.0080 | 0.0063 | -2.66 | -2.70 | 379 / 281 |
| 3k | 1 | 1.31 | 1.29 | 0.0126 | 0.0113 | -2.55 | -2.59 | 383 / 293 |
| 0 | 1 | 0.58 | 0.56 | 0.1240 | 0.1074 | -2.20 | -2.23 | 312 / 336 |
| 48k | 10 | 3.47 | 2.83 | 0.0057 | 0.0043 | -2.85 | -2.84 | 146 / 176 |
| 12k | 10 | 1.37 | 1.35 | 0.0102 | 0.0087 | -2.77 | -2.77 | 183 / 198 |
| 3k | 10 | 0.84 | 0.78 | 0.0155 | 0.0104 | -2.66 | -2.69 | 246 / 197 |
| 0 | 10 | 0.25 | 0.21 | 0.1368 | 0.1092 | -2.33 | -2.39 | 279 / 207 |

**The previous family's plan buys almost nothing.**
- At the same budget, a chained step scores within 0.06 in mlog of a cold run on the same payload, and wins on about as many payloads as it loses.
- It costs the same CPU, except at 48k, where the agree exit comes about 20% sooner.
- Even `max_iters=0`, the carried plan plus the polishes, does no better than the cold start's shadow prices plus the polishes.
- After a real change of stock or gear, a step is in effect a cold run.

**`quick` almost never fires on family chains.** It costs x0.85–0.92, because the polish moves a carried plan by more than its guard allows after a 10% stock change. `carry_stall=1` is again no better than a plain cut: x0.56–0.59 for about 6k's mlog.

**Per step**, steps 1–3 score better than step 0 at every budget. That is the payloads, not the carry: the pairs above show the same payloads score the same cold. Steps 6+ come from fewer than 30 chains and are noisy.

### 2.4 Generated cases (batch `a_gen`)

`quick_dev`, each step cold, 3 seeds (`scripts/gen.py`). Its best known comes from every run of iterations 1–3, a stronger reference than real3's, which would make gaps look larger, not smaller.

| variant | x1 cpu x | x1 mean | x1 mlog | x1 p99 | x10 cpu x | x10 mean | x10 mlog | x10 p99 |
|---|---|---|---|---|---|---|---|---|
| 48k | 1.00 | 0.0019 | -2.87 | 0.042 | 1.00 | 0.0019 | -2.94 | 0.017 |
| 3k | 0.34 | 0.0093 | -2.72 | 0.210 | 0.35 | 0.0038 | -2.87 | 0.081 |
| 1.5k | 0.33 | 0.0078 | -2.74 | 0.156 | 0.32 | 0.0054 | -2.82 | 0.101 |
| 750 | 0.33 | 0.0083 | -2.72 | 0.129 | 0.32 | 0.0051 | -2.84 | 0.120 |
| 0 | 0.19 | 0.0310 | -2.53 | 0.613 | 0.10 | 0.0278 | -2.70 | 0.483 |

**The cheap end is easier on generated cases than on real ones.**
- 48k is about the same on both (-2.87 against -2.81 at streak 1).
- `max_iters=0` misses far less on generated cases (-2.53 against -2.20 at streak 1, worst 0.6% against 9%). So does 3k (-2.72 against -2.53).
- Generated payloads rarely have the chest-choice deadlock of §2.1; `quick_dev` has half the choices.

The cheap end is where real cases differ, so it has to be tuned on real3, not on the generated sets.

## 3. Verdict: where to aim

**The curve is the same shape for every kind of run, and has no knee to aim at.**
- From 48k down to about 6k, mlog falls by about 0.05–0.1 a halving of CPU: about -2.8 at x1 and -2.55 to -2.7 at x0.3.
- Cold singles, income steps and family steps sit within about 0.1 of each other at every budget. Streak 10 is a little better than streak 1, except on payloads with 26+ upgrades.
- In absolute terms, at 3k (x0.23–0.33) the mean miss is about 0.01% of flow and the p99 about 0.1–0.27%, against 0.003–0.005% and 0.05–0.13% at 48k.

**Later steps of a chain are not cheaper under the current carry.**
- An income step after a week's income and a new family's step cost and score the same as a cold run at every budget (§2.2, §2.3).
- The shortcuts that trust the carried plan (`quick`, `carry_stall=1`) are no better than a plain cut, and on income chains they get worse step by step.
- A cheaper later step needs a better warm start (§2.1's chest deadlock is one reason the carried plan plus the polishes is not enough), not a smaller budget.

**Below about x0.3, `solve` is structurally wasteful.**
- Its 4 chains each end in the full polishes, which cost about as much as a `max_iters=0` run. Cutting `max_iters` below 6k saves almost nothing.
- One chain halves the CPU but is unreliable on small payloads.
- The real cases' cheap-end misses are mostly one structural failure (chests against plans) that the search covers up at a cost.

**Suggested aim:** about x0.2–0.25 of 48k on every kind of run, at the accuracy of 12k–24k today (mlog about -2.65 to -2.8, p99 about 0.1–0.2%).
- This is the region where the current cost structure, not the search, is the limit, so it has the most to gain.
- The CPU is in payloads with 11+ upgrades (80–85% of it). Up to 10 upgrades, 3k is already close to 48k.

The levers, in order:

1. Fewer or cheaper polishes at low budgets: polish only the best one or two chains, or fewer polish rounds. Existing knobs cover part of this (batch `b` below).
2. A joint chest-and-plan move in the start or polish (§2.1). It is the main cheap-end failure on real cases and would also make the carried plan a better warm start.
3. Then the cheap heuristics and the category split of the brief, measured against this curve.

**Proposed batch `b`** (not run): real3 singles, dev and test, 3 seeds, existing knobs only, about 1 hour:
- `bud_a`;
- `max_iters=3000,chains=2`, `max_iters=6000,chains=1`, `max_iters=12000,chains=1`, `max_iters=12000,chains=2`;
- `max_iters=3000,polish_rounds=1`, `max_iters=6000,chains=2,polish_rounds=1`, `max_iters=6000,chains=2,response_polish=1`.

It maps whether longer and fewer chains, or lighter polishes, already move the low end of the curve.
