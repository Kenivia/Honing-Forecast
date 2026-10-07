# Optimizer

Lives in `crates/core/src/optimizer/v35`. The module header there and `docs/Optimizer.md` describe the algorithm; both warn that details change often, so this note sticks to what stays true.

## What it does

A single-chain simulated annealer over a `StateBundle`, maximising the average-gold metric (see `Evaluation pipeline.md`). It runs for a fixed number of evaluations, not a time budget, and returns the best state seen.

Decision variables:

- each upgrade's `State` (which juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`.

Things worth knowing about its shape:

- The metric has no natural scale, so there is no fixed temperature schedule. An adaptive scaler adjusts acceptance to track a target acceptance rate over the run.
- It restarts from the best known state very often.
- On restart it may copy one upgrade's state onto an identical upgrade, since identical upgrades often want identical plans.
- In the browser it periodically posts the best state so far as progress.

## Rules a neighbour move must respect

Evaluation assumes these; breaking them corrupts results without an error.

- `special_state` stays a permutation of upgrade indices. Never reorder the upgrade array.
- A state's length never changes.
- A book id used on an upgrade must be one that exists for that piece type and level. Id 0 means none.
- Advanced-honing strategy indices stay within their maximum.
- Update the state's hash after changing it.
- For normal honing, taps the player has already done are fixed and must not be modified.
- Normal-honing states are kept in a restricted form: for each juice pool, one streak from the first tap and one streak from the last. This is an empirical simplification, not a proven one, and the perturbation code depends on it.

## Version folders and the test harness: defunct

Each optimizer version is a full copy in its own folder behind a cargo feature, originally so versions could be benchmarked against each other. That tooling is largely defunct and is expected to be replaced or reworked soon. Do not build on it without asking.

- `optimizer/Old/` is a dead archive. It does not compile into anything.
- `crates/arena`, `scripts/optimizer_test.ps1`, `scripts/autorun.ps1`, `scripts/optimizer_visualizer` and `test_cases/optimizer_results` belong to the benchmarking workflow. `crates/arena` is commented out of the workspace.
- `crates/core/src/verification` compares the analytical evaluation with Monte Carlo simulation and with cached results in `test_cases/verification_results`. It is **commented out** of `lib.rs` pending a proper test suite for the core; the files are untouched, and `crates/arena` was commented out with it because it is the only thing that calls it.

What remains load-bearing: the `v35` / `active_version` feature must be enabled for `solve` to exist. There is still no automated check on the optimizer's *search*, so verify changes there by other means and say so. The evaluator's material valuation does have unit tests, in `materials.rs`.
