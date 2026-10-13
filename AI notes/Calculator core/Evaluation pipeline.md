# Evaluation pipeline

How a payload becomes a number. For the maths itself read `docs/Saddlepoint Approximation.pdf`; for a flow diagram, `docs/Average Evaluation.md`.

## Entry points

Both are in `crates/wasm/src/lib.rs` and both start by turning the `Payload` into a `StateBundle`.

- **Optimize**: runs the optimizer, re-evaluates the best state, and returns the whole `StateBundle`. While running it posts intermediate best states straight to the page from Rust, so it must run inside a Web Worker.
- **Histogram**: evaluates one fixed state (no search) and returns, keyed by material label, a cumulative distribution of consumption, the chance that each ownership band is enough, and the average gold under every plan in the payload.

The optimizer and the histogram share the same evaluation code, so a change to the evaluator affects both the plan chosen and the numbers shown.

## One evaluation

1. Refresh each upgrade's probability distribution from its `State`.
2. Refresh each upgrade's per-material cost supports.
3. Compute the free-tap outcome probabilities: the chance that exactly the first k upgrades in the free-tap order get skipped.
4. For each free-tap outcome and each material, compute the expected gold for that material, then take the weighted sum.

Step 4 needs, at each price threshold, the probability that total consumption stays below it and a size-biased version of the same quantity. Those two probabilities are the whole computational problem.

**Sign**: the metric is a gold *value*. It is usually negative (a cost) and the optimizer maximises it.

## Method selection

`saddlepoint_approximation_wrapper` picks one of, in this order:

- **Trivial**: the threshold is outside the possible range.
- **Lattice** (`core/lattice.rs`): the whole distribution of the sum, exactly, as an array over the lattice it lives on. Used when lattice points times outcomes is within `Tuning::max_lattice_work`, which is mostly juices and books. `one_dimension_average_gold` builds it once and reads every threshold of the material from it, plain and biased, so it never reaches the wrapper there.
- **Brute force** (`core/brute.rs`): exact convolution with pruning, for one threshold. Used when the product of the outcome counts is small, or when the threshold is very close to the minimum or maximum, where pruning is effective and the approximation is weak.
- **Saddlepoint** (`core/saddlepoint_approximation.rs`): the Lugannani-Rice formula with the second continuity correction on the sum's lattice, which starts at the minimum and whose span is the gcd of the gaps of the upgrades with more than one outcome. Needs the root of the cumulant function's first derivative, found by `core/root_finder.rs`. `Tuning::root_finder` picks the finder, so a second one is a new enum variant and a match arm.
- **Edgeworth**: in place of the main formula near the mean, where that is singular. It takes the cumulants at zero, so it needs no root.
- **Conditioning**: when either of the last two is outside [0, 1], or the main formula's slope term (`1 + k4/8 - 5 k3^2/24` at the saddlepoint, plain probabilities only) is not positive, the sum is split on the outcomes of the upgrade with the most variance and each part goes through the wrapper again. Two levels deep at most, then the value is clamped. Only a NaN there, or a root that does not converge, is a panic.

**The saddlepoint formula is the weak part, not its implementation.** It is asymptotic in the number of comparable upgrades, and the pity tap puts some 15% of an upgrade's probability on its last outcome. With one upgrade outweighing the rest it has been 0.2 off inside [0, 1] with a positive slope term, so nothing flags it, and the optimizer found such a state and was paid 3 million gold of error for it (`test_cases/scenarios/regressions`). The lattice route is what makes that case right; the conditioning only catches what leaves the range. In the stress sweep, which packs its thresholds where the formula is worst, three in ten of the saddlepoint's answers are more than 0.01 off with 2 or 3 upgrades in the sum, one in eight with 4 to 7, and under 1% with 32 or more. Raising `max_lattice_work` trades time for that: 200,000 cost 5% on the `juice` set and a million 60%.

`FLOAT_TOL` is smaller than the gap between f64s past about 8 million, so thresholds are compared with `float_tol(value)`. With the bare constant a threshold equal to a reachable total counted as short of it.

The root finder's safeguards are empirical; treat changes there with care. Check a change anywhere on this path with `pnpm arena stress` and `pnpm arena verify` (`Arena.md`).

Every function on this path takes a `&mut Probe` and counts which method it used. See `Arena.md`.

## Invariants

These fail silently or panic at a distance, and none is obvious from a single file.

- **Rehash after mutating a state.** Distributions and collapsed supports are cached by the state's hash. Changing a state's contents without updating its hash leaves stale caches in use. Use the provided update methods.
- **Iteration follows free-tap order.** The support iterators walk upgrades in `special_state` order, not array order, and "skip the first k" refers to that order. The upgrade array itself is never reordered.
- **Compute before read.** The free-tap probabilities are read from a cache keyed by the current free-tap order and must have been computed for it first. Every metric does this at its start; a new caller must too.
- **Collapsed supports have no zero-probability entries.** Collapsing merges equal values and drops negligible probabilities; the cumulant code relies on that.
- **Supports are evenly spaced.** All current distributions have a constant gap between values, which the cumulant fast path and the lattice correction both assume.
- **Plans are precomputed.** `PreparationOutput::bands` holds one band list per plan per material, built once per chest state (`Domain model.md`, "Chests"). The evaluator never recomputes them, so a payload with many plans costs extra evaluations, not extra parsing.
- **A threshold the price does not change at is free.** `one_dimension_average_gold` skips its two saddlepoint evaluations, because they are multiplied by the price gap. `ValuationPlan::bands` already merges such entries wherever merging is sound, so in practice this fires only on the top entry, which has to stay for the base term. See `Domain model.md`.

## Leftovers

`metric_type` exists but only the average-gold metric is live. `success_prob.rs` is a shelved metric, kept because the histogram reuses `one_dimension_prob`. `bound.rs` is an abandoned bounding attempt that now only supplies an initial guess. `helpers.rs` contains helpers from an earlier design.
