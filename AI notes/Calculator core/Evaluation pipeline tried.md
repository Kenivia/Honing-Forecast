# Evaluation pipeline: what was tried

History behind `Evaluation pipeline.md`: measurements, bugs that were fixed, and what was dropped. Only read this when trying something new on the evaluator. All of it is October 2026 unless dated otherwise.

The overnight test of the saddlepoint's replacements is archived with its driver, results and conclusions in `ignored/core experiments/FINDINGS.md` at the repository root (gitignored). It has everything tried against the approximations' weakness with one dominant upgrade, and the code of the alternatives that were dropped (the size-biased pair, the split). Its reference was `coarse_cells=512`, within 1e-8 of flow of `max_lattice_work=2e9` on `pilot` and 17 times faster. What was measured against the code before the evaluator's speed-up is in `Arena tried.md`.

## The metric's saddlepoint, and why it went

The metric used to take the leftover from one saddlepoint (`leftover`, called "direct"), with brute force near the ends and the coarse lattice at 16 points where it found no root. Against that direct route, c8:

- one evaluation 1.3 times as long, worst error 1.5e-4 of flow against 3.5e-3, mean 1e-5 against 5e-5;
- solves 1.3 times as long on `pilot` and `nh_quick` (one job), 0.78 times on 175 upgrades (where the optimizer's own work dominates), 1.33 to 1.52 times in the browser;
- plans under the reference the same within noise on `nh_dev`, `weekly_dev`, `chests` and `stress`, and some 13k gold better on `saddlepoint_out_of_range_t4_n27` (3 seeds), the case of one dominant upgrade the saddlepoint misjudges;
- the optimizer's own number off the reference by 9.4e-5 of flow at worst, against 1.9e-3.

The reference (c64) is 1e-6 of flow off on average and 1e-5 at worst; one evaluation at 64 points is 5 times c8's.

The Edgeworth leftover was never reached: over 2,631 solves at c8 (98 million state evaluations, 1 billion coarse lattices) the coarse lattice never failed.

## Tried on the coarse lattice and dropped

None was worth its code.

- A step chosen per sum from an estimate of the work, as fine as a budget of multiplications an outcome a threshold allows, falling back below some points a standard deviation. Per unit of time it beat a fixed `coarse_cells` once accuracy mattered (6.3 times the old direct route at a worst of 2.7e-6 of flow, against c64 at 7.0 and 6.8e-6), but no budget was as fast as c8 without falling back and bringing the fallback's errors with it.
- A mirror, the grid of max - S where the thresholds reach less from the top. It halves the work of a lone threshold near the top, but a material's thresholds span both sides and the 1e-16 tails already bound the grid, so a whole evaluation was unchanged. A mirror of the exact lattice would admit nothing more, since its size check is on the whole range.
- Brute force in place of the coarse lattice for small sums: same accuracy, 2% slower.

## No root, and `one_cumulant`

- No root used to be a panic. A weekly chain of the arena hit it under every optimizer (`arena_data/scenarios/regressions/no_root_week_inc_0022`). The Edgeworth value now stands in; nothing is known to reach it since the cause below was fixed.
- **The cause was `one_cumulant`, not the root finder.** The evenly spaced path counts q from the end the tilt takes the weight off. Tilted hard, the weight sits at the far end, and the mean and variance come out as the difference of two nearly equal numbers: on that payload a juice of 4 upgrades (steps of 20 to 50) with 23 owned, whose biased search ran to theta near -1, where one upgrade's mean came out -2.7e-12 against 3.3e-21 and the biased K' was off by 1 at theta -1 and by 24,000 at -1.4, jumping by 0.08 within 1e-8 of theta. So there was no point with a residual under `y_value_tol`. The heavy-end path fixed it; it is rare: 120 states of `nh_quick` evaluate to the same bits and as fast as before, and `stress` on it gives the same table.

## The root finder

- `taylor_skip` at 0.001: the variance read off the series was within 2e-8 of the true one over 6 million steps.
- Before the walk on the series was held to the same limit as the step, Newton walked off the series and reported false roots: that killed 3 jobs in 7600.
- 0.1 standard deviations failed on a sum of one bimodal upgrade, which is why the range bounds the series and not the standard deviation.
- **An id twice on a tap used to count twice in the chance** and once in the cost, so such a plan scored far above any real one. No optimizer has made one but an experiment of October 2026 (`Optimizer tried.md`).

## Where the time went (October 2026)

After the speed-up a solve was 2.8 times faster on the `pilot` set and 5.5 times on `stress`. Of what was left, on the explore singles:

- **The advanced honing distributions were 59% of it**, and 75 to 96% of a solve with advanced upgrades: 300 to 1500 cache misses in `compute_adv_dist_wrapper`, 5 to 11 ms each. A miss became 1 to 2 ms. The cost was copying: every memo hit cloned three vectors and every branch allocated three more; the one-buffer layout fixed it.
  - Sharing one memo between the strategies of a start is another 2 to 3 times, at 60 to 160 MB while it runs. Not done.
  - Every start position there is (some 8,400 of them, 361 strategies each) is too much to ship. The two fresh ones with double balls, 722 distributions, are 0.6 MB, which is what the wasm carries.
- Of the rest: two fifths cumulant evaluations, an eighth the lattice, and the remainder spread thin over rebuilding supports, the memo's keys, `shape`, brute force and the free-tap probabilities.
- **The annealer itself, outside the evaluator, was 29% of the `pilot` set** and 19% of `explore` (`decide_s`, `Arena.md`), and 12% of `pilot` after its own clean-up (`Optimizer tried.md`, "What a move costs").
- The cumulant loop is bound by its chain of running sums, which a compiler cannot vectorise. Plain sums against a table of powers could be, at more work an outcome in scalar code. Not tried.
