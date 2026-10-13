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
- A chest state has one entry per option, none negative, summing to the chest's count. Change it through `set_chest_state`, or call `apply_chest_state` after, or the bands are stale.
- A book id used on an upgrade must be one that exists for that piece type and level. Id 0 means none.
- Advanced-honing strategy indices stay within their maximum.
- Update the state's hash after changing it.
- For normal honing, taps the player has already done are fixed and must not be modified.
- Normal-honing states are kept in a restricted form: for each juice pool, one streak from the first tap and one streak from the last. This is an empirical simplification, not a proven one, and the perturbation code depends on it.

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
