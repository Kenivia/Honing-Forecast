# Evaluation pipeline

How a payload becomes a number. For the maths itself read `docs/Saddlepoint Approximation.pdf`; for a flow diagram, `docs/Average Evaluation.md`.

## Entry points

Both are in `crates/wasm/src/lib.rs` and both start by turning the `Payload` into a `StateBundle`.

- **Optimize**: runs the optimizer, re-evaluates the best state, and returns the whole `StateBundle`. While running it posts intermediate best states straight to the page from Rust, so it must run inside a Web Worker.
- **Histogram**: evaluates one fixed state (no search) and returns, per material, a cumulative distribution of consumption, the chance that each ownership level is enough, and the average gold under the UI's treatment plans.

The optimizer and the histogram share the same evaluation code, so a change to the evaluator affects both the plan chosen and the numbers shown.

## One evaluation

1. Refresh each upgrade's probability distribution from its `State`.
2. Refresh each upgrade's per-material cost supports.
3. Compute the free-tap outcome probabilities: the chance that exactly the first k upgrades in the free-tap order get skipped.
4. For each free-tap outcome and each material, compute the expected gold for that material, then take the weighted sum.

Step 4 needs, at each price threshold, the probability that total consumption stays below it and a size-biased version of the same quantity. Those two probabilities are the whole computational problem.

**Sign**: the metric is a gold *value*. It is usually negative (a cost) and the optimizer maximises it.

## Method selection

`saddlepoint_approximation_wrapper` picks one of:

- **Trivial**: the threshold is outside the possible range.
- **Brute force** (`core/brute.rs`): exact convolution with pruning. Used when the outcome space is small, or when the threshold is very close to the minimum or maximum, where pruning is effective and the approximation is weak.
- **Saddlepoint** (`core/saddlepoint_approximation.rs`): the Lugannani-Rice formula with a lattice continuity correction. Needs the root of the cumulant function's first derivative, found by `core/root_finder.rs`.
- **Edgeworth fallback**: used near the mean, where the main formula is singular, and when the main formula returns something out of range.

A failure to converge or a result outside [0, 1] after the fallback is a panic, not a soft error. The root finder's safeguards are empirical; treat changes there with care.

## Invariants

These fail silently or panic at a distance, and none is obvious from a single file.

- **Rehash after mutating a state.** Distributions and collapsed supports are cached by the state's hash. Changing a state's contents without updating its hash leaves stale caches in use. Use the provided update methods.
- **Iteration follows free-tap order.** The support iterators walk upgrades in `special_state` order, not array order, and "skip the first k" refers to that order. The upgrade array itself is never reordered.
- **Compute before read.** The free-tap probabilities are read from a cache keyed by the current free-tap order and must have been computed for it first. Every metric does this at its start; a new caller must too.
- **Collapsed supports have no zero-probability entries.** Collapsing merges equal values and drops negligible probabilities; the cumulant code relies on that.
- **Supports are evenly spaced.** All current distributions have a constant gap between values, which the cumulant fast path and the lattice correction both assume.
- **Four breakpoints.** Parts of the UI metric assume the four-entry material layout the frontend sends.

## Leftovers

`metric_type` exists but only the average-gold metric is live. `success_prob.rs` is a shelved metric, kept because the histogram reuses part of it. `bound.rs` is an abandoned bounding attempt that now only supplies an initial guess. `helpers.rs` contains helpers from an earlier design.
