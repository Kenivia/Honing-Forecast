# State space: what was measured

The measurements behind `State space.md`, October 2026. Only read this when trying something new on the shape of a plan. `Arena.md`, "The state space", has the commands. Nothing in the optimizer was changed by the first round; the response came out of it.

## The landscape

`pnpm arena landscape` and `arena_data/landscape_plot.py`. On `chain_0011_t4_n7` (7 upgrades of 76 to 219 taps, juice only), front streak against back streak:

- 20,661 interior peaks, all of them at the first length of a new pity. Between two of them each juice more costs a steady few hundred gold.
- The peaks form one top on the upgrades of up to 112 taps; on the 219 tap ones several.
- With books, or juice owned for part of the taps (the `lone` cases), a quarter to all of the peaks are at a pity, the rest where an owned amount runs out.
- Against 100 times the lattice work the default tuning is off by 13 to 2300 gold (sd) over a grid, but by 1 to 130 from one cell to the next, and the best cell is the same on 6 upgrades of 7.

## Is the streak restriction sound

`pnpm arena tiny` and `pnpm arena exhaust`: 60 cases of one to three upgrades, far enough in that every plan can be tried (1 to 3 million states each, the exact metric, every free tap order), and 30 lone upgrades with 20 taps left. Juices and books both.

- **82 of 90 have their best plan among the streaks.** The other 8 lose 0.0001% to 0.24% of flow, 5 of them under 0.004%. The front streak that skips a tap is the 0.24%.
- **The annealer of then found the best streak plan**: the best of 4 seeds is it on all 90, the worst of 4 misses on 12 by up to 1.6%.

On 94 `explore` singles, starting from the best known state (8 seeds) and letting each upgrade in turn take its best plan (`pnpm arena ascend`):

- of the streaks only: better on 18, by 0.002% of flow on average and 0.07% at most. This is the annealer's slack.
- of every plan: better on 22, by 0.005% on average and 0.15% at most. Better than the streaks on 14, by 0.10% at most. This is what the restriction costs.

## The response, as first written

- It was exact within a gold where the lattice answers, and off by sd 170 to 1200 gold on the 219 tap upgrades above.
- **From the best streak plan it reached the true best on all 90 small cases.**
- **From nothing, with the free tap order and the advanced upgrades of the best known state**, it matched or beat the best known on 82 of 94 `explore` singles and lost up to 0.18% of flow on the rest, in 2.7 responses an upgrade.
- On the small cases, from nothing and the given order, 11 of its 13 misses (up to 14%) were the order.
- It was slow, 0.01 to 0.6 seconds a response: every table entry was its own evaluation of a material. The core's version takes one pass over the sum of the other upgrades for a whole table.

Later in October 2026 the response learned to mix two ids of a pool, and `State::compute_streak_info` was found to read the back streak from the wrong end (`Optimizer tried.md`). The `ascend` numbers above are from before both.

## Why a block in the middle

Two lone upgrades whose best plan is not a streak plan (`lone_022`, `lone_029`), worked out exactly with `lone_exact` (`Arena.md`):

- In `lone_022` Red runs out on tap 8 and the best two juices are on taps 8 and 9; sliding them to the front costs 5,140 gold, to the back 138.
- `lone_029`'s hole at its second tap is a tie (0.1 gold); its plan is really a block on taps 3 to 11.
- Evaluator and `monte_carlo::exact_single` agree to the third decimal on both, so those plans are real, not evaluator error.
