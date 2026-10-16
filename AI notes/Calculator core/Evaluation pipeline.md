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
4. For each material and each free-tap outcome, compute the expected gold for that material, then take the weighted sum.

Step 4 needs, at each price threshold t, the expected leftover E[(t - S)+] = t F(t) - mean F'(t), where F' is the size-biased distribution function. That is the whole computational problem. The histogram also needs the plain F(t) alone.

## What an evaluation does not redo

The optimizer changes one upgrade and evaluates again, so nearly everything is as it was. Each of these returns what computing again would have, to rounding:

- **Steps 1 and 2 run only for an upgrade whose state hash moved.** The distributions of the state before are kept in `Upgrade::spare` and swapped back when a rejected move returns to it.
- **Step 3 is skipped while the free-tap order and budget are the ones last computed for** (`special_done`).
- **Step 4 is kept per material and free-tap outcome** in `StateBundle::gold_memo` (`material_average_gold`). The key is everything the number depends on: the material, its bands, the evaluator's tuning, and for every upgrade either its state hash (honed) or what a free tap leaves of it (skipped). An upgrade that does not use the material counts as a constant. The honed part is a sum, so the order of the honed upgrades does not change the key. That makes a result depend, at the 1e-8 level, on which order it was first computed under; a seed still repeats exactly. `Tuning::max_memo_size` of 0 turns it off. **Anything new that a material's result depends on has to go into the key.**
- **A support keeps its mean, its cumulants at zero and whether it is evenly spaced** (`Support::summarise`), set wherever its collapsed pairs are.

**Sign**: the metric is a gold *value*. It is usually negative (a cost) and the optimizer maximises it.

## Method selection

`saddlepoint_approximation_wrapper` picks one of, in this order:

- **Trivial**: the threshold is outside the possible range.
- **Lattice** (`core/lattice.rs`): the whole distribution of the sum, exactly, as an array over the lattice it lives on. Used when lattice points times outcomes is within `Tuning::max_lattice_work`, which is mostly juices and books. The metric goes through `lattice_table`, which answers every threshold of a material for every free-tap outcome in one pass: skipping k upgrades leaves the sum of the rest, so the sums are built from the last upgrade backwards, and nothing past the largest threshold is kept. `lattice` is the same for one outcome and is what the wrapper uses.
- **Coarse lattice** (`Coarse` in `lattice.rs`, `Tuning::coarse_cells`, `max_coarse_work`; 0 cells by default, 64 in `Tuning::reference`): where the exact lattice is too big, the sum on a grid of that many points a standard deviation (a whole number of spans where there is a lattice). Each outcome's mass is split between the two grid points around it, which keeps every mean, so t F - mean F' is exactly the leftover of the gridded sum, whose only error is the variance the split adds; half of that times the density is taken off. Its plain probability is a difference of leftovers half a step either side and only 1e-2 good, so it is for the leftover. Past `max_coarse_work` multiplications the routes below answer.
- **Brute force** (`core/brute.rs`): exact convolution with pruning, for one threshold. Used when the product of the outcome counts is small, or when the threshold is very close to the minimum or maximum, where pruning is effective and the approximation is weak.
- **The leftover from one saddlepoint** (`leftover` in `core/saddlepoint_approximation.rs`, the metric's route, called "direct"): E[(x - S)+] = (1/2 pi i) int e^(K(s) - s x) / ((2/span) sinh(span s / 2))^2 ds at a lattice point x, which is 1/s^2 off the lattice. Its pole at 0 is double and has no 1/w part, so it gives (x - mean) Phi(w) + phi(w) [(x - mean)/w - (x - mean)/w^3 + 1/(s_span^2 sqrt K'')], with w and s the plain saddlepoint's and s_span = (2/span) sinh(span s / 2). Between lattice points the leftover is a straight line, so it is taken at the points either side and joined, the second root seeded from the first. Within 0.01 standard deviations of the mean the terms cancel to nothing and the integral of the Edgeworth series stands in, less span^2/12 times the density for the lattice. It is exact for a normal sum and unmoved by a constant added to the sum. Where the sum is small enough or the threshold is near either end, brute force answers instead; where it finds no root or gives what no distribution on the range could (`DirectFallback`), a coarse lattice at 16 points answers, and past that the Edgeworth leftover.
- **Saddlepoint** (`saddlepoint_approximation`, plain probabilities only, for the histogram and `stress`): the Lugannani-Rice formula with the second continuity correction on the sum's lattice, which starts at the minimum and whose span is the gcd of the gaps of the upgrades with more than one outcome. Both saddlepoint routes need the root of the cumulant function's first derivative, found by `core/root_finder.rs`. `Tuning::root_finder` picks the finder, so a second one is a new enum variant and a match arm. What the thresholds of one material share (range, span, mean, cumulants at zero) is a `Shape`, made once.
- **Edgeworth**: in place of the main formula near the mean, where that is singular. It takes the cumulants at zero, so it needs no root.
- **Conditioning**: when either of the last two is outside [0, 1], or the main formula's slope term (`1 + k4/8 - 5 k3^2/24` at the saddlepoint) is not positive, or the root finder finds no root, the sum is split on the outcomes of the upgrade with the most variance and each part goes through the wrapper again. Two levels deep at most, then the value is clamped. Only a NaN there is a panic.
  - No root used to be a panic. A weekly chain of the arena hit it under every optimizer (`test_cases/scenarios/regressions/no_root_week_inc_0022`). Now the Edgeworth value stands in, untrusted, so the split is what answers; nothing is known to reach it since the cause below was fixed.
  - **The cause was `one_cumulant`, not the root finder.** The evenly spaced path counts q from the end the tilt takes the weight off. Tilted hard, the weight sits at the far end, and the mean and variance come out as the difference of two nearly equal numbers: on that payload a juice of 4 upgrades (steps of 20 to 50) with 23 owned, whose biased search ran to theta near -1, where one upgrade's mean came out -2.7e-12 against 3.3e-21 and the biased K' was off by 1 at theta -1 and by 24,000 at -1.4, jumping by 0.08 within 1e-8 of theta. So there was no point with a residual under `y_value_tol`. Now, where the variance keeps fewer than 8 digits, the same sums are taken from the heavy end, each weight its own exponential. That is rare: 120 states of `nh_quick` evaluate to the same bits and as fast as before, and `stress` on it gives the same table.

**The approximations are the weak part, not their implementation.** They are asymptotic in the number of comparable upgrades, and the pity tap puts some 15% of an upgrade's probability on its last outcome, so with one upgrade outweighing the rest they are off by up to 0.2 in a probability with nothing to flag it, and their error leans optimistic. The lattice route is what makes that case right. Everything tried against it, the measurements, and the code of the alternatives that were dropped (the size-biased pair, the split) are in `core experiments/FINDINGS.md` at the repository root (gitignored, an archive).

**The optimizer and what is reported use different evaluators.** The optimizer uses the default tuning (direct saddlepoint leftover). Every number a person or the arena judges by is `Tuning::reference()`, the coarse lattice at 64 points a standard deviation, which is 1e-6 of flow off on average and 1e-5 at worst: the histogram's `metric_per_plan` (`histogram.rs` swaps the tuning around `ui_average_gold_metric` only, so the distribution graphs and chances stay on the default), the arena's `metric_ref` and its Monte Carlo checks. The optimizer never sees the reference. Plans chosen by the cheaper one were no worse under the reference, but its own number of them flattered them by up to 2.7e-3 of flow.

The coarse lattice's work goes with the square of its points; at 64 one evaluation is 3 to 6.5 times the default's, which is why the optimizer does not use it.

`FLOAT_TOL` is smaller than the gap between f64s past about 8 million, so thresholds are compared with `float_tol(value)`. With the bare constant a threshold equal to a reachable total counted as short of it.

The root finder's safeguards are empirical; treat changes there with care. Check a change anywhere on this path with `pnpm arena stress` and `pnpm arena verify` (`Arena.md`).

## The cost of a root

Evaluating the cumulants is the unit of cost: one pass over every outcome of every upgrade. A leftover takes one root at each of the two lattice points around the threshold, the second seeded from the first. All of it is in `root_finder.rs` and `leftover`:

- **A last step shorter than `Tuning::taylor_skip` standard deviations is taken on the Taylor series** (`KsTuple::moved`) in place of the evaluation that would only confirm it. At 0.001 the variance read off the series was within 2e-8 of the true one over 6 million steps. Never from cumulants that came off a series themselves.
  - **The limit is on where the series is read, not on the step that was proposed.** The proposed step can be short while the residual is large, and Newton on the series then walks to where the series means nothing and reports a root there. That killed 3 jobs in 7600 before the walk was held to the same limit.
  - What bounds the series is the range, not the standard deviation: 0.1 standard deviations failed on a sum of one bimodal upgrade.
- **The cumulants at zero come from the supports**, not from an evaluation.
- **`one_cumulant` takes its moments in units of outcomes** where the support is evenly spaced: running sums of running sums give the factorial moments of the outcome count with one multiplication an outcome, and the cumulants are scaled by the gap after. The count starts from the light end of the tilt, and from the heavy end where that loses the variance (see "Method selection").
- **An id twice on a tap counts once** (`get_extra_arr`), as its cost always did. Before, the chance counted it twice and such a plan scored far above any real one. No optimizer has made one but an experiment of October 2026.

`taylor_skip=0,max_memo_size=0` gives the root finding of before, which is how a doubt about these is settled. What was measured against the code before all of this is in `Arena.md`, "The evaluator before and after its speed-up".

## Where the time goes now

A solve is 2.8 times faster on the `pilot` set and 5.5 times on `stress`. Of what is left, on the explore singles:

- **The advanced honing distributions were 59% of it**, and 75 to 96% of a solve with advanced upgrades: 300 to 1500 cache misses in `compute_adv_dist_wrapper`, 5 to 11 ms each. A miss is now 1 to 2 ms. The cost was copying: every memo hit cloned three vectors and every branch allocated three more. The states' distributions are now written end to end into one buffer, in the same order of additions, so the output is the same to the bit (`dists_are_unchanged`). In the arena no solve has a miss at all (`Arena.md`, "The advanced honing table"). The browser has none for an upgrade not yet started (`Frontend/Wasm contract.md`) and still has them for one in progress, once per start position while the page lives.
  - Sharing one memo between the strategies of a start is another 2 to 3 times, at 60 to 160 MB while it runs.
  - Every start position there is (some 8,400 of them, 361 strategies each) is too much to ship. The two fresh ones with double balls, 722 distributions, are 0.6 MB.
- Of the rest: two fifths cumulant evaluations, an eighth the lattice, and the remainder spread thin over rebuilding supports, the memo's keys, `shape`, brute force and the free-tap probabilities.
- **The annealer itself, outside the evaluator, was 29% of the `pilot` set** and 19% of `explore` (`decide_s`, `Arena.md`), and is 12% of `pilot` now. See `Optimizer.md`, "What a move costs".
- The cumulant loop is bound by its chain of running sums, which a compiler cannot vectorise. Plain sums against a table of powers could be, at more work an outcome in scalar code.

Every function on this path takes a `&mut Probe` and counts which method it used. See `Arena.md`.

## Invariants

These fail silently or panic at a distance, and none is obvious from a single file.

- **Rehash after mutating a state.** Distributions and collapsed supports are cached by the state's hash. Changing a state's contents without updating its hash leaves stale caches in use. Use the provided update methods.
- **Iteration follows free-tap order.** The support iterators walk upgrades in `special_state` order, not array order, and "skip the first k" refers to that order. The upgrade array itself is never reordered.
- **Compute before read.** The free-tap probabilities are read from a cache keyed by the current free-tap order and must have been computed for it first. Every metric does this at its start; a new caller must too.
- **Collapsed supports have no zero-probability entries.** Collapsing merges equal values and drops negligible probabilities; the cumulant code relies on that.
- **Supports are evenly spaced.** All current distributions have a constant gap between values, which the lattice correction assumes. The cumulant fast path checks it (`Support::arithmetic`) and falls back where an outcome in the middle was dropped.
- **Distributions follow the state hash.** Nothing is rebuilt for an upgrade whose supports carry its state's hash, so whatever else a distribution came to depend on would have to move that hash.
- **Plans are precomputed.** `PreparationOutput::bands` holds one band list per plan per material, built once per chest state (`Domain model.md`, "Chests"). The evaluator never recomputes them, so a payload with many plans costs extra evaluations, not extra parsing.
- **A threshold the price does not change at is free.** `one_dimension_average_gold` skips its two saddlepoint evaluations, because they are multiplied by the price gap. `ValuationPlan::bands` already merges such entries wherever merging is sound, so in practice this fires only on the top entry, which has to stay for the base term. See `Domain model.md`.

## Leftovers

`metric_type` exists but only the average-gold metric is live. `success_prob.rs` is a shelved metric, kept because the histogram reuses `one_dimension_prob`. `bound.rs` is an abandoned bounding attempt that now only supplies an initial guess. `helpers.rs` contains helpers from an earlier design.
