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
| `memetic`, `cem`, `lns` | Not reviewed yet | |

## Pieces, and where they stand

| Piece | Status | Gain | Cost | Evidence |
|---|---|---|---|---|
| `response_points` 128 | **In production** | Up to 0.96 on lone upgrades; `small` +0.007 | None measurable | `price` report 1 |
| Kick loop, `kick_stall=32`, all four kick types | Candidate; in tree, off (`kick_stall` 0) | real_test +0.0056 (hit 96.6 → 97.6), quick_dev +0.0016 (11 wins, 0 losses; 95.5 → 97.8), `speed_test` +0.0006, `large` +0.0077 (53 → 70), income +0.0001; no payload lost | x1.4–2.7 CPU | `price` report 8–10 |
| Surrogate loop after `solve`: `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach` | Candidate; in tree, off (`sur_stall` 0) | Against `cut_a`: real_test +0.0064 (hit 96.6 → 97.9), quick_dev +0.0018 (95.5 → 97.3), `speed_test` +0.0016, `large` +0.0066, advanced, free_taps and juice to 100, income +0.0002; no payload lost | x2.0–3.3 CPU | `surrogate` report 5–11 |
| Screened polish in that loop: `sur_screen=true` | Candidate with it | Same gain within 0.01 on every payload (10-seed gap sum 0.288 against 0.276) | 13–24% less CPU than the loop | `surrogate` report 10 |
| Chest amounts from the grid plus 4 random draws | Candidate; `price` `chestboth` only, not in `solve` yet | Fixes `single_1104_serca_n1` and `single_1166_serca_n1` | None measurable | `price` report 7–8 |

Against `solve` at 96k in the same batch the kick loop is ahead or level everywhere: real_test +0.0016, quick_dev +0.0008 (8 wins, 1 loss), `large` +0.0020 (1, 1), at x0.86–1.49 the CPU. At the old cut its `large` lead was +0.0073.

## What makes a result good

**Free tap order.**

- Insertion of a normal upgrade within a free tap's reach is what makes an order good: without those moves `price` loses 0.13–0.48. `solve`'s `order_polish` already has them.
- `solve` misses some order basins at any budget, 384k included. The kick loop reaches most of them.

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

**Chest amounts.** A fixed step grid (halves in `solve`, thirds in `price`) misses lone-chest optima. `solve` finds those only through its random-amount moves in the annealer: without them (`chest_move_rate=0`) it loses real_test -0.079 (up to -1.0 a payload), `speed_test` -0.022 (`single_1208` -3.1) and income -0.0044.

**Starts.** Started from `solve`'s finished plan, `price` ties the kick loop, and the two split the order basins (the loop `chain_0017:3`, `price` the rest). From its own start it is behind, so the kick loop is worth having and its start is not.

## Where `solve` stands

**Weak:** free tap order basins on some multi-upgrade payloads (mostly fixed by the kick loop), joint moves of several upgrades' plans (the surrogate loop reaches most of these), and chest amounts off its grid.

**Strong:** it is hard to beat at matched CPU anywhere else. `real_test` hit is 96.8% and income is within 0.001.

## Open

**Joint moves of several upgrades' plans.** `single_3375`, `single_3397_serca_n4` and `b10_week_inc_0004` step 3 are reached by `price`'s kick loop with its own polish: responses in turn over several rounds, with order and chest moves between.

- In their end states several upgrades' plans move together: juice tails shortened across a piece type, or one upgrade moved from past the reach to the front.
- Under the surrogate loop, `single_3397_serca_n4` is 0.002, `single_3375` 0.005 and `b10_week_inc_0004:3` 0.008 at 10 seeds (`solve` 0.042, 0.045, 0.039). A re-plan with order moves in it reaches them, with or without the surrogate.
- Open: the kick loop and the surrogate loop together.
- `single_3300` was on this list by mistake. Its gap of 0.071 is one seed in three at 0.213; `solve` hits it on 9 of 10 seeds. `b10_single_3300` is still hard (4 of 10).

**How the free tap cut steers the search.** The new cut takes `single_1033_serca_n21` under `lns` from 0 of 10 hits to 10 of 10 and `b10_chain_0017:2` under `solve` from 0 to 6. That is far over its own error, and it is not the reach of order moves. What is cut is the low end of the outcomes, so it may be neutral moves near the front of the order.

## Noise

- `real_27a2bc40` swings ±0.2 between seeds, and most of the kick loop's real_test gain is this payload.
- `chain_1051` ends each seed at either 0 or 0.190.
- `single_1208` and `single_1000` swing between 0.05 and 0.5 in `price`.
- `single_1056_serca_n3` ends each seed at either 0 or 0.936 in `surrogate`.
- `single_0071_serca_n19` under `price` sticks at 1.2 on 1 to 3 seeds in 10, at either cut.
- `chain_0017` and `b10_single_3300` swing by 0.1 to 0.2 under any change, in every algorithm.
