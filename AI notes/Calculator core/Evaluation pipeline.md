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

Step 4 needs, at each price threshold, the probability that total consumption stays below it and a size-biased version of the same quantity. Those two probabilities are the whole computational problem.

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
- **Brute force** (`core/brute.rs`): exact convolution with pruning, for one threshold. Used when the product of the outcome counts is small, or when the threshold is very close to the minimum or maximum, where pruning is effective and the approximation is weak.
- **Saddlepoint** (`core/saddlepoint_approximation.rs`): the Lugannani-Rice formula with the second continuity correction on the sum's lattice, which starts at the minimum and whose span is the gcd of the gaps of the upgrades with more than one outcome. Needs the root of the cumulant function's first derivative, found by `core/root_finder.rs`. `Tuning::root_finder` picks the finder, so a second one is a new enum variant and a match arm. What the thresholds of one material share (range, span, mean, cumulants at zero) is a `Shape`, made once.
- **Edgeworth**: in place of the main formula near the mean, where that is singular. It takes the cumulants at zero, so it needs no root.
- **Conditioning**: when either of the last two is outside [0, 1], or the main formula's slope term (`1 + k4/8 - 5 k3^2/24` at the saddlepoint, plain probabilities only) is not positive, the sum is split on the outcomes of the upgrade with the most variance and each part goes through the wrapper again. Two levels deep at most, then the value is clamped. Only a NaN there, or a root that does not converge, is a panic.

**The saddlepoint formula is the weak part, not its implementation.** It is asymptotic in the number of comparable upgrades, and the pity tap puts some 15% of an upgrade's probability on its last outcome. With one upgrade outweighing the rest it has been 0.2 off inside [0, 1] with a positive slope term, so nothing flags it, and the optimizer found such a state and was paid 3 million gold of error for it (`test_cases/scenarios/regressions`). The lattice route is what makes that case right; the conditioning only catches what leaves the range. In the stress sweep, which packs its thresholds where the formula is worst, three in ten of the saddlepoint's answers are more than 0.01 off with 2 or 3 upgrades in the sum, one in eight with 4 to 7, and under 1% with 32 or more. Raising `max_lattice_work` trades time for that: 200,000 cost 5% on the `juice` set and a million 60%.

`FLOAT_TOL` is smaller than the gap between f64s past about 8 million, so thresholds are compared with `float_tol(value)`. With the bare constant a threshold equal to a reachable total counted as short of it.

The root finder's safeguards are empirical; treat changes there with care. Check a change anywhere on this path with `pnpm arena stress` and `pnpm arena verify` (`Arena.md`).

## The cost of a root

Evaluating the cumulants is the unit of cost: one pass over every outcome of every upgrade. A pair of probabilities took 8 of them and takes about 3.5. All of it is in `root_finder.rs` and `probability_pair`:

- **The biased cumulants are a function of the plain ones** (`KsTuple::biased`), so the finder works on the plain ones and the plain root's cumulants give the biased search its first step for nothing (`Tuning::seed_biased`). The plain probability is computed first for this. A seed that is a hair off can bracket the search away from its root, so one that does not converge is run again unseeded.
- **A last step shorter than `Tuning::taylor_skip` standard deviations is taken on the Taylor series** (`KsTuple::moved`) in place of the evaluation that would only confirm it. At 0.001 the variance read off the series was within 2e-8 of the true one for the plain root and 7e-6 for the biased, over 6 million steps; at 0.01 the biased one was 2e-4 off. Never from cumulants that came off a series themselves.
  - **The limit is on where the series is read, not on the step that was proposed.** The proposed step can be short while the residual is large, and Newton on the series then walks to where the series means nothing and reports a root there. That killed 3 jobs in 7600 before the walk was held to the same limit.
  - What bounds the series is the range, not the standard deviation: 0.1 standard deviations failed on a sum of one bimodal upgrade.
- **The cumulants at zero come from the supports**, not from an evaluation.
- **`one_cumulant` takes its moments in units of outcomes** where the support is evenly spaced: running sums of running sums give the factorial moments of the outcome count with one multiplication an outcome, and the cumulants are scaled by the gap after.

`taylor_skip=0,seed_biased=false,max_memo_size=0` gives the root finding of before, which is how a doubt about these is settled. What was measured against the code before all of this is in `Arena.md`, "The evaluator before and after its speed-up".

## Where the time goes now

A solve is 2.8 times faster on the `pilot` set and 5.5 times on `stress`. Of what is left, on the explore singles:

- **The advanced honing distributions were 59% of it**, and 75 to 96% of a solve with advanced upgrades: 300 to 1500 cache misses in `compute_adv_dist_wrapper`, 5 to 11 ms each. A miss is now 1 to 2 ms. The cost was copying: every memo hit cloned three vectors and every branch allocated three more. The states' distributions are now written end to end into one buffer, in the same order of additions, so the output is the same to the bit (`dists_are_unchanged`). In the arena no solve has a miss at all (`Arena.md`, "The advanced honing table"). The browser has none for an upgrade not yet started (`Frontend/Wasm contract.md`) and still has them for one in progress, once per start position while the page lives.
  - Sharing one memo between the strategies of a start is another 2 to 3 times, at 60 to 160 MB while it runs.
  - Every start position there is (some 8,400 of them, 361 strategies each) is too much to ship. The two fresh ones with double balls, 722 distributions, are 0.6 MB.
- Of the rest: two fifths cumulant evaluations, an eighth the lattice, and the remainder spread thin over rebuilding supports, the memo's keys, `shape`, brute force and the free-tap probabilities.
- **The annealer itself, outside the evaluator, was 29% of the `pilot` set** and 19% of `explore` (`decide_s`, `Arena.md`), and is 12% of `pilot` now. See `Optimizer.md`, "What a move costs".
- The cumulant loop is bound by its chain of running sums, which a compiler cannot vectorise. Plain sums against a table of powers could be, at more work an outcome in scalar code.
- A pair of probabilities needs two roots because the biased one has its own. A partial expectation read off the plain saddlepoint would need one, and would be a different approximation.

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
