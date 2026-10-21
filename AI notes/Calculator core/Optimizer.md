# Optimizer

Lives in `crates/core/src/optimizer`. The module header there describes the algorithm; details change often, so this note sticks to what stays true. Every arena measurement behind the defaults, and everything tried and taken out, is in `Optimizer tried.md`; read it only when trying something new here.

## What it does

Several short chains of simulated annealing over a `StateBundle`, each polished, the best kept, maximising the average-gold metric (see `Evaluation pipeline.md`). It runs for at most a fixed number of evaluations, not a time budget, and returns the best state seen. Its constants are fields of `Tuning`. "The shape of a solve" below has the parts; the rest of this section is one chain.

Decision variables:

- each upgrade's `State` (which juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`;
- how each chest stack is opened, `chest_states`. It starts as given (the payload's, or everything on the first option) and travels with the state everywhere `special_state` does. See "Chests" below.

Things worth knowing about its shape:

- The metric has no natural scale, so there is no fixed temperature schedule. An adaptive scaler adjusts acceptance to track a target acceptance rate over the run.
- It restarts from the best known state very often.
- On restart it may copy one upgrade's state onto an identical upgrade, since identical upgrades often want identical plans. Identical means normal honing too (an advanced block shares its index with a low normal level, and its state is not a normal state), and the same artisan and taps done, so a state's length never changes.
- A move changes two upgrades: that is what moves juice between them.
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
4. **Leaving early**: once `agree_exit` chains have ended on the same metric; or, going on from a plan (below), once `carry_stall` chains in a row have not bettered it. Too few agreeing chains leave on a plan that is reliably not the best on small payloads.
5. **Merge** (`merge`): the best chain's result takes from each other chain's, where the metric agrees, its free tap order, its plans for a whole piece type (they share juices and books), and its plan for each upgrade.
6. **Exchange** (`exchange`): uses of a juice or a book go from one upgrade to another that takes the same, 1, 2, 4 up to 64 at a time: the last taps that take it of the one, the first that do not of the other. Then the polishes again, and again while it moves anything. Not under a streak length above 1.

A chained run (`Previous`, see "Chained runs") puts the plan of the run before in front of all this.

**The free tap order is only moved while the budget can pay for a free tap** (`StateBundle::affords_a_free_tap`); otherwise those moves are wasted. `special_floor` keeps a move of the order from being drawn with nothing as its chance, which used to lock the order out when the first upgrade cost more than the budget.

**What is still missed** is small and in few payloads, the hardest being one piece with many levels sharing a partial stock of juice, where every run ends on another split of it.

## Rules a neighbour move must respect

Evaluation assumes these; breaking them corrupts results without an error.

- `special_state` stays a permutation of upgrade indices. Never reorder the upgrade array.
- A state's length never changes.
- A state may be any plan: the pass of responses at the end does not keep to streaks. Reading its streaks (`compute_streak_info`) then gives the nearest streak state, which is what a perturbation goes on from.
- A chest state has one entry per option, none negative, summing to the chest's count. Change it through `set_chest_state`, or call `apply_chest_state` after, or the bands are stale.
- A book id used on an upgrade must be one that exists for that piece type and level. Id 0 means none.
- Never the same id twice on one tap. The evaluator counts it once, but a move that makes one is a bug, and the arena treats such a state as an error (`Arena.md`).
- Advanced-honing strategy indices stay within their maximum.
- Update the state's hash after changing it.
- For normal honing, taps the player has already done are fixed and must not be modified.
- Normal-honing states are kept in a restricted form: for each juice pool, one streak from the first tap and one streak from the last. This is an empirical simplification, not a proven one, and the perturbation code depends on it. It is nearly free (`State space.md`), and the responses reach the plans outside it.

## Chests

Two things change a chest state, and neither draws from the rng or touches the chain where no stack has a choice, so a payload without one runs as if there were no chests:

- **A chest move** (`chest_move`, `Tuning::chest_move_rate` of the moves): some chests of one stack go from one option to another, at most all of the option's early in the run and one at the end. It is a move like any other: evaluated, then accepted or rejected.
- **A polish** (`chest_polish`, every `Tuning::chest_polish_every` evaluations of a chain, once at its start and with the other polishes at its end): from the best state, with the upgrades as they are, every pair of options of every stack is tried in steps that halve, for as long as anything helps. Its evaluations count against `max_iters`, and the chain goes on from the result.

Opening the chests well is the easy part; what is left of the gap with chests is in the upgrades and the free tap order. Two kinds of payload stay stuck: a stack offering a book or scroll no upgrade uses yet (the chests are worth nothing until a state uses it, and the state is worse until the chests are there), and a large free tap budget with a stack that adds to it.

A move that goes on after a chest move must compute the free tap probabilities first: `neighbour` reads them for the current budget, and a chest move may have made a budget that no evaluation has seen.

## Responses

`optimizer/response.rs` works out the best plan of one normal upgrade with the rest of the state as it is, over every plan and not only the streaks (`State space.md` has why that is possible). Three things can use it, each a `Tuning` field:

- **`response_polish`, on, 3**: at the end, from the best state, each normal upgrade in turn takes its best plan where the metric agrees, for up to 3 passes. A solve can therefore return a state that is not a streak state. This is nearly all of what responses gain.
- **`response_every`, off**: the same single pass every so many evaluations, the chain going on from the result.
- **`response_rate`, off**: the share of normal honing moves that are a response of one upgrade in place of a random change of its streaks (`respond`). An upgrade whose state and whose rest of the state are as they were when it last responded is not asked again; where none is left the move is a random one. `response_kick` changes another upgrade at random first. More responses are worse: a response costs 15 to 100 evaluations, taken from the free tap order, the advanced upgrades and the chests.

A response is charged what it took, in evaluations: its table evaluations, and the steps of its recursion at `response_steps` to one. That is about its time, within a factor of two either way.

Inside a response:

- **One id a pool, then two** (`response_mixed`, on): the search over every plan keeps to one id a pool. From what it finds, or from the plan the upgrade has where that is better, each pool then takes its best taps with any two of its ids, a tap taking either (`improve`), where the table fits (2 million cells). A regular book on the first taps and an enhanced one after is the usual case; without it a state with such a plan is one the responses cannot keep.
- **`response_joint`**: an upgrade with two pools is searched over both at once while that is at most this many steps, and pool by pool from its plan past that (`Response::improve`).
- **`response_points`**: where the lattice does not answer, what an amount of a material is worth is taken at that many amounts and joined by lines.
- The recursion's table is kept between responses in a thread local, stamped, because most of it is never reached and clearing it was most of a response.

## Streak length

The game can do ten taps at once, so a plan may be asked to change what it takes only every so many taps. The payload's `min_resolution` is that number (`StateBundle::block`; `Tuning::streak_length` overrides it, which is how the arena varies it). The frontend sends the profile's `min_resolution`, 1 or 10 by the "Force x10 streak length" checkbox in the optimizer control panel (saved with the profile).

- **The rule**: from an upgrade's next tap on, the taps go in blocks of that many, and every tap of a block takes the same. So every streak, of taking or of not taking, is a multiple of it but the last before the pity, which the pity cuts.
- **A random move** rounds its front streak down to a block and starts its back streak where a block does, and changes a streak by two blocks at least, or the rounding would undo it.
- **A response** chooses once a block and plays the block through, so it is the best of the plans that keep the rule, and its table is only kept where a block starts. `Model::filled` gives a plan cut at its pity its whole last block back: a pool left alone while another moves the pity out would otherwise stop inside a block.
- **A state that is given off the grid stays so** until a move or a response replaces it.
- **What the rule costs is in what is owned**: an owned amount that is not a whole number of blocks is either left over or topped up at the buy price. The search itself still finds the best plan that keeps the rule.

## What a move costs

Outside the evaluator an iteration is some 5 microseconds. Things that keep it there:

- **Accepting or rejecting copies every upgrade's state**, changed or not. `State::copy_from` does it without allocating and leaves a payload with the same hash alone.
- `clone_from_essence` skips a payload that is already the one wanted.
- **The state hash is `ahash` with fixed seeds**, in place of SipHash. The seeds are fixed because a seed has to repeat.

Of what is left, two thirds is `neighbour` (two `WeightedIndex` built per move, and the streaks written back over the whole payload) and a quarter the copy on accepting. Changing the first without changing what the rng is asked for takes care.

**A restart does not bring the streaks back** unless asked to. `clone_from_essence` restores a payload and leaves its `streak_cache`, and `perturb_normal` builds the next payload from that cache, so the first move of a normal upgrade after a restart goes on from where the chain was. `Tuning::fresh_streaks` clears the cache there.

## Chained runs

`solve` takes an optional `Previous`: the payload of the last run, its best `StateBundle`, and a blob the optimizer left for itself (returned in `Solved`, capped at `Tuning::max_blob_bytes`, still empty). The frontend does not pass one yet; it still seeds a run through the payload's `state` and `special_state`.

What a run does with it (`Tuning::carry`):

- **The plan is carried over** (`Previous::carry`): an upgrade that is still there keeps its state, less the taps done since (its first slots), and its place in the free tap order; a new one goes last and starts empty; a chest stack whose key and count are as they were is opened as it was. The arena's `carry::carried` is the same rule, kept apart because it reads a stored state.
- **It is used only where it beats the state given**, which is the default state unless the payload has one.
- **It is polished first** (`carry_polish`), and that is a result like a chain's.
- **The first chain goes on from it**, the others start afresh as in a cold run. A chain that starts from the carried plan almost never leaves it, so only the fresh ones say whether it is still the best.
- **The run ends** as a cold one does, when a fresh chain ends on the same metric as the carried plan (`agree_exit` counts the polished plan and the chain from it), or when `carry_stall` chains in a row have not bettered the best. That second rule is for a run with a plan to go on from only; on a cold one it costs hit rate. Leaving as soon as the chain from the carried plan finds nothing better drifts: week after week the plan is kept past its time.
- **A state that comes with the payload is such a plan too**, which is what the frontend sends when the user optimizes again: one chain goes on from it and the rest start fresh.
- The cost does not go to nothing as the stock grows: a run with mostly empty plans and a metric near zero still takes about half of a full one.

## Testing a change

Use the arena (`Arena.md`): snapshot the variants, run them against each other, and read the gap, hit rate and counters. All randomness must come from the solver's own `rng` so that a seed repeats. `scramble` exposes the neighbour moves for making random valid states.
