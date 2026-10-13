# Optimizer

Lives in `crates/core/src/optimizer`. The module header there and `docs/Optimizer.md` describe the algorithm; both warn that details change often, so this note sticks to what stays true.

## What it does

A single-chain simulated annealer over a `StateBundle`, maximising the average-gold metric (see `Evaluation pipeline.md`). It runs for a fixed number of evaluations, not a time budget, and returns the best state seen. Its constants are fields of `Tuning`.

Decision variables:

- each upgrade's `State` (which juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`;
- how each chest stack is opened, `chest_states`. It starts as given (the payload's, or everything on the first option) and travels with the state everywhere `special_state` does. See "Chests" below.

Things worth knowing about its shape:

- The metric has no natural scale, so there is no fixed temperature schedule. An adaptive scaler adjusts acceptance to track a target acceptance rate over the run.
- It restarts from the best known state very often.
- On restart it may copy one upgrade's state onto an identical upgrade, since identical upgrades often want identical plans. Identical means normal honing too: an advanced block shares its index with a low normal level, and its state is not a normal state.
- In the browser it periodically posts the best state so far as progress.

## Rules a neighbour move must respect

Evaluation assumes these; breaking them corrupts results without an error.

- `special_state` stays a permutation of upgrade indices. Never reorder the upgrade array.
- A state's length never changes.
- A state may be any plan: the pass of responses at the end does not keep to streaks. Reading its streaks (`compute_streak_info`) then gives the nearest streak state, which is what a perturbation goes on from.
- A chest state has one entry per option, none negative, summing to the chest's count. Change it through `set_chest_state`, or call `apply_chest_state` after, or the bands are stale.
- A book id used on an upgrade must be one that exists for that piece type and level. Id 0 means none.
- Advanced-honing strategy indices stay within their maximum.
- Update the state's hash after changing it.
- For normal honing, taps the player has already done are fixed and must not be modified.
- Normal-honing states are kept in a restricted form: for each juice pool, one streak from the first tap and one streak from the last. This is an empirical simplification, not a proven one, and the perturbation code depends on it. Measured in October 2026 (`State space.md`): the best plan is a streak plan on 82 of 90 small cases, and the rest lose 0.24% of flow at most.

## Chests

Two things change a chest state, and neither draws from the rng or touches the chain where no stack has a choice, so a payload without one runs as it did before chests:

- **A chest move** (`chest_move`, `Tuning::chest_move_rate` of the moves, 0.15): some chests of one stack go from one option to another, at most all of the option's early in the run and one at the end. It is a move like any other: evaluated, then accepted or rejected.
- **A polish** (`chest_polish`, every `Tuning::chest_polish_every` evaluations, 6000, and once at the start): from the best state, with the upgrades as they are, every pair of options of every stack is tried in steps that halve, for as long as anything helps. Its evaluations count against `max_iters`, and the chain goes on from the result.

What the arena said (October 2026, the `chests` set, 453 payloads, 3 seeds, sessions 28 to 32; `Arena.md` has the set):

- **Opening the chests well is the easy part.** Against the random state of before, any way of moving them takes the hit rate from 28% to about 80%, which is where the optimizer is without chests. On the worst payloads every variant ends with the same chests or nearly, and what is left of the gap is in the upgrades and the free tap order.
- **Rates of 0.05 to 0.3 cannot be told apart.** The polish on top takes a little off the tail: mean gap 0.08% of flow against 0.16%, and it wins the comparison payload by payload against every variant without it by a few payloads in 453.
- **Tried and taken out, for no measurable gain**: a rate that fades while chest moves find no new best (as `special_affinity` does), a polish at the start only, and a chest move with an ordinary move in the same step (8 to 16% more time).
- **What it costs**: 4.5% of CPU time on the `pilot` set at one job (session 33), all of it on payloads with a choice.
- **Still stuck, 11 payloads in 453 by more than 0.5% of flow, and four times the evaluations do not free them** (session 30). The ones looked at are of two kinds. A stack offers a book or scroll that no upgrade uses yet: the chests are worth nothing there until a state uses it, and the state is worse until the chests are there (`single_1178_t4_n4`). Or a large free tap budget and a stack that adds to it (`single_1231_t4_n24`), where runs with the same chests differ by the free tap order. Neither is fixed.

A move that goes on after a chest move must compute the free tap probabilities first: `neighbour` reads them for the current budget, and a chest move may have made a budget that no evaluation has seen.

## Responses

`optimizer/response.rs` works out the best plan of one normal upgrade with the rest of the state as it is, over every plan and not only the streaks (`State space.md` has why that is possible). Three things can use it, each a `Tuning` field:

- **`response_polish`, on, 3**: at the end, from the best state, each normal upgrade in turn takes its best plan where the metric agrees, for up to 3 passes. A solve can therefore return a state that is not a streak state.
- **`response_every`, off**: the same single pass every so many evaluations, the chain going on from the result.
- **`response_rate`, off**: the share of normal honing moves that are a response of one upgrade in place of a random change of its streaks (`respond`). An upgrade whose state and whose rest of the state are as they were when it last responded is not asked again; where none is left the move is a random one. `response_kick` changes another upgrade at random first.

A response is charged what it took, in evaluations: its table evaluations, and the steps of its recursion at `response_steps` to one. That is about its time, within a factor of two either way.

What the arena said (October 2026, sessions 42 to 46, the gap as a share of flow, hit within 1e-4):

| set | before | `response_polish=3` |
| --- | --- | --- |
| `explore` singles 100 to 199, 3 seeds | hit 79.7%, 90th percentile 0.056% | 90.3%, 0.009% |
| `chests`, 1 seed | 79.5%, 0.055% | 85.0%, 0.032% |
| `chains`, 1 seed | 68.8%, 0.093% | 81.2%, 0.065% |

- **The pass at the end is nearly all of the gain**, for 1 to 2% more time.
- **`response_every` at 6000 or 2000 and `response_rate` at 0.03 are not told apart from it** (90.7, 90.7 and 90.3% on the singles). The rate at 0.03 takes 16% less time, because what a response is charged is on the safe side.
- **More responses are worse**: a rate of 0.1 gave 87.7% on those singles, and on 495 other singles a rate of 0.3 gave 80% with a mean gap four times the baseline's and a worst of 25%. A response costs 15 to 100 evaluations, and those are taken from the free tap order, the advanced upgrades and the chests.
- **Every normal move a response** (rate 1) was the worst on `pilot`: mean gap 1.6% against 0.14%.
- **What is still missed is not only the order**: with the pass at the end, singles with neither free taps nor advanced honing are hit 88% of the time. Those are states no single upgrade can leave.

Inside a response:

- **One id a pool.** A plan that mixes two book ids of a pool is not looked for.
- **`response_joint`**: an upgrade with two pools is searched over both at once while that is at most this many steps, and pool by pool from its plan past that (`Response::improve`). Always pool by pool gave the same results in the same time on the singles.
- **`response_points`, 16**: where the lattice does not answer, what an amount of a material is worth is taken at 16 amounts and joined by lines.
- The recursion's table is kept between responses in a thread local, stamped, because most of it is never reached and clearing it was most of a response.

## Streak length

The game can do ten taps at once, so a plan may be asked to change what it takes only every so many taps. The payload's `min_resolution` is that number (`StateBundle::block`; `Tuning::streak_length` overrides it, which is how the arena varies it). The frontend still sends 1.

- **The rule**: from an upgrade's next tap on, the taps go in blocks of that many, and every tap of a block takes the same. So every streak, of taking or of not taking, is a multiple of it but the last before the pity, which the pity cuts.
- **A random move** rounds its front streak down to a block and starts its back streak where a block does, and changes a streak by two blocks at least, or the rounding would undo it.
- **A response** chooses once a block and plays the block through, so it is the best of the plans that keep the rule, and its table is only kept where a block starts. `Model::filled` gives a plan cut at its pity its whole last block back: a pool left alone while another moves the pity out would otherwise stop inside a block.
- **A state that is given off the grid stays so** until a move or a response replaces it.

What it was measured to give up (October 2026):

- **The search is not what is lost.** On the 90 small cases, with every plan that keeps the rule tried, the best of 4 solves is the best there is at 5 and at 10, and misses once at 2, by 0.02%, on a plan that mixes two book ids. Every plan returned over 100 `explore` singles kept the rule (session 48).
- **The rule itself costs**, against the best known without it, on 100 `explore` singles and 3 seeds (sessions 47 and 48):

| streak length | hit | median gap | 90th percentile | mean | worst |
| --- | --- | --- | --- | --- | --- |
| 1 | 90.3% | 0 | 0.009% | 0.026% | 1.5% |
| 2 | 62.3% | 0.001% | 0.30% | 0.11% | 2.4% |
| 5 | 54.0% | 0.008% | 0.71% | 0.19% | 3.3% |
| 10 | 45.0% | 0.014% | 0.85% | 0.32% | 7.6% |

- **The cost is in what is owned**: at 10, payloads that own no juice lose 0.07% on average and 0.37% at most; those that own some lose 0.3 to 0.5% on average. An owned amount that is not a whole number of blocks is either left over or topped up at the buy price. The worst are one or two upgrades with a few taps left.
- It is a fifth to a quarter faster at 10, there being fewer plans.

## What a move costs

Outside the evaluator an iteration is some 5 microseconds, a third of what it was. What was taken out, none of which changes a result (the `pilot` runs of session 25 have the metrics of session 21):

- **Accepting or rejecting copies every upgrade's state**, changed or not. `State::copy_from` does it without allocating and leaves a payload with the same hash alone.
- **A perturbed state was hashed three times**, and `apply_new_streak_info` built a hash set of a few ids on every call.
- **A restart rehashed every upgrade**; `clone_from_essence` now skips a payload that is already the one wanted.
- **The state hash is `ahash` with fixed seeds**, in place of SipHash. The seeds are fixed because a seed has to repeat.

Of what is left, two thirds is `neighbour` (two `WeightedIndex` built per move, and the streaks written back over the whole payload) and a quarter the copy on accepting. Changing the first without changing what the rng is asked for takes care.

**A restart does not bring the streaks back.** `clone_from_essence` restores a payload and leaves its `streak_cache`, and `perturb_normal` builds the next payload from that cache. So the first move of a normal upgrade after a restart goes on from where the chain was, not from the best state. Left as it is: clearing the cache there changes every search, and needs a run of the arena to say whether for the better.

## Chained runs

`solve` takes an optional `Previous`: the payload of the last run, its best `StateBundle`, and a blob the optimizer left for itself (returned in `Solved`, capped at `Tuning::max_blob_bytes`). Nothing reads it yet. The frontend does not pass one either; it still seeds a run through the payload's `state` and `special_state`.

## Testing a change

Use the arena (`Arena.md`): snapshot the variants, run them against each other, and read the gap, hit rate and counters. All randomness must come from the solver's own `rng` so that a seed repeats. `scramble` exposes the neighbour moves for making random valid states.
