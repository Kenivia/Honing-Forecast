# State space: what not to try

Warnings for `State space.md`. Read before trying something new on the shape of a plan. `Arena.md`, "The state space", has the commands.

## The landscape

- Don't hill-climb the streak length as if the metric were smooth: it is a sawtooth whose peaks are the first length of each new pity (20,661 interior peaks on `chain_0011_t4_n7`), and with books or part-owned juice up to three quarters of the peaks are not at a pity but where an owned amount runs out instead.
- Don't chase the default tuning's error as noise: it is smooth between neighbouring cells (1 to 130 gold sd) even though far off over a grid (13 to 2300 gold against 100 times the lattice work), and the best cell is the same on 6 upgrades of 7.

## The streak restriction

- Don't widen the search to every plan to gain accuracy: on 90 exhaustively solved small cases 82 have a streak plan as best, the 8 others lose 0.0001% to 0.24% of flow. On 94 `explore` singles every plan beats the streaks by 0.10% at most. Reach the rest with the response instead.
- The largest thing the streaks miss is a front streak that skips a tap (`.0.00000000...`), 0.24% of flow.
- A best plan can be a block in the middle (`lone_022`: Red runs out on tap 8, the juices belong on taps 8 and 9; sliding them to the front costs 5,140 gold, to the back 138). Evaluator and `monte_carlo::exact_single` agree on it, so it is real, not evaluator error. A tie in the metric (`lone_029`, 0.1 gold) is not a plan to chase.

## The response

- Don't enumerate plans, and don't evaluate a table entry as its own material evaluation: the first response did and took 0.01 to 0.6 s. One pass over the sum of the other upgrades fills a whole table.
- Don't start the response from nothing and expect the free-tap order to come out: on small cases 11 of its 13 misses (up to 14%) were the order. It does not choose it.
- Don't trust the table where the lattice does not answer on long upgrades: off by sd 170 to 1200 gold on the 219 tap ones.
- Don't read `State::compute_streak_info`'s back streak from the front end: it was read from the wrong end once, which silently skewed results taken before the fix (`Optimizer tried.md`).
