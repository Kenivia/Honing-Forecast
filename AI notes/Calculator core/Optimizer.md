# Optimizer

Lives in `crates/core/src/optimizer`. The module header there and `docs/Optimizer.md` describe the algorithm; both warn that details change often, so this note sticks to what stays true.

## What it does

Several short chains of simulated annealing over a `StateBundle`, each polished, the best kept, maximising the average-gold metric (see `Evaluation pipeline.md`). It runs for at most a fixed number of evaluations, not a time budget, and returns the best state seen. Its constants are fields of `Tuning`. "The shape of a solve" below has the parts; the rest of this section is one chain.

Decision variables:

- each upgrade's `State` (which juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`;
- how each chest stack is opened, `chest_states`. It starts as given (the payload's, or everything on the first option) and travels with the state everywhere `special_state` does. See "Chests" below.

Things worth knowing about its shape:

- The metric has no natural scale, so there is no fixed temperature schedule. An adaptive scaler adjusts acceptance to track a target acceptance rate over the run.
- It restarts from the best known state very often.
- On restart it may copy one upgrade's state onto an identical upgrade, since identical upgrades often want identical plans. Identical means normal honing too: an advanced block shares its index with a low normal level, and its state is not a normal state.
- In the browser it periodically posts the best state so far as progress.

## The shape of a solve

`solve` in `optimizer/simulated_annealing.rs`. Everything here is a `Tuning` field, and the defaults are what is described.

1. **A starting free tap order** (`greedy_order`): the upgrades by what a free tap saves for what it takes, whole and one piece's next level at a time, where either beats the order given.
2. **Chains** (`anneal`): the evaluations (`max_iters`) are shared between several chains, each from the same start with its own random numbers, each ending in the polishes. How many is by size (`chains` 0): a chain gets `chain_iters_per_upgrade` evaluations an upgrade, no less than `chain_iters_min`, and there are never fewer than 4. One long chain settles into one of many plans that nothing near betters; more evaluations do not get it out, another chain lands elsewhere.
3. **The polishes** (`polish`, `polish_rounds` rounds while any of them moves the best state):
   - the free tap order (`order_polish`): each upgrade to each place a free tap can still reach;
   - the responses (`response_polish`, see "Responses");
   - the advanced upgrades (`adv_polish`): each entry of a strategy takes the best of its 19 values;
   - the chests (`chest_polish`).
4. **Leaving early**: once `agree_exit` chains, 5, have ended on the same metric; or, going on from a plan (below), once `carry_stall` chains in a row have not bettered it.
5. **Merge** (`merge`): the best chain's result takes from each other chain's, where the metric agrees, its free tap order, its plans for a whole piece type (they share juices and books), and its plan for each upgrade.
6. **Exchange** (`exchange`): uses of a juice or a book go from one upgrade to another that takes the same, 1, 2, 4 up to 64 at a time: the last taps that take it of the one, the first that do not of the other. Then the polishes again, and again while it moves anything. Not under a streak length above 1.

A chained run (`Previous`, see "Chained runs") puts the plan of the run before in front of all this.

What the arena said (October 2026; `Arena.md` has the sets and the baseline `v0`, which is all of the above off and one chain of 24,000). Hit is within 1e-4 of the best known as a share of flow, near within 1e-3, spread the best seed less the worst, time is CPU over `v0`'s:

| `nh_dev`, 158 payloads, 4 seeds | hit | near | mean gap | spread | time |
| --- | --- | --- | --- | --- | --- |
| `v0` | 78.5% | 90.1% | 0.055% | 0.18% | 1 |
| `v0` with 48,000 evaluations | 81.5% | 92.3% | 0.038% | 0.086% | 2.0 |
| the polishes at the end, 24,000 | 82.3% | | 0.035% | 0.038% | 1.0 |
| and the order's fixes (floor, reach, greedy start) | 82.3% | 92.1% | 0.028% | 0.026% | 1.0 |
| 48,000 in 4 chains | 88.9% | 97.2% | 0.010% | 0.011% | 2.2 |
| and mixed responses | 92.4% | 99.1% | 0.0045% | 0.0067% | 2.2 |
| and merge and exchange | 94.5% | 99.1% | 0.0032% | 0.0058% | 2.2 |
| chains by size, leaving at 3 that agree | 94.3% | 98.9% | 0.0035% | 0.0060% | 1.7 |
| and fresh streaks (`v1`) | 93.8% | 98.7% | 0.0037% | 0.0071% | 1.6 |
| leaving at 5 that agree, and no order moves without a free tap: the defaults, `v6` | 94.3% | 98.7% | 0.0034% | 0.0061% | 1.85 |

- **On the set kept out of it** (`nh_test`, 157 payloads, 8 seeds, sessions 73 and 94): 76.9% to 94.3%, near 88.6% to 99.3%, mean gap 0.060% to 0.0040%, spread 0.15% to 0.012%, payloads whose seeds all agree 64% to 88%, 1.9 times the time. No payload is worse than under `v0`, 50 are better.
- **By size** on `nh_test`: 4 to 7 upgrades 83% to 92% in 1.3 times the time, 8 to 15 83% to 97% in 1.5, 16 to 31 71% to 94% in 2.2, 32 and up 55% to 92% in 2.4. Small payloads leave early, large ones use all of it. The sets below were run with `v1`, which differs only in those last two changes.
- **With advanced honing** (`explore` singles 100 to 199, 3 seeds, session 74): 88.0% to 94.7%, near 95.7% to 98.7%, 1.6 times the time. One payload of 100 is worse, by 0.27% of flow: two upgrades, where the best plan takes an enhanced book on the first tap only, `v0` found it on one seed in three and the chains agree on the other plan.
- **With chests** (the `chests` set, 453 payloads, 1 seed, session 75): 83.4% to 92.5%, near 93.2% to 98.0%, 61 payloads better and 9 worse (0.21% at most), 1.9 times the time. The two stuck payloads of "Chests" below are among the better ones, by 1.3% and 8.6%.
- **At a streak length of 10** (`nh_quick`, 4 seeds, session 76): better than `v0` at the same length on 11 payloads of 40 and worse on none, spread 0.56% to 0.012%.
- **One job at a time** (`nh_quick`, 1 seed): 39.2 CPU seconds became 75.3 under `v6`, 1.9 times.
- **On the 90 small cases whose every plan was tried** (`exhaust`, `State space.md`): one solve reaches the best there is on all 90, where the worst of 4 solves of before missed 12. On the lone upgrade too large to try every plan of, it beats the best that was enumerated by 2.8%, with a plan that mixes two ids.
- **More evaluations in one chain buy little**, and two chains of 24,000 beat one of 48,000 on `v0`'s own runs (83% against 82%).
- **A chain wants some 375 evaluations an upgrade.** 4 to 7 upgrades hit most with 12 chains of 4,000, 32 and up with 4 of 12,000, and worst the other way round (71% against 79%). 250 and 500 an upgrade are not told apart from 375.
- **Leaving at 2 chains that agree costs a point of hit**, at 3 nothing that could be measured on `nh_dev`, for a quarter less time overall and two thirds less on small payloads. But on the weekly chains (`weekly_test`, 1 seed, session 92) payloads of 1 to 7 upgrades lost 2 points at 3: on some, chains agree on a plan that is not the best, reliably (a few weeks of `week_hone_0014` lost 0.3% to 1.3% of flow). 5, the default, gave back half of it (honing chains 96.2% to 97.3%, never leaving 98.2%) for a fifth more time there, and nothing on large payloads, which use all their chains anyway.
- **Moves of the free tap order are wasted while the budget cannot pay for one free tap**, and `special_floor` made a fifth of all moves such. Nothing about the order is tried then (`StateBundle::affords_a_free_tap`).
- **The free tap order could lock itself out**: a move of the order was drawn with the chance that the first upgrade is free tapped, which is nothing when that upgrade costs more than the budget. `special_floor` and the polish of the order end that. It was the worst case of `v0` (12% of flow).
- **What is still missed** is small and in few payloads: 39 runs in 632, 31 of them on 10 payloads, the worst 0.3% of flow. The hardest are one piece with many levels sharing a partial stock of juice, where every run ends on another split of it.

Tried and taken out, for no gain: going back to the best state without an evaluation (a fifth more time), longer between going back, one upgrade a move in place of two (73% against 80%: two is what moves juice between upgrades), a polish in which the upgrades that share a material agree on its price, trying a lost free tap order again with the plans of the upgrades it moves, the exchanges on every chain's result (a seventh more time), and 64,000 evaluations (94.8%, a fifth more time).

**The evaluator used to score the same id twice on a tap, and well**: the chance counted it twice, the cost once. A move that changed one pool of an upgrade and left the others made such states a few times in a thousand runs, and they looked like new bests. No other run of 191,000 in the database has one. The evaluator now counts it once.

**Why that move could make them: the crossover changed a state's length.** It copied a plan onto an upgrade of the same piece type and level whatever its progress, and one further along has fewer taps left, so its state took the other's length (on one payload the level 19 armour came out at 48, 57, 61 and 77 slots, under `v0` too). The streaks kept between moves then overran the new length, which every other move clips and that one did not. Now the crossover copies only between upgrades with the same artisan and taps done, so a state's length never changes (`v7`: as `v6` on `nh_dev` and `nh_test`).

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
- **A polish** (`chest_polish`, every `Tuning::chest_polish_every` evaluations of a chain, 6000, once at its start and with the other polishes at its end): from the best state, with the upgrades as they are, every pair of options of every stack is tried in steps that halve, for as long as anything helps. Its evaluations count against `max_iters`, and the chain goes on from the result.

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

- **One id a pool, then two** (`response_mixed`, on): the search over every plan keeps to one id a pool. From what it finds, or from the plan the upgrade has where that is better, each pool then takes its best taps with any two of its ids, a tap taking either (`improve`), where the table fits (2 million cells). A regular book on the first taps and an enhanced one after is the usual case. It took the hit rate on `nh_dev` from 88.9% to 92.9%: without it a state with such a plan is one the responses cannot keep.
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

**A restart does not bring the streaks back.** `clone_from_essence` restores a payload and leaves its `streak_cache`, and `perturb_normal` builds the next payload from that cache. So the first move of a normal upgrade after a restart goes on from where the chain was, not from the best state. `Tuning::fresh_streaks` clears the cache there. On `nh_dev` it hit as often and took a tenth less time (sessions 64 and 68), which is within the noise on the first and not on the second.

**Reading a state's streaks used to look for the back streak at the wrong end** (`State::compute_streak_info` took the slots before `len - front_len` in place of those after `front_len`). So wherever the streaks were read from a payload and not kept from the last move, the back streak was lost or one was made up: after a copy between identical upgrades, on a state given in the payload, on a carried plan. Fixed in October 2026. On cold runs it changed nothing that could be measured.

## Chained runs

`solve` takes an optional `Previous`: the payload of the last run, its best `StateBundle`, and a blob the optimizer left for itself (returned in `Solved`, capped at `Tuning::max_blob_bytes`, still empty). The frontend does not pass one yet; it still seeds a run through the payload's `state` and `special_state`.

What a run does with it (`Tuning::carry`):

- **The plan is carried over** (`Previous::carry`): an upgrade that is still there keeps its state, less the taps done since (its first slots), and its place in the free tap order; a new one goes last and starts empty; a chest stack whose key and count are as they were is opened as it was. The arena's `carry::carried` is the same rule, kept apart because it reads a stored state.
- **It is used only where it beats the state given**, which is the default state unless the payload has one.
- **It is polished first** (`carry_polish`), and that is a result like a chain's.
- **The first chain goes on from it**, the others start afresh as in a cold run. A chain that starts from the carried plan almost never leaves it, so only the fresh ones say whether it is still the best.
- **The run ends** as a cold one does, when a fresh chain ends on the same metric as the carried plan (`agree_exit` counts the polished plan and the chain from it), or when `carry_stall` chains in a row, 4, have not bettered the best. That second rule is for a run with a plan to go on from only: on a cold one it cost a point and a half of hit.
- **A state that comes with the payload is such a plan too**, which is what the frontend sends when the user optimizes again. Before, every chain started from it and agreed with it. On `nh_dev` with `v0`'s result as the given state (the `seeded` folder, 4 seeds, session 91), hit 90.8% became 94.0% in the same time, which is what a cold run hits.

What the arena said (October 2026, the weekly chains of `Arena.md`, 2 seeds, a chained run against a cold one of the same build on the same payload):

| `weekly_test`, 36 chains, 991 payloads | hit, nothing honed | time | hit, honing | time |
| --- | --- | --- | --- | --- |
| `v0`, cold | 67.2% | 0.51 | 85.7% | 0.65 |
| the defaults (`v6`), cold | 93.3% | 1 | 97.2% | 1 |
| `v6` chained, `carry_stall` 4 | 97.9% | 0.79 | 97.8% | 0.83 |
| `v2` chained, `carry_stall` 2, against `v2` cold | 96.1% against 94.3% | 0.56 | 96.8% against 96.6% | 0.60 |

Sessions 79, 81 and 95. Over `v0`'s time a cold run takes 1.9 and 1.5 times, a chained one 1.5 and 1.3.

- **A carried plan is worth more than a fresh chain**: with nothing leaving early a chained run hit 4 to 8 points more often than a cold one for the same evaluations (`weekly_dev`, sessions 59 and 60).
- **Leaving as soon as the chain from the carried plan finds nothing better drifts.** That chain almost never leaves the plan it starts from, so week after week the plan is kept past its time: by the last ten weeks of the chains with nothing honed the hit rate was 72% against 90% (session 60). It takes fresh chains to say the plan is still the best, and 3 of them not bettering it (`carry_stall` 4) was the least that kept the hit rate of a cold run on `weekly_dev`; 2 lost two points there (sessions 71 and 72). On `weekly_test` 1 was already as good as cold.
- **Without the polish of the carried plan** the same number of chains hit 2 to 4 points less.
- **The cost does not go to nothing as the stock grows.** In the last ten weeks the plans are mostly empty and the metric near zero, and a run still takes half of what it did in the first weeks.
- Not tried: telling how much a payload changed, and a blob. The carried `StateBundle` has been all that was needed.

## Testing a change

Use the arena (`Arena.md`): snapshot the variants, run them against each other, and read the gap, hit rate and counters. All randomness must come from the solver's own `rng` so that a seed repeats. `scramble` exposes the neighbour moves for making random valid states.
