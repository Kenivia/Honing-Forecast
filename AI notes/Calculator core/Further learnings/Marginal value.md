# Marginal value (shelved)

The calculator's "Est. gold value" column: for each juice or book some upgrade can take (with a market price), the market price under which the optimizer would use more of it. Shelved since 2026-10-21: the column is commented out in `MaterialDist.vue` and `start_marginal` (`CharWorkerUtils.ts`) is not called. The Rust (`ui/marginal.rs`, `use_more_prices`) and the wasm op (`marginal_value_wrapper`) are still there.

## How it works

The plan is polished once (`polish_only`), then for each row the price is searched downwards from market over a fixed `POLISHES`, each a polish of that plan with the row's market and taxed prices scaled to the trial price:

- away from market in steps of a tenth of the price's order of magnitude, doubling, until a price uses more;
- then bisected for the polishes left; the answer is the middle of the bracket;
- never found: the lowest price tried, shown as "<"; "-" if that reached 0.

"Uses more" is the row's expected usage (`avg_used`) rising past a relative 1e-6.

## What not to try

- Don't score every row as one bump in plan 0 (gross spend): for a juice or book it is about market price times the chance of running short, never over market, so it cannot say whether buying is worth it. Re-polishing under the user's plan while scoring in plan 0 also gave negative values (to -10k per book on `single_3048_serca_n37`).
- Don't average over purchases (re-polish with the row at 1000x market and divide what that costs the other rows by the units bought): with a prohibitive price the fixed plan only uses juice where even the worst case stays within stock, so purchases are credited with what flexible use of the stock saves; as stock grew the value went from about 5% of market to about 3x and back down.
- Don't search a range both ways: 16% of a solve, most of it upward searches on unused rows that can never move.
- Don't expect re-polishing at a plan's own price to move it (82 rows never did), which is why the polish is cheap.

## Where it was left

Full solves with buying disabled (`crates/arena/examples/no_buy.rs`: the row's market price times 1000, a full `solve` at stock x and x + d, scored in the optimizer's plan at the reference tuning, (V(x + d) - V(x)) / d; `SWEEP=name:label:points:spacing` sweeps the stock, `ENVELOPE=...` re-solves each point carried from its neighbours'):

- Value mostly falls smoothly with stock, with a small rise at the very start. Carrying x's plan into the x + d solve changed nothing.
- Two kinds of blip: real ones, where owning more makes a better plan possible, and optimizer misses (a neighbour's plan was 511 and 915 gold better).
- Don't use d = 1: seed spread is 15 to 45% on the larger scenarios, under 10% for most rows at d = 4. A larger d averages over later units that are worth less, which flipped buy against market for one row. Cost is one full solve per row per d, 4 to 5 s each on the large scenarios.
- Not yet tried: d = 4 with a two-way carry (x cold, x + d from x, x again from x + d's plan, keep the better x), which removed the misses in the envelope test; and deciding whether the column means the next unit or the next few on average.
