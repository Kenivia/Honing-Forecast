# Optimizer lab: archetypes of batch 1

Each attempt takes one of these as its overarching approach, on the problem of `brief.md`. The design inside it is the attempt's own: representation, moves, schedules and constants. Every attempt must handle all four kinds of decision (normal plans, advanced plans, the free tap order, chests), even if some are handled simply, and must use the given and carried plans as candidates.

## price: price coordination (Lagrangian / dual decomposition)

The upgrades interact only through the shared stocks. So coordinate them by prices: give every material a price, let every normal upgrade choose its best plan alone under those prices (`Response::linear`, no evaluation), and search over the price vector rather than over plans, adjusting prices from what the plans then use against what is owned (subgradient, a fixed point on marginal values, bisection per material, or several price trajectories from different starts). The true metric judges the plans a price vector gives. Prices can never express everything (a stock that runs out on some outcomes and not others is not linear), so finish with a primal phase: true-metric responses (`Response::of`), and handle the free tap order, advanced upgrades and chests by their own steps inside or between price rounds.

## surrogate: a cheap moment surrogate, verified by the true metric

Each upgrade's use of each material is independent of the others', so the mean, variance and higher cumulants of every material's total are sums over upgrades, and over the free-tap outcomes a mixture. From those, the expected gold of a material (the expected leftover past each band threshold) can be approximated in microseconds, for example with a normal or a shifted gamma approximation, and updated incrementally when one upgrade changes. Search the surrogate far harder than the true evaluator allows (many restarts, exhaustive local moves), and use the true metric sparingly: to check the best few, to calibrate the surrogate at the incumbent (a correction term, trust regions), and to choose between candidates. Where the surrogate and the truth disagree is the interesting part; measure it.

## memetic: a population with structured recombination and local improvement

Keep a population of complete plans. Children combine parents along the problem's structure (whole upgrades, whole piece types that share juices and books, chest stacks), the free tap order by an order-preserving permutation crossover; mutations change streaks, orders and chests. Each child is improved by a short local search before it competes (best responses on a few upgrades, order moves, chest moves). Keep the population diverse (distance, niching or replacement rules) and the best always. The value of the population is in recombining good parts found in different places; check that recombination, not the local search alone, is what earns.

## cem: cross-entropy method / estimation of distribution

Keep a probability model over the decisions instead of states: per normal upgrade and pool, a distribution over streak lengths and ids (or over blocks taken); for the free tap order, a model of permutations (a Plackett-Luce model or a position matrix); per chest stack, a distribution over how its count is split; per advanced upgrade, over its indices. Each round, sample a batch, evaluate (cheaply, where samples share upgrades), refit the model to the best fraction with smoothing, and keep the best ever seen. Mind dependencies between upgrades that share a stock (a model of independent parts may need a joint factor, or a polish of each sample). Finish with best responses on the best.

## lns: large neighbourhood search (destroy and repair)

From the best state, repeatedly free a structured part of it and rebuild that part jointly with the rest fixed: all upgrades of a piece type (they share juices and books), the upgrades that use a scarce material, a window of the free tap order, one chest stack together with the upgrades that use what it gives. The repair is where the strength is: rebuild the freed upgrades together (best responses in turn until none moves, a small joint enumeration of block plans, a price-guided rebuild), not one at a time from scratch. Accept by a criterion that can leave a local optimum (late acceptance, threshold acceptance, record-to-record), and weigh the neighbourhoods adaptively by what they have gained.
