# Evaluation pipeline

How a payload becomes a number. What failed or was ruled out on this path is in `Evaluation pipeline tried.md`; read it only when trying something new here.

## Entry points

These are in `crates/wasm/src/lib.rs` (`optimize_average_wrapper`, `histogram_wrapper`, `marginal_value_wrapper`) and all start by turning the `Payload` into a `StateBundle`.

- **Optimize**: runs the optimizer, re-evaluates the best state, and returns the whole `StateBundle`. While running it posts intermediate best states straight to the page from Rust, so it must run inside a Web Worker.
- **Histogram**: evaluates one fixed state (no search) and returns, keyed by material label, a cumulative distribution of consumption, the chance that each ownership band is enough, and the average gold under every plan in the payload. With the payload's `dist_plan` set, also the distribution of the gold that plan spends (`gold_dist.rs`, below).
- **Marginal value** (`marginal.rs`, `marginal_value_wrapper`): the market price of a juice or book under which the optimizer would use more of it, searched by re-polishing the plan at lower prices.

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
- **Step 4 is kept per material and free-tap outcome** in `StateBundle::gold_memo` (`material_average_gold`). The key is everything the number depends on: the material, its bands, the evaluator's tuning, and for every upgrade either its state hash (honed) or what a free tap leaves of it (skipped). An upgrade that does not use the material counts as a constant. The honed part is a sum, so the order of the honed upgrades does not change the key. That makes a result depend, at rounding level, on which order it was first computed under; a seed still repeats exactly. `Tuning::max_memo_size` of 0 turns it off. **Anything new that a material's result depends on has to go into the key.**
- **A support keeps its mean, its cumulants at zero and whether it is evenly spaced** (`Support::summarise`), set wherever its collapsed pairs are.
- **Advanced honing distributions** come from a memoised recursion (`compute_adv_dist_wrapper`) whose states are written end to end into one buffer; the output is the same to the bit as computing each alone (`dists_are_unchanged`). In the arena they come from a table on disk (`Arena.md`, "The advanced honing table"), in the browser from a table built into the wasm for an upgrade not yet started (`Frontend/Wasm contract.md`), and otherwise are worked out once per start position while the page lives.

**Sign**: the metric is a gold *value*. It is usually negative (a cost) and the optimizer maximises it.

## Method selection

The code is in two folders of `crates/core/src/core`: `mean/` for the metric (`average.rs`, `lattice.rs`, `special.rs`, the free-tap outcomes) and `probability/` for the plain probability (`saddlepoint.rs`, `root_finder.rs`, `cumulants.rs`, `bound.rs`, `brute.rs`, `shape.rs`, `success_prob.rs`). Each imports what it needs of the other: the metric takes `Shape` and the cumulants from `probability/`, and the probability takes the exact `lattice` from `mean/`. What the thresholds of one material share (range, span, mean, cumulants at zero) is a `Shape` (`shape.rs`), made once.

**The metric** (`one_dimension_average_gold`), per material and free-tap outcome, in this order:

- **Lattice** (`lattice_table` in `mean/lattice.rs`): the whole distribution of the sum, exactly, as an array over the lattice it lives on. Used when lattice points times outcomes is within `Tuning::max_lattice_work`, which is mostly juices and books. It answers every threshold of a material for every free-tap outcome in one pass: skipping k upgrades leaves the sum of the rest, so the sums are built from the last upgrade backwards, and nothing past the largest threshold is kept. The check for "small enough" is on the whole range, not on what is kept. A threshold at or past the top of the sum is answered (1, 1) outright, in the coarse lattice too: summing the masses leaves rounding that a threshold of a billion turns into gold.
- **Coarse lattice** (`Coarse`, `fallback_table`; `Tuning::coarse_cells`, finer in `Tuning::reference`; `max_coarse_work`): where the exact lattice is too big, the sum on a grid of that many points a standard deviation (a whole number of spans where there is a lattice). Each outcome's mass is split between the two grid points around it, which keeps every mean, so t F - mean F' is exactly the leftover of the gridded sum, whose only error is the variance the split adds; half of that times the density is taken off. Negligible mass at either end and everything past the largest threshold is dropped. Its plain probability is a difference of leftovers half a step either side and too coarse, so it is never used as a probability. Its work goes with the square of its points.
- **Edgeworth leftover** (`leftover` in `average.rs`): off the range the trivial answer, else the integral of the Edgeworth series, less span^2/12 times the density for the lattice, no lower than what any distribution of that mean has. A last resort for when the coarse lattice fails (no known case reaches it).

**The metric has no saddlepoint in it.** The saddlepoint's error leans optimistic and differs from state to state, so an optimizer scored by it prefers states for being misjudged; the coarse lattice's error is far smaller and has no lean.

**The plain probability** (`saddlepoint_approximation_wrapper`, for the histogram's graphs and chances and `stress`), in this order:

- **Trivial**: the threshold is outside the possible range.
- **Lattice** (`lattice` in `mean/lattice.rs`): the same exact lattice for one outcome. No coarse lattice here, whatever the tuning.
- **Saddlepoint** (`saddlepoint_approximation`): the Lugannani-Rice formula with the second continuity correction on the sum's lattice, which starts at the minimum and whose span is the gcd of the gaps of the upgrades with more than one outcome. It needs the root of the cumulant function's first derivative, found by `root_finder.rs`. `Tuning::root_finder` picks the finder, so a second one is a new enum variant and a match arm. Used where the product of outcome counts is past `max_brute_size` and the threshold is not near either end.
- **Edgeworth**: in place of the main formula near the mean, where that is singular. It takes the cumulants at zero, so it needs no root.
- **Conditioning**: when either of the last two is outside [0, 1], or the main formula's slope term (`1 + k4/8 - 5 k3^2/24` at the saddlepoint) is not positive, or the root finder finds no root, the sum is split on the outcomes of the upgrade with the most variance and each part goes through again. Two levels deep at most (`MAX_CONDITION_DEPTH`), then the value is clamped. Only a NaN there is a panic. With no root the Edgeworth value stands in, untrusted, so the split is what answers.
- **Brute force** (`brute.rs`): exact convolution with pruning, for one threshold, where the sum is small or the threshold is near either end.

**The approximations are the weak part, not their implementation.** They are asymptotic in the number of comparable upgrades, and the pity tap puts a large share of an upgrade's probability on its last outcome, so with one upgrade outweighing the rest they can be badly off in a probability with nothing to flag it, and their error leans optimistic. The lattices are what make that case right.

**The optimizer and what is reported use different evaluators.** The optimizer uses the default tuning. Every number a person or the arena judges by is `Tuning::reference()`, a finer coarse lattice: the histogram's `metric_per_plan` (`histogram.rs` swaps the tuning around `ui_average_gold_metric` only, so the distribution graphs and chances are the plain probability either way), the arena's `metric_ref` and its Monte Carlo checks.

`FLOAT_TOL` is smaller than the gap between f64s for large values, so thresholds are compared with `float_tol(value)`; with the bare constant a threshold equal to a reachable total counts as short of it.

The root finder's safeguards are empirical; treat changes there with care. Check a change anywhere on this path with `pnpm arena stress` and `pnpm arena verify` (`Arena.md`).

## The cost of a root

Evaluating the cumulants is the unit of cost: one pass over every outcome of every upgrade. Only the plain probability takes roots; the metric takes none. All of it is in `root_finder.rs` and `cumulants.rs`:

- **A last step shorter than `Tuning::taylor_skip` standard deviations is taken on the Taylor series** (`KsTuple::moved`) in place of the evaluation that would only confirm it. Never from cumulants that came off a series themselves.
  - **The limit is on where the series is read, not on the step that was proposed.** The proposed step can be short while the residual is large, and Newton on the series would then walk to where the series means nothing and report a root there.
  - What bounds the series is the range, not the standard deviation.
- **The cumulants at zero come from the supports**, not from an evaluation.
- **`one_cumulant` takes its moments in units of outcomes** where the support is evenly spaced: running sums of running sums give the factorial moments of the outcome count with one multiplication an outcome, and the cumulants are scaled by the gap after. The count starts from the light end of the tilt. Where that keeps too few digits of the variance (tilted hard, the weight at the far end, mean and variance a difference of two nearly equal numbers), the same sums are taken from the heavy end, each weight its own exponential.
- **An id twice on a tap counts once** (`get_extra_arr`), in the chance as in the cost.

`taylor_skip=0,max_memo_size=0` gives the plain root finding, which is how a doubt about these is settled.

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
- **A threshold the price does not change at is free.** `one_dimension_average_gold` skips its leftover evaluation, because it is multiplied by the price gap. `ValuationPlan::bands` already merges such entries wherever merging is sound, so in practice this fires only on the top entry, which has to stay for the base term. See `Domain model.md`.

## The gold distribution

`gold_distribution` (`gold_dist.rs`) gives the whole distribution of the gold one plan spends, not just its mean. **The metric's methods cannot give it**: the mean splits into one 1-D problem per material by linearity, but the spend is a sum of piecewise-linear functions of every material's total, and the totals move together (more taps cost more of everything at once), so it needs their joint distribution. It is sampled instead (`GOLD_SAMPLES` runs):

- **What a run uses comes from the evaluator's own supports**: the free-tap outcome k from `special_probs()`, the first k upgrades in the order at `access_min(true)`, a normal upgrade's outcome indexing `cost_dist[row].support`. Each row costs the same per outcome as the metric sees it, and `gold` (`materials.rs`, shared with the arena) is the function the metric takes the mean of.
- **The free-tap outcome and normal taps are stratified** (`tap_map_generator`): each outcome appears its expected number of times, then shuffled.
- **Advanced honing is played out** (`advanced_honing/sim.rs`), because the evaluator keeps only the three marginals and the joint matters: taking taps, juice and scroll at one shared quantile is badly off the CDF on grace-scroll strategies. The runs of each strategy are cached in a thread local for the life of the worker (`ADV_RUNS`), from a seed of their own, and shuffled per use; playing them out is most of the time otherwise.
- **The ends are deterministic**, as on the material graphs: luckiest is the most free taps with a live chance and every upgrade on its first live outcome, pity the fewest and the last. Their odds are log10 (a product over a hundred upgrades underflows) and the larger of that one run's chance and the sampled fraction, since a whole region can spend the same as an end (0 gold, where the stock covers it). An advanced upgrade's juice and scroll are taken at the same end as its taps there.
- **Seeded** (`SmallRng`, 0), so the same state draws the same curve.

It costs several times a histogram, so the frontend asks for it only for a final state (`Frontend/Wasm contract.md`). The arena test `gold_distribution_matches_monte_carlo` checks its mean against the metric and its curve against `monte_carlo::simulate`, which plays free taps out by the game rules.

## Leftovers

`metric_type` exists but only the average-gold metric is live (`metric_router`; 0 is unreachable). `success_prob.rs` is a shelved metric, kept because the histogram reuses `one_dimension_prob`. `bound.rs` is an abandoned bounding attempt that now only supplies the root finder's initial guess. `helpers.rs` holds `my_dbg!` and leftover helpers from an earlier design.
