# price: price coordination

`crates/core/src/optimizer/lab/price.rs`, one file, registered as `price` in `lab/mod.rs`.

## The algorithm as built

Work is counted in evaluations: each evaluation 1 + 0.05 per upgrade it recomputes, a response its own `work(response_steps)` (at least 0.1), a shadow price call 2. Budget `lab_effort * (28000 + 2600 n)`.

1. **Candidates.** The given state put on the grid by `Model::filled` alone and after `snap_to_block`, each carried plan the same two ways, and the state that uses nothing (always valid). Each is checked with `check::invalid` before it is evaluated; the best is the incumbent. Every evaluated state competes for the best kept (an essence), so nothing worse than a candidate is returned.
2. **Free tap order.** A greedy order by leapstone efficiency (expected cost at shadow prices, from `Response::linear(..).score`, times `base_chance / special_cost`), among the lowest remaining level of each piece; kept if it evaluates better.
3. **Price phase (25%).** Trajectories from the incumbent: every upgrade takes its best plan at prices p, then p moves toward the shadow prices of that state with step 1/(k+2), clamped to each row's band price range, up to 6 steps or until no plan moves. Normal upgrades by `Response::linear(..).better`; advanced upgrades by the plan of least priced mean use, from a table of the mean use of every row by each of the 19^2 plans (set the plan, `update_this_prob_dist` + `update_this_support`, read `Support::mean`; no evaluation). Even trajectories also open every chest stack all on its option worth most at p, a free-tap leapstone priced by a finite difference of the metric in the budget. Trajectories after the second start from randomly perturbed prices (kinked rows uniform in their range, others times e^±0.3); without any kinked row only four are run. Then a coordinate step on the best: each kinked row's price halfway toward either end, kept where the priced state evaluates better.
4. **Primal polish** of the best start (35% share), then the next `round(1 + effort)` starts (20% between them). A round: true responses (`Response::of(..).better`) on every normal upgrade in random order, accepted if the metric gains; each advanced slot over all 19 indices (first round), then every plan within 2 in each slot; free tap order insertions into the reached positions (moves where the order alone loses under 0.2% retried with the moved upgrade replanned, `reach + 2` such a call); chest moves of all, a third, a ninth ... of an option's count to another. Rounds use the default `response_joint` until one gains nothing, then one more with a joint limit of 500 000.
5. **Kicks** from the best until 90% of the budget or `30 + 40 effort` kicks in a row without gain: a random stack reopened (15%), a window of the order shuffled (15%), an advanced upgrade set at random (15%), an exchange (two upgrades sharing a kinked row, one repriced cheap and one dear on it, 20%), all normal upgrades of a piece type taking their true response at once (15%), else a piece type (or all) repriced at perturbed shadow prices. Each is polished without the deep round.
6. **Final** deep polish of the best with what is left; the best re-checked and evaluated.

## Dev numbers (quick_dev, seed 0, against `current` of session 6)

Session 12 (all three efforts), base from session 6. No invalid state, no panic, on any run (2094 runs).

| effort | diff cold | diff chained | hit cold | hit chained | x |
|---|---|---|---|---|---|
| 0.5 | -0.0015 | +0.0000 | 90.8 | 93.1 | 0.57 |
| 1 | -0.0022 | -0.0009 | 92.6 | 93.7 | 0.97 |
| 2 | +0.0006 | +0.0013 | 95.1 | 96.6 | 1.71 |

Pilot (session 4/5, earlier build): -0.005 cold, +0.0001 chained at x 0.24. Sessions 7-11 are the tuning steps (budget, advanced pricing, deep joint).

## What earned

- Pricing the free-tap leapstone in the chest choice: on chain_1033 the price phase alone went from 86M to 134M (the polished best), where without it only the polish got there.
- Advanced plans by price: chain_0001_t4_n18 (11 advanced upgrades sharing the juice) is a joint allocation no slot-by-slot search leaves; pricing them all at once found it.
- The deep joint limit in the polish: week_hone_0004 (long armour upgrades, two pools) gained ~0.3% of flow.
- The exchange kick: chain_1044 (scarce juice between two upgrades).

## Where it is weak

- The price phase helps little where the start is the empty plan and the shadow prices there are a poor guide (chain_0017_t4_n14, single_3300): the linear responses stay near empty and the polish does all the work. 0.5 beating 1 on cold is that payload's noise (-0.62 at 1).
- Long upgrades with two or three pools (single_0002_t4_n3, len 150): `better` falls back to `improve`, which is local; the deep round is too costly to run often.
- week_hone_0004 steps 1-3 and single_3044_serca_n13 stay 0.1% behind at every effort.
- Kicks are blind; most fail. The work counter is a guess at CPU (x tracks it overall, not per payload).

## What I would try next

- Run the trajectories from the polished best too (its shadow prices are a far better start than the empty plan's), and per-upgrade prices (a Lagrangian per upgrade and row, moved by its own use against its share of the stock).
- Price free-tap order positions (a free tap's value per leapstone at the current prices) to choose the order inside the price loop rather than by insertions.
- Spend the deep joint only on upgrades whose response moved in the last kicks.
