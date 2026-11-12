# Cold singles at x0.2–0.25 of `solve` 48k

**Verdict: the aim is met at 12k's accuracy, not yet at 24k's.** 12k with `adv_radius=1,coarse_shared=true,coarse_cache=true` is x0.22 / 0.19 of 48k's CPU on `real3_dev` (streak 1 / 10) at mlog -2.61 / -2.74 (48k -2.79 / -2.84), and x0.26 / 0.26 at -2.67 / -2.76 on `real3_test`; adding `size_power=0.5` gives -2.63 / -2.76 at x0.23 / 0.19 (test -2.69 / -2.78 at x0.28). That is what 12k had at x0.41 / 0.40. Of the saving, the advanced polish's radius is x0.80, the shared coarse pass x0.68 and the cached cells x0.90–0.95; none changes accuracy, so they are worth having at any `max_iters`. 16k with them is x0.26 / 0.24 at -2.66 / -2.77 (test x0.31, -2.69 / -2.77); 24k's -2.69 / -2.79 would need about x1.5 more.

- Every search knob tried (chain polishes, the exchange's loops and stops, chest flips, chain counts, `max_iters` by size) is on the curve of `max_iters` to within about 0.02 of mlog; the only one above it is `size_power=0.5`, by 0.01–0.03.
- The rest of a large run's time is spread over parts of 5–30% each (§6), so the next x0.9 is one of several small evaluator changes.
- Candidates, in tree and off: `adv_radius=1`, `coarse_shared`, `coarse_cache`, `size_power=0.5`. Removed: `exchange_flat`, `exchange_rank`, `min_chains`.

Aim (user, 2026-11-12): cold starts at x0.2–0.25 of `solve` 48k's CPU, as accurate as possible; chained runs left out for now. Any heuristic may be tried, and ones that only work in one regime can be pieced together. Judged on `real3_dev` (both streaks, 3 seeds); finalists confirmed on `real3_test`. Metrics as in `budget/report.md`: gap in % of flow against the best known per payload hash, mlog = mean log10(gap + 0.001), RMS, hit (≤ 0.01), p90/p99/worst; CPU as a ratio of sums against 48k in the same session.

## 1. Code (snapshot `cold_a`, repeats `bud_a` seed for seed)

- **Time by phase** (`Probe::time`, trace only), stored per run as `us_<phase>` counters, wall µs: `start` (greedy order, shadow start, warm plans), `chains` (each chain's anneal, polish included), `moves` (the annealing loop alone), `polish` (every end polish, chains' and after), `order`/`resp`/`adv`/`chest`/`flip` (the polishes' parts), `merge`, `exchange`, `post` (everything after the chains). A chain's polish is `chains` - `moves`.
- **`chain_polish`**: `screen` ends each chain on one round of the polishes with one pass of responses, `none` on none; the best `polish_top` results then get the full polish before the merge.
- **`chest_flip`**: each stack with a choice opened all on each option but its main one, the normal upgrades re-planned by responses (two passes at most), kept where it beats the best. `all` in every polish round, `final` once after the exchanges (then a full polish where it gained).
- **The exchange's loops**: `exchange_rounds` (2), `exchange_powers` (7 amounts), `refine_rounds` (3), and `exchange_stop` (a pair's amounts stop at the first that does not gain).

## 2. Smoke tests (148 `real3_dev` scenarios matching `r3_0`, 3 seeds, scratch)

Shares of the run's wall time. Not judged on: a small subset, and its best known is only what has been run on it.

- **The exchange is most of the cheap end's time at streak 1**: 77% of `max_iters=0`, 31% of 6k with 2 chains, 21% of 12k screened; 6% of 48k. At streak 10 it is 8% at 6k.
- **And it is worth it**: without it `max_iters=0` falls from -2.29 to -1.92 (streak 1) and 6k with 2 chains from -2.63 to -2.53. Fewer amounts (`exchange_powers=3`) or one pass (`exchange_rounds=1`) save 3–10% of the run and lose little; `exchange_stop` saves 26% at 6k with 2 chains for -0.05 at streak 1 (none at 10), but at `max_iters=0` it loses 0.2.
- **`chest_flip=final` at `max_iters=0`**: -2.29 to -2.37 at streak 1, -2.40 to -2.57 at 10, the worst payload 4.23 to 2.99, for 5–10% more time. `all` does as well for 25–60% more. At 6k with 2 chains `all` changes nothing.
- **`chain_polish` at 12k**: screened -2.69 / -2.83 at about x0.3, none -2.64 / -2.76.

## 3. Batch `a` (snapshot `cold_a`, `real3_dev`, 1266 runs a variant a streak, 2 h 28 min)

`scripts/cold.py cold_a [--size] [--phases]`, `scripts/mix.py cold_a [--chests]`. Streak 1 / streak 10:

| variant | cpu x | mlog | hit% | p99 |
| --- | --- | --- | --- | --- |
| 48k | 1.00 / 1.00 | -2.80 / -2.85 | 94.8 / 93.8 | 0.043 / 0.105 |
| 24k | 0.60 / 0.61 | -2.72 / -2.79 | 90.8 / 91.4 | 0.078 / 0.153 |
| 24k screen | 0.55 / 0.58 | -2.69 / -2.78 | 90.0 / 90.8 | 0.063 / 0.179 |
| 12k | 0.41 / 0.40 | -2.62 / -2.75 | 84.4 / 88.2 | 0.074 / 0.153 |
| 12k screen | 0.35 / 0.35 | -2.59 / -2.73 | 83.4 / 86.9 | 0.083 / 0.168 |
| 12k none | 0.31 / 0.31 | -2.56 / -2.69 | 81.0 / 85.2 | 0.122 / 0.197 |
| 6k | 0.33 / 0.30 | -2.54 / -2.68 | 79.8 / 84.4 | 0.111 / 0.205 |
| 6k, 2 chains | 0.25 / 0.22 | -2.52 / -2.65 | 78.1 / 82.2 | 0.143 / 0.283 |
| … `refine_rounds=1` | 0.23 / 0.22 | -2.51 / -2.65 | 77.8 / 81.9 | 0.143 / 0.283 |
| … `exchange_stop` | 0.22 / 0.22 | -2.49 / -2.65 | 75.7 / 82.2 | 0.143 / 0.283 |
| … `polish_rounds=1` | 0.21 / 0.19 | -2.49 / -2.62 | 76.7 / 80.7 | 0.177 / 0.347 |
| 3k | 0.30 / 0.25 | -2.45 / -2.63 | 74.2 / 82.4 | 0.193 / 0.231 |
| m0 | 0.14 / 0.08 | -2.14 / -2.29 | 61.8 / 69.9 | 1.95 / 1.86 |
| m0 `chest_flip=final` | 0.18 / 0.10 | -2.22 / -2.41 | 64.0 / 73.7 | 0.631 / 0.624 |
| m0 `chest_flip=all` | 0.25 / 0.15 | -2.23 / -2.42 | 63.7 / 74.9 | 0.631 / 0.463 |

- **Every knob is on the curve of `max_iters`.** At x0.21–0.25 the best are -2.49 to -2.52 / -2.62 to -2.65, against -2.62 / -2.75 at 12k: the aim is about 0.1 of mlog away, or x1.6 of CPU at the same accuracy.
- **A setting per size does not close it.** The best variant per streak × size (`mix.py`, picked in sample on dev) is -2.54 at x0.23 / -2.69 at x0.22, 0.02–0.04 over the best single setting.
- **Where the CPU is.** Payloads of 11–25 and 26+ upgrades are 45% and 42% of 48k's CPU at streak 1 (43% and 46% at 10); up to 10 upgrades is 13%. An evaluated state costs 50 µs up to 10 upgrades, 120–130 µs at 11–25 and 330–370 µs at 26+ (§5 has where). On 26+ at 6k with 2 chains the polishes and the exchange evaluate more states than the 6k moves (13.7k in all), and the time is: advanced polish 33% / 43%, responses 16% / 7%, exchange 14% / 2%, moves 28% / 38%. At 11–25, streak 1: exchange 20%, responses 11%, advanced 5%.
- **The advanced polish** tries all 19 values of every entry of every advanced upgrade, in every chain's polish and again in each refine round; on 26+ at streak 10 it is 42% of 12k and 66% of `max_iters=0`.
- **`chest_flip=final` only pays without chains.** At m0 it halves the mean gap (0.091 to 0.053 / 0.090 to 0.048) and the p99 falls by two thirds for x0.03; at 6k with 2 chains and above it is +3% CPU for nothing at streak 1 (-2.66 against -2.65 at 10): the chains' chest moves already find what it finds.
- **`exchange_stop`** at 12k: x0.38 -2.60 against x0.41 -2.62 at streak 1; at 10 the exchange is 1% of the time.

## 4. Round 2 (snapshot `cold_b`, repeats `cold_a` seed for seed)

Code:

- **`adv_radius`**: the advanced polish walks each entry within that many values of its best so far, until a step gains nothing, in place of trying all 19 values (0, the default, tries all).
- **`exchange_rank`**: per id, each user's one tap (or block) given up and taken on alone is evaluated (2 evaluations a user); only the best that many pairs by the sum of the two are tried (0, the default, tries every pair). The exchange is now `shifted` and `put`, the same moves in the same order.
- **`exchange_flat=false`** leaves out ids at one price throughout.

Smoke (148 `r3_0` scenarios, 3 seeds, scratch; streak 1 / 10):

- Repeats: `cold_b` with nothing set and `exchange_stop` equal `cold_a` on 444 of 444 runs.
- `adv_radius=1` takes the advanced polish's time to an eighth (this subset has little advanced honing, so nothing else moved).
- `exchange_flat=false` changed nothing: every juice and book the exchange touched there has more than one price.
- `coarse_cells=4` at 6k with 2 chains: moves 9% faster, the run 6% faster, hit 79 to 76%. The coarse lattice is not where the time is.
- `exchange_rank` at 6k with 2 chains: rank 12 is 0.85 s a run for -2.58 / -2.65, against 0.91 s for -2.58 / -2.72 with `exchange_stop` (and 1.24 s, -2.63 at streak 1, with neither in §2). At `max_iters=0` it takes the run from 0.59 to 0.25 s at streak 1 for -2.27 to -2.25 (against `refine_rounds=1`), but loses at 10.

Batch `b` (`real3_dev`, 23 variants, 1 h 48 min): the references 48k, 12k, 6k with 2 chains; `adv_radius` 1 and 2; `exchange_rank=12` (with and without `exchange_stop`); `max_lattice_work` 50k and 20k, `special_tol=3e-4`, `max_memo_size=262144` at 12k; `magic_number=0.25` at 12k and 6k with 2 chains; and S = `adv_radius=1,exchange_stop=true` at 12k (plain, screened, with rank 12), 16k and 24k screened, 8k and 6k with 2 chains, and m0 with `chest_flip=final`.

`scripts/cold.py cold_b [--size] [--phases]`, `scripts/mix.py cold_b`. Streak 1 / streak 10:

| variant | cpu x | mlog | hit% | p99 |
| --- | --- | --- | --- | --- |
| 48k | 1.00 / 1.00 | -2.80 / -2.85 | 94.7 / 93.8 | 0.043 / 0.110 |
| 24k S screen | 0.49 / 0.53 | -2.67 / -2.78 | 88.9 / 90.8 | 0.077 / 0.179 |
| 16k S screen | 0.36 / 0.38 | -2.61 / -2.75 | 84.9 / 88.8 | 0.096 / 0.144 |
| 12k | 0.41 / 0.40 | -2.62 / -2.75 | 84.4 / 88.2 | 0.074 / 0.153 |
| 12k `adv_radius=1` | 0.34 / 0.31 | -2.62 / -2.75 | 84.4 / 88.2 | 0.074 / 0.153 |
| 12k `adv_radius=2` | 0.35 / 0.32 | -2.62 / -2.75 | 84.4 / 88.2 | 0.074 / 0.153 |
| 12k S | 0.32 / 0.31 | -2.60 / -2.75 | 83.1 / 88.2 | 0.078 / 0.153 |
| 12k S screen | 0.28 / 0.30 | -2.57 / -2.73 | 82.0 / 87.0 | 0.084 / 0.168 |
| 12k S, rank 12 | 0.31 / 0.31 | -2.60 / -2.73 | 82.7 / 87.1 | 0.084 / 0.160 |
| 12k rank 12 | 0.38 / 0.39 | -2.61 / -2.73 | 83.3 / 87.0 | 0.083 / 0.160 |
| 12k `special_tol=3e-4` | 0.40 / 0.39 | -2.62 / -2.76 | 84.9 / 89.2 | 0.078 / 0.117 |
| 12k `max_lattice_work=20000` | 0.41 / 0.39 | -2.61 / -2.74 | 84.6 / 88.2 | 0.080 / 0.149 |
| 12k `magic_number=0.25` | 0.41 / 0.39 | -2.62 / -2.74 | 85.3 / 88.5 | 0.084 / 0.158 |
| 8k, 2 chains, S | 0.21 / 0.22 | -2.51 / -2.68 | 76.7 / 84.8 | 0.134 / 0.232 |
| 6k, 2 chains | 0.25 / 0.22 | -2.51 / -2.65 | 78.1 / 82.1 | 0.143 / 0.283 |
| … `adv_radius=1` | 0.21 / 0.17 | -2.51 / -2.65 | 78.3 / 82.2 | 0.140 / 0.283 |
| … S | 0.18 / 0.17 | -2.49 / -2.65 | 75.8 / 82.2 | 0.154 / 0.283 |
| … rank 12 | 0.21 / 0.22 | -2.49 / -2.63 | 76.2 / 81.2 | 0.164 / 0.283 |
| … `magic_number=0.25` | 0.25 / 0.22 | -2.51 / -2.67 | 77.4 / 84.1 | 0.157 / 0.270 |
| m0 S `chest_flip=final` | 0.10 / 0.06 | -2.13 / -2.40 | 56.9 / 73.2 | 0.546 / 0.560 |

- **`adv_radius=1` is free.** Every payload ends the same to the digit (mlog, hit, p99 alike) for 17% less CPU at 12k streak 1 and 23% at 10; at 26+ upgrades 0.48 to 0.34 and 0.44 to 0.28. A radius of 2 is no better. It moves the whole curve left; nothing else in this batch does.
- **`exchange_rank` loses to `exchange_stop`** at the same CPU (12k: -2.61 / -2.73 at x0.38 against S's -2.60 / -2.75 at x0.32), and with it adds nothing.
- **The evaluator's cut, lattice and memo sizes change neither CPU nor accuracy** at 12k; `magic_number=0.25` is within the noise.
- **Picking per size again does not close the gap**: in sample, -2.66 at x0.36 / -2.76 at x0.29.
- After `adv_radius`, 12k on 26+ is moves 45% / 62%, the chains' polish 38% / 34% (responses 23% / 11%), the exchange 11% / 2%.

## 5. Round 3: where an evaluation goes (snapshot `cold_c`, repeats `cold_b` seed for seed)

**Profile.** `samply` needs Administrator rights on Windows, so timers went into the evaluator for one scratch build (never a snapshot): `Probe::time` around the distribution updates, the keys, the free tap DP, each material's table (base materials and juices apart) and the coarse lattice's parts. On one roster's 41–46 upgrade payloads at 12k `adv_radius=1`: evaluations are 72–87% of the wall time; **the base materials' coarse lattices alone are 60–76%**, at 22–27 coarse lattices an evaluation; the juices' and books' exact lattices 12–23%; updating distributions, the keys and the free tap DP 1–3% each. Each free tap outcome of a base material had a coarse lattice of its own, a pass over every upgrade, where the exact lattice already builds them all in one pass from the last upgrade back.

**`coarse_shared`** (off by default, in the memo's key only when on): a material's coarse lattices for every outcome in one pass from the last upgrade back, each read off as the pass reaches it, on the finest of their steps. Off, the code does the same arithmetic in the same order (`Grid::add` is the old loop's body).

- Repeats: off equals `cold_b` on 1080 of 1080 runs (12k and 6k with 2 chains, `adv_radius=1`, 148 `r3_0` scenarios, 3 seeds), metric and evaluations; the `cold_c` snapshot equals the timed build on both.
- Error against the reference on the 844 best known `real3_dev` states (`bench --states`): mean 3.8e-4 against 4.3e-4 % of flow, p99 2.5e-3 against 3.0–3.5e-3, worst the same. The given and scrambled states move by as little; their larger errors (to 40% of flow on scrambles 200 moves away) are the free tap cut's and the same with it off.
- Smoke (148 `r3_0` scenarios, 3 seeds; streak 1 / 10): 12k `adv_radius=1` from 474 / 323 CPU s to 261 / 178 (x0.55), mlog -2.72 / -2.84 to -2.72 / -2.80, hit 91.9 / 91.4 to 93.7 / 89.2; 6k with 2 chains x0.55 / 0.56, -2.63 / -2.72 to -2.62 / -2.71. With it the coarse lattices are 33% of the wall, the juices 14%, the free tap DP 7%.

Batch `c` (13 variants on `real3_dev`, then 4 on `real3_test`, 1 h 24 min): A = `adv_radius=1,coarse_shared=true`. `scripts/cold.py cold_c --split dev|test [--size] [--phases]`, `scripts/mix.py cold_c --split dev`. Streak 1 / streak 10:

| dev | cpu x | mlog | hit% | p99 |
| --- | --- | --- | --- | --- |
| 48k | 1.00 / 1.00 | -2.79 / -2.84 | 94.5 / 93.8 | 0.043 / 0.110 |
| 48k A | 0.60 / 0.58 | -2.77 / -2.84 | 93.5 / 93.7 | 0.041 / 0.110 |
| 24k A | 0.35 / 0.34 | -2.69 / -2.79 | 89.8 / 91.2 | 0.079 / 0.132 |
| 20k A | 0.31 / 0.30 | -2.68 / -2.78 | 88.2 / 90.8 | 0.080 / 0.160 |
| 16k A | 0.27 / 0.26 | -2.65 / -2.77 | 87.4 / 90.0 | 0.085 / 0.146 |
| 16k A `coarse_cells=6` | 0.26 / 0.25 | -2.62 / -2.76 | 86.2 / 90.0 | 0.086 / 0.145 |
| 16k A `exchange_stop` | 0.26 / 0.25 | -2.63 / -2.77 | 85.8 / 90.0 | 0.087 / 0.146 |
| 12k `adv_radius=1` | 0.34 / 0.31 | -2.62 / -2.75 | 84.4 / 88.2 | 0.075 / 0.153 |
| **12k A** | **0.23 / 0.21** | **-2.62 / -2.74** | 85.2 / 87.6 | 0.080 / 0.140 |
| 12k A `coarse_cells=6` | 0.22 / 0.20 | -2.60 / -2.73 | 84.2 / 87.7 | 0.085 / 0.159 |
| 8k A, 2 chains | 0.16 / 0.14 | -2.52 / -2.68 | 77.7 / 84.3 | 0.132 / 0.249 |
| 6k A, 2 chains | 0.14 / 0.12 | -2.50 / -2.66 | 76.9 / 83.0 | 0.153 / 0.270 |
| m0 A `chest_flip=final` | 0.09 / 0.04 | -2.20 / -2.39 | 62.6 / 74.4 | 0.546 / 0.560 |

| test | cpu x | mlog | hit% | p99 |
| --- | --- | --- | --- | --- |
| 48k | 1.00 / 1.00 | -2.80 / -2.85 | 92.9 / 93.6 | 0.073 / 0.109 |
| 20k A | 0.37 / 0.39 | -2.71 / -2.80 | 88.8 / 91.8 | 0.194 / 0.520 |
| 16k A | 0.33 / 0.34 | -2.69 / -2.78 | 87.9 / 90.7 | 0.194 / 0.520 |
| 12k A | 0.27 / 0.28 | -2.67 / -2.76 | 87.0 / 89.2 | 0.194 / 0.280 |

- **The aim is met at 12k's accuracy.** 12k A is x0.23 / 0.21 for the -2.62 / -2.74 that 12k had at x0.41 / 0.40 in batch `a`; `coarse_shared` alone takes 12k `adv_radius=1` from x0.34 / 0.31 to x0.23 / 0.21 at the same mlog. 16k A is -2.65 / -2.77 at x0.27 / 0.26, 24k A -2.69 / -2.79 at x0.35.
- **Test agrees** but its CPU ratios are higher (12k A x0.27 / 0.28): it has fewer large payloads (48k's s/run 3.80 against 5.16), where `coarse_shared` saves most.
- **Test's tails are restarts on small payloads, not the evaluator.** Its p99 and worst (0.19–0.74 at streak 1, 0.28–1.18 at 10, against 48k's 0.07 / 0.11) come from payloads of 1–10 upgrades that miss on one seed of three: `e482eed2_b16830` (1 upgrade) 0.37–0.74, `7a97b994_cf2d0d` 0.27–0.77, `27a2bc40_2991b9` 0.5–1.2 (where 48k also misses 0.65–0.70 on a seed at streak 10). 16k and 20k end identical there: both are 4 chains of 4000 on a small payload, 48k is 12.
- `coarse_cells=6` and `exchange_stop` are on the curve; m0 with `chest_flip=final` stays far below it.
- **Where the next CPU is worth most.** By size, going 12k A to 24k A gains 0.10 mlog for 0.46 s a run on 4–10 upgrades, 0.10 for 0.80 s on 11–25 and 0.09 for 1.47 s on 26+ (streak 1): each size gains about half as much per CPU second as the one below it. Picked per size in sample, -2.63 at x0.22 / -2.74 at x0.17 (`mix.py`), 4–10 at 20–24k, 11–25 at 12k, 26+ at 6–8k with 2 chains.

## 6. Round 4 (snapshot `cold_d`, repeats `cold_c` seed for seed)

**Profile again** (temporary `Probe::time` timers, scratch build, 12k A on the 148 `r3_0` scenarios, one seed). At 11+ upgrades evaluations are 73–80% of the wall: the coarse lattices' pass 28–32%, reading each outcome off it 8% (a copy of the grid, its cdf and moments), the outcomes' shapes 5–6%, the free tap DP 5–14% (order moves that miss its cache; most at streak 10), the memo's lookups 5%, the exact lattices 1–10% (32–34% at 4–10 upgrades). No one part is left to take a large share off.

- The outcomes' steps are within 1.09–1.15 of each other, so a step per outcome would save 3–10% of the pass: not worth it.
- An add is 30–40 outcome pairs into 6–8 cells, convolved with 25–33 grid points. Done twice, the binning of the pairs adds 12.4% of the run, the convolution 5.7%.
- Built for `x86-64-v2` (`floor` in an instruction, as it is in wasm) the run takes the same time: not a library call's cost.
- A material's step is the same as at its last evaluation only 12–39% of the time, so the cells cannot be kept as they are.

Code:

- **`coarse_cache`** (with `coarse_shared`; in the memo's key): steps on a ladder of eighth powers of two (up to 9% finer), and each upgrade's cells and the variance they add kept by upgrade, state, material and step (`CoarseMemo`, 32k entries, cleared when full; a copy of the state starts without it, so the chains' and polishes' copies cost nothing). Only the optimizer's evaluation keeps them; responses, shadow prices and the UI do not. Off, the code computes as before (`bin` and `Grid::convolve` are the old loop's two halves).
- **`size_power`** (0, off), `size_ref` (16), `size_cap` (48000): `max_iters` times (`size_ref` / upgrades) to that power, at most `size_cap`. At 12k and 0.5: 24k at 4 upgrades, 17k at 8, 8.5k at 32.
- **`min_chains`** (4): the fewest chains `chains` 0 makes (removed after batch `d`).

Checks:

- Repeats: `cold_d` 12k A equals `cold_c` on 444 of 444 runs (148 `r3_0` scenarios, 3 seeds), metric and evaluations.
- Error against the reference on the 844 best known dev states: mean 3.79e-4 against 3.76e-4 for `coarse_shared` alone, p99 3.6e-3 against 2.5e-3 (streak 1). One given state is 2.2e-2 against 4.2e-3; the shared pass alone at `coarse_cells=8.3` (a 4% finer step, no cache) gives it 2.3e-2, so it is the grid's step, not the cache.
- Smoke (148 `r3_0` scenarios, 3 seeds; streak 1 / 10): the cache takes 12k A from 255 / 175 CPU s to 241 / 160 (x0.94 / 0.92), mlog -2.72 / -2.80 to -2.71 / -2.83; with `size_power=0.5`, 240 / 154 s, -2.74 / -2.83.

Batch `d` (C = A + `coarse_cache=true`; 13 variants on `real3_dev`, then 5 on `real3_test`; dev 53 min, test stopped once and resumed): dev 48k, 12k A, C at 20k, 16k, 12k, 8k with `min_chains=2`; `size_power=0.5` at 16k, 12k, 8k; `size_power=1` at 12k; `min_chains=2` at 12k plain, with `size_power` 0.5 and 1. Test: 48k, C at 16k and 12k, C `size_power=0.5` at 16k and 12k. `scripts/cold.py cold_d --split dev|test [--size]`, `scripts/mix.py cold_d --split dev`. Streak 1 / streak 10 (sp = `size_power`, mc2 = `min_chains=2`):

| dev | cpu x | mlog | hit% | p99 |
| --- | --- | --- | --- | --- |
| 48k | 1.00 / 1.00 | -2.79 / -2.84 | 94.3 / 93.8 | 0.043 / 0.110 |
| 20k C | 0.30 / 0.28 | -2.68 / -2.79 | 88.6 / 90.8 | 0.064 / 0.126 |
| 16k C | 0.26 / 0.24 | -2.66 / -2.77 | 87.7 / 89.8 | 0.079 / 0.153 |
| 16k C sp 0.5 | 0.26 / 0.23 | -2.66 / -2.77 | 88.3 / 89.7 | 0.081 / 0.135 |
| 12k A | 0.23 / 0.21 | -2.61 / -2.74 | 85.1 / 87.6 | 0.080 / 0.140 |
| **12k C** | **0.22 / 0.19** | **-2.61 / -2.74** | 84.8 / 88.2 | 0.079 / 0.160 |
| **12k C sp 0.5** | **0.23 / 0.19** | **-2.63 / -2.76** | 86.7 / 89.7 | 0.099 / 0.186 |
| 12k C sp 1 | 0.24 / 0.20 | -2.63 / -2.76 | 85.9 / 89.8 | 0.117 / 0.170 |
| 12k C mc2 | 0.20 / 0.19 | -2.59 / -2.73 | 82.8 / 87.5 | 0.117 / 0.172 |
| 12k C sp 0.5 mc2 | 0.20 / 0.18 | -2.60 / -2.73 | 84.4 / 87.0 | 0.101 / 0.225 |
| 12k C sp 1 mc2 | 0.21 / 0.19 | -2.62 / -2.74 | 85.2 / 88.2 | 0.115 / 0.199 |
| 8k C sp 0.5 | 0.19 / 0.15 | -2.56 / -2.72 | 80.9 / 87.1 | 0.117 / 0.220 |
| 8k C mc2 | 0.15 / 0.13 | -2.53 / -2.69 | 79.0 / 85.0 | 0.114 / 0.232 |

| test | cpu x | mlog | hit% | p99 |
| --- | --- | --- | --- | --- |
| 48k | 1.00 / 1.00 | -2.80 / -2.85 | 92.8 / 93.6 | 0.073 / 0.109 |
| 16k C | 0.31 / 0.32 | -2.69 / -2.77 | 87.8 / 89.6 | 0.171 / 0.353 |
| 16k C sp 0.5 | 0.32 / 0.33 | -2.72 / -2.78 | 89.2 / 90.5 | 0.112 / 0.387 |
| 12k C | 0.26 / 0.26 | -2.67 / -2.76 | 86.1 / 89.1 | 0.193 / 0.265 |
| 12k C sp 0.5 | 0.28 / 0.28 | -2.69 / -2.78 | 87.1 / 90.4 | 0.194 / 0.527 |

- **`coarse_cache` is free**: 12k A to C takes x0.23 / 0.21 to x0.22 / 0.19 (5% and 10% less) at the same mlog. Less than the smoke's x0.94 at streak 1.
- **`size_power=0.5` is a small, steady gain**: +0.02 mlog at 12k for +0.01 of CPU at streak 1 and none at 10 on dev, +0.02 / +0.02 for +0.02 on test; at 16k level on dev, +0.03 / +0.01 on test. On test it takes 1–3 upgrades' worst from 0.367 to 0.194 (48k's); its worse p99 at streak 10 is `27a2bc40_2991b9` (4–10 upgrades), which 48k misses too (0.70). A power of 1 is no better.
- **`min_chains=2` is on the curve**: about 10% less CPU for 0.02 less mlog, as 12k to 8k-and-a-bit. Removed.
- Picked per size in sample the frontier is x0.20 / 0.15 at -2.62 / -2.75 and x0.25 / 0.17 at -2.66 / -2.77: about 0.02 over the best single setting, so little is left in piecing sizes together.
- Repeats: after removing `exchange_flat`, `exchange_rank` and `min_chains`, the tree equals `cold_d` on 444 of 444 runs for 12k C and 12k C sp 0.5 (148 `r3_0` scenarios, 3 seeds).
