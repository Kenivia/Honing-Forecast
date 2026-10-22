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

## The speed round (2026-10-22)

Aim: as accurate (the near rate, 1e-3 of flow, within about a point) in less time, the Choices and Income pages first. Sets in `Arena.md` (`quick_dev`, `warm_dev`, `opt_test`); `w0` is the code before, sessions 102 to 117. Hit is within 1e-4 of flow of the best known, near within 1e-3. CPU is at 16 jobs, so only comparable within a session.

**The hit criterion is well clear of the evaluators' error.** Every run of sessions 102 to 104 that missed by more than 1e-4 (56) was scored again at 8, 64 and 512 coarse cells: all three see the miss, its size the same within 3e-5 of flow. A miss is the search's, not the evaluator's.

**Where the time went before**: 80 to 90% of a solve is evaluations; by size, payloads of 16 to 31 upgrades were half the CPU and 32 and up another 28%, under 8 upgrades 4%. The real pages' payloads (`choices`, `income`, every one at a streak length of 10) were hit on every run, cold or chained, yet took 28,000 to 31,000 evaluations each, a choice's option as much as a cold run: chains ran until 5 agreed, or 4 in a row stalled after a carried plan.

How the defaults came about, each against the one before on `quick_dev` (698 runs a variant, 2 seeds); the held-out set then changed two of them (below):

| | hit | near | CPU |
| --- | --- | --- | --- |
| `w0` | 95.3 | 98.7 | 1 |
| leaving once the first 3 results agree and none elsewhere (`unanimous`) | 95.3 | 98.7 | 0.80 |
| and `chain_iters_min` 2000 (was 4000) | 94.8 | 98.9 | 0.79 |
| and the exchange under a streak length (`exchange_blocks`) | 95.1 | 98.9 | 0.79 |
| and the shadow price start, 4 rounds | 95.0 | 98.4 | 0.79 |
| and `chain_iters_per_upgrade` 250 (was 375) | 95.7 | 98.6 | 0.72 |

- `unanimous` alone ended on the same plan as `w0` on every run, 20% sooner. The shorter chains matter on large payloads: at 32 upgrades and up hit went from 90% to 98% at the same CPU (6 chains of 8,000 in place of 4 of 12,000), at 16 to 31 the same hit for 40% less CPU.
- Not kept: `max_iters` 32,000 (93.0 / 97.6) or 40,000 (93.8 / 98.3) from the shadow start; `chain_iters_per_upgrade` 150 or 200 (within a point of 250 either way, session 110); `agree_exit` 4 (the same as 5 here, unanimous does it better).
- **A chain's last improvement** (`w0`, session 104): median at 43% of the chain, 10% under 8 upgrades, 58 to 71% from 8 up with a 90th percentile of 95 to 99%. Large chains improve until late, so stopping a stalled chain would save little where the time is.

**Starts.**
- **The polished start for every chain** (`start_polish`, dropped): 11% fewer evaluations, hit 95.3 to 93.4: chains from one polished start agree on a plan that is not the best (fresh chain steps 100 to 86%).
- **The shadow price start** (developed by a subagent, `shadow_diag` and `shadow_oracle` in `crates/arena/src/space.rs`, `nh_quick`): with the order and advanced upgrades of a solve, the empty plan is 23% of flow off it on average (median 17%), 4 rounds of shadow prices 0.63% (median 0.064%, 90th percentile 2%) for about 650 evaluations. With the identity order it was 3.2%: the order is most of what is left. Damping by a running average beat a fixed 0.5 from 4 rounds on.
- **The prices of the solved plan itself** give a worse start (mean 5.3%, 90th percentile 20%, median 0.003%): where stock is owned, its price at the best plan is the leftover's, and under it every upgrade takes too much. So no learned predictor of the prices (the small neural net considered) could beat the averaged fixed point by much; none was built.

**Going on from a plan** (`warm_dev`: the choices' stars, income, weekly and generated chains, 650 runs a variant):
- **One fresh chain after the carried plan's** (`carry_stall` 1) and the carried chain cut to a quarter: options 15 times cheaper, hit 99.3 to 96.6, near 100 to 98.3 (session 106). Uncut: 99.0 / 99.3 at a quarter of the evaluations. Every miss was a large juice option on a large character (Paradise Level 10, 256 Lava's and 768 Glacier's Breath on 61 upgrades): the new juice has to be spread over many upgrades at once. The shadow start halved the worst of them (0.12% to 0.05% of flow).
- **On chains of runs it drifts**: weekly income chains (`week_inc`) at a streak length of 10 missed 4 to 8 of 30 steps at `carry_stall` 1, 2 to 4 of them by more than 1e-3, against 1 to 3 at 4. The generated chains' income steps (5 to 25% of use at once) and finished upgrades missed more too. Weekly honing chains missed nothing whatever the setting.
- **The other plans of the run before** (`elites` 4, `warm_polish` 5) helped chains (generated: 0 of 18 missed against 6 of 18 at `carry_stall` 1) and not the stars.
- Tried and dropped: a run searching afresh every third (an age in the blob, `thorough_every`), and a run whose polish moved the carried plan searching longer (`settle`). Half the chained runs' polish moved nothing and those never missed, but the drift on weekly income chains came from runs whose polish did move it, and neither rule took it back.
- **Hence `quick`**: the short search only where the frontend knows the payload is a variant of the one gone on from (a choice's option), the default elsewhere.

**Iterated local search** (developed by a subagent, not kept): from the polished cold start, kick (redraw 2 to 4 upgrades by impact, empty one piece's levels within two, move the free tap order, a chest), polish, keep if better, every polish charged to the budget. `nh_quick`, 3 seeds: 83.3 / 97.5 against the chains' 91.7 / 98.3 at 0.83 of their CPU; with 8-upgrade kicks and 40 stalled kicks to stop 86.7 / 98.3. On `pilot` it won, 91.7 against 85.4. It stalls near 85% whatever the budget: on a large payload a polish is mostly responses and few kicks fit.

**Exchange under a streak length**: amounts of 1 to 64 blocks, both plans put back on the grid before the metric decides. On the streak-10 singles hit 93.5 to 94.9, nothing else moved, at no measurable cost.

**The result** (`opt_test`, held out throughout, 1,316 runs a variant, 2 seeds, session 116, best known with session 117): the defaults (`w11`) against `w0`, hit 97.5 to 98.1, near 99.8 to 99.9, mean gap 0.0011% to 0.0008% of flow, 27% less CPU, 38% fewer evaluations. No group got worse by more than a run. Choices options: 98.8 / 100 both, CPU 0.61 of `w0`'s; with `quick` 97.5 / 100, worst 0.028% of flow, 0.27 of `w0`'s. The other plans of the run before in the blob (`elites` 4): 98.3 / 99.9 for 4% more CPU, which is why `elites` is 0. One job at a time (`cpu_check`, session 118, all of it hit): `w0` 72 CPU seconds, the defaults 38, with `quick` 24; options 36, 16 and 8; income's later weeks 16, 8 and 3.

**In the browser** (the income fixture, Choices bonus chests then Paradise, from opening the tab to the last run, stubbed prices, the page of before built from HEAD on another port): Toneema 10.3 + 8.8 s to 2.1 + 1.4 s, Keniwei 14.0 + 13.4 to 3.3 + 2.3, Kenipope (61 upgrades) 8.4 + 7.5 to 3.5 + 2.2, Kenivia 4.0 + 3.3 to 1.1 + 0.7, Kenibrush 1.8 + 1.7 to 0.5 + 0.2; the worths shown the same within a hundred gold. Income: Toneema 116 s to 70, Kenivia 47 to 16. That includes keeping the jobs' workers between runs.

**What the held-out set caught** (`opt_test`, session 113). The defaults as above, with every fresh chain from the shadow start and `chain_iters_min` 2000, did worse than `w0` there: near 98.1 against 100. The misses were small payloads (1 to 9 upgrades) that stopped at some 6,000 evaluations where `w0` took 20,000 to 48,000: three short chains from one shadow start agreed on a plan that was not the best. On those seven payloads, 4 seeds (sessions 114, 115): `w0` 86% hit, those defaults 25%, without the shadow start 82%, with `chain_iters_min` 4000 68%, without `unanimous` 43%; every other chain from the shadow start 61%, and with `chain_iters_min` 4000 as well 79%. Hence both: payloads under 8 upgrades are 4% of the time, so the longer chains there cost nothing that shows.

**`quick` needs a guard.** One stalled chain on every option missed the large juice options of the held-out characters by up to 1.4% of flow. Polishing the base's plan under the option's payload gains nothing on 229 of 288 options (session 111), and the most on exactly those: Paradise Level 10 juice at 0.05 to 0.4 of what the upgrades cost, leapstones at 0.01 to 0.14. So an option whose polish gained more than 1e-3 of that gets the whole search.

**`quick` for the Income page, not kept.** Every income chain at full length (`income_all`, session 119): the later weeks hit 100 / 100 at `w0`, 100 / 100 at the defaults, 98.4 / 100 with `quick` (worst 0.065% of flow), at 1,268, 785 and 316 CPU seconds; in the browser Toneema's chain 70 s to 36, Kenivia's 16 to 12. But the generated weekly income chains (`week_inc`, `warm_dev`, session 120) still drift under it, guard and all: hit 94.4 to 84.7, near 100 to 98.6, worst 0.12% of flow; the generated chains 90.7 to 83.3 hit, 97.2 to 96.3 near. Those stand for accounts other than the fixture's, so Income keeps the whole search. Turning it on is `quick: true` in `warm` (`IncomeJobs.ts`), after the hash.
