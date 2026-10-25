# Evaluation pipeline: what not to try

Warnings for `Evaluation pipeline.md`: things that failed or were ruled out. Read before trying something new on the evaluator.

The overnight test of the saddlepoint's replacements is archived with its driver, results and conclusions in `ignored/core experiments/FINDINGS.md` at the repository root (gitignored): everything tried against the approximations' weakness with one dominant upgrade, and the code of the dropped alternatives (the size-biased pair, the split). Earlier speed measurements are in `Arena tried.md`.

## The metric

- Don't put a saddlepoint back in the metric (the old "direct" leftover): its error leans optimistic and differs by state, so the optimizer prefers misjudged states. The coarse lattice has a worst error of 1.5e-4 of flow against 3.5e-3, for 1.3 times the time per evaluation, and plans came out equal or better (13k gold on `saddlepoint_out_of_range_t4_n27`).
- Don't expect the Edgeworth leftover to be exercised: the coarse lattice never failed over 2,631 solves (98 million evaluations).
- Don't sum the lattice's masses up to a threshold at or past the top: rounding leaves a fraction of a gold per material at a threshold of a billion, which summed over juices showed as a gain from spending nothing ("-0" / "-1" gold). Answer (1, 1) outright.

## The coarse lattice

- Don't pick the step per sum from a work budget: it beat a fixed `coarse_cells` per unit of time at high accuracy, but no budget was as fast as the default without falling back, and the fallback brought its errors with it.
- Don't add a mirror (the grid of max - S): it halves a lone threshold near the top, but a material's thresholds span both sides and the tails already bound the grid, so a whole evaluation was unchanged.
- Don't use brute force in place of the coarse lattice for small sums: same accuracy, 2% slower.

## No root and `one_cumulant`

- Don't treat "no root" as the root finder's fault: the cause was `one_cumulant` counting from the light end of the tilt. Tilted hard, mean and variance are a difference of two nearly equal numbers (off by 24,000 in K' at theta -1.4), so no point had a residual under tolerance. The heavy-end path fixes it and is free where it is not needed. Regression: `arena_data/scenarios/regressions/no_root_week_inc_0022`.
- Don't let an id count twice in the chance while it counts once in the cost (`get_extra_arr`): such a plan scores far above any real one, and an optimizer experiment found it.

## The root finder

- Don't hold the walk on the Taylor series to the same limit as the proposed step: Newton walked off the series and reported false roots, which killed 3 jobs in 7600. Bound where the series is read.
- Don't bound the series by standard deviations (0.1 failed on a sum of one bimodal upgrade); bound it by the range.

## Where the time goes

- Don't clone distributions on every advanced-honing memo hit or allocate per branch: that was 59% of a solve (75 to 96% with advanced upgrades). The one-buffer layout of `compute_adv_dist_wrapper` fixed it.
- Don't ship every advanced start position in the wasm (some 8,400 of them, 361 strategies each): too large. Only the fresh ones (0.6 MB) are built in.
- Not done: sharing one memo between the strategies of a start is another 2 to 3 times, at 60 to 160 MB while it runs. A plain-sums-against-a-table-of-powers cumulant loop that a compiler could vectorise is untried.

## The gold distribution

- Don't look for an exact route: the lattices are per material and the spend needs the joint of every material's total.
- Don't take an advanced upgrade's taps, juice and scroll at one shared quantile of their marginals (comonotone): up to 0.106 off the CDF, worst on grace scroll strategies. Play it out (`advanced_honing/sim.rs`).
- Don't play out the advanced runs afresh per call or use `StdRng`: 96 ms native on the dummy scenario against 37 ms with cached runs and `SmallRng`.
- Don't compute it on every intermediate result: it costs five to six times a histogram in wasm, so it is final-state only. Untried speed-ups: looping over runs on the outside to keep totals in cache, and 50k runs (half the time, 0.6% off over the curve).
- Don't use plain chances for the pity odds: the luckiest underflowed to 0 on 96 upgrades. They are log10.
- The site once drew a gold graph from its own Monte Carlo (removed in October 2025); this one replaces it.
