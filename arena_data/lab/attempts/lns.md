# lns: large neighbourhood search (destroy and repair)

Code: `crates/core/src/optimizer/lab/lns.rs` (one file, registered in `lab/mod.rs`).

## The algorithm as built

- **Work** is counted in evaluations (an evaluation is 1, a shadow price call 2, a response its `work(response_steps)`). Budget `lab_effort * (18000 + 2400 n)`; the search stops early once 45% of the budget has gone by without the best moving by 1e-6 of it.
- **Candidates**: the given state, every carried plan, and the given with empty normal plans, each put on the rules first (`make_valid`: `snap_to_block` only where an upgrade breaks the grid, advanced slots, order, chest states); the best is the start and the floor.
- **Start**: free tap order candidates (greedy build, below; ratio of what a free tap saves at shadow prices to `special_cost / base_chance`, raw and with each piece in level order); up to 3 sweeps of best responses over every upgrade (`Response::of` + `better`; advanced upgrades by a full scan of each slot); one joint chest move per stack with a choice; the order built again and one sweep.
- **Order build** (`build_order`): from a place on, each place in turn takes the upgrade that does best there (true metric), while the place is reached by the free taps. Full at the start, 8 tries a place inside moves. The evaluator's own cleaning keeps the order valid; it is never forced into level order (free tapping only the high levels of a piece is often best).
- **Destroy and repair arms** (13, adaptive weights): the freed set is two upgrades (same piece type likelier), the advanced upgrades, a piece type, a piece, or the users of a material whose bands vary (row by price times use), at most 10. Each is rebuilt from
  - **Empty**: plans emptied, then sweeps;
  - **Price**: shadow prices times lognormal noise (0.35), every freed upgrade its `Response::linear` best, then sweeps;
  - **Ladder** (the joint one): each freed normal upgrade gets candidates, its linear best at the shadow prices with the row (or every juice/book) scaled by 0, 0.3, 0.6, 1, 1.6, 3; the best pair is enumerated where two are freed, else every upgrade at one scale together, then each in turn takes its best candidate (3 rounds); advanced ones get 12 tries of moving a slot's index from one to another (1, 2, 3, 5 steps);
  - then best responses in turn over the set until none moves (3 sweeps at most).
  - Order arms: a window of 1-3 places where free taps reach plus one from anywhere, put back one at a time at the best place; and the order rebuilt from a random reached place on. Moved upgrades get one sweep.
  - Chest arm: one stack, each option all on it with the order rebuilt (where it gives free taps) and the users of all its options' materials (plus the reached upgrades for leaps) swept once, the best kept; then halving transfers between options with plans fixed; then 2 sweeps of the users.
- **Acceptance**: record-to-record, a state within `3e-4 * |best|` of the best (shrinking to 0 over the budget) is accepted, else the move is undone from an essence; back to the best after 20 moves without one.
- **Weights**: `w = 0.8 w + 0.2 reward / relative work`, reward 4 new best, 2 better than now, 0.3 accepted worse, floor 0.05.
- **End**: one sweep over every upgrade from the best, the best returned.

## Dev numbers (quick_dev, seed 0, against `current`, `dev_lns.sqlite`)

Session 16 (final code):

| effort | diff cold | diff chained | hit cold | x cpu |
| --- | --- | --- | --- | --- |
| 0.5 | -0.0226 | -0.0172 | 87.7% | 0.56 |
| 1 | -0.0142 | -0.0115 | 89.4% | 1.11 |
| 2 | -0.0098 | -0.0068 | 93.1% | 2.04 |

No invalid states, no panics. `current` itself hits 96.3%. Session 14 is the same code with the dev switches still in (same numbers, effort 1).

**Joint repair vs one at a time** (session 15, effort 1, temporary `LNS_SOLO` switch since removed: no destroy, no price or ladder start, each freed upgrade one response on its own; order and chest arms as they are): diff -0.0556, hit 75.9%, x 0.92, against -0.0142, 89.4%, 1.05. The joint rebuild is most of what earns. The ladder and advanced transfers alone took effort 1 from -0.023 to -0.014 (sessions 13 to 14).

Pilot sessions 1-8, 10-12 are the development history on `pilot` (2 seeds).

## Where it is weak

- **Free tap order basins**: the worst payloads (`chain_0017_t4_n14` -1.3, `single_0087_t4_n9` -0.58, week_hone) differ from `current` only or mostly in the order; greedy place-by-place building and insertion windows miss orders that move several upgrades at once.
- Chains (generated, with free taps) are the weakest kind (-0.08); b10 and the real characters' pages are near even but take 1.6x the CPU there, chains and weeks 0.6x: the budget is not shaped like `current`'s.
- The joint chest move costs a sweep per option; stacks with many options are only sampled.
- Advanced upgrades are coordinate scans plus pair transfers, no joint enumeration.

## What I would try next

- An order move that swaps a whole set in and out of the reached prefix (knapsack on the free tap value) and rebuilds the plans of everything whose skip chance changed.
- Shaping the budget by evaluation cost rather than upgrade count, and spending the stall slack on restarts from the second best.
- Ladder candidates for advanced upgrades from linear prices, so they join the pair enumeration.
