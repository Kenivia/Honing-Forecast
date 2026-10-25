# State space of normal honing

The shape of the metric over one upgrade's state, the streak restriction (`Optimizer.md`, "Rules a neighbour move must respect"), and how the exact best plan of one upgrade is found. The arena tools that measure it are in `crates/arena/src/space.rs` (`Arena.md`, "The state space"). What not to repeat is in `State space tried.md`; read it only when trying something new here.

## What a plan is

- **A state has more slots than taps that matter.** Its length is the pity of the upgrade with nothing used. Anything used shortens the pity, so the slots from the pity tap on are never reached, and the pity tap itself takes nothing (the game does not allow it), unless the chance already reached 1 before it. Many states are the same plan. `Model::canonical` empties those slots; compare plans only after it.
- **Taps can come in blocks** that must take the same (`StateBundle::block`, `Model::block`); a plan is filled to whole blocks (`Model::filled`).
- **A back streak is anchored to the last slot**, so it has to cross the dead slots before it does anything, and it always runs up to the tap before the pity.
- **The pity tap is known without an evaluation.** Artisan after n taps is the start plus `ARTISAN_MULTIPLIER * rate` times the sum of the chances, so the pity depends on how many taps took each id before it and not on which. A juice usually adds the base chance and a tap past the tenth is worth twice that, hence a tap of pity for every two juices. `Model::pity` is this.

## The landscape

- **Along a streak's length the metric is a sawtooth, and every peak is the first length of a new pity.** Between two of them each juice more costs a steady amount; at one the metric jumps.
- **The peaks alone form a smooth curve**, with one top on short upgrades and possibly several, by amounts the size of the evaluator's own error, on long ones.
- **What is owned makes kinks too**: peaks also sit where an owned amount runs out.
- **The default tuning's error is smooth** from one cell to the next, so it rarely moves which plan is best.

## The streak restriction

Each juice pool is one streak from the first tap and one from the last. The best plan is a streak plan on most cases, and what the streaks miss is small:

- a block in the middle that stops before the last taps (`......00..`);
- a front streak that skips a tap (`.0.00000000...`);
- two book ids mixed in one pool (`01101111`).

What the restriction gives up is the same size as what the annealer leaves inside it, and the responses (`Optimizer.md`) reach the plans outside it.

**Why a block in the middle.** Moving juice one tap later saves juice and costs base materials: later taps are reached less often, so the juice is bought on fewer runs, but success comes later, so more taps are paid for. The best place is where the two cross. Owned stock that is worthless when left over makes the base material side step: while it covers a material, succeeding a tap later costs nothing in it, and once it runs out every tap costs full price, so the crossing lands on the tap where the stock runs out, which can be mid-run. Without owned stock the front is where juice pays most, so a front streak, plus a back streak for the juices that pull the pity in, is what one expects there (reasoned, not measured).

## The best plan of one upgrade

The metric is a sum over materials, and the upgrades are independent, so with the rest of the state fixed it is an average over the tap this upgrade succeeds on of what that outcome is worth. The free tap probabilities do not depend on the plans. So:

- **What an amount of a material is worth is a table**, by pinning the upgrade's support to the amount (`Support::pin`) and taking that material's part of the metric: one entry per tap for the base materials, one per use for each id (`Response::of`).
- **A plan's value is then a sum over its taps** (`Response::score`), and the best of every plan there is, streak or not, comes from the last tap backwards, over the tap and how many times each id was used (`Response::best`). No enumeration, at any length.
- It is exact where the evaluator is: within a gold of it where the lattice answers, and off by the approximation's error where it does not.
- It does not choose the free tap order, and each upgrade taking its best plan in turn stops at states no single upgrade can leave.

This is the core's `optimizer/response.rs`, with one evaluation pass per table, and the optimizer runs a pass of it at the end (`Optimizer.md`, "Responses"). The arena's `exhaust`, `ascend` and `landscape` use it.

## Not done

- Moves that go from one pity to the next, and a state without the dead slots.
- Chests with a choice and advanced honing were left as given throughout the measurements.
