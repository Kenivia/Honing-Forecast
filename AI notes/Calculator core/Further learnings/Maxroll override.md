# Maxroll override (dead code)

`optimizer/maxroll.rs` (payload `maxroll`, lab `algo=maxroll`) plans like maxroll.gg's upgrade calculator, for comparison only; evaluation stays the band metric. The frontend options (override radios, special-leap checkbox, annotation, Introduction demo) were removed on 2026-11-18 because it cannot represent Maxroll accurately. The Rust, the payload field and the store fields are kept; don't re-add the UI unless asked. Lab `algo=simple` (`lab/simple.rs`) is the plainer rule. Numbers are in `Optimizer lab results.md` (prototypes, `simple`) and `arena_data/lab/iteration4/simple/report.md`.

## Checking against the site

On maxroll.gg's calculator: select "Optimal materials", press Recalculate, tick "Show material values", expand a row for per-tap values. On 2026-11-18 its expected gold matched an independent DP to 1 gold (Serca weapon +20 with and without juice, T4 weapon +12 with books), and its data tables match ours. Its logic is a client-side wasm DP (decompiled).

## How the site differs from our model

- Silver is not valued.
- Partial juice on the last juiced tap is allowed (about 0.007% gain; our all-or-nothing cannot express it).
- A free tap's value per special leapstone is base chance × expected remaining gold (with juice and books, without unlock) / special cost. That is not `greedy_order` (which drops juice cost and picks by the band metric), so the override orders free taps by the site's value.

## What the override adds

Choices made for it: own buy prices; silver priced like any material; a Maxroll pool worked out with the other pool held as its override, independent of the order the options were picked in. Owned juice and books (a departure from the site): charged taps ranked by gold saved a unit if reached, books included, only bound and roster-bound stock as free, worst case affordable; the plan is those taps joined with the buy-price plan, values from one pass. Reach-weighted ranking was slightly better (4.05% against 4.21% of flow on `real3na`), but if-reached is easier to explain in one line. Counting tradable stock as free loses (`Optimizer tried.md`).
