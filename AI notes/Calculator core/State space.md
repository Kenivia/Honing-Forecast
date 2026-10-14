# State space of normal honing

What was measured in October 2026 about the shape of the metric over an upgrade's state and about the streak restriction (`Optimizer.md`, "Rules a neighbour move must respect"). The tools are in `crates/arena/src/space.rs`; `Arena.md` has the commands. Nothing in the optimizer was changed by this.

## What a plan is

- **A state has more slots than taps that matter.** Its length is the pity of the upgrade with nothing used. Anything used shortens the pity, so the slots from the pity tap on are never reached, and the pity tap itself takes nothing (the game does not allow it). Many states are the same plan. `Model::canonical` empties those slots; compare plans only after it.
- **A back streak is anchored to the last slot**, so it has to cross the dead slots before it does anything, and it always runs up to the tap before the pity.
- **The pity tap is known without an evaluation.** Artisan after n taps is the start plus `ARTISAN_MULTIPLIER * rate` times the sum of the chances, so the pity depends on how many taps took each id before it and not on which. A juice usually adds the base chance and a tap past the tenth is worth twice that, hence a tap of pity for every two juices. `Model::pity` is this.

## The landscape

`pnpm arena landscape` and `scripts/arena/landscape_plot.py`. On `chain_0011_t4_n7` (7 upgrades of 76 to 219 taps, juice only), front streak against back streak:

- **Along a streak's length the metric is a sawtooth, and every peak is the first length of a new pity**: 20,661 interior peaks, all of them there. Between two of them each juice more costs a steady few hundred gold; at one the metric jumps.
- **The peaks alone form a smooth curve**, with one top on the upgrades of up to 112 taps. On the 219 tap ones it has several, by amounts the size of the evaluator's own error there.
- **What is owned makes kinks too.** With books, or juice owned for part of the taps (the `lone` cases), a quarter to all of the peaks are at a pity, the rest where an owned amount runs out.
- **The default tuning's error is smooth.** Against 100 times the lattice work it is off by 13 to 2300 gold (sd) over a grid, but by 1 to 130 from one cell to the next, and the best cell is the same on 6 upgrades of 7.

## Is the streak restriction sound

`pnpm arena tiny` and `pnpm arena exhaust`: 60 cases of one to three upgrades, far enough in that every plan can be tried (1 to 3 million states each, the exact metric, every free tap order), and 30 lone upgrades with 20 taps left. Juices and books both.

- **82 of 90 have their best plan among the streaks.** The other 8 lose 0.0001% to 0.24% of flow, 5 of them under 0.004%.
- What the streaks miss:
  - a block in the middle that stops before the last taps (`......00..`);
  - a front streak that skips a tap (`.0.00000000...`, the 0.24%);
  - two book ids mixed in one pool (`01101111`).
- **The annealer finds the best streak plan**: the best of 4 seeds is it on all 90, the worst of 4 misses on 12 by up to 1.6%.

On 94 `explore` singles, starting from the best known state (8 seeds) and letting each upgrade in turn take its best plan (`pnpm arena ascend`):

- of the streaks only: better on 18, by 0.002% of flow on average and 0.07% at most. This is the annealer's slack.
- of every plan: better on 22, by 0.005% on average and 0.15% at most. Better than the streaks on 14, by 0.10% at most. This is what the restriction costs.

So the restriction is nearly free, and what it gives up is the same size as what the annealer leaves inside it.

## The best plan of one upgrade

The metric is a sum over materials, and the upgrades are independent, so with the rest of the state fixed it is an average over the tap this upgrade succeeds on of what that outcome is worth. The free tap probabilities do not depend on the plans. So:

- **What an amount of a material is worth is a table**, by pinning the upgrade's support to the amount (`Support::pin`) and taking that material's part of the metric: one entry per tap for the base materials, one per use for each id (`Response::of`).
- **A plan's value is then a sum over its taps** (`Response::score`), and the best of every plan there is, streak or not, comes from the last tap backwards, over the tap and how many times each id was used (`Response::best`). No enumeration, at any length.
- It is exact where the evaluator is: within a gold of it where the lattice answers, and off by the saddlepoint's error where it does not (sd 170 to 1200 gold on the 219 tap upgrades above).

Each upgrade taking its best plan in turn (`ascend`):

- **From the best streak plan it reached the true best on all 90 small cases.**
- **From nothing, with the free tap order and the advanced upgrades of the best known state**, it matched or beat the best known on 82 of 94 `explore` singles and lost up to 0.18% of flow on the rest, in 2.7 responses an upgrade. It stops at states no single upgrade can leave.
- **It does not choose the free tap order.** On the small cases, from nothing and the given order, 11 of its 13 misses (up to 14%) were the order.
- As written it is slow, 0.01 to 0.6 seconds a response: every table entry is its own evaluation of a material. One pass over the sum of the other upgrades would give a whole table.

## Since

The response is in the core now (`optimizer/response.rs`), with one evaluation pass per table in place of one per entry, and the optimizer runs a pass of it at the end. `Optimizer.md`, "Responses", has what was tried and what it gave. The arena's `exhaust`, `ascend` and `landscape` use the core's.

Later in October 2026 the response learned to mix two ids of a pool, which was the largest single gain of that round, and `State::compute_streak_info` was found to read the back streak from the wrong end (`Optimizer.md`). The `ascend` numbers above are from before both.

## Not done

- Moves that go from one pity to the next, and a state without the dead slots.
- Chests with a choice and advanced honing were left as given throughout.
