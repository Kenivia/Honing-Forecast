# Optimizer

Lives in `crates/core/src/optimizer`. The module header there and `docs/Optimizer.md` describe the algorithm; both warn that details change often, so this note sticks to what stays true.

## What it does

A single-chain simulated annealer over a `StateBundle`, maximising the average-gold metric (see `Evaluation pipeline.md`). It runs for a fixed number of evaluations, not a time budget, and returns the best state seen. Its constants are fields of `Tuning`.

Decision variables:

- each upgrade's `State` (which juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`;
- how each chest stack is opened, `chest_states`. **Placeholder**: `start_solver` assigns a random valid state once (`randomize_chest_state`, from the solver's rng, drawing nothing for a chest with no choice) and no move changes it afterwards. It travels with the state everywhere `special_state` does.

Things worth knowing about its shape:

- The metric has no natural scale, so there is no fixed temperature schedule. An adaptive scaler adjusts acceptance to track a target acceptance rate over the run.
- It restarts from the best known state very often.
- On restart it may copy one upgrade's state onto an identical upgrade, since identical upgrades often want identical plans.
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

## Chained runs

`solve` takes an optional `Previous`: the payload of the last run, its best `StateBundle`, and a blob the optimizer left for itself (returned in `Solved`, capped at `Tuning::max_blob_bytes`). Nothing reads it yet. The frontend does not pass one either; it still seeds a run through the payload's `state` and `special_state`.

## Testing a change

Use the arena (`Arena.md`): snapshot the variants, run them against each other, and read the gap, hit rate and counters. All randomness must come from the solver's own `rng` so that a seed repeats. `scramble` exposes the neighbour moves for making random valid states.
