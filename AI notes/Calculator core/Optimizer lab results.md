# Optimizer lab: results

What the lab has found, by topic. How the lab works is in `Optimizer lab.md`; what not to try again is in `Optimizer tried.md`. Numbers are in % of flow against `solve` 48k; evidence is in each investigation's `report.md`, under `arena_data/lab/`.

**Baselines** in `runs.sqlite`:

- `lab_r1`: `solve` 48k.
- `lab_r1:max_iters=96000`: `solve` at 96k.
- `i2g:kick_stall=32,kick_types=windowq+chest+adv+piece`: the kick loop. For new runs `i2i:kick_stall=32` is the same behaviour, but it has no stored runs.

## Prototypes

| Prototype | Verdict | Evidence |
|---|---|---|
| `price` | Take pieces, not the algorithm. Its kick loop is its strength and its own start its weakness. | `iteration2/price/report.md` |
| `surrogate`, `memetic`, `cem`, `lns` | Not reviewed yet | |

## Pieces, and where they stand

| Piece | Status | Gain | Cost | Evidence |
|---|---|---|---|---|
| `response_points` 128 | **In production** | Up to 0.96 on lone upgrades; `small` +0.007 | None measurable | `price` report 1 |
| Kick loop, `kick_stall=32`, all four kick types | Candidate; code in `iteration2/price/final.patch`, off | quick_dev +0.0023 (hit 94.7 → 97.1), `large` +0.012, real_test +0.006, `speed_test` +0.0007, income 0; no payload lost | x1.2–2.3 CPU | `price` report 8–10 |
| Chest amounts from the grid plus 4 random draws | Candidate; `price` `chestboth` only, not in `solve` yet | Fixes `single_1104_serca_n1` and `single_1166_serca_n1` | None measurable | `price` report 7–8 |

Against `solve` at 96k the kick loop is ahead (`large` +0.0073, quick_dev +0.0011, real_test +0.0016) at x0.8–1.3 the CPU.

## What makes a result good

**Free tap order.**

- Insertion of a normal upgrade within a free tap's reach is what makes an order good: without those moves `price` loses 0.14–0.47. `solve`'s `order_polish` already has them.
- `solve` misses some order basins at any budget, 384k included. The kick loop reaches most of them.

**Kicks.**

- A kick reaches a new basin only if what it moved is re-planned before a greedy repair runs. Otherwise `order_polish` puts a shuffled window straight back.
- It must also move something that matters: a window starting past the free taps' reach changes nothing.
- The moved upgrades must be read before the evaluation, which cleans the order (`clean_special_state`). One build read them after and lost every `chain_0017` gain.
- Each set has its own kick: adv carries `large`, chest carries real_test and `speed_test`, and window carries the order basins. All four together match the best single kick on each set. The piece kick does nothing alone, but `large` and real_test are worse without it in the mix.
- Stall length: 8 gets about half of 32, and 96 adds little to 32 for twice the CPU.
- Kicks add to the annealing and do not replace it: halving the annealing to pay for them loses at matched CPU.

**Chest amounts.** A fixed step grid (halves in `solve`, thirds in `price`) misses lone-chest optima. `solve` finds those only through its random-amount moves in the annealer.

**Starts.** Started from `solve`'s finished plan, `price` ties the kick loop. From its own start it is behind, so the kick loop is worth having and its start is not.

## Where `solve` stands

**Weak:** free tap order basins on some multi-upgrade payloads (mostly fixed by the kick loop), joint moves of several upgrades' plans, and chest amounts off its grid.

**Strong:** it is hard to beat at matched CPU anywhere else. `real_test` hit is 96.8% and income is within 0.001.

## Open

**Joint moves of several upgrades' plans.** `single_3300`, `single_3375`, `single_3397_serca_n4` and `week_inc_0004` step 3 are reached only by `price`'s kick loop with its own polish: responses in turn over several rounds, with order and chest moves between.

- In their end states several upgrades' plans move together: juice tails shortened across a piece type, or one upgrade moved from past the reach to the front.
- Re-planning once after a kick is not enough, and neither is any `solve` kick so far.

## Noise

- `real_27a2bc40` swings ±0.2 between seeds, and most of the kick loop's real_test gain is this payload.
- `chain_1051` ends each seed at either 0 or 0.190.
- `single_1208` and `single_1000` swing between 0.05 and 0.5 in `price`.
