# Optimizer lab: results

What the lab has found, by topic. How the lab works is in `Optimizer lab.md`; what not to try again is in `Optimizer tried.md`. Numbers are in % of flow against `solve` 48k; evidence is in each investigation's `report.md`, under `arena_data/lab/`.

**Baselines** in `runs.sqlite`:

- `lab_r1`: `solve` 48k.
- `lab_r1:max_iters=96000`: `solve` at 96k.
- `i2g:kick_stall=32,kick_types=windowq+chest+adv+piece`: the kick loop at the old cut.
- `cut_a`: `solve` 48k at the free tap cut of 2026-10-31; `cut_a:max_iters=96000` is 96k. Every build since repeats this one, not those above. The three above used the old cut. Their metrics stay comparable, but on order-basin payloads the new cut alone lifts `solve`, so gains measured against `lab_r1` run high there. Every row below is against `cut_a` or a plain baseline that repeats it. Evidence: `iteration2/free_tap_cut/report.md`.
- `pr_a`: the `price` re-run at the new cut (batches `price/runs/ra`–`rd`); repeats `cut_a` on all 7,948 shared runs. It holds `solve` 6k–96k, the effort curves of `price`, `memetic`, `cem` and `lns` (0.25–2), and the kick loop (`pr_a:kick_stall=32`) with its variants. The curves are in `price` report 11.

## Prototypes

| Prototype | Verdict | Evidence |
|---|---|---|
| `price` | Take pieces, not the algorithm. Its kick loop is its strength and its own start its weakness. | `iteration2/price/report.md` |
| `surrogate` | Take one piece: its descent, as a re-plan between a kick and `solve`'s polish (`sur_stall`), with `sur_screen` to cut its CPU. The order moves among the plans carry most of it; the model adds a little through its error, and true moves with price noise do as well at more CPU. Its calibration is worth nothing; its own start and budget are its weakness. | `iteration2/surrogate/report.md` |
| `memetic` | Take two pieces: its polish as a `solve` knob (`mem_polish`), and its generations from `solve`'s endpoint. Its own start is its weakness. | `iteration2/memetic/report.md` |
| `cem` | Take no piece of its model. Its restarts are its strength, and a cap of 10 places in its order polish was most of its loss. Run after `solve`, its restarts reach `b10_week_inc_0004:3` and `single_3375`, which the kick loop misses. | `iteration2/cem/report.md` |
| `lns` | Take one piece: its rebuild of a piece type, as a kick type in `solve`'s kick loop (`rebuild`). Its destroy step carries it; its ladder, acceptance and adaptive weights carry nothing. Its own start, which drops low levels from the free tap order, is its weakness. | `iteration2/lns/report.md` |

## Pieces, and where they stand

| Piece | Status | Gain | Cost | Evidence |
|---|---|---|---|---|
| `response_points` 128 | **In production** | Up to 0.96 on lone upgrades; `small` +0.007 | None measurable | `price` report 1 |
| Kick loop, `kick_stall=32`, all four kick types | Candidate; in tree, off (`kick_stall` 0) | real_test +0.0056 (hit 96.6 → 97.6), quick_dev +0.0016 (11 wins, 0 losses; 95.5 → 97.8), `speed_test` +0.0006, `large` +0.0077 (53 → 70), income +0.0001; no payload lost | x1.4–2.7 CPU | `price` report 8–10 |
| Surrogate loop after `solve`: `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach` | Candidate; in tree, off (`sur_stall` 0) | Against `cut_a`: real_test +0.0064 (hit 96.6 → 97.9), quick_dev +0.0018 (95.5 → 97.3), `speed_test` +0.0016, `large` +0.0066, advanced, free_taps and juice to 100, income +0.0002; no payload lost | x2.0–3.3 CPU | `surrogate` report 5–11 |
| Screened polish in that loop: `sur_screen=true` | Candidate with it | Same gain within 0.01 on every payload (10-seed gap sum 0.288 against 0.276) | 13–24% less CPU than the loop | `surrogate` report 10 |
| Memetic's polish after `solve`: `mem_polish=true` | Candidate; in tree, off | quick_dev +0.0014 (4 wins, 0 losses; 95.5 → 96.3), `large` +0.0021, 0 elsewhere; no payload lost | x1.03–1.17 CPU | `memetic` report 4 |
| Kick loop with `mem_polish` | Candidate pair | quick_dev +0.0019 (97.6), `large` +0.0093 (73.3), real_test +0.0058, `speed_test` +0.0007; the union of both's order basins | The kick loop's own; x1.56–1.85 on the regime sets against its x2.2–3.0 | `memetic` report 4 |
| Memetic generations from `solve`'s endpoint (lab: `algo=memetic,lab_on=fromsolve,lab_off=oreplan`) | Lab only; for a replacement | quick_dev +0.0015, `speed_test` +0.0019 (98.6), `large` +0.0061, real_test +0.0036, income +0.0003; no payload lost | x1.30–1.41, `solve` included | `memetic` report 3–4 |
| cem's restarts after `solve` (lab: `algo=cem,lab_effort=0.5,lab_on=fullreach+fromsolve+ownstart`) | Lab only; a complement to the kick loop | quick_dev +0.0008 (8 wins, 0 losses), real_test +0.0033, income +0.0001; no payload lost. With `kick_stall=32,mem_polish=true`, offline better-of-two (not a run): quick_dev +0.0021, real_test +0.0061 (with L instead: +0.0022, +0.0070) | x1.21–1.54, `solve` included; with the kick loop x2.34 on quick_dev (with L x2.96) | `cem` report 4 |
| Rebuild kick with the kick loop and `mem_polish`: `kick_stall=32,mem_polish=true,kick_types=window+chest+adv+piece+rebuild` | Candidate; in tree, off (`rebuild` only when named) | Against `cut_a`: quick_dev +0.0021 (9 wins, 0 losses; 97.6), real_test +0.0044, `speed_test` +0.0007, `large` +0.0045, advanced, free_taps and juice to 100, income +0.0001; no payload lost. Takes `b10_week_inc_0004:3`, `single_0085` and `single_3375` to 0 on every seed, and keeps the kick loop's order basins. Against the pair without it: `large` -0.0047 and real_test -0.0014, on the noisy `single_0159` and `real_27a2bc40` | x0.97–1.04 of the pair on the main sets, x1.23–1.25 on the regime sets | `lns` report 5 |
| lns from `solve`'s endpoint (lab: `algo=lns,lab_effort=0.5,lab_on=fromsolve`) | Lab only; superseded by the rebuild kick | quick_dev +0.0006 (4, 0), regime sets to 100; no payload lost. The rebuild kick alone matches it at the same CPU and also takes `single_3375` | x1.47, `solve` included | `lns` report 4–5 |
| Chest amounts from the grid plus 4 random draws | Candidate; `price` `chestboth` only, not in `solve` yet | Fixes `single_1104_serca_n1` and `single_1166_serca_n1` | None measurable | `price` report 7–8 |

Against `solve` at 96k in the same batch the kick loop is ahead or level everywhere: real_test +0.0016, quick_dev +0.0008 (8 wins, 1 loss), `large` +0.0020 (1, 1), at x0.86–1.49 the CPU. At the old cut its `large` lead was +0.0073.

## What makes a result good

**Free tap order.**

- Insertion of a normal upgrade within a free tap's reach is what makes an order good: without those moves `price` loses 0.13–0.48. `solve`'s `order_polish` already has them.
- `solve` misses some order basins at any budget, 384k included. The kick loop reaches most of them.
- An order polish capped at a fixed number of places loses long free tap reaches. cem's cap of 10 cost `large` 0.10 and `single_0031_serca_n36` 0.368 on every seed; the plans that looked wrong followed the order.

**Kicks.**

- A kick reaches a new basin only if what it moved is re-planned before a greedy repair runs. Otherwise `order_polish` puts a shuffled window straight back.
- It must also move something that matters: a window starting past the free taps' reach changes nothing.
- The moved upgrades must be read before the evaluation, which cleans the order (`clean_special_state`). One build read them after and lost every `chain_0017` gain.
- Each set has its own kick: adv carries `large`, chest carries real_test, window carries quick_dev and the order basins, and chest and window share `speed_test`. All four together match the best single kick on each set. The piece kick does nothing alone, but `large` and real_test are worse without it in the mix.
- Stall length: 8 gets about half of 32 (a fifth on `large`), and 96 adds little to 32 for twice the CPU.
- Kicks add to the annealing and do not replace it: halving the annealing to pay for them loses at matched CPU.

**Re-planning after a kick.**

- A kick needs a re-plan before `order_polish`: the plans of every normal upgrade, with order and advanced moves among them, in rounds. Without any, `chain_0017:3` stays at 0.13–0.18. Without the order moves the 10-seed gap sum is 0.399 against 0.276; without the plans 0.656.
- With the order moves the descent ends on the new basin itself, and the polish only confirms it.
- A moment surrogate for those steps adds a little over true moves (0.276 against 0.303), mostly through its error. True moves on band prices perturbed at random (log spread 0.1) reach 0.264, at +87% of the loop's CPU on `large` and about +30% elsewhere. Only `single_3116` needs the surrogate's own plan steps. A plain normal model is enough.
- A polish of one round first, in full only where that beats the best, keeps the gain at 13–24% less CPU.
- Every win moves several plans together (median 7, none a single plan; two thirds with the order).
- Kick kinds trade payloads: price `single_0044`, rand the order basins, chest and empty real_test. Stall 32, not 16.

**Paired order moves.**

- One upgrade out of the free taps and another in at once (memetic's toggles), each retried with both upgrades re-planned, takes `chain_0017:2/3`, `b10_chain_0017:3` and `b10_single_3300` from `solve`'s endpoint on every seed.
- Without the re-plan it gains nothing (+0.0001 against +0.0014), and without the pairs a quarter (+0.0004). `solve`'s `order_polish` makes single insertions only.
- Runs of 2–3 to the front or the end add nothing.

**A population from `solve`'s endpoint.**

- Memetic's generations from `solve`'s endpoint take the joint-move payloads that no polish or kick loop reaches: `b10_week_inc_0004:3`, `single_3397`, `chain_1051`, `single_3375`, `week_hone_0009:6`. The gains are children's responses after recombination.
- It is level with the surrogate loop on quick_dev, `speed_test`, income and `large`, at about 0.65 of its CPU (across batches).
- Recombination adds a little (income, the regime sets). Price seeds and the re-plan of failed order moves add nothing there.

**Restarts.**

- Restarts from one cheap start, each a short descent (random partial redraws kept if better) and a polish, reach `b10_week_inc_0004:3` and `single_3375`, which kicks from the best miss. Each lands in another basin; they need no model and no special start (cem with its refit and piece-type factor off is level).
- They do not improve a polished state: started from `solve`'s endpoint, cem's samples gain nothing (quick_dev +0.0001 at x2.9).
- Moving juice between upgrades (`solve`'s exchanges) is part of what reaches advanced plans with normal ones: in cem it takes `real_08927991` from 0.196 to 0.067.

**Joint re-splits of a shared juice.**

- A piece type whose upgrades share a juice can sit where each juice tail is optimal with the others fixed, so no single response, kick-and-polish or pair of order moves leaves it.
- Emptying or re-pricing the piece type's plans at once, then responses in turn, does. On `single_3375` the tails of N0.10–N0.17 shorten together; on `single_0085`, N5.10–N5.15. The rebuild kick does this (`lns` report 5).

**Chest amounts.** A fixed step grid (halves in `solve`, thirds in `price`) misses lone-chest optima. `solve` finds those only through its random-amount moves in the annealer: without them (`chest_move_rate=0`) it loses real_test -0.079 (up to -1.0 a payload), `speed_test` -0.022 (`single_1208` -3.1) and income -0.0044.

**Starts.** Started from `solve`'s finished plan, `price` ties the kick loop, and the two split the order basins (the loop `chain_0017:3`, `price` the rest). From its own start it is behind, so the kick loop is worth having and its start is not. Memetic's start fails the same way: every payload it loses is fixed by starting from `solve`'s endpoint (advanced plans with normal ones, `real_08927991`; chest amounts off its grid, `single_1104`; `chain_0001`, `week_hone_0004`, `week_hone_0012`). lns's start builds the free tap order place by place on empty plans, and putting a higher level ahead of a lower one of its piece drops the lower one from the free taps (`single_0071`, `single_3300`, `single_0087`, `chain_0017:0/1`, the same on every seed). Memetic's polish on its endpoint takes all of these but `single_0071`.

## Where `solve` stands

**Weak:** free tap order basins on some multi-upgrade payloads (mostly fixed by the kick loop; the kick loop and `mem_polish` together take them all), joint moves of several upgrades' plans (the surrogate loop, memetic's generations from `solve`'s endpoint and cem's restarts after it reach most of these; the rebuild kick reaches the known ones), and chest amounts off its grid.

**Strong:** it is hard to beat at matched CPU anywhere else. `real_test` hit is 96.8% and income is within 0.001.

## Open

**Joint moves of several upgrades' plans.** `single_3375`, `single_3397_serca_n4` and `b10_week_inc_0004` step 3 are reached by `price`'s kick loop with its own polish: responses in turn over several rounds, with order and chest moves between.

- In their end states several upgrades' plans move together: juice tails shortened across a piece type, or one upgrade moved from past the reach to the front.
- Under the surrogate loop, `single_3397_serca_n4` is 0.002, `single_3375` 0.005 and `b10_week_inc_0004:3` 0.008 at 10 seeds (`solve` 0.042, 0.045, 0.039). A re-plan with order moves in it reaches them, with or without the surrogate.
- Under memetic's generations from `solve`'s endpoint, `b10_week_inc_0004:3` is 0, `single_3397` 0.003, `single_3375` 0–0.019 and `chain_1051` 0, on 3 seeds.
- cem's restarts after `solve` reach `b10_week_inc_0004:3` (0) and `single_3375` (0.009) at effort 0.5, on 3 seeds.
- The rebuild kick added to the kick loop with `mem_polish` takes `b10_week_inc_0004:3`, `single_0085` and `single_3375` to 0 and `single_3397` to 0.011, on 3 seeds. The surrogate loop alone takes `chain_1051` and `single_3116`, which it misses.
- Open: the kick loop with `mem_polish` and the rebuild kick, together with the surrogate loop, as one run; which half of the rebuild kick (emptied or re-priced plans) does the work.
- `single_3300` was on this list by mistake. Its gap of 0.071 is one seed in three at 0.213; `solve` hits it on 9 of 10 seeds. `b10_single_3300` is still hard (4 of 10).

**cem's `chain_0017:0/1` basin.** Every cem variant ends at 0.2–0.75 there, against `solve`'s 0.02–0.03, with another free tap order and plainer plans. Neither the uncapped polish nor the exchanges leave it.

**How the free tap cut steers the search.** The new cut takes `single_1033_serca_n21` under `lns` from 0 of 10 hits to 10 of 10 and `b10_chain_0017:2` under `solve` from 0 to 6. That is far over its own error, and it is not the reach of order moves. What is cut is the low end of the outcomes, so it may be neutral moves near the front of the order.

## Noise

- `real_27a2bc40` swings ±0.2 between seeds, and most of the kick loop's real_test gain is this payload.
- `chain_1051` ends each seed at either 0 or 0.190.
- `single_1208` and `single_1000` swing between 0.05 and 0.5 in `price`.
- `single_1056_serca_n3` ends each seed at either 0 or 0.936 in `surrogate`.
- `single_0071_serca_n19` under `price` sticks at 1.2 on 1 to 3 seeds in 10, at either cut.
- `chain_0017` and `b10_single_3300` swing by 0.1 to 0.2 under any change, in every algorithm.
- `single_0159` swings from 0.01 to 0.59 between variants under memetic, and `large`'s rows follow it.
- `free_taps` `single_0087` ends each seed at 0 or 0.2–0.6 under cem's ablations.
