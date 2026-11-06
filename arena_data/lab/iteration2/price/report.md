# `price` investigation

**Verdict: take pieces, not the algorithm.** Its namesake, the shadow price trajectories, carries no weight, and its own start is its weakness. Its kick loop is its strength: that loop now exists in `solve`, off by default (`kick_stall=32`), and loses no payload (`final.patch`). What is settled from here is in `AI notes/Calculator core/Optimizer lab results.md`.

Batches are in `runs/`, and runs are in `arena_data/lab/runs.sqlite` under the snapshots named below.

**Re-run at the new free tap cut.** Snapshot `pr_a`, batches `ra`–`rd`, run 2026-11-03 to 11-06. `pr_a` repeats `cut_a` on all 7,948 runs they share. Every number below is from that re-run unless marked **(old cut)**. It is against plain `pr_a` (`solve` 48k at 128 points), or against `price` 1 in the same batch; cold, 3 seeds, real_test 10; income chained, 3 seeds. CPU is compared only inside a batch.

The old-cut marks cover what could not be re-run:
- knobs since removed: `order_kicks`, `windowr`, `windowa`, `insertr`, `replan`, the plain window kick, and the re-plan in `order_polish`;
- `solve` at 384k;
- quick_dev chained;
- the end-state readings (part by part).

The old cut's snapshots were `lab_r1` and `i2a`–`i2g`. What the cut changed is in `iteration2/free_tap_cut/report.md`.

**What moved at the new cut.** The verdict and every conclusion stand. These details changed:
- `price` 2 is no longer ahead of `solve` 48k on quick_dev: -0.0002, was +0.0007 (section 3).
- `single_3300_t4_n23` is now found by `solve` itself (sections 4, 10).
- `fromsolve` and the kick loop split the order basins between them (sections 8, 10): the loop takes `chain_0017` step 3 and `fromsolve` takes the rest. At the old cut `fromsolve` took them all.
- Against `solve` 96k, the kick loop's lead on `large` shrank to +0.0020 (1 win, 1 loss), from +0.0073 (3 wins, 0 losses) (section 9).
- `kick_stall=8` gets about half the gain on real_test and quick_dev, but a fifth on `large` (section 9).
- `chest_move_rate=0`, now run on every set, loses far more than `single_1104` (section 5).

## 1. The best response is inexact where the lattice does not answer (`response_points`)

`Response` takes a material's worth exactly where the lattice answers. Elsewhere it samples 16 evenly spaced amounts (`Tuning::response_points`) and joins them by straight lines (`StateBundle::worth` in `response.rs`). On some payloads that line misses the true worth by enough to pick the wrong plan.

`pnpm arena ascend` (batch `rd`) runs responses in turn: "cold a" starts from nothing, "any" from the best known. Cold a against the best known, in % of flow:

| payload | n | 16 points | 32 | 64 | 128 | accurate evaluator, 16 points (old cut) |
|---|---|---|---|---|---|---|
| small `single_0049_t4_n1` | 1 | -0.424 | 0 | 0 | 0 | -0.424 |
| small `single_0039_t4_n1` | 1 | -0.032 | -0.009 | -0.009 | 0 | -0.032 |
| small `single_0002_t4_n3` | 3 | -0.086 | -0.0003 | 0 | 0 | -0.085 |
| speed_test `single_1235_t4_n1` | 1 | -0.962 | | | 0 | |

- **It is the points, not the evaluator (old cut).** `max_brute_size`/`max_lattice_work` at their maximum change nothing, and nor does `response_joint`.
- **With one upgrade there is nothing to coordinate**, so an exact response is the optimum. These losses were errors in the response, not separate basins.
- **Cost:** none measurable. The ascend times match from 16 to 128 points: small 19.9 s and 20.5 s, speed_test 1193 s and 1209 s, real_test 203 s and 200 s.
- "Any" ends at the best known on every payload, at every point count.

In whole solves (`small` from batch `ra`, `pilot` from `rd`), against today's default of 128 points:

| variant | small diff | small hit % | pilot diff | pilot hit % | x CPU (small / pilot) |
|---|---|---|---|---|---|
| `solve` 48k | 0 | 100 | 0 | 89.6 | 1.00 |
| `solve` 48k, `response_points=16` | **-0.0067** | **95.0** | 0 | 89.6 | 1.26 / 1.00 |
| `price` effort 1 | 0 | 100 | +0.0038 | 100 | 0.97 / 0.72 |
| `price` effort 1, 16 points | **-0.0346** | **80.0** | +0.0038 | 100 | 1.01 / 0.73 |

- **`price`'s whole loss on `small` was this.** At 16 points it loses -0.035; at 128 it hits every payload.
- **Production `solve` gains too.** At 16 points it misses `small` `single_0025_t4_n2` by 0.13%. At the old cut this was a gain that every batch 1 run had missed, the 384k ones included (the gap the reference runs left open).
- `pilot` does not move at all: there the lattice answers, so the points are never used.
- **CPU on `large` (old cut)**, 10 heavy payloads, 2 seeds: `solve` 48k at 128 and at 256 points gives +0.0002 at x0.99 CPU.

**Production fix:** the `Tuning` default is now 128 points. The wasm package is rebuilt, so the site uses it too. `response_points=16` keeps the old production for comparison.

## 2. The multi-upgrade losses on the held-out set are real basins

Cold ascend at 16 and 128 points ends at the same four places:

| payload | n | cold ascend |
|---|---|---|
| `real_08927991_4514f9` | 12 | -0.40 |
| `real_27a2bc40_2991b9` | 10 | -0.40 |
| `real_8f8ac564_8bb1fc` | 10 | -0.27 |
| `real_07f47ad0_30bfd5` | 5 | -0.14 |

These are fixed points of the exact responses, reached from nothing and not from the best known. The batch 1 review (at 16 points, now in `ignored/junkyard/lab_iteration1_16pt`) put these down to back streaks, and that diagnosis stands for these only.

On `speed_test`, 128 points leave most multi-upgrade losses as they were: `single_3479_t4_n11` -1.15, `single_1208_t4_n14` -0.65. A few shrink:

| payload | 16 points | 128 points |
|---|---|---|
| `single_3301_t4_n4` | -0.158 | -0.088 |
| `single_3033_t4_n25` | -0.443 | -0.363 |
| `single_3447_t4_n22` | -0.187 | -0.174 |

## 3. `price` at matched CPU

From batch `ra`, cold unless marked. "diff" is the mean over seeds less `solve` 48k's, in % of flow; "hit" is within 0.01% of the best known.

| set | price 1 | price 2 | `solve` at similar CPU |
|---|---|---|---|
| real_test | -0.0113, hit 92.4, x0.78 | -0.0057, hit 92.1, x1.39 | 24k: -0.0047, hit 93.9, x0.64; 96k: +0.0040, x1.76 |
| real_income_test (chained) | -0.0010, x0.92 | -0.0004, x1.67 | 24k: -0.0002, x0.73 |
| quick_dev | -0.0016, x0.90 | -0.0002, x1.62 | 96k: +0.0008, x1.65 |
| quick_dev chained (old cut) | -0.0001, x1.10 | +0.0019, x1.95 | 96k: +0.0009, x1.58 |
| speed_test | -0.0039, x0.66 | -0.0028, x1.17 | 24k: -0.0004, x0.61 |
| large | -0.0085, hit 43, x0.95 | -0.0088, hit 53, x1.74 | 96k: +0.0057, x1.81 |
| advanced | +0.0028, x1.07 | +0.0027, x2.04 | **every `solve` ≤ 0**, 96k hit 94.4 |
| free_taps | -0.0348, x0.58 | -0.0013, x1.12 | 6k/12k lose -0.036 (one order basin); 24k -0.0001 |

- **`price` is behind `solve` at matched CPU everywhere but `advanced`.** At the old cut it was also ahead on quick_dev at effort 2 (+0.0007). The new cut lifts `solve` on the order basins that `price` won there.
- **Its losses and wins are a few payloads, not a spread.** At effort 2 against `solve` 48k: real_test 0 wins and 2 losses, quick_dev 12 and 10, speed_test 3 and 9.
- **`free_taps` at effort 1 is one payload**, `single_0071_serca_n19` (-0.42). It is the order basin that `solve` 6k and 12k fall into. With the ratio start off it is -0.014 (section 5).
- **Effort is not CPU across sets.** Effort 1 costs x0.58 (`free_taps`) to x1.07 (`advanced`) of `solve` 48k.
- At effort 0.125 it can run out before it gets anywhere: on speed_test `chain_1032_t4_n8:1`, one seed ends at -14.8%. That is budget, not a mechanism.
- **The old production (`response_points=16`) on every set:** -0.0067 on `small`, -0.0004 on quick_dev, -0.0003 on speed_test, and nothing on real_test (the lattice answers there).
- The effort curves of every prototype are in section 11.

## 4. What separates `price` from `solve`, payload by payload

**Free tap order basins: `price` wins.** At efforts 1 and 2, `price` beats `solve` 48k by 0.02 to 0.18 on:
- quick_dev: `chain_0017_t4_n14:3`, `b10_chain_0017_t4_n14:3`, `b10_single_3300_t4_n23`, `b10_week_inc_0004_t4_n9:3`;
- speed_test: `single_3375_t4_n21`, `single_3397_serca_n4`, `week_hone_0009_serca_n21:6`.

`single_3300_t4_n23`, one of them at the old cut, is now found by `solve` itself.

The end states were compared part by part at the old cut (free tap order, chests, plans) **(old cut)**. There, `solve`'s worse runs had a different free tap order, and `solve` at 384k sat in them too (-0.09 to -0.39%).

`price`'s order move differs from `solve`'s `order_polish` in one way. A move that loses less than 0.2% is tried again with the moved upgrade's plan replaced by its best response (`order_moves` in `price.rs`). `order_polish` keeps every plan as it is.

**Chest choices: `price` loses on lone serca upgrades.** The response is exact there: `ascend` from nothing reaches the best known at 128 points. The losses are in the chests:

- `single_1104_serca_n1`: -0.39 at effort 0.5, -0.26 at 1, -0.13 at 2. End state (old cut): the same plan, but a stack opened as 14 Special Leap in place of 9 Special Leap and 5 Leaps.
- `single_1166_serca_n1` (-0.05): the plan and three stacks differ together (old cut).

`price`'s chest move tries all of an option, then a third of that (14, 4, 1, so never 5), and keeps the first gain in one pass. `solve`'s `chest_polish` halves its step and repeats while anything moves, and the annealer moves random amounts.

**Advanced and normal plans together: `price` loses on `real_test`.** On `real_08927991_4514f9` (n=12) it loses -0.17 at effort 1 and -0.13 at effort 2. End state (old cut): the free tap order and chests are the same. The best puts nearly every advanced upgrade on strategy 5, and with that gives material 6 to about ten more taps of each of the four normal upgrades. `price` spreads the advanced upgrades over strategies 0–8. Responses only plan normal upgrades, and `price` moves advanced ones a slot at a time, so neither reaches it.

**Also lost, not analysed:** `week_hone_0004_t4_n9` steps 1–3 (-0.08 to -0.15). The loss is identical on every seed and at every effort, at both cuts, and `memetic` loses it nearly as often.

## 5. Ablations: which piece of `price` does the work

Batch `rb`. `Tuning::lab_off` leaves pieces out (`lab_off=order`, joined by `+`). Each row is against `price` 1 in the same batch.

| left out | quick_dev | speed_test | real_test | evaluations | CPU |
|---|---|---|---|---|---|
| `order` (free tap order moves in the polish) | **-0.127**, 98 worse | **-0.484**, 118 worse | -0.035, 12 worse | x0.33–0.66 | x0.99–1.15 |
| `kicks` (the whole kick phase) | -0.013, 43 worse | -0.017 | -0.064 | x0.21–0.29 | x0.27–0.35 |
| `prices` (the price trajectories, `price`'s namesake) | -0.0006 (6 better, 7 worse) | -0.0031 (3 better, 13 worse) | -0.0064 (worst one payload: -0.30) | x0.90–1.01 | x0.90–1.00 |
| `replan` (moved upgrade's response in an order move) | -0.0001 | +0.0005 | +0.0028 | x1.04–1.08 | x0.94–0.95 |
| `ratio` (start order by leapstone efficiency) | +0.0001 | +0.0003 | +0.0003 | x1.00–1.01 | x1.00–1.01 |

- **The order moves are most of `price`.** Without them it is behind by -0.13 to -0.48%.
- **The kicks are next**, but leaving them out also leaves 65–75% of the budget unspent, so this row is not at matched cost.
- **The shadow price trajectories are worth about nothing on average.** They are its namesake, yet they help on a few payloads and hurt on a few. `speed_test` leans their way.
- **The replan and the ratio start are worth nothing.** The ratio start costs one order basin on `free_taps` (`single_0071_serca_n19`; +0.034 on the set with it off).

**From the `solve` side, the replan does nothing either (old cut).** The same move added to `solve`'s `order_polish` (up to 4 or 16 a polish) moved no set by more than 0.0001, at the same CPU. `Optimizer tried.md` already had this ("trying a lost free tap order again with the plans of the upgrades it moves"); it now carries the number.

**`price`'s order basin wins do not hang on any one piece.** With `replan`, `ratio` or `prices` left out it still reaches them on some seeds (`chain_0017_t4_n14:2/3`, `single_3300_t4_n23`, `b10_chain_0017_t4_n14:3`), and they come more often at effort 2 than at 1. Only `order` loses them all, by 0.1 to 9.7% a seed.

What remains is the structure: kick, then polish with order moves and full response passes in turn, many times over. That is the iterated local search `Optimizer tried.md` rules out *in place of* chains (83.3% hit against 91.7%). These payloads say it finds order basins that chains do not.

**Chests: the annealer's random amounts matter well beyond one payload.** `solve` with `chest_move_rate=0` keeps only the end polish's halving steps. It loses `single_1104_serca_n1` by 0.39%, as `price` does.

At the old cut it was run on part of quick_dev and speed_test only. Run on every set, it loses:

| set | diff | payloads lost |
|---|---|---|
| real_test | -0.079 | 7; `real_16ca4a20_08bcc1` -1.02, `real_27a2bc40` -1.00, `real_fbaad097` -0.81 |
| speed_test | -0.022 | 6; `single_1208` -3.1, `single_1056_serca_n3` -0.94, `single_1104` -0.39, `chain_1051` -0.06 to -0.13 |
| income | -0.0044 | 5 |
| quick_dev | -0.0004 | |

Nothing moves elsewhere. `single_1166_serca_n1` needs neither: at the old cut `solve` found it with both chest moves off (-0.002), from the plan side.

## 6. Where this leaves `price`

**Verdict: take pieces, not the algorithm.** Its namesake, the price trajectories, carries no weight. What it does well is find free tap order basins through repeated kick-and-polish, and that is the one thing `solve` at any budget misses. What it does badly is all one kind: two parts of the state that have to move together, where `price` moves one at a time:

- advanced strategies with normal plans (`real_08927991`);
- chest amounts that a fixed step grid cannot hit (`single_1104_serca_n1`);
- plans with chests (`single_1166_serca_n1`).

Each of the next experiments isolates one of these:

1. **Order basins in `solve`.** A kick-and-polish phase after the chains, restricted to order-window kicks, at a fixed share of the budget: is it the kick-polish loop or the order kick itself? Against `price` with `lab_off` of every other kick type.
2. **Joint advanced and normal move.** Every advanced upgrade of a piece type to one strategy, then responses for the normal upgrades that share its material; in `price` first, where `real_08927991` is lost.
3. **Random chest amounts in `price`** in place of the 1, 1/3, 1/9 grid; this should take `single_1104_serca_n1`.

## 7. The three experiments (queue `e_*` at the old cut; batches `ra`, `rb`)

"diff" is the mean per payload against the base, in % of flow; "gap" is to the best known.

**1. Order kicks in `solve` (`order_kicks`): null (old cut; the knob is removed).** A window of the best state's order is shuffled and polished, and kept where better, after the chains.

| set | `=16` diff | `=64` diff | evals `=16` / `=64` |
|---|---|---|---|
| quick_dev | +0.0003 (2 wins, 0 losses) | +0.0004 (2 / 0) | x1.09 / x1.37 |
| speed_test | +0.0002 (2 / 0) | +0.0002 (2 / 0) | x1.06 / x1.23 |
| large | 0 | +0.0019 (1 / 0) | x1.34 / x2.37 |
| real_test, advanced, free_taps, juice, small | 0 | 0 | up to x1.6 |
| real_income_test (gap) | 0.0051, same | 0.0051, same | CPU x1.08 / x1.32 |

Across every payload, 64 kicks win 5 times and never lose. The biggest win is `b10_single_3300` (gap 0.25 to 0.15).

So `solve`'s end state is a local optimum against a window shuffle plus its own polish. The kick alone does not explain `price`'s order basins.

**Price with only the window kick** (`lab_off=kchest+kadv+kexchange+kpiece+kperturb`) is as weak as `price` with no kicks at all. Against `price` 1:

| set | diff |
|---|---|
| real_test | -0.064 |
| speed_test | -0.016 (the same -2.75 on `single_1208`) |
| quick_dev | -0.0047 |
| large | -0.018 |
| income gap | 0.0122 vs 0.0019 |

But it is the payloads with order basins that it wins:

| payload | gap, window only | gap, all kicks |
|---|---|---|
| `chain_0017` steps | 0–0.084 | 0.038–0.181 |
| `single_3300` | 0 | 0.071 |

The mirror holds: `lab_off=kwindow` loses on exactly those (`chain_0017`, `b10_chain_0017`, `single_3300`, `b10_single_3300`, by 0.10–0.37) and is neutral elsewhere.

So the window kick is the order-basin tool, and the other kick types carry the rest. The difference from `solve` has to lie in what runs around the kick. Either it is `price`'s state when it kicks, or it is the many kicks until a stall (30 + 40 × effort without a gain) against a fixed 16 or 64. It is not the polish's insertion moves, since `solve`'s `order_polish` has them too.

**2. Joint advanced kick (`lab_on=advall`): no.** It changes nothing on `real_08927991` over 10 seeds. Its largest loss is `real_27a2bc40` (-0.13), a payload where every random change to `price` swings by about ±0.1–0.2 over 10 seeds.

| set | diff |
|---|---|
| advanced | -0.002 |
| large | -0.001 |
| real_test | -0.004 |
| elsewhere | about 0 |

The gap `price` leaves on advanced plans is not one of moving several advanced upgrades together.

**3. Random chest amounts (`lab_on=chestrand`): it fixes what it was meant to, and swings others.**

- **Fixed:** `single_1104_serca_n1` (gap 0.262 to 0) and `single_1166_serca_n1` (0.048 to 0.002). `single_3397_serca_n4`, fixed by it at the old cut, `price` now reaches without it.
- **Lost:** `chain_1051` (-0.06 to -0.13 on three steps), `single_1000` (-0.05) and `real_27a2bc40` (-0.18). `single_1208` now gains (+0.15), where at the old cut it lost.
  - Four uniform draws in `1..have` seldom land on the small amounts that the 1/3, 1/9 grid reaches.
  - So random amounts should come in addition to the grid, not in its place.
- **Net:** about 0 everywhere (real_test -0.005, quick_dev +0.0003, speed_test +0.0006); income gap 0.0017 vs 0.0019.

**What this leaves:**

1. Order basins are found by `price`'s kick loop around a window shuffle, not by the shuffle bolted onto `solve`'s end. Next to try is a stall-bounded loop in `solve` (kick, polish, keep, until N without a gain) that draws from more than one kick type.
2. Chest amounts want both the grid and random draws.
3. The joint advanced move is ruled out.

## 8. The follow-ups (queue `f_*` at the old cut; batches `ra`, `rb`)

The numbers come from `cmp.py` and `pay.py`.

**Kick types in `price`, one off at a time, against `price`:**

| kick off | real_test | quick_dev | speed_test | large |
|---|---|---|---|---|
| kchest | -0.0028 | -0.0010 | -0.0013 | -0.0076 |
| kadv | -0.0040 | -0.0001 | -0.0004 | -0.0158 |
| kexchange | -0.0021 | -0.0002 | +0.0003 | -0.0082 |
| kpiece | -0.0022 | +0.0005 | -0.0010 | -0.0127 |
| kperturb | -0.0010 | +0.0001 | -0.0012 | -0.0223 |

- On real_test, quick_dev and speed_test no single kick type carries the loop. Each costs at most a few thousandths, while all of them off cost -0.064 on real_test (section 7).
- On `large` each one costs -0.008 to -0.022, as much as the five together (-0.018). `large` is 10 payloads at 3 seeds, and `price` swings there (hit 43%), so read it as that spread, not as five kicks each needed.
- So the loop needs kicks of several kinds, and which kind hardly matters.

**`chestboth` (the 1/3 grid plus 4 random amounts) is the right chest move, and the losses of section 7 are noise.** It fixes the lone-chest payloads exactly as `chestrand` does:

| payload | before | `chestboth` |
|---|---|---|
| `single_1104_serca_n1` | 0.262 | 0 |
| `single_1166_serca_n1` | 0.048 | 0.002 |

But its losses match `chestrand`'s to the last digit on quick_dev and speed_test, so the fine steps cannot have caused them. It differs from `chestrand` only on `real_27a2bc40` (+0.06). Seed by seed:

- `chain_1051` is bimodal. Every `price` variant ends each seed at a gap of either 0 or about 0.19, and the two random-amount variants shift the random stream the same way.
- `single_1208` and `single_1000` swing the same way, between 0.05 and 0.8.

**A stall-bounded kick loop in `solve` works** (`kick_stall=32`, all four kick types: window, chest, adv, piece). Each kick is polished and kept where better, until 32 in a row gain nothing. The window kick here is the final one (section 10); the plain window kick this table first ran with is removed.

| set | diff vs `solve` 48k | wins / losses | hit % | cpu |
|---|---|---|---|---|
| real_test | +0.0056 | 1 / 0 | 96.6 → 97.6 | x1.54 |
| large | +0.0077 | 2 / 0 | 53.3 → 70.0 | x2.71 |
| quick_dev | +0.0016 | 11 / 0 | 95.5 → 97.8 | x1.92 |
| speed_test | +0.0006 | 3 / 0 | 96.3 → 96.7 | x1.38 |
| income (chained) | +0.0001 | 0 / 0 | 98.0 → 98.1 | x1.53 |

- It never loses on any of the 954 payload steps, in any set.
- The real_test number is almost all from `real_27a2bc40` (+0.204), a noisy payload.
- Against `solve` at 96k: section 9.

**`price` from `solve`'s endpoint (`lab_on=fromsolve`)** starts `price` from `solve`'s finished plan, with `solve`'s cost outside its budget.

| set | diff vs `solve` (with `prices` off) | hit % |
|---|---|---|
| quick_dev | +0.0014 (+0.0015) | 96.6 (96.8) |
| real_test | +0.0034 (+0.0051) | 97.9 |
| large | +0.0043 (+0.0059) | 60.0 |
| speed_test | +0.0006 (+0.0009) | 97.4 |
| income | +0.0003 | 98.8 |

- CPU is about x1.6–1.9 on the main sets, up to x2.2 on the small ones.
- It never loses a payload to `solve`.
- At the old cut quick_dev was +0.0024. The new cut gives `solve` part of what `fromsolve` added there.
- Against the `solve` kick loop it ties overall: quick_dev -0.0001 (3 wins, 4 losses), speed_test +0.0002, real_test -0.0005, `large` -0.0017. The two split the order basins:

  | payload | gap, kick loop | gap, `fromsolve` (prices off) |
  |---|---|---|
  | `chain_0017` step 2 | 0.072 | 0 |
  | `chain_0017` step 3 | 0 | 0.175 |
  | `b10_single_3300` | 0.071 | 0 |
  | `single_3397_serca_n4` | 0.025 | 0 |
  | `single_3375_t4_n21` | 0.047 | 0 |
  | `b10_week_inc_0004` step 3 | 0.064 | 0 |

  It loses a little (0.01–0.02) on `single_0159`, `single_0161` and `chain_0001`. At the old cut it also took `chain_0017` step 3.
- With the price trajectories off it does as well or better; again they carry no weight.
- On `single_1208` and `single_1000` it brings `price`'s gaps from 0.38 and 0.12 to 0.015 and 0.002.

So `price`'s own start is its weakness, and its kick loop is its strength.

## 9. What carries the kick loop in `solve` (queue `g_*` at the old cut; batches `ra`, `rb`)

Each kick type alone at `kick_stall=32`, the stall length, a CPU-matched variant, and `price` from `solve`'s endpoint without the window kick.

**Each kick type alone, diff vs `solve` 48k:**

| kick types | real_test | large | quick_dev | speed_test | cpu (real_test / large) |
|---|---|---|---|---|---|
| all four | +0.0056 | +0.0077 | +0.0016 | +0.0006 | x1.54 / x2.71 |
| adv | +0.0027 | +0.0077 | +0.0005 | +0.0000 | x1.06 / x3.45 |
| chest | +0.0062 | +0.0000 | +0.0000 | +0.0003 | x1.84 / x0.99 |
| window | +0.0001 | +0.0013 | +0.0014 | +0.0003 | x1.22 / x1.95 |
| piece | +0.0000 | +0.0001 | +0.0000 | +0.0000 | x1.27 / x1.32 |

The window kick is the final one (section 10). The plain window kick this row first ran with gave real_test 0, `large` +0.0018 and quick_dev +0.0003 (old cut).

- **Each set has its own kick.** Adv carries `large`, chest carries real_test (`real_27a2bc40`), window carries quick_dev, and chest and window share `speed_test`. All four together are about as good as the best one alone on every set.
- The piece kick does nothing in `solve`; its response is what `response_polish` already finds.
- Income is unchanged by any of them.

**The stall length:**

| | real_test | large | quick_dev | cpu (real_test / large) |
|---|---|---|---|---|
| `kick_stall=8` | +0.0025 | +0.0014 | +0.0007 | x1.12 / x1.29 |
| `kick_stall=32` | +0.0056 | +0.0077 | +0.0016 | x1.54 / x2.71 |
| `kick_stall=96` | +0.0062 | +0.0079 | +0.0019 | x2.61 / x4.86 |

8 gets about half on real_test and quick_dev, and a fifth on `large`. 96 adds little to 32, at about double the CPU.

**Kicks against annealing at matched CPU.**
- **Half the annealing plus the kicks** (`max_iters=24000,kick_stall=32`) costs x1.0–1.5 of `solve` 48k, and is no better than it:
  - real_test +0.0028, with 1 loss;
  - quick_dev +0.0003, with 8 wins and 11 losses;
  - `large` -0.0020 (2 wins, 2 losses);
  - `advanced`, `free_taps` and `juice` all slightly negative.
- **Against `solve` at 96k**, `kick_stall=32` at 48k is ahead or level everywhere:

  | set | diff | wins / losses | cpu |
  |---|---|---|---|
  | real_test | +0.0016 | | x0.87 |
  | quick_dev | +0.0008 | 8 / 1 | x1.17 |
  | `speed_test` | +0.0004 | 3 / 1 | x0.86 |
  | `large` | +0.0020 | 1 / 1 | x1.49 |
  | income | +0.0001 | | |

  At the old cut the `large` lead was +0.0073 (3 wins, 0 losses).
- So the kicks add to the annealing; they do not replace it.

**The window kick is what takes `price` from `solve`'s endpoint into the order basins.** With `kwindow` off, `fromsolve` (prices off) drops on quick_dev from +0.0015 to +0.0009. Payload by payload:

| payload | gap with the window kick | gap without |
|---|---|---|
| `chain_0017` step 2 | 0 | 0.072 |
| `b10_chain_0017` step 3 | 0 | 0.051 |
| `b10_single_3300` | 0 | 0.122 |

**Why the same kick fails in `solve`.** `solve`'s polish runs `order_polish` first, which can move the shuffled upgrades straight back before their plans adapt to the new order. `price`'s polish re-plans every normal upgrade first and moves the order after.

`kick_types=windowr` is a window kick that re-plans the shuffled upgrades (`respond_one`, kept where better) before the polish. It was smoke-tested on `chain_0017` and `single_3300` (quick_dev, 3 seeds; old cut, kicks since removed):

| | wins vs `i2b` | `chain_0017` step 3 gap | `b10_single_3300` gap |
|---|---|---|---|
| `window` | 2 | 0.145 | 0.174 |
| `windowr` | 7 | 0 | 0.110 |
| `windowa` (re-plans all normal upgrades) | 6 | 0 | 0.110 |

So a kick only reaches a basin if the parts it moved get to re-plan for their new place before a greedy repair runs. The full sets were in queue `h_*`.

## 10. Re-plan before the polish (queues `h_*`, `i_*` at the old cut; batch `ra`)

**`windowr` on the full sets (queue `h_*`, old cut, against `i2b`):**

| | quick_dev | speed_test | large | real_test | income |
|---|---|---|---|---|---|
| `window` alone | +0.0003 (2 wins) | +0.0002 | +0.0018 | 0 | 0 |
| `windowr` alone | +0.0014 (7 wins) | +0.0004 | +0.0019 | 0 | 0 |
| all four, `window` | +0.0009 (10 wins) | +0.0005 | +0.0109 | +0.0056 | 0 |
| all four, `windowr` | +0.0015 (13 wins) | +0.0007 | +0.0115 | +0.0056 | 0 |

- None of them lost anywhere, and the CPU was the same as with `window` or lower.
- Against the old loop, the `windowr` loop won 5 payloads and lost none. All 5 were order basins (`chain_0017` steps 2–3, `b10_single_3300`, `single_3397_serca_n4`).

**Two more kicks, smoke-tested on the order-basin payloads** (quick_dev and speed_test, 3 seeds; gaps; old cut, kicks since removed):

| | `chain_0017` step 2 | step 3 | `b10_single_3300` | `single_3300` | `single_3397_serca_n4` | `single_3375` | `week_inc_0004` step 3 |
|---|---|---|---|---|---|---|---|
| `i2b` (`solve`) | 0.109 | 0.145 | 0.250 | 0.071 | 0.062 | 0.038 | 0.058 |
| `windowr` | 0.055 | 0 | 0.110 | 0.071 | 0.024 | 0.038 | 0.058 |
| `windowq` (`windowr` starting within the free taps' reach) | 0 | 0 | 0 | 0.071 | 0.024 | 0.038 | 0.058 |
| `insertr` (an upgrade from anywhere moved within reach; it and the next two re-planned) | 0.109 | 0 | 0.039 | 0.071 | 0.039 | 0.038 | 0.058 |
| `price` from `solve` | 0.055 | 0 | 0 | 0 | 0 | 0 | 0 |

The same payloads at the new cut (`windowq` is today's `window`):

| | `chain_0017` step 2 | step 3 | `b10_single_3300` | `single_3300` | `single_3397_serca_n4` | `single_3375` | `b10_week_inc_0004` step 3 |
|---|---|---|---|---|---|---|---|
| `solve` | 0.072 | 0.175 | 0.162 | 0 | 0.062 | 0.047 | 0.064 |
| `window` alone | 0 | 0 | 0.039 | 0 | 0.024 | 0.047 | 0.064 |
| the final loop (all four) | 0.072 | 0 | 0.071 | 0 | 0.025 | 0.047 | 0.064 |
| `price` from `solve`, prices off | 0 | 0.175 | 0 | 0 | 0 | 0 | 0 |

- On long orders most windows fall past the reach, where a shuffle changes nothing. `windowq` puts every kick where it counts, so it does the most per kick.
- Mixed with the chest, adv and piece kicks, the window kick gets a quarter of the kicks. In the mix, `chain_0017` step 2 and `b10_single_3300` end at 0.072 and 0.071, against 0 and 0.039 for the window kick alone.
- `replan` (every normal upgrade re-planned after any kick; old cut) helped only `single_3300` (0.037). With the chest, adv and piece kicks alone it did nothing.

**What `price` still finds that no `solve` kick does.** From `solve`'s endpoint, with prices off (batches `rb`, `rc`):
- With its whole kick loop off, `price` fixes none of the last four payloads. With its order moves off it loses `single_3397` (0.022) and `single_3375` (0.036).
- With any one kick type off, it still fixes `single_3375`, `b10_week_inc_0004` and `single_3397`, so any of its kicks will do there. The perturb kick is the one exception: without it `single_3375` stays at 0.023.
- `b10_single_3300` needs its window kick (0.122 without), and its perturb kick (0.071 without).

In the end states (old cut), the gains are joint moves of several upgrades:
- **`single_3375`:** upgrade 18 moves from past the reach to the front of the order. The juice tails of upgrades 10–17 all shorten and 18–19 swap plans.
- **`week_inc_0004` step 3:** the order is the same; upgrades 6, 7 and 8 shift their plans together.

`price`'s polish takes each upgrade's response in turn, for several rounds, with its order and chest moves in between. A kick followed by that polish walks to these states. `solve`'s polish after the same kicks does not, and re-planning the moved upgrades once (`windowr`, `replan`) was not enough. This is left open; the next prototypes' reviews may throw light on it.

**The lesson:** a kick reaches a new basin only if the parts it moved re-plan for their new place before a greedy repair runs, and only if it moves something that matters (within the reach).

**The final mix, diff vs `solve` 48k:**

| | quick_dev | speed_test | large | real_test | income | cpu (quick / large / real) |
|---|---|---|---|---|---|---|
| `window`+chest+adv+piece | +0.0016 (11 wins) | +0.0006 | +0.0077 | +0.0056 | +0.0001 | x1.92 / x2.71 / x1.54 |
| `window`+chest+adv | +0.0019 (13 wins) | +0.0003 | +0.0060 | +0.0032 | +0.0001 | x2.19 / x2.99 / x1.64 |

At the old cut, against `i2b`, the mix with the piece kick gave quick_dev +0.0023 (14 wins), `large` +0.0121, real_test +0.0056 and `speed_test` +0.0007.

- **The mix with the piece kick** loses no payload anywhere. Its quick_dev hit rate is 97.8% (`solve` 95.5%), above `price` from `solve`'s endpoint (96.8%, +0.0015).
  - Against the `windowr` loop (old cut) it won 5 payloads and lost none, all on `chain_0017` and `b10_single_3300`.
  - Against `solve` at 96k it is ahead or level on every set (section 9).
- **Without the piece kick**, the mix is worse on `large` and real_test and a little better on quick_dev. On its own the piece kick does nothing (section 9), but it is kept.

**What `solve` keeps:**
- `kick_stall` (off, 0) and `kick_types` (`window+chest+adv+piece`).
- `window` is the `windowq` kick: a window starting within the reach is shuffled, and the shuffled upgrades are re-planned before the polish.
- `windowr`, `windowa`, `insertr` and `replan` were removed.

**One detail matters:** the moved upgrades must be taken before the evaluation. `clean_special_state` pushes adv upgrades, and any upgrade ahead of a lower level of its own piece, to the back. Read after the evaluation, `special_state[at..end]` can name other upgrades. One cleaned build did that, and it lost every `chain_0017` gain (0.05–0.2% on 6 of 9 runs). With the fix, `i2i:kick_stall=32` repeats `i2g` exactly (57 of 57 runs).

## 11. Effort curves of every prototype at the new cut

Batch `ra`, the same sets and seeds. Each cell is diff vs `solve` 48k, (hit %), and x CPU. `surrogate` is in its own report.

| variant | real_test | quick_dev | speed_test | large | income (chained) | advanced | free_taps | juice | small |
|---|---|---|---|---|---|---|---|---|---|
| `solve` 6k | -0.0096 (84) x0.32 | -0.0070 (88) x0.32 | -0.0031 (92) x0.27 | -0.0184 (37) x0.44 | -0.0013 (95) x0.38 | -0.0040 (78) x0.37 | -0.0361 (92) x0.30 | -0.0018 (89) x0.28 | +0.0000 (100) x0.36 |
| `solve` 12k | -0.0039 (90) x0.44 | -0.0044 (90) x0.43 | -0.0042 (94) x0.39 | -0.0037 (47) x0.50 | -0.0007 (96) x0.50 | -0.0010 (89) x0.43 | -0.0366 (89) x0.37 | +0.0010 (94) x0.36 | +0.0000 (100) x0.68 |
| `solve` 24k | -0.0047 (94) x0.64 | -0.0027 (93) x0.64 | -0.0004 (95) x0.61 | -0.0196 (40) x0.64 | -0.0002 (98) x0.73 | -0.0019 (89) x0.59 | -0.0001 (97) x0.53 | -0.0008 (92) x0.55 | +0.0000 (100) x0.99 |
| `solve` 48k | 0 (97) x1 | 0 (96) x1 | 0 (96) x1 | 0 (53) x1 | 0 (98) x1 | 0 (94) x1 | 0 (97) x1 | 0 (94) x1 | 0 (100) x1 |
| `solve` 96k | +0.0040 (98) x1.76 | +0.0008 (96) x1.65 | +0.0002 (97) x1.60 | +0.0057 (57) x1.81 | -0.0000 (98) x1.16 | +0.0000 (94) x1.89 | +0.0001 (97) x1.92 | +0.0000 (94) x1.87 | +0.0000 (100) x1.04 |
| `price` 0.125 | -0.0434 (82) x0.14 | -0.0111 (89) x0.19 | -0.0868 (89) x0.13 | -0.0621 (30) x0.16 | -0.0043 (91) x0.20 | +0.0009 (97) x0.17 | -0.0023 (92) x0.10 | -0.0021 (92) x0.12 | -0.0014 (98) x0.20 |
| `price` 0.25 | -0.0279 (89) x0.26 | -0.0075 (89) x0.31 | -0.0090 (89) x0.21 | -0.0322 (37) x0.29 | -0.0017 (94) x0.34 | +0.0028 (100) x0.31 | -0.0028 (92) x0.18 | -0.0013 (92) x0.21 | -0.0028 (97) x0.32 |
| `price` 0.5 | -0.0204 (92) x0.45 | -0.0046 (90) x0.52 | -0.0070 (91) x0.38 | -0.0221 (47) x0.53 | -0.0011 (97) x0.55 | +0.0010 (97) x0.60 | -0.0212 (86) x0.34 | -0.0014 (92) x0.38 | -0.0000 (100) x0.54 |
| `price` 1 | -0.0113 (92) x0.78 | -0.0016 (92) x0.90 | -0.0039 (93) x0.66 | -0.0085 (43) x0.95 | -0.0010 (96) x0.92 | +0.0028 (100) x1.07 | -0.0348 (92) x0.58 | -0.0009 (92) x0.66 | -0.0000 (100) x0.97 |
| `price` 2 | -0.0057 (92) x1.39 | -0.0002 (94) x1.62 | -0.0028 (94) x1.17 | -0.0088 (53) x1.74 | -0.0004 (98) x1.67 | +0.0027 (97) x2.04 | -0.0013 (92) x1.12 | -0.0004 (92) x1.30 | -0.0000 (100) x1.82 |
| `memetic` 0.25 | -0.0391 (75) x0.36 | -0.0219 (86) x0.35 | -0.3603 (76) x0.25 | -0.1369 (30) x0.29 | -0.0054 (85) x0.49 | -0.0133 (89) x0.22 | -0.0139 (78) x0.13 | +0.0003 (94) x0.18 | -0.0000 (100) x0.16 |
| `memetic` 0.5 | -0.0310 (84) x0.59 | -0.0118 (91) x0.54 | -0.0864 (89) x0.48 | -0.0846 (43) x0.49 | -0.0026 (91) x0.76 | -0.0133 (92) x0.35 | -0.0057 (89) x0.25 | -0.0023 (92) x0.31 | -0.0000 (100) x0.37 |
| `memetic` 1 | -0.0232 (88) x0.91 | -0.0045 (93) x0.81 | -0.0056 (93) x0.84 | -0.0341 (47) x0.82 | -0.0010 (95) x1.12 | -0.0100 (92) x0.62 | -0.0017 (89) x0.46 | -0.0018 (92) x0.55 | -0.0000 (100) x0.78 |
| `memetic` 2 | -0.0116 (92) x1.61 | +0.0001 (95) x1.61 | -0.0005 (96) x1.63 | -0.0081 (63) x1.48 | +0.0001 (99) x2.01 | +0.0018 (94) x1.24 | -0.0013 (92) x0.86 | -0.0004 (94) x1.09 | -0.0000 (100) x1.85 |
| `cem` 0.25 | -0.0372 (64) x0.24 | -0.0316 (82) x0.24 | -0.0314 (80) x0.21 | -0.2238 (30) x0.17 | -0.0051 (87) x0.33 | -0.0060 (83) x0.20 | -0.0727 (81) x0.12 | -0.0100 (81) x0.13 | +0.0000 (100) x0.17 |
| `cem` 0.5 | -0.0146 (73) x0.48 | -0.0194 (85) x0.47 | -0.0204 (84) x0.40 | -0.1781 (33) x0.33 | -0.0030 (91) x0.66 | -0.0041 (86) x0.39 | -0.0520 (83) x0.22 | -0.0077 (83) x0.25 | +0.0000 (100) x0.32 |
| `cem` 1 | -0.0078 (81) x0.95 | -0.0121 (88) x0.92 | -0.0097 (86) x0.78 | -0.1301 (33) x0.66 | -0.0017 (94) x1.30 | -0.0011 (89) x0.76 | -0.0097 (83) x0.44 | -0.0034 (86) x0.50 | +0.0000 (100) x0.61 |
| `cem` 2 | -0.0019 (87) x1.90 | -0.0087 (90) x1.81 | -0.0044 (88) x1.54 | -0.0997 (33) x1.28 | -0.0010 (96) x2.58 | -0.0003 (89) x1.49 | -0.0069 (86) x0.84 | -0.0001 (92) x0.97 | +0.0000 (100) x1.21 |
| `lns` 0.25 | -0.0493 (79) x0.25 | -0.0442 (85) x0.30 | -0.0290 (85) x0.17 | -0.1379 (40) x0.28 | -0.0045 (90) x0.30 | -0.0096 (89) x0.29 | -0.1798 (64) x0.15 | -0.0015 (92) x0.20 | -0.0028 (97) x0.20 |
| `lns` 0.5 | -0.0397 (81) x0.43 | -0.0261 (87) x0.55 | -0.0173 (88) x0.31 | -0.0979 (47) x0.50 | -0.0039 (91) x0.56 | -0.0022 (94) x0.52 | -0.1735 (75) x0.28 | -0.0015 (92) x0.38 | -0.0014 (98) x0.40 |
| `lns` 1 | -0.0335 (84) x0.79 | -0.0169 (90) x1.03 | -0.0100 (91) x0.57 | -0.0909 (50) x0.95 | -0.0032 (92) x1.07 | +0.0025 (97) x0.99 | -0.1636 (78) x0.52 | -0.0014 (92) x0.75 | -0.0014 (98) x0.81 |
| `lns` 2 | -0.0271 (87) x1.57 | -0.0104 (92) x1.89 | -0.0097 (91) x1.07 | -0.0769 (50) x1.83 | -0.0030 (93) x2.02 | +0.0031 (100) x1.85 | -0.1186 (89) x1.00 | -0.0013 (92) x1.42 | -0.0014 (98) x1.48 |

- **No prototype beats `solve` at matched CPU on the main sets.** At effort 2 `memetic` reaches `solve` 48k on quick_dev (+0.0001) and income (+0.0001), at x1.6–2.0 the CPU.
- **`memetic` collapses at low effort on `speed_test`** (-0.36 at 0.25, -0.086 at 0.5) and is near `solve` from effort 1.
- **`cem` and `lns` stay well behind on `large`** (-0.08 to -0.22 at every effort). `lns` also stays behind on `free_taps` (-0.12 to -0.18), mostly from `single_0071_serca_n19` (-1.24), the order basin of section 3.
- **On `advanced` only `cem` stays behind.** `price` is ahead from effort 0.25, `lns` from 1, `memetic` at 2.
