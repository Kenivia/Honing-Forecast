# Optimizer: what was tried

History behind `Optimizer.md`: the arena measurements behind each default, bugs that were fixed, and what was taken out. Only read this when trying something new on the optimizer. All of it is October 2026. `Arena.md` has the sets, and `Arena tried.md` the baseline `v0` (one chain of 24,000 and a pass of responses at the end) and the snapshot labels.

## The round that made the current shape

Hit is within 1e-4 of the best known as a share of flow, near within 1e-3, spread the best seed less the worst, time is CPU over `v0`'s:

| `nh_dev`, 158 payloads, 4 seeds | hit | near | mean gap | spread | time |
| --- | --- | --- | --- | --- | --- |
| `v0` | 78.5% | 90.1% | 0.055% | 0.18% | 1 |
| `v0` with 48,000 evaluations | 81.5% | 92.3% | 0.038% | 0.086% | 2.0 |
| the polishes at the end, 24,000 | 82.3% | | 0.035% | 0.038% | 1.0 |
| and the order's fixes (floor, reach, greedy start) | 82.3% | 92.1% | 0.028% | 0.026% | 1.0 |
| 48,000 in 4 chains | 88.9% | 97.2% | 0.010% | 0.011% | 2.2 |
| and mixed responses | 92.4% | 99.1% | 0.0045% | 0.0067% | 2.2 |
| and merge and exchange | 94.5% | 99.1% | 0.0032% | 0.0058% | 2.2 |
| chains by size, leaving at 3 that agree | 94.3% | 98.9% | 0.0035% | 0.0060% | 1.7 |
| and fresh streaks (`v1`) | 93.8% | 98.7% | 0.0037% | 0.0071% | 1.6 |
| leaving at 5 that agree, and no order moves without a free tap: the defaults, `v6` | 94.3% | 98.7% | 0.0034% | 0.0061% | 1.85 |

- **On the set kept out of it** (`nh_test`, 157 payloads, 8 seeds, sessions 73 and 94): 76.9% to 94.3%, near 88.6% to 99.3%, mean gap 0.060% to 0.0040%, spread 0.15% to 0.012%, payloads whose seeds all agree 64% to 88%, 1.9 times the time. No payload is worse than under `v0`, 50 are better.
- **By size** on `nh_test`: 4 to 7 upgrades 83% to 92% in 1.3 times the time, 8 to 15 83% to 97% in 1.5, 16 to 31 71% to 94% in 2.2, 32 and up 55% to 92% in 2.4. Small payloads leave early, large ones use all of it. The sets below were run with `v1`, which differs only in those last two changes.
- **With advanced honing** (`explore` singles 100 to 199, 3 seeds, session 74): 88.0% to 94.7%, near 95.7% to 98.7%, 1.6 times the time. One payload of 100 is worse, by 0.27% of flow: two upgrades, where the best plan takes an enhanced book on the first tap only, `v0` found it on one seed in three and the chains agree on the other plan.
- **With chests** (the `chests` set, 453 payloads, 1 seed, session 75): 83.4% to 92.5%, near 93.2% to 98.0%, 61 payloads better and 9 worse (0.21% at most), 1.9 times the time. The two stuck payloads of "Chests" below are among the better ones, by 1.3% and 8.6%.
- **At a streak length of 10** (`nh_quick`, 4 seeds, session 76): better than `v0` at the same length on 11 payloads of 40 and worse on none, spread 0.56% to 0.012%.
- **One job at a time** (`nh_quick`, 1 seed): 39.2 CPU seconds became 75.3 under `v6`, 1.9 times.
- **On the 90 small cases whose every plan was tried** (`exhaust`, `State space tried.md`): one solve reaches the best there is on all 90, where the worst of 4 solves of before missed 12. On the lone upgrade too large to try every plan of, it beats the best that was enumerated by 2.8%, with a plan that mixes two ids.
- **More evaluations in one chain buy little**, and two chains of 24,000 beat one of 48,000 on `v0`'s own runs (83% against 82%).
- **A chain wants some 375 evaluations an upgrade.** 4 to 7 upgrades hit most with 12 chains of 4,000, 32 and up with 4 of 12,000, and worst the other way round (71% against 79%). 250 and 500 an upgrade are not told apart from 375.
- **Leaving at 2 chains that agree costs a point of hit**, at 3 nothing that could be measured on `nh_dev`, for a quarter less time overall and two thirds less on small payloads. But on the weekly chains (`weekly_test`, 1 seed, session 92) payloads of 1 to 7 upgrades lost 2 points at 3: on some, chains agree on a plan that is not the best, reliably (a few weeks of `week_hone_0014` lost 0.3% to 1.3% of flow). 5, the default, gave back half of it (honing chains 96.2% to 97.3%, never leaving 98.2%) for a fifth more time there, and nothing on large payloads, which use all their chains anyway.
- **Moves of the free tap order were wasted while the budget cannot pay for one free tap**: `special_floor` made a fifth of all moves such, hence `affords_a_free_tap`.
- **The free tap order could lock itself out**: a move of the order was drawn with the chance that the first upgrade is free tapped, which is nothing when that upgrade costs more than the budget. `special_floor` and the polish of the order end that. It was the worst case of `v0` (12% of flow).
- **What is still missed**: 39 runs in 632, 31 of them on 10 payloads, the worst 0.3% of flow.

Tried and taken out, for no gain: going back to the best state without an evaluation (a fifth more time), longer between going back, one upgrade a move in place of two (73% against 80%: two is what moves juice between upgrades), a polish in which the upgrades that share a material agree on its price, trying a lost free tap order again with the plans of the upgrades it moves, the exchanges on every chain's result (a seventh more time), and 64,000 evaluations (94.8%, a fifth more time).

## The same id twice on a tap

**The evaluator used to score the same id twice on a tap, and well**: the chance counted it twice, the cost once. A move that changed one pool of an upgrade and left the others made such states a few times in a thousand runs, and they looked like new bests. No other run of 191,000 in the database has one. The evaluator now counts it once.

**Why that move could make them: the crossover changed a state's length.** It copied a plan onto an upgrade of the same piece type and level whatever its progress, and one further along has fewer taps left, so its state took the other's length (on one payload the level 19 armour came out at 48, 57, 61 and 77 slots, under `v0` too). The streaks kept between moves then overran the new length, which every other move clips and that one did not. Now the crossover copies only between upgrades with the same artisan and taps done (`v7`: as `v6` on `nh_dev` and `nh_test`).

## Chests

The `chests` set, 453 payloads, 3 seeds, sessions 28 to 32 (the rate is 0.15, the polish every 6000):

- **Opening the chests well is the easy part.** Against the random state of before, any way of moving them takes the hit rate from 28% to about 80%, which is where the optimizer is without chests. On the worst payloads every variant ends with the same chests or nearly.
- **Rates of 0.05 to 0.3 cannot be told apart.** The polish on top takes a little off the tail: mean gap 0.08% of flow against 0.16%, and it wins the comparison payload by payload against every variant without it by a few payloads in 453.
- **Tried and taken out, for no measurable gain**: a rate that fades while chest moves find no new best (as `special_affinity` does), a polish at the start only, and a chest move with an ordinary move in the same step (8 to 16% more time).
- **What it costs**: 4.5% of CPU time on the `pilot` set at one job (session 33), all of it on payloads with a choice.
- **Still stuck, 11 payloads in 453 by more than 0.5% of flow, and four times the evaluations do not free them** (session 30). The book nobody uses yet is `single_1178_t4_n4`; the large free tap budget is `single_1231_t4_n24`, where runs with the same chests differ by the free tap order. Neither is fixed.

## Responses

Sessions 42 to 46, the gap as a share of flow, hit within 1e-4:

| set | before | `response_polish=3` |
| --- | --- | --- |
| `explore` singles 100 to 199, 3 seeds | hit 79.7%, 90th percentile 0.056% | 90.3%, 0.009% |
| `chests`, 1 seed | 79.5%, 0.055% | 85.0%, 0.032% |
| `chains`, 1 seed | 68.8%, 0.093% | 81.2%, 0.065% |

- **The pass at the end is nearly all of the gain**, for 1 to 2% more time.
- **`response_every` at 6000 or 2000 and `response_rate` at 0.03 are not told apart from it** (90.7, 90.7 and 90.3% on the singles). The rate at 0.03 takes 16% less time, because what a response is charged is on the safe side.
- **More responses are worse**: a rate of 0.1 gave 87.7% on those singles, and on 495 other singles a rate of 0.3 gave 80% with a mean gap four times the baseline's and a worst of 25%.
- **Every normal move a response** (rate 1) was the worst on `pilot`: mean gap 1.6% against 0.14%.
- **What is still missed is not only the order**: with the pass at the end, singles with neither free taps nor advanced honing are hit 88% of the time. Those are states no single upgrade can leave.
- Mixing two ids of a pool (`response_mixed`) took the hit rate on `nh_dev` from 88.9% to 92.9%, the largest single gain of the round.
- Always pool by pool (`response_joint`) gave the same results in the same time on the singles.

## Streak length

- **The search is not what is lost.** On the 90 small cases, with every plan that keeps the rule tried, the best of 4 solves is the best there is at 5 and at 10, and misses once at 2, by 0.02%, on a plan that mixes two book ids. Every plan returned over 100 `explore` singles kept the rule (session 48).
- **The rule itself costs**, against the best known without it, on 100 `explore` singles and 3 seeds (sessions 47 and 48):

| streak length | hit | median gap | 90th percentile | mean | worst |
| --- | --- | --- | --- | --- | --- |
| 1 | 90.3% | 0 | 0.009% | 0.026% | 1.5% |
| 2 | 62.3% | 0.001% | 0.30% | 0.11% | 2.4% |
| 5 | 54.0% | 0.008% | 0.71% | 0.19% | 3.3% |
| 10 | 45.0% | 0.014% | 0.85% | 0.32% | 7.6% |

- **The cost is in what is owned**: at 10, payloads that own no juice lose 0.07% on average and 0.37% at most; those that own some lose 0.3 to 0.5% on average. The worst are one or two upgrades with a few taps left.
- It is a fifth to a quarter faster at 10, there being fewer plans.

## What a move cost

An iteration outside the evaluator went to a third of what it was. What was taken out, none of which changes a result (the `pilot` runs of session 25 have the metrics of session 21):

- accepting or rejecting allocated a copy of every upgrade's state;
- a perturbed state was hashed three times, and `apply_new_streak_info` built a hash set of a few ids on every call;
- a restart rehashed every upgrade;
- the state hash was SipHash.

`fresh_streaks` on `nh_dev` hit as often and took a tenth less time (sessions 64 and 68), which is within the noise on the first and not on the second.

**Reading a state's streaks used to look for the back streak at the wrong end** (`State::compute_streak_info` took the slots before `len - front_len` in place of those after `front_len`). So wherever the streaks were read from a payload and not kept from the last move, the back streak was lost or one was made up: after a copy between identical upgrades, on a state given in the payload, on a carried plan. On cold runs fixing it changed nothing that could be measured.

## Chained runs

The weekly chains of `Arena.md`, 2 seeds, a chained run against a cold one of the same build on the same payload:

| `weekly_test`, 36 chains, 991 payloads | hit, nothing honed | time | hit, honing | time |
| --- | --- | --- | --- | --- |
| `v0`, cold | 67.2% | 0.51 | 85.7% | 0.65 |
| the defaults (`v6`), cold | 93.3% | 1 | 97.2% | 1 |
| `v6` chained, `carry_stall` 4 | 97.9% | 0.79 | 97.8% | 0.83 |
| `v2` chained, `carry_stall` 2, against `v2` cold | 96.1% against 94.3% | 0.56 | 96.8% against 96.6% | 0.60 |

Sessions 79, 81 and 95. Over `v0`'s time a cold run takes 1.9 and 1.5 times, a chained one 1.5 and 1.3.

- **A carried plan is worth more than a fresh chain**: with nothing leaving early a chained run hit 4 to 8 points more often than a cold one for the same evaluations (`weekly_dev`, sessions 59 and 60).
- **Leaving as soon as the chain from the carried plan finds nothing better drifts**: by the last ten weeks of the chains with nothing honed the hit rate was 72% against 90% (session 60). 3 fresh chains not bettering it (`carry_stall` 4) was the least that kept the hit rate of a cold run on `weekly_dev`; 2 lost two points there (sessions 71 and 72). On `weekly_test` 1 was already as good as cold. On a cold run the stall rule cost a point and a half of hit.
- **Without the polish of the carried plan** the same number of chains hit 2 to 4 points less.
- **A given state used to start every chain**, and every chain agreed with it. On `nh_dev` with `v0`'s result as the given state (the `seeded` folder, 4 seeds, session 91), hit 90.8% became 94.0% in the same time, which is what a cold run hits.
- **The cost does not go to nothing as the stock grows**: in the last ten weeks a run still takes half of what it did in the first weeks.
- Not tried: telling how much a payload changed, and a blob. The carried `StateBundle` has been all that was needed.

## Marginal value

- **Until 2026-10-21 it was one bump scored in plan 0** (gross spend) for every row: one more step char-bound, re-polished, gold saved per unit. For a juice or book that is about its market price times the chance of running short, never over market (a unit worth more is already bought on demand), so it could not say whether buying is worth it. It also re-polished under the user's plan while scoring in plan 0, which gave negative values (to -10k per book on `single_3048_serca_n37`, whose plan credits tradable leftovers at the taxed price).
- **An average over purchases** (when the plan buys, re-polish with the row at 1000x market and divide what that costs the other rows by the units bought; otherwise the bump, scored in the optimizer's plan) was built and dropped the same day. With a prohibitive price the fixed plan only uses juice where even the worst case stays within stock, so the comparison credits purchases with what flexible use of the stock saves: as stock grew the value went from about 5% of market to about 3x and back down.
- **A range both ways** (doubling steps up and down until usage moves): 16% of a solve over 30 `normal` scenarios, most of it upward searches on unused rows that can never move. Only the lower side was wanted.

### Full solves with buying disabled (where it was left)

`crates/arena/examples/no_buy.rs`, kept for picking this up: the row's market price times 1000, a full `solve` at stock x and at x + d (d in uses of one tap, times the block), scored in the optimizer's plan at the reference tuning, (V(x + d) - V(x)) / d. `SWEEP=name:label:points:spacing` sweeps the stock from 0; `ENVELOPE=...` re-solves each point carried from its neighbours' plans; with neither, each row of the first N scenarios at its own stock, d = 1, 2, 4, 8, two seeds.

- **Mostly falls smoothly with stock** (single_3005 Lava 132, 112, 99 ... 11), with a small rise at the very start (single_3000 Glacier 8, 17, 49, then down). The 5% to 3x swing of the average-over-purchases method is gone.
- **Carrying x's plan into the x + d solve changed nothing**: on these scenarios the cold solve already finds the same plan.
- **Two kinds of blip.** Real ones, where owning more makes a better plan possible (single_3000 Glacier 320 to 400: usage 33 to 45, no neighbour's plan beats either solve). And optimizer misses (640 to 720: a neighbour's plan was 511 and 915 gold better, which smoothed 0.2, 33 to 10.3, 10.3).
- **Seed spread falls with d**: about 15 to 45% at d = 1 on the larger scenarios, under 10% for most rows at d = 4, about the same at 8. Small scenarios were the same over seeds at every d. A larger d averages over later units that are worth less (single_3001 Enhanced 19-20 Weapon: 78, 39, 19, 10, as only one more book is used), which flipped buy against market for one row (single_3012 11-14 Armor, market 265: 285 to 407 at d = 1, 2; 182 to 242 at d = 4, 8).
- single_3003 Glacier stays noisy at every d (26 against 70 at d = 1): the x solve itself varies.
- Cost: one full solve per row per d plus the baseline; 4 to 5 s a solve on single_3003 and single_3012.
- **Not yet tried**: d = 4 with a two-way carry (x cold, x + d from x, x again from x + d's plan, keep the better x), which is what removed the misses in the envelope test. Also open: whether the column should mean the next unit (small d, noisier) or the next few on average.
