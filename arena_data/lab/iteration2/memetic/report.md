# `memetic` investigation

**Verdict: take two pieces, not the algorithm. Its start is its weakness; its polish and its generations are its strength.**

- **Its polish, as a `solve` knob (`mem_polish`).** After `refine`, `solve`'s best takes memetic's polish once.
  - **Gain:** quick_dev +0.0014 (4 wins, 0 losses; hit 95.5 → 96.3), `large` +0.0021, 0 elsewhere. No payload is lost on any set.
  - **Cost:** x1.03–1.17 the CPU.
  - **What it reaches:** the order basins `chain_0017:2/3`, `b10_chain_0017:3` and `b10_single_3300`, on every seed.
  - **Why:** `solve`'s `order_polish` makes single insertions. Memetic's makes paired moves (one upgrade out of the free taps and another in, at once), and retries a failed move with the moved upgrades re-planned. Both parts are needed: without the re-plan it gains +0.0001, without the pairs +0.0004. Runs of 2–3 add nothing.
- **With the kick loop**, the two take the union of their order basins at the kick loop's CPU: quick_dev +0.0019 against +0.0016, `large` +0.0093 against +0.0077, real_test +0.0058 against +0.0056. On the regime sets it is cheaper than the loop alone (x1.56–1.85 against x2.21–3.00).
- **Its generations, from `solve`'s endpoint (`fromsolve`, re-plan off).**
  - **Gain:** quick_dev +0.0015, speed_test +0.0019 (hit 98.6), `large` +0.0061, real_test +0.0036, income +0.0003. No payload is lost.
  - **Cost:** x1.30–1.41 the CPU, `solve` included.
  - **What it reaches:** the joint-move payloads `b10_week_inc_0004:3`, `single_3397`, `chain_1051`, `single_3375` and `week_hone_0009:6`, which no polish and no kick loop reaches. Children's responses after recombination find them.
  - **Against L** (surrogate report, at x1.95–2.94 in batch `a`): level on quick_dev, speed_test, income and `large`, behind on real_test. That difference is `real_27a2bc40`, a noisy payload. The CPU comparison crosses batches; FS's own ratio is the same in `a` and `b` to the second decimal.
- **What carries no weight:**
  - responses in chunks of 4;
  - fresh shadow prices;
  - price seeds (mutated copies do as well or better);
  - runs of 2–3;
  - the re-plan of failed order moves inside the population. It is about half of the prototype's CPU and gains nothing there, though `mem_polish` in `solve` needs it.
- **Memetic from its own start is behind `solve` everywhere but on these payloads.** Its losses are advanced plans with normal ones, chest amounts off its grid, and plans only the annealer finds (`real_08927991`, `chain_0001`, `single_1104`, `single_1056`, `week_hone_0004`, `week_hone_0012`). From `solve`'s endpoint it loses none of them.

Scripts are in `scripts/` (`dump.py`/`an.py` for the stored runs, `rep.py` for repeat checks, `attr.py` for the trace). `an.py` needs `runs.pkl` from `dump.py` (not kept). All numbers are % of flow.

## 0. State of the work

| Round | Snapshot | Batch | Tests | Status |
|---|---|---|---|---|
| a | `mem_a` (14df7c21 + switches) | `iteration2/memetic/a` | every main piece ablated at effort 1; `fromsolve` with and without generations; kick loop and L in the same session | read, section 3 |
| b | `mem_b` | `iteration2/memetic/b` | `mem_polish` in `solve` and its order moves; with the kick loop; `fromsolve` ablated (`cross`, `pseed`, `oreplan`, `toggle`) and at effort 0.5 | read, section 4 |

- Plain `mem_a` and `mem_b` repeat `cut_a` on every set (3,128 runs). FS repeats across `mem_a` and `mem_b` (3,128 runs).
- The final code drops the null switches `chunk` and `freshprice`. It repeats `mem_a` and `mem_b` exactly (smoke, 78 runs), and `lab_algos_repeat_and_are_valid` passes.

Switches in `memetic.rs`:
- `lab_off`: `cross`, `gens`, `restart`, `improve`, `pseed`, `oreplan`, `toggle`, `runs`.
- `lab_on`: `fromsolve`.
- Removed as null: `chunk`, `freshprice` (section 3).

Knob in `solve`: `mem_polish` (false). After `refine` and before the loops, the best state takes `memetic::polish_state`, in rounds while it gains (at most 4). Each round is:
- responses in chunks of 4, one evaluation a chunk;
- a full advanced scan;
- the order moves: window insertions, in/out toggles and runs of 2–3, each retried with the moved upgrades re-planned;
- the chest halving and pair moves.

Trace: `mem_gain`, `mem_wins` and `mem_cost` at iter = phase × 16 + move (`scripts/attr.py`).

## 1. Where it stands (stored effort curve, `pr_a` batch `ra`)

Against `solve` 48k in the same batch (price report 11 has the whole curve):

| set | effort 1 | effort 2 | evals / CPU at 1 |
|---|---|---|---|
| real_test | -0.0232 (0 wins, 3 losses) | -0.0116 (0, 3) | x0.17 / x0.91 |
| quick_dev | -0.0045 (5, 24) | +0.0001 (10, 14) | x0.15 / x0.81 |
| speed_test | -0.0056 (7, 12) | -0.0005 (7, 5) | x0.13 / x0.84 |
| large | -0.0341 (1, 3) | -0.0081 (2, 3) | x0.30 / x0.82 |
| income | -0.0010 (1, 10) | +0.0001 (4, 0) | x0.28 / x1.12 |

- **It spends on responses, not evaluations:** x0.13–0.30 the evaluations of `solve` at x0.5–1.1 the CPU.
- **Its wins are the open payloads, and it reaches them at low effort.** Mean gap over 3 seeds:

| payload | `solve` 48k | 96k | kick loop | mem 0.25 | mem 0.5 | mem 1 | mem 2 |
|---|---|---|---|---|---|---|---|
| `b10_single_3300` | 0.162 | 0.162 | 0.071 | 0 | 0 | 0 | 0 |
| `b10_week_inc_0004:3` | 0.064 | 0.035 | 0.064 | 0.014 | 0 | 0 | 0 |
| `b10_chain_0017:3` | 0.090 | 0.039 | 0 | 0 | 0 | 0 | 0 |
| `single_3397_serca_n4` | 0.062 | 0.062 | 0.025 | 0.002 | 0.001 | 0.002 | 0.003 |
| `single_3375` | 0.047 | 0.033 | 0.047 | 0.038 | 0.023 | 0.005 | 0 |
| `single_0085` | 0.037 | 0.037 | 0.037 | 0 | 0 | 0 | 0 |
| `chain_1051:1` | 0.123 | 0.123 | 0.123 | 0.062 | 0 | 0 | 0 |
| `week_hone_0009_serca_n21:6` | 0.023 | 0.023 | 0 | 0.046 | 0 | 0 | 0 |
| `chain_0017:2` | 0.072 | 0.072 | 0.072 | 0.271 | 0.140 | 0.109 | 0 |
| `chain_0017:3` | 0.175 | 0.116 | 0 | 0.304 | 0.156 | 0.070 | 0 |
| `single_0044` | 0.079 | 0.066 | 0.078 | 0.176 | 0.114 | 0.045 | 0.018 |

- **Its losses are the families `price` and `surrogate` lose:**

| payload | `solve` 48k | mem 1 | mem 2 | kind |
|---|---|---|---|---|
| `real_08927991_4514f9` | 0.002 | 0.377 | 0.284 | advanced with normal plans |
| `chain_0001:0` | 0.010 | 0.165 | 0.013 | advanced and order |
| `single_0159` | 0.056 | 0.383 | 0.170 | advanced-heavy, `large` |
| `week_hone_0004:3` | 0 | 0.083 | 0.105 | the same loss as `price`, steps 1–3 |
| `single_1104_serca_n1` | 0 | 0.392 | 0.262 | chest amount off its grid |
| `single_1000` | 0.002 | 0.241 | 0.120 | |

- **The collapse at low effort on speed_test is one payload.**
  - At 0.25, `single_1000` is -64.1 on 2 of 3 seeds (gap 94 and 97), which is -0.298 of the set's -0.360. Without it the set is -0.063 at 0.25 and -0.012 at 0.5.
  - The budget binds there: spent/budget 1.02, 6 children, no restart (smoke, 3 seeds). Responses on that payload are charged so much work that the anchor polish cannot finish one pass. This is budget, not a mechanism (as `price` at 0.125).
- **Streak length 10, a lead.** On quick_dev at effort 1, memetic is -0.0012 on the 137 `b10_` payloads against -0.0066 on the other 212, and `solve`'s own gap is higher on `b10_` (0.0034 against 0.0026). Three of the open payloads above are `b10_` ones whose plain forms `solve` solves.

## 2. Reading the code

- **Budget:** `effort × (20000 + 3000 n)`, charged as evaluations plus `response.work`. The generations stop at 80% of it, and the final polish gets the rest, or one chunk if nothing is left.
- **Shadow prices are taken once, at the best seed**, which on a cold run is the given (often empty) plan. Every later price seed and restart mixes those with market prices. `freshprice` tests prices taken at the anchor.
- **The polish order is the reverse of `solve`'s:** responses (chunks of 4, one evaluation a chunk), then adv, order and chest. `solve` starts with the order.
- **The polish's order moves differ from `solve`'s `order_polish`:**
  - each failed window move is retried with the moved and displaced upgrades re-planned together;
  - it adds in/out toggles between the free-tapped and the cleaned-out upgrades, and runs of 2–3 to the front or the end, each re-planned on failure;
  - the windows are not limited to the free taps' reach.
- **The chest polish halves from count/2** (7, 3, 1 on 14 chests) and the mutation moves all, half or 1. Neither reaches 9+5 on `single_1104_serca_n1`, which `solve` gets from its random amounts.
- **No bug found** that changes results; the order is read before the evaluation cleans it.

## 3. Batch `a`: every piece at effort 1, and from `solve`'s endpoint

Everything ran in one session. Diff is against `mem_a` (`solve` 48k), with CPU in the same batch. M is memetic at effort 1; FS is M with `fromsolve`; FSg is FS with `gens` off. 3 seeds, real_test 10.

| variant | real_test | quick_dev | speed_test | large | income | CPU quick_dev / large |
|---|---|---|---|---|---|---|
| M | -0.0232 | -0.0045 (5 wins, 24 losses) | -0.0056 | -0.0341 | -0.0010 | x0.82 / 0.83 |
| `cross` off | -0.0330 | **-0.0087** (5, 42) | -0.0096 | -0.0213 | **-0.0047** | x0.80 / 0.82 |
| `gens` off | -0.0924 | -0.0333 (5, 75) | -0.0382 | -0.0786 | -0.0442 | x0.38 / 0.52 |
| `restart` off | -0.0277 | -0.0116 (5, 38) | -0.0094 | -0.0406 | -0.0038 | x0.46 / 0.58 |
| `improve` off | -0.0258 | -0.0122 (5, 48) | -0.0098 | -0.0645 | -0.0038 | x0.69 / 0.72 |
| `pseed` off | -0.0292 | -0.0054 (5, 22) | -0.0073 | -0.0144 | -0.0012 | x0.83 / 0.89 |
| `chunk` off | -0.0231 | -0.0053 (5, 23) | -0.0055 | -0.0263 | -0.0007 | x0.81 / 0.85 |
| `oreplan` off | -0.0200 | -0.0040 (5, 21) | -0.0043 | -0.0288 | -0.0006 | **x0.46 / 0.57** |
| `toggle` off | -0.0232 | **-0.0078** (4, 25) | -0.0056 | -0.0125 | -0.0011 | x0.75 / 0.74 |
| `runs` off | -0.0167 | -0.0042 (5, 22) | -0.0053 | -0.0131 | -0.0009 | x0.76 / 0.80 |
| `freshprice` | -0.0231 | -0.0047 (5, 23) | -0.0054 | -0.0206 | -0.0010 | x0.83 / 0.80 |
| FS | +0.0036, hit 97.1 | **+0.0018** (6, 0), 96.9 | **+0.0018**, 98.1 | +0.0035 | +0.0003 | x1.66 / 1.55 |
| FSg | 0 | +0.0015 (4, 0), 96.4 | +0.0001 | +0.0022 | 0 | x1.27 / 1.30 |
| kick loop | +0.0056, 97.6 | +0.0016 (11, 0), 97.8 | +0.0006, 96.7 | +0.0077 | +0.0001 | x1.91 / 2.57 |
| L | +0.0064, 97.9 | +0.0018 (8, 0), 97.3 | +0.0016, 98.3 | +0.0066 | +0.0002 | x2.05 / 2.94 |

- FS's CPU on real_test and speed_test is x1.64 and x1.78, against the kick loop's x1.53 and x1.37 and L's x2.22 and x1.95.
- On the regime sets, FS is level with or ahead of `solve`: advanced +0.0015 (hit 97.2), juice +0.0018 (97.2), free_taps 0. FS, FSg, the kick loop and L lose no payload on any set (worst -0.002 on income).
- `large` is 10 payloads at 3 seeds. Its rows among M's ablations follow `single_0159`, which ranges from 0.10 to 0.59 across variants, so they are noise. Most of real_test's spread is `real_27a2bc40`.

**Mean gaps on the payloads that move** (3 seeds):

| payload | `solve` | M | FS | FSg | kick | L | M's ablation that loses it |
|---|---|---|---|---|---|---|---|
| `b10_single_3300` | 0.162 | 0 | 0 | 0 | 0.071 | 0.039 | `toggle` 0.213 |
| `chain_0017:3` | 0.175 | 0.070 | 0 | 0 | 0 | 0 | |
| `chain_0017:2` | 0.072 | 0.109 | 0.026 | 0 | 0.072 | 0 | |
| `b10_chain_0017:3` | 0.090 | 0 | 0 | 0 | 0 | 0 | `oreplan` 0.051, `toggle` 0.061 |
| `b10_week_inc_0004:3` | 0.064 | 0 | 0 | 0.064 | 0.064 | 0.014 | `gens` 0.043, `cross` 0.014 |
| `single_3375` | 0.047 | 0.005 | 0.019 | 0.047 | 0.047 | 0.005 | `cross` 0.027, `restart` 0.033 |
| `single_3397_serca_n4` | 0.062 | 0.002 | 0.003 | 0.049 | 0.025 | 0.003 | `improve` 0.035, `gens` 0.034 |
| `chain_1051:0–2` (sum) | 0.252 | 0 | 0 | 0.252 | 0.189 | 0.066 | `restart` 0.191 |
| `single_0085` | 0.037 | 0 | 0.018 | 0.037 | 0.037 | 0 | |
| `week_hone_0009_serca_n21:6` | 0.023 | 0 | 0 | 0.023 | 0 | 0 | |
| `single_3116` | 0.020 | 0.057 | 0.018 | 0.018 | 0.018 | 0.002 | |
| `real_27a2bc40` (10 seeds) | 0.274 | 0.716 | 0.148 | 0.274 | 0.070 | 0.044 | |

**Its own losses are its start.** FS brings each of M's losses back to `solve`'s gap:

| payload | M | FS |
|---|---|---|
| `real_08927991_4514f9` | 0.377 | 0.002 |
| `chain_0001` steps 0–3 | 0.05–0.17 | 0.01–0.03 |
| `week_hone_0004` steps 0–3 | 0.06–0.10 | 0–0.003 |
| `week_hone_0012` steps 2–8 | up to 0.07 | 0 |
| `single_0100` | 0.152 | 0.001 |
| `single_1104_serca_n1` | 0.392 | 0 |
| `single_1056_serca_n3` | 0.606 | 0 |
| `single_1000` | 0.241 | 0.002 |
| juice `single_0034` | 0.062 | 0 |

These are what `solve`'s annealing finds: advanced strategies with normal plans, chest amounts off a grid, and plans no response sequence reaches from nothing. Given that start, memetic's search costs nothing anywhere.

**Two parts, two kinds of payload** (attribution from the trace, quick_dev):

- **The polish takes the order basins.** FSg reaches `b10_single_3300`, `chain_0017:2/3` and `b10_chain_0017:3`, all in its first polish of `solve`'s state.
  - On `b10_single_3300`, the gains are window moves with re-plan (0.075 a run), toggles (0.044), plain window moves (0.023) and re-planned runs and toggles.
  - On `chain_0017:3`, they are re-planned toggles (0.070), the response sweep (0.034) and re-planned window moves (0.022).
  - `solve`'s `order_polish` makes single insertions only. Memetic's polish also makes paired moves: an in/out toggle moves one upgrade out of the free taps and another in at once, and runs move 2–3 together. Without toggles, M loses `b10_single_3300` (0.213) and `b10_chain_0017:2/3`.
- **The generations take the joint-move payloads.** FSg, with no generations, leaves `b10_week_inc_0004:3`, `single_3375`, `single_3397`, `chain_1051`, `single_0085` and `week_hone_0009:6` exactly where `solve` left them. FS takes them all.
  - On `b10_week_inc_0004:3` every gain is in children: responses after recombination (0.046 a run, 3.7 new bests) and the recombined child itself (0.018). Neither polish finds any.
  - With crossover off, `b10_week_inc_0004:3` (0.014) and `single_3375` (0.027) slip.

**The pieces inside memetic:**

- **Recombination helps, about 2x on the mean**, less than iteration 1's 4x. Without it: quick_dev -0.0087 against -0.0045 (42 losses against 24), speed_test -0.0096 against -0.0056, income -0.0047 against -0.0010, real_test -0.033 against -0.023, at the same CPU.
- **The child's local search and the restarts help**, but leaving them out also saves CPU. `improve` off is -0.0122 at x0.69, and `restart` off -0.0116 at x0.46. `solve` 24k is -0.0027 at x0.64 (price report 11), so neither is a good trade.
- **Responses in chunks of 4, one evaluation per chunk, do nothing.** With each response checked alone, it is quick_dev -0.0053, speed_test -0.0055 and income -0.0007 at the same CPU, and the same on the open payloads.
- **Fresh shadow prices at each restart do nothing** on any set.
- The switches `chunk` and `freshprice` are removed from the code.
- **Price seeds are about worth nothing.** Mutated copies in their place: quick_dev -0.0054, income -0.0012, speed_test -0.0073, real_test -0.029. Every open payload is still reached.
- **The re-plan after a failed order move is worth nothing and costs about half the CPU.**
  - Without it: quick_dev -0.0040 at x0.46 against -0.0045 at x0.82; speed_test -0.0043 at x0.53; income -0.0006 at x0.46; real_test -0.0200 at x0.45.
  - It only loses `b10_chain_0017:2/3` (0.049, 0.051).
  - Its charged work is 7% of the total, but it is half the CPU: responses after an order move cost about twice what they are charged.
  - This is the third time this re-plan has failed, after `price`'s `replan` and the same move in `solve`'s `order_polish`.
- **Toggles carry `b10_single_3300` and `b10_chain_0017`.** Runs of 2–3 do nothing measurable.
- **Effort.** Spent/budget is 0.46 on quick_dev: the stall rules end runs, not the budget (as in `surrogate`).

## 4. Batch `b`: memetic's pieces swapped into `solve`, and from its endpoint

One session. Diff is against `mem_b` (`solve` 48k), with CPU in the same batch. FS is `algo=memetic,lab_effort=1,lab_on=fromsolve`; its cost includes `solve`'s.

| variant | real_test | quick_dev | speed_test | large | income | CPU quick_dev / speed_test / large |
|---|---|---|---|---|---|---|
| `mem_polish` | 0 | **+0.0014** (4, 0), 96.3 | 0 | +0.0021 | 0 | x1.08 / 1.12 / 1.11 |
| — without the re-plan (`oreplan`) | 0 | +0.0001 (1, 0) | 0 | +0.0002 | 0 | x1.01 / 1.01 / 1.02 |
| — without toggles | 0 | +0.0004 (2, 0) | 0 | +0.0003 | 0 | x1.06 / 1.09 / 1.07 |
| — without runs | 0 | +0.0014 (4, 0) | 0 | +0.0021 | 0 | x1.07 / 1.10 / 1.09 |
| kick loop | +0.0056 | +0.0016 (11, 0), 97.8 | +0.0006 | +0.0077, 70.0 | +0.0001 | x1.91 / 1.37 / 2.65 |
| kick loop + `mem_polish` | +0.0058 | **+0.0019** (10, 0), 97.6 | +0.0007 | **+0.0093**, 73.3 | +0.0001 | x1.93 / 1.48 / 2.61 |
| FS | +0.0036 | +0.0018 (6, 0), 96.9 | +0.0018, 98.1 | +0.0035 | +0.0003 | x1.66 / 1.78 / 1.59 |
| FS, `cross` off | +0.0027 | +0.0017 (5, 0) | +0.0018 | +0.0025 | 0.0000 | x1.64 / 1.76 / 1.54 |
| FS, `pseed` off | +0.0020 | +0.0019 (7, 0), 97.1 | +0.0015 | +0.0096 | +0.0004 | x1.68 / 1.79 / 1.72 |
| FS, `oreplan` off | +0.0036 | +0.0015 (7, 0) | **+0.0019, 98.6** | +0.0061 | +0.0003 | **x1.35 / 1.41 / 1.30** |
| FS, `toggle` off | +0.0036 | +0.0012 (4, 0) | +0.0018 | +0.0063 | +0.0002 | x1.60 / 1.73 / 1.51 |
| FS, effort 0.5 | +0.0022 | +0.0010 (3, 0) | +0.0014 | +0.0024 | +0.0001 | x1.41 / 1.43 / 1.33 |

- **Regime sets.** `mem_polish` moves nothing on them (x1.03–1.06).
  - FS: advanced and juice +0.0015 and +0.0018 (hit 97.2).
  - FS with `pseed` off: 100 on advanced, free_taps and juice.
  - FS with `cross` off: 94.4 on advanced and juice.
  - Kick loop + `mem_polish`: x1.85, 1.56 and 1.58, against the loop's x3.00, 2.21 and 2.55.
- **No variant loses a payload anywhere** (worst -0.005, on income, for FS with `cross` off).

**Mean gaps on the payloads that move** (3 seeds; every 0 below is 0 on all 3):

| payload | `solve` | `mem_polish` | its `oreplan` off | its `toggle` off | kick | kick + `mem_polish` | FS | FS `cross` off | FS `pseed` off | FS `oreplan` off | FS `toggle` off | FS 0.5 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `chain_0017:2` | 0.072 | 0 | 0.055 | 0.072 | 0.072 | 0 | 0.026 | 0.025 | 0.026 | 0 | 0.072 | 0.072 |
| `chain_0017:3` | 0.175 | 0 | 0.175 | 0.175 | 0 | 0 | 0 | 0 | 0 | 0.116 | 0.175 | 0.175 |
| `b10_chain_0017:3` | 0.090 | 0 | 0.090 | 0.051 | 0 | 0 | 0 | 0 | 0 | 0.039 | 0 | 0 |
| `b10_single_3300` | 0.162 | 0 | 0.162 | 0.071 | 0.071 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `b10_week_inc_0004:3` | 0.064 | 0.064 | 0.064 | 0.064 | 0.064 | 0.064 | 0 | 0 | 0 | 0 | 0 | 0 |
| `single_3397_serca_n4` | 0.062 | 0.057 | 0.058 | 0.057 | 0.025 | 0.014 | 0.003 | 0.002 | 0.004 | 0.003 | 0.003 | 0.001 |
| `single_3375` | 0.047 | 0.047 | 0.047 | 0.047 | 0.047 | 0.047 | 0.019 | 0.024 | 0.036 | 0 | 0.019 | 0.036 |
| `chain_1051:0–2` (sum) | 0.252 | 0.252 | 0.252 | 0.252 | 0.189 | 0.189 | 0 | 0 | 0.062 | 0 | 0 | 0.066 |
| `week_hone_0009_serca_n21:6` | 0.023 | 0.023 | 0.023 | 0.023 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `single_0085` | 0.037 | 0.037 | 0.037 | 0.037 | 0.037 | 0.037 | 0.018 | 0.027 | 0 | 0.018 | 0.018 | 0.037 |
| `single_0044` | 0.079 | 0.063 | 0.079 | 0.079 | 0.078 | 0.059 | 0.063 | 0.060 | 0.031 | 0.059 | 0.023 | 0.063 |
| `single_0159` | 0.056 | 0.056 | 0.056 | 0.056 | 0.014 | 0.008 | 0.054 | 0.056 | 0.032 | 0.034 | 0.054 | 0.055 |
| `real_27a2bc40` (10 seeds) | 0.274 | 0.274 | 0.274 | 0.274 | 0.070 | 0.065 | 0.148 | 0.181 | 0.214 | 0.148 | 0.148 | 0.201 |

**The polish in `solve`:**
- `mem_polish` takes exactly the four order basins that FSg took in batch `a`, on every seed.
- **Both the paired moves and the re-plan carry it.**
  - Without the re-plan none of the four is reached (+0.0001).
  - Without toggles only `chain_0017:2/3` stay lost, and `b10_*` are halved.
  - Runs change nothing, to the fourth place.
- In the prototype the re-plan was worth nothing (section 3). From `solve`'s endpoint it is the half of the polish that matters, because a single plain move there is already a local optimum.
- This qualifies `Optimizer tried.md`'s "trying a lost free tap order again with the plans of the upgrades it moves". For single insertions it is null (`price`, and in `solve`'s `order_polish`). With paired in/out moves it is what reaches these basins.
- **It complements the kick loop.** The loop keeps `chain_0017:2` and `b10_single_3300` at 0.072 and 0.071; the polish takes both. Together they lose nothing either one has, and add `large`'s `single_0044` and `single_0159`.

**The generations from `solve`'s endpoint:**
- **They are the only memetic piece that reaches the joint-move payloads:** `b10_week_inc_0004:3`, `single_3397`, `chain_1051`, `single_3375`, `week_hone_0009:6`, part of `single_0085`. They are also the only `solve`-side search beside L that does.
- **The re-plan is not needed for them,** and leaving it out saves a fifth of the CPU (x1.35 against x1.66 on quick_dev). It costs `chain_0017:3` (0.116) and `b10_chain_0017:3` (0.039), which `mem_polish` takes. So a production version would run the polish once and the generations without the re-plan.
- **Price seeds are not needed.** Mutated copies are level on quick_dev and income, slightly behind on real_test (+0.0020 against +0.0036), ahead on `large` (+0.0096) and the regime sets (100).
- **Recombination helps a little:** income 0 against +0.0003, real_test +0.0027 against +0.0036, the regime sets, `single_3375` and `single_0085`.
- **Toggles in the generations matter only for `chain_0017:2/3`.**
- **Effort 0.5 is a worse trade than effort 1 with the re-plan off:** at about the same CPU it loses `chain_0017`, `single_3375` and `chain_1051:2`.

## 5. Pieces

| Piece | Verdict | Where | Evidence | Cost | Why |
|---|---|---|---|---|---|
| `mem_polish` in `solve` | **Helps, candidate** | `chain_0017:2/3`, `b10_chain_0017:3`, `b10_single_3300` | quick_dev +0.0014 (4, 0), `large` +0.0021; no payload lost | x1.03–1.17 | paired in/out order moves with the moved plans re-planned; `order_polish`'s single insertions cannot leave these |
| Re-plan of failed order moves (`oreplan`) | **Needed in `mem_polish`; nothing in the population** | as above | `mem_polish` without it +0.0001; memetic without it equal at x0.56 the CPU | half the prototype's CPU | a lone move from `solve`'s endpoint is already optimal; its pair with plans is not |
| In/out toggles | **Needed** | `b10_single_3300`, `b10_chain_0017`, `chain_0017:2/3` | `mem_polish` without them +0.0004; memetic without them loses `b10_single_3300` (0.213) | | a joint move of two upgrades' places |
| Runs of 2–3 | **Nothing** | | identical without them in `mem_polish`; memetic level | | |
| Kick loop + `mem_polish` | **Helps, best order-basin mix** | union of both | quick_dev +0.0019, `large` +0.0093, real_test +0.0058 | the loop's own (x1.48–2.61); less on the regime sets | they reach different basins |
| Generations from `solve`'s endpoint (`fromsolve`, `oreplan` off) | **Helps, candidate for a replacement** | joint-move payloads | quick_dev +0.0015, speed_test +0.0019 (98.6), `large` +0.0061, income +0.0003; level with L on those sets | x1.30–1.41, `solve` included | children's responses after recombination move several plans at once |
| Recombination | **Small help** | income, regime sets, `single_3375` | from `solve`: income +0.0003 against 0, real_test +0.0036 against +0.0027; own start: quick_dev -0.0045 against -0.0087 | none | |
| Child local search, restarts | **Help, not a trade** | | own start: -0.0122 and -0.0116 without them, at x0.69 and x0.46 | | |
| Price seeds | **Nothing** | | mutated copies level or better from `solve` (`large` +0.0096, regime 100) | | |
| Responses in chunks of 4 (`chunk`) | **Nothing; removed** | | level on every set | | |
| Fresh shadow prices at restarts (`freshprice`) | **Nothing; removed** | | level on every set | | |
| Its own start | **Its weakness** | `real_08927991`, `chain_0001`, `single_1104`, `single_1056`, `week_hone_0004`, `week_hone_0012`, `single_0100`, `single_1000` | all fixed by `fromsolve` | | advanced with normal plans, chest amounts off its grid, plans only annealing finds |
| Effort below 1 | **Budget-starved** | speed_test `single_1000` | -0.298 of -0.360 at 0.25 | | spent/budget 1.02, 6 children |

## Not yet answered

- **`mem_polish` and the kick loop with the surrogate loop L,** or the generations from `solve`'s endpoint with L. Each pair reaches different payloads. Deferred to the final report's design stage.
- **A production form of the generations:** a `solve` phase in place of `fromsolve`, with mutated seeds and no re-plan. The prototype's numbers are the evidence; it was not built here.
