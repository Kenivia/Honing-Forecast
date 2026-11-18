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

## The calculator's Maxroll override with owned stock (batch `b`, 2026-11-18)

`algo=maxroll` (`lab/maxroll.rs`, snapshot `sim_b`) runs the calculator's override (`optimizer/maxroll.rs`) with juice, books and the free tap order all on. Owned juice and books go to the charged taps of the buy price plans with the most gold saved a unit, as far as the stock covers every tap given them; the plan is those taps and the buy price plan's. Switches: `ifreached` (rank by gold saved if the tap is reached, Maxroll's per-tap value, instead of reach × that), `nostock` (owned stock ignored, the rule before), `allstock` (tradable stock free as well as bound and roster). Default = reach-weighted, bound + roster.

Set `real3na_dev`/`_test` (`scripts/strip_adv.py`): the real3 cold singles at streak 1 with advanced honing removed, 724 cases (108 had any). Best known from default `solve` at 3 seeds (plus earlier runs of identical payloads). `scripts/noadv.py`; gap in % of flow, `diff` = gain over `nostock`, `+`/`-` payloads better/worse by 0.01%.

| variant | mean | p50 | p90 | worst | hit % | diff | + | − |
|---|---|---|---|---|---|---|---|---|
| ignore owned (`nostock`) | 5.77 | 4.66 | 13.45 | 28.0 | 12.7 | | | |
| **reach, bound + roster (default)** | **4.05** | **2.44** | **10.22** | **28.0** | **14.6** | **+1.72** | **403** | **1** |
| if reached, bound + roster | 4.21 | 2.58 | 10.72 | 28.0 | 14.6 | +1.55 | 384 | 0 |
| reach, all stock | 4.36 | 2.60 | 10.68 | 52.3 | 13.5 | +1.41 | 404 | 70 |
| if reached, all stock | 4.51 | 2.69 | 10.98 | 52.3 | 13.5 | +1.26 | 399 | 61 |
| `solve` default | 0.005 | 0 | 0.009 | 0.41 | 91.3 | | | |

- The default is best in every group. Its gain is mostly where bound juice is 10% of flow or more (390 cases: 6.38 → 3.48).
- Treating tradable stock as free loses on 70 cases (worst 52% of flow): the metric values a used tradable unit at its sell price.
- What is left (4% of flow mean) is chests (557 of 724 have them; the rule leaves them as given) and owned base materials, which the rule prices at market.
- Shipped in the calculator: if reached, bound + roster (the user's pick, 2026-11-18): 0.17% of flow behind reach-weighted, but one line to explain in the frontend. `algo=maxroll`'s default stays reach-weighted, so `sim_b`'s rows keep their meaning.
- `allstock` was removed afterwards; its rows in `sim_b` stay as evidence.
