# `price` investigation

**Verdict: take pieces, not the algorithm.** Its namesake, the shadow price trajectories, carries no weight, and its own start is its weakness. Its kick loop is its strength: that loop now exists in `solve`, off by default (`kick_stall=32`), and loses no payload (`final.patch`). What is settled from here is in `AI notes/Calculator core/Optimizer lab results.md`.

Batches are in `runs/`, and runs are in `arena_data/lab/runs.sqlite` under the snapshots named below.

## 1. The best response is inexact where the lattice does not answer (`response_points`)

`Response` takes a material's worth exactly where the lattice answers. Elsewhere it samples 16 evenly spaced amounts (`Tuning::response_points`) and joins them by straight lines (`StateBundle::worth` in `response.rs`). On some payloads that line misses the true worth by enough to pick the wrong plan.

`pnpm arena ascend` (responses in turn; "cold a" from nothing, "any" from the best known), in % of flow:

| payload | n | 16 points | 32 | 64 | 128 | accurate evaluator, 16 points |
|---|---|---|---|---|---|---|
| small `single_0049_t4_n1` | 1 | -0.424 | 0 | 0 | 0 | -0.424 |
| small `single_0039_t4_n1` | 1 | -0.032 | -0.009 | -0.009 | 0 | -0.032 |
| small `single_0002_t4_n3` | 3 | -0.085 | 0 | +0.00025 | +0.00025 | -0.085 |
| speed_test `single_1235_t4_n1` | 1 | -0.962 | | | 0 | |

- **It is the points, not the evaluator.** `max_brute_size`/`max_lattice_work` at their maximum change nothing, and nor does `response_joint`.
- **With one upgrade there is nothing to coordinate**, so an exact response is the optimum. These losses were errors in the response, not separate basins.
- **Cost:** none measurable. The ascend times and the solves' CPU are unchanged from 16 to 128 points.

In whole solves (scratch database, `small` and `pilot`, 3 seeds, cold):

| variant | small diff | small hit % | pilot diff | x CPU |
|---|---|---|---|---|
| `solve` 48k | 0 | 95.0 | 0 | 1.00 |
| `solve` 48k, `response_points=64` | **+0.0070** | **100** | 0 | 0.79 (small), 0.99 (pilot) |
| `price` effort 1 | -0.0280 | 80.0 | +0.0023 | 0.77 |
| `price` effort 1, 64 points | **+0.0065** | **100** | +0.0023 | 0.74 |

- **`price`'s whole loss on `small` was this.** At 64 points it goes from -0.028 to +0.0065 and hits every payload.
- **Production `solve` gains too.** On small `single_0025_t4_n2` it finds +0.139% that every batch 1 run missed, the 384k ones included (the gap the reference runs left open).
- `pilot` does not move at all: there the lattice answers, so the points are never used.
- **CPU on `large`** (10 heavy payloads, 2 seeds): `solve` 48k at 128 and at 256 points gives +0.0002 at x0.99 CPU.

**Production fix:** the `Tuning` default is now 128 points. The wasm package is rebuilt, so the site uses it too. Iteration 1 is re-run with it (snapshot `lab_r1`); `lab_r1:response_points=16` keeps the old production for comparison.

## 2. The multi-upgrade losses on the held-out set are real basins

Cold ascend at any accuracy still ends at -0.40 (`real_08927991_4514f9`, n=12), -0.41 (`real_27a2bc40_2991b9`), -0.27 (`real_8f8ac564_8bb1fc`) and -0.14 (`real_07f47ad0_30bfd5`). These are fixed points of the exact responses, reached from nothing and not from the best known. The batch 1 review (at 16 points, now in `ignored/junkyard/lab_iteration1_16pt`) put these down to back streaks, and that diagnosis stands for these only.

On `speed_test` (`_n1x` payloads), 128 points leave every multi-upgrade loss as it was (`single_3479_t4_n11` -1.15, `single_1208_t4_n14` -0.65).

## 3. `price` on the re-run, at matched CPU

From the re-run (snapshot `lab_r1`, 128 points), cold unless marked; diff is the mean over seeds less `solve` 48k's, in % of flow; hit is within 0.01% of the best known.

| set | price 1 | price 2 | `solve` at similar CPU |
|---|---|---|---|
| real_test | -0.0115, hit 92.4, x0.80 | -0.0059, hit 92.4, x1.44 | 24k: -0.0049, hit 93.7, x0.64; 96k: +0.0041, x1.73 |
| real_income_test (chained) | -0.0008, x0.99 | -0.0003, x1.74 | 24k: -0.0002, x0.75 |
| quick_dev | -0.0010, x0.92 | +0.0007, x1.64 | 96k: +0.0013, x1.64 |
| quick_dev chained | -0.0001, x1.10 | **+0.0019**, x1.95 | 96k: +0.0009, x1.58 |
| speed_test | -0.0035, x0.66 | -0.0029, x1.16 | 24k: -0.0006, x0.61 |
| large | -0.0117, hit 47, x0.98 | -0.0034, x1.75 | 96k: +0.0052, x1.83 |
| advanced | +0.0009, x1.03 | +0.0027, x1.91 | **every `solve` ≤ 0**, 96k hit 94.4 |
| free_taps | -0.0016, x0.53 | -0.0009, x1.02 | 6k/12k lose -0.04 (one order basin); 24k -0.0005 |

- **`price` is behind `solve` at matched CPU nearly everywhere**, by less than it seemed in batch 1. It is ahead on `advanced`, and on `quick_dev` at effort 2.
- **Its losses and wins are a few payloads, not a spread.** At effort 2 against `solve` 48k: `real_test` 0 wins and 2 losses, `quick_dev` 16 and 10, `speed_test` 3 and 8.
- **Effort is not CPU across sets.** Effort 1 costs x0.53 (`free_taps`) to x1.10 (`quick_dev` chained) of `solve` 48k.
- At effort 0.125 it can run out before it gets anywhere: `speed_test` `chain_1032_t4_n8:1`, one seed, -14.9% after 1164 evaluations. No mechanism, only budget.
- The old production (`response_points=16`) is now measured on every set: -0.0068 on `small`, about -0.0005 on `quick_dev` and `speed_test`, nothing on `real_test` (the lattice answers there).

## 4. What separates `price` from `solve`, payload by payload

Each case below compares end states part by part (free tap order, chests, plans) across every run.

**Free tap order basins: `price` wins.** On every payload where `price` beats `solve` (`chain_0017_t4_n14:2/3`, `single_3300_t4_n23`, `single_3397_serca_n4`, `chain_0001_t4_n18:1`), `solve`'s worse runs have a different free tap order, and `solve` at 384k sits in them too (-0.09 to -0.39%). `price`'s order move differs from `solve`'s `order_polish` in one way. A move that loses less than 0.2% is tried again with the moved upgrade's plan replaced by its best response (`order_moves` in `price.rs`). `order_polish` keeps every plan as it is.

**Chest choices: `price` loses on lone serca upgrades.** The response is exact there: `ascend` from nothing reaches the best known at 128 and at 1024 points. The losses are in the chests:

- `single_1104_serca_n1` (-0.39%, every effort): the same plan, but a stack opened as 14 Special Leap in place of 9 Special Leap and 5 Leaps.
- `single_1166_serca_n1` (-0.05%): the plan and three stacks differ together.

`price`'s chest move tries all of an option, then a third of that (14, 4, 1, so never 5), and keeps the first gain in one pass. `solve`'s `chest_polish` halves its step and repeats while anything moves, and the annealer moves random amounts.

**Advanced and normal plans together: `price` loses on `real_test`.** On `real_08927991_4514f9` (n=12, -0.07 to -0.2% at effort 1–2) the free tap order and chests are the same. The best puts nearly every advanced upgrade on strategy 5, and with that gives material 6 to about ten more taps of each of the four normal upgrades. `price` spreads the advanced upgrades over strategies 0–8. Responses only plan normal upgrades, and `price` moves advanced ones a slot at a time, so neither reaches it.

## 5. Ablations: which piece of `price` does the work

Snapshot `i2a`, cold, 3 seeds. `Tuning::lab_off` leaves pieces out (`lab_off=order`, joined by `+`). Plain `i2a` price repeats the re-run's `price` 1 exactly, so each row below is against that. The machine was in use during this batch, so the cost column is evaluations, not CPU.

| left out | quick_dev | speed_test | real_test | evaluations |
|---|---|---|---|---|
| `order` (free tap order moves in the polish) | **-0.143**, 97 worse | **-0.473**, 119 worse | -0.037, 13 worse | x0.33–0.66 |
| `kicks` (the whole kick phase) | -0.011, 42 worse | -0.019 | -0.067 | x0.21–0.29 |
| `prices` (the price trajectories, `price`'s namesake) | +0.0002 (10 better, 7 worse) | -0.0027 | -0.017 (one payload: -0.67) | x0.91–1.00 |
| `replan` (moved upgrade's response in an order move) | +0.0000 | -0.0001 | +0.0034 | x1.03–1.09 |
| `ratio` (start order by leapstone efficiency) | +0.0001 | -0.0000 | +0.0049 | x1.00 |

- **The order moves are most of `price`.** Without them it is behind by -0.14 to -0.47%.
- **The kicks are next**, but leaving them out also leaves 70–80% of the budget unspent, so this row is not at matched cost.
- **The shadow price trajectories are worth about nothing on average.** They are its namesake, yet they help on a few payloads and hurt on as many.
- **The replan and the ratio start are worth nothing.**

**From the `solve` side, the replan does nothing either.** The same move added to `solve`'s `order_polish` (up to 4 or 16 a polish) moved no set by more than 0.0001, at the same CPU. `Optimizer tried.md` already had this ("trying a lost free tap order again with the plans of the upgrades it moves"); it now carries the new number.

**`price`'s order basin wins do not hang on any one piece.** With `replan`, `ratio` or `prices` left out it still reaches them on some seeds (`chain_0017_t4_n14:3`, `single_3300_t4_n23`, `b10_single_3300_t4_n23`), and they come more often at effort 2 than at 1. Only `order` loses them all, by 0.4 to 6.5%. What remains is the structure: kick, then polish with order moves and full response passes in turn, many times over. That is the iterated local search `Optimizer tried.md` rules out *in place of* chains (83.3% hit against 91.7%). These payloads say it finds order basins that chains at 384k do not.

**Chests: random amounts find what halving steps do not.** `solve` with `chest_move_rate=0` (the end polish's halving steps only) loses `single_1104_serca_n1` by 0.39%, as `price` does. Its random-amount chest moves in the annealer are what find it. `single_1166_serca_n1` needs neither: `solve` finds it with both off (-0.002), from the plan side.

## 6. Where this leaves `price`

**Verdict: take pieces, not the algorithm.** Its namesake, the price trajectories, carries no weight. What it does well is find free tap order basins through repeated kick-and-polish, and that is the one thing `solve` at any budget misses. What it does badly is all one kind: two parts of the state that have to move together, where `price` moves one at a time:

- advanced strategies with normal plans (`real_08927991`);
- chest amounts that a fixed step grid cannot hit (`single_1104_serca_n1`);
- plans with chests (`single_1166_serca_n1`).

Each of the next experiments isolates one of these (not run yet):

1. **Order basins in `solve`.** A kick-and-polish phase after the chains, restricted to order-window kicks, at a fixed share of the budget: is it the kick-polish loop or the order kick itself? Against `price` with `lab_off` of every other kick type.
2. **Joint advanced and normal move.** Every advanced upgrade of a piece type to one strategy, then responses for the normal upgrades that share its material; in `price` first, where `real_08927991` is lost.
3. **Random chest amounts in `price`** in place of the 1, 1/3, 1/9 grid; this should take `single_1104_serca_n1`.

## 7. The three experiments (`i2b`, queue `e_*`)

Every set cold (`real_test` 10 seeds, the rest 3); `real_income_test` chained, 3 seeds. "diff" is the mean per payload against the base, in % of flow; "gap" is to the best known. With nothing set, `i2b` price repeats `i2a` price exactly.

**1. Order kicks in `solve` (`order_kicks`): null.** A window of the best state's order shuffled and polished, kept where better, after the chains.

| set | `=16` diff | `=64` diff | evals `=16` / `=64` |
|---|---|---|---|
| quick_dev | +0.0003 (2 wins, 0 losses) | +0.0004 (2 / 0) | x1.09 / x1.37 |
| speed_test | +0.0002 (2 / 0) | +0.0002 (2 / 0) | x1.06 / x1.23 |
| large | 0 | +0.0019 (1 / 0) | x1.34 / x2.37 |
| real_test, advanced, free_taps, juice, small | 0 | 0 | up to x1.6 |
| real_income_test (gap) | 0.0051, same | 0.0051, same | CPU x1.08 / x1.32 |

Across every payload, 64 kicks win 5 times and never lose. The biggest win is `b10_single_3300` (gap 0.25 to 0.15). On `chain_0017` they still leave a gap of 0.02–0.15.

So `solve`'s end state is a local optimum against a window shuffle plus its own polish. The kick alone does not explain `price`'s order basins.

**Price with only the window kick** (`lab_off=kchest+kadv+kexchange+kpiece+kperturb`): it is as weak as `price` with no kicks at all.

| set | diff |
|---|---|
| real_test | -0.065 |
| speed_test | -0.018 (the same -2.9 on `single_1208`) |
| quick_dev | -0.002 |
| large | -0.012 |
| income gap | 0.089 vs 0.021 |

But it is the payloads with order basins that it wins:

| payload | gap, window only | gap, all kicks |
|---|---|---|
| `chain_0017` steps | 0.03–0.06 | 0.04–0.18 |
| `single_3300` | 0 | 0.07 |

The mirror holds: `lab_off=kwindow` loses on exactly those (`chain_0017`, `single_3300`, by 0.13–0.19) and is neutral elsewhere.

So the window kick is the order-basin tool, and the other kick types carry the rest. The difference from `solve` has to lie in what runs around the kick: either `price`'s state when it kicks, or the many kicks until a stall (30 + 40 × effort without a gain) against a fixed 16 or 64. It is not the polish's insertion moves, since `solve`'s `order_polish` has them too.

**2. Joint advanced kick (`lab_on=advall`): no.** It changes nothing on `real_08927991` over 10 seeds; the smoke run's loss was noise. Its largest loss is `real_27a2bc40` (-0.13), a payload where every random change to `price` swings by about ±0.1–0.2 over 10 seeds.

| set | diff |
|---|---|
| large | +0.004 |
| advanced | +0.002 |
| real_test | -0.004 |
| elsewhere | about 0 |

The gap `price` leaves on advanced plans is not one of moving several advanced upgrades together.

**3. Random chest amounts (`lab_on=chestrand`): it fixes what it was meant to, and loses the fine steps.**

- **Fixed:** `single_1104_serca_n1` (gap 0.262 to 0), `single_1166_serca_n1` (0.05 to 0) and `single_3397_serca_n4`. These now beat `solve` too.
- **Lost:** `chain_1051` (-0.06 to -0.13 on three steps), `single_1000`, `single_1208` and `real_27a2bc40`.
  - Four uniform draws in `1..have` seldom land on the small amounts that the 1/3, 1/9 grid reaches.
  - So random amounts should come in addition to the grid, not in its place. This is not run yet.
- **Net:** about 0 everywhere (real_test -0.005, quick_dev +0.0002); income gap 0.0185 vs 0.0209.

**What this leaves:**

1. Order basins are found by `price`'s kick loop around a window shuffle, not by the shuffle bolted onto `solve`'s end. Next to try is a stall-bounded loop in `solve` (kick, polish, keep, until N without a gain) that draws from more than one kick type.
2. Chest amounts want both the grid and random draws.
3. The joint advanced move is ruled out.

## 8. The follow-ups (`i2c`, queue `f_*`)

These use the same sets and seeds as section 7. From here the numbers come from `cmp.py` and `pay.py`.

**Kick types in `price`, one off at a time, against `price`:**

| kick off | real_test | quick_dev | speed_test | large |
|---|---|---|---|---|
| kchest | -0.003 | -0.000 | -0.002 | -0.007 |
| kadv | -0.004 | +0.000 | -0.001 | +0.001 |
| kexchange | -0.002 | +0.000 | -0.000 | -0.002 |
| kpiece | -0.002 | +0.001 | -0.001 | -0.004 |
| kperturb | -0.001 | +0.001 | -0.002 | -0.002 |

- No single kick type carries the loop. Each costs at most a few thousandths, while all of them off cost -0.065 on real_test (section 7).
- The chest kick matters most on `large` and `speed_test`.
- So the loop needs kicks of several kinds, and which kind hardly matters.

**`chestboth` (the 1/3 grid plus 4 random amounts) is the right chest move, and the losses of section 7 were noise.** It fixes the lone-chest payloads exactly as `chestrand` did:

| payload | before | `chestboth` |
|---|---|---|
| `single_1104_serca_n1` | 0.262 | 0 |
| `single_1166_serca_n1` | 0.05 | 0 |

But its losses were identical to the last digit with `chestrand`'s, so the fine steps cannot have caused them. Seed by seed:

- `chain_1051` is bimodal. Every `price` variant ends each seed at a gap of either 0 or 0.190, and the two random-amount variants shift the random stream the same way.
- `single_1208` and `single_1000` swing the same way, between 0.05 and 0.5.

**A stall-bounded kick loop in `solve` works** (`kick_stall=32`, all four kick types: window, chest, adv, piece). Each kick is polished and kept where better, until 32 in a row gain nothing.

| set | diff vs `i2b` | wins / losses | hit % | cpu |
|---|---|---|---|---|
| real_test | +0.0056 | 1 / 0 | 96.8 → 97.9 | x1.54 |
| large | +0.0109 | 2 / 0 | 56.7 → 66.7 | x2.1 |
| quick_dev | +0.0009 | 10 / 0 | 94.7 → 96.3 | x1.72 |
| speed_test | +0.0005 | 3 / 0 | | |
| income (chained) | +0.0000 | | | x1.37 |

- It never loses on any of the 632 payloads.
- The real_test number is almost all from `real_27a2bc40` (+0.205), a noisy payload.
- Against `solve` at 96k it is ahead on `large` (+0.006) and real_test (+0.0016), and level elsewhere.
- In the smoke test, 16 kicks gained nothing on the `chain_0017` payloads; the order basins stay out of reach.

**`price` from `solve`'s endpoint (`lab_on=fromsolve`)** starts `price` from `solve`'s finished plan, with `solve`'s cost outside its budget.

| set | diff vs `solve` | hit % |
|---|---|---|
| quick_dev | +0.0024 | 96.8 |
| real_test | +0.0034 (+0.0051 with `prices` off) | |
| large | +0.008 | |
| income | +0.0003 | |

- CPU is about x1.75.
- Against the `solve` kick loop it ties overall. It wins the order basins:

  | payload | gap, kick loop | gap, `fromsolve` |
  |---|---|---|
  | `chain_0017` | 0.07–0.15 | 0 |
  | `b10_single_3300` | 0.17 | 0 |
  | `single_3397_serca_n4` | 0.06 | 0 |

  It loses a little on `single_0159`/`0161` and `chain_0001`.
- With the price trajectories off it does as well or better; again they carry no weight.
- On `single_1208` and `single_1000` it brings `price`'s gaps from 0.2–0.5 to 0.03 or less.

So `price`'s own start is its weakness, and its kick loop is its strength.

## 9. What carries the kick loop in `solve` (`i2c`, queue `g_*`)

Same sets and seeds. Each kick type alone at `kick_stall=32`, the stall length, a CPU-matched variant, and `price` from `solve`'s endpoint without the window kick.

**Each kick type alone, diff vs `i2b`:**

| kick types | real_test | large | quick_dev | speed_test | cpu (real_test / large) |
|---|---|---|---|---|---|
| all four | +0.0056 | +0.0109 | +0.0009 | +0.0005 | x1.40 / x2.36 |
| adv | +0.0027 | +0.0113 | +0.0007 | +0.0000 | x0.97 / x2.95 |
| chest | +0.0063 | +0.0000 | +0.0000 | +0.0003 | x1.88 / x0.92 |
| window | +0.0000 | +0.0018 | +0.0003 | +0.0002 | x1.05 / x1.73 |
| piece | +0.0000 | +0.0000 | +0.0000 | +0.0000 | x1.17 / x1.22 |

- Each set has its own kick: adv carries `large`, chest carries real_test (`real_27a2bc40`) and `speed_test`, and adv and window share quick_dev. All four together are about as good as the best one alone on every set.
- The piece kick does nothing in `solve`; its response is what `response_polish` already finds.
- Income is unchanged by any of them.

**The stall length:**

| | real_test | large | quick_dev | cpu (real_test / large) |
|---|---|---|---|---|
| `kick_stall=8` | +0.0025 | +0.0046 | +0.0002 | x1.02 / x1.20 |
| `kick_stall=32` | +0.0056 | +0.0109 | +0.0009 | x1.40 / x2.36 |
| `kick_stall=96` | +0.0062 | +0.0126 | +0.0010 | x2.44 / x4.22 |

8 gets about half, and 96 adds little to 32 at about double the CPU.

**Kicks against annealing at matched CPU.**
- `max_iters=24000,kick_stall=32` (half the annealing) costs about what `solve` 48k costs on real_test (x1.03) and loses to it in places: real_test +0.0027 but 1 loss, quick_dev -0.0007 with 9 losses, `advanced`, `free_taps` and `juice` all slightly negative.
- Against `solve` at 96k, `kick_stall=32` at 48k is ahead on `large` (+0.006, 3 wins, 0 losses) and real_test (+0.0016) at x0.9–1.3 the CPU, and level on quick_dev and `speed_test`.
- So the kicks add to the annealing; they do not replace it.

**The window kick is what takes `price` from `solve`'s endpoint into the order basins.** With `kwindow` off, `fromsolve` (prices off) loses on quick_dev (+0.0024 → +0.0016). Payload by payload:

| payload | gap with the window kick | gap without |
|---|---|---|
| `chain_0017` step 2 | 0.055 | 0.109 |
| `chain_0017` step 3 | 0 | 0.058 |
| `single_3300` | 0 | 0.071 |
| `b10_single_3300` | 0 | 0.122 |

**Why the same kick fails in `solve`.** `solve`'s polish runs `order_polish` first. It can move the shuffled upgrades straight back before their plans adapt to the new order. `price`'s polish re-plans every normal upgrade first and moves the order after. A window kick that re-plans the shuffled upgrades (`respond_one`, kept where better) before the polish, `kick_types=windowr`, was smoke-tested on `chain_0017` and `single_3300` (quick_dev, 3 seeds):

| | wins vs `i2b` | `chain_0017` step 3 gap | `b10_single_3300` gap |
|---|---|---|---|
| `window` | 2 | 0.145 | 0.174 |
| `windowr` | 7 | 0 | 0.110 |
| `windowa` (re-plans all normal upgrades) | 6 | 0 | 0.110 |

So a kick only reaches a basin if the parts it moved get to re-plan for their new place before a greedy repair runs. The full sets are in queue `h_*`.

## 10. Re-plan before the polish (`i2d`–`i2g`, queues `h_*`, `i_*`)

**`windowr` on the full sets (queue `h_*`), diff vs `i2b`:**

| | quick_dev | speed_test | large | real_test | income |
|---|---|---|---|---|---|
| `window` alone | +0.0003 (2 wins) | +0.0002 | +0.0018 | 0 | 0 |
| `windowr` alone | +0.0014 (7 wins) | +0.0004 | +0.0019 | 0 | 0 |
| all four, `window` | +0.0009 (10 wins) | +0.0005 | +0.0109 | +0.0056 | 0 |
| all four, `windowr` | +0.0015 (13 wins) | +0.0007 | +0.0115 | +0.0056 | 0 |

- None of them loses anywhere, and the CPU is the same as with `window` or lower.
- Against the old loop (`i2c:kick_stall=32`), the `windowr` loop wins 5 payloads and loses none. All 5 are order basins (`chain_0017` steps 2–3, `b10_single_3300`, `single_3397_serca_n4`).

**Two more kicks, smoke-tested on the order-basin payloads (quick_dev and speed_test, 3 seeds; gaps):**

| | `chain_0017` step 2 | step 3 | `b10_single_3300` | `single_3300` | `single_3397_serca_n4` | `single_3375` | `week_inc_0004` step 3 |
|---|---|---|---|---|---|---|---|
| `i2b` (`solve`) | 0.109 | 0.145 | 0.250 | 0.071 | 0.062 | 0.038 | 0.058 |
| `windowr` | 0.055 | 0 | 0.110 | 0.071 | 0.024 | 0.038 | 0.058 |
| `windowq` (`windowr` starting within the free taps' reach) | 0 | 0 | 0 | 0.071 | 0.024 | 0.038 | 0.058 |
| `insertr` (an upgrade from anywhere moved within reach; it and the next two re-planned) | 0.109 | 0 | 0.039 | 0.071 | 0.039 | 0.038 | 0.058 |
| `price` from `solve` | 0.055 | 0 | 0 | 0 | 0 | 0 | 0 |

- On long orders most windows fall past the reach, where a shuffle changes nothing. `windowq` puts every kick where it counts, so it does the most per kick.
- Mixed with the chest, adv and piece kicks, `windowq` gets a quarter of the kicks, and `b10_single_3300` ends at 0.039 rather than 0.
- `replan` (every normal upgrade re-planned after any kick) helps only `single_3300` (0.037), and with the chest, adv and piece kicks alone it does nothing.

**What `price` still finds that no `solve` kick does.** From `solve`'s endpoint, with prices off:
- With its whole kick loop off, `price` fixes none of the four last payloads, and with its order moves off it loses `single_3397` and `single_3375`.
- With any one kick type off, it still fixes `single_3375`, `week_inc_0004` and `single_3397`, so any of its kicks will do there. `single_3300` needs its window kick.

In the end states, the gains are joint moves of several upgrades:
- **`single_3375`:** upgrade 18 moves from past the reach to the front of the order. The juice tails of upgrades 10–17 all shorten and 18–19 swap plans.
- **`week_inc_0004` step 3:** the order is the same; upgrades 6, 7 and 8 shift their plans together.

`price`'s polish takes each upgrade's response in turn, for several rounds, with its order and chest moves in between. A kick followed by that polish walks to these states. `solve`'s polish after the same kicks does not, and re-planning the moved upgrades once (`windowr`, `replan`) is not enough. This is left open; the next prototypes' reviews may throw light on it.

**The lesson:** a kick reaches a new basin only if the parts it moved re-plan for their new place before a greedy repair runs, and only if it moves something that matters (within the reach).

**The final mix (queue `i_*`, `i2g`), diff vs `i2b`:**

| | quick_dev | speed_test | large | real_test | income | cpu (quick / large / real) |
|---|---|---|---|---|---|---|
| `windowq`+chest+adv+piece | +0.0023 (14 wins) | +0.0007 | +0.0121 | +0.0056 | +0.0000 | x1.71 / x2.31 / x1.43 |
| `windowq`+chest+adv | +0.0025 (15 wins) | +0.0004 | +0.0083 | +0.0032 | +0.0001 | x2.04 / x2.86 / x1.45 |

- **The mix with the piece kick** loses no payload anywhere. Its quick_dev hit rate is 97.1% (`solve` 94.7%), level with `price` from `solve`'s endpoint (96.8%, +0.0024).
  - Against the `windowr` loop it wins 5 payloads and loses none, all on `chain_0017` and `b10_single_3300`.
  - Against `solve` at 96k it is ahead: `large` +0.0073 (3 wins, 0 losses), quick_dev +0.0011 (10 wins, 1 loss), real_test +0.0016, `speed_test` +0.0005, at x0.8–1.3 the CPU.
- **Without the piece kick**, the mix is worse on `large` and real_test. On its own the piece kick does nothing (section 9), but it is kept.

**What `solve` keeps:**
- `kick_stall` (off, 0) and `kick_types` (`window+chest+adv+piece`).
- `window` is now the `windowq` kick: a window starting within the reach is shuffled, and the shuffled upgrades are re-planned before the polish.
- `windowr`, `windowa`, `insertr` and `replan` were removed.

**One detail matters:** the moved upgrades must be taken before the evaluation. `clean_special_state` pushes adv upgrades, and any upgrade ahead of a lower level of its own piece, to the back. Read after the evaluation, `special_state[at..end]` can name other upgrades. One cleaned build did that, and it lost every `chain_0017` gain (0.05–0.2% on 6 of 9 runs). With the fix, `i2i:kick_stall=32` repeats `i2g` exactly (57 of 57 runs).
