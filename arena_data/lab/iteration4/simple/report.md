# Simple strategy: Maxroll's rule

## Verdict

Maxroll's rule (each normal upgrade takes the plan that is best with every material at its buy price) is exact where everything is bought at market, and far off on real cases: **8.4% of flow mean, 9% hit**, against 0.010% / 87% for the current default `solve`, at about x0.01 its CPU. The loss is not the rule's arithmetic but what it ignores: owned stock (including chests) whose leftovers are worthless, chest choices, and advanced honing (left as given).

## What was built

`algo=simple` (`crates/core/src/optimizer/lab/simple.rs`, snapshot `sim_a`): prices are each row's last band price (buy price) in the optimizer's plan; every normal upgrade takes `Response::linear` at those prices, the joint search over its pools at any size (`better` with no step cap); then `greedy_order`; advanced plans and chests stay as given. One evaluation.

Maxroll's own calculator (read from its client code, 2026-11-16): a per-piece expected-gold DP over (failed taps, artisan) with every material at its user price; its "material values" are the per-tap break-even prices of that DP. With no owned stock and no free taps this is the same problem as `Response::linear`.

## Evidence

**Exact where it should be.** 30 real3 singles without advanced honing (4+ upgrades), stock, chests and free taps removed (`scripts/zeroed.py`): `simple` and `solve` reach the same metric on all 30, at x0.08 the CPU.

**Real cases** (batch `a`, cold `real3_dev` + `real3_test`, 1,662 singles, 1 seed; `scripts/simple.py`; % of flow against the best known):

| group | n | default `solve` mean / hit | `simple` mean / p50 / p90 / hit | `max_iters=0` mean / hit |
|---|---|---|---|---|
| all | 1662 | 0.0097 / 87.1 | 8.42 / 7.23 / 17.9 / 9.1 | 0.140 / 69.0 |
| streak 1 / 10 | 831 each | 0.008 / 0.012 | 8.47 / 8.38 | 0.136 / 0.145 |
| no advanced | 1232 | 0.0042 / 91.2 | 5.69 / 4.27 / 13.5 / 11.4 | 0.111 / 73.3 |
| has advanced | 430 | 0.0255 / 75.3 | 16.3 / 13.7 / 29.4 / 2.8 | 0.225 / 56.5 |
| no chests | 418 | 0.0048 / 90.2 | 9.19 / 4.54 / 27.5 / 23.2 | 0.043 / 70.1 |
| no advanced, no chests | 266 | 0.0059 / 91.7 | 3.24 / 0.66 / 11.0 / 35.7 | 0.012 / 76.7 |

**Why.** The real cases value leftover bound and roster stock at nothing (`Worthless, Worthless, TaxedSell`):

- Owned base materials make taps nearly free, so `solve` uses little juice; `simple` prices taps at market and buys juice and books. `r3_7a97b994_305802_f0` (80k Red, 315k Blue owned, no juice): `simple` puts 38 juices on, `solve` none; 18–20% of flow.
- Owned juice, mostly from roster-bound chests, costs nothing to use; `simple` charges its market price and leaves it.
- Advanced upgrades stay on strategy 0 (`r3_ca67b754_c104ea_f0`: 54% of flow).
- Chest choices are not made.

## Not done

- The rule with band-aware prices (that is the shadow start, `shadow_prices`), or with advanced honing and chests chosen.
- Maxroll's state-dependent advanced honing policy.
