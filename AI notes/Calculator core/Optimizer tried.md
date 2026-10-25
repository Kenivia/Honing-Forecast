# Optimizer: what not to try

Warnings behind `Optimizer.md`: what failed or was ruled out, with the one measurement that says so. "Hit" is within 1e-4 of the best known as a share of flow, "near" within 1e-3. `Arena.md` has the sets and `Arena tried.md` the traps of running them.

## Evaluations and chains

- Don't spend more evaluations in one chain: 48,000 in one chain gave 81.5% hit for twice the time, while 48,000 in 4 chains gave 88.9%, and two chains of 24,000 beat one of 48,000. 64,000 evaluations in total gave 94.8% against 94.3% for a fifth more time.
- Don't make chains short for small payloads or long for large ones: 4 to 7 upgrades hit most with many short chains, 32 and up with a few long ones, and the opposite choice was worst (71% against 79%). `chain_iters_per_upgrade` 150, 200, 250, 375 and 500 are within a point of each other; the shadow start makes more, shorter chains better on large payloads.
- Don't lower `chain_iters_min` for small payloads or start every chain from the shadow start: on the held-out set payloads of 1 to 9 upgrades stopped at some 6,000 evaluations where the old search took 20,000 to 48,000, because three short chains from one start agree on a plan that is not the best (hit on seven such payloads 25%, against 86% before). Alternating starts and a floor on chain length restore it; small payloads are 4% of the time, so the longer chains cost nothing.
- Don't leave at fewer than 5 agreeing results without `unanimous`: 3 that agree looked free on cold generated singles, but on weekly chains payloads of 1 to 7 upgrades lost 2 points, since chains agree reliably on a plan that is not the best (a few weeks of `week_hone_0014` lost 0.3 to 1.3% of flow). `agree_exit` 4 is no better than 5 with `unanimous`.
- Don't use `max_iters` 32,000 or 40,000 with the shadow start: 93.0% and 93.8% hit against 95.0%.
- Don't stop a stalled chain early to save time: a chain's last improvement is at a median of 43% of it, and 58 to 71% from 8 upgrades up, so little is saved where the time is.
- Don't add: going back to the best state without an evaluation (a fifth more time), longer between going back, one upgrade a move in place of two (73% against 80%: two is what moves juice between upgrades), a polish in which upgrades that share a material agree on its price, trying a lost free tap order again with the plans of the upgrades it moves, exchanges on every chain's result (a seventh more time).

## Starts

- Don't start every chain from one polished start (`start_polish`): 11% fewer evaluations but hit 95.3 to 93.4, since chains from one start agree on a plan that is not the best.
- Don't start from the prices of the solved plan itself: mean 5.3% of flow off, 90th percentile 20%. Where stock is owned, its price at the best plan is the leftover's, and under it every upgrade takes too much. So don't build a learned predictor of the prices either; the averaged fixed point (4 rounds, mean 0.63% off, from 23% for the empty plan) is hard to beat, and damping by a running average beats a fixed 0.5.
- Don't expect the shadow start to fix the free tap order: with the identity order it is 3.2% off against 0.63%, so the order is most of what is left.
- Don't use iterated local search (kick a few upgrades, one piece, the order or a chest, polish, keep if better) in place of chains: 83.3% hit against 91.7% for chains at 0.83 of the CPU; it stalls near 85% whatever the budget, since a polish on a large payload is mostly responses and few kicks fit.

## The free tap order

- Don't draw a move of the order with the chance that the first upgrade is free tapped: that is nothing when that upgrade costs more than the budget, and it locked the order out (the worst case of the old optimizer, 12% of flow). `special_floor` and the order polish end it.
- Don't allow order moves while the budget cannot pay for a free tap: `special_floor` made a fifth of all moves such (`affords_a_free_tap`).

## Same id twice on a tap

- Don't let a move put the same id twice on a tap. The evaluator once scored it well above any real plan (the chance counted twice, the cost once), and such states looked like new bests.
- Don't let the crossover copy between upgrades whose progress differs: a state took the other's length (the level 19 armour came out at 48, 57, 61 and 77 slots), the streaks kept between moves then overran it, and that produced such states. Copy only between upgrades with the same artisan and taps done.

## Chests

- Don't tune the chest move rate: 0.05 to 0.3 cannot be told apart. Any way of moving chests takes hit from 28% (random chest state) to about 80%, where the optimizer is without chests; the polish on top only trims the tail (mean gap 0.08% of flow against 0.16%).
- Don't add a rate that fades while chest moves find no new best, a polish at the start only, or a chest move with an ordinary move in the same step: no measurable gain, the last 8 to 16% more time.
- Don't expect more evaluations to free the stuck payloads: four times the evaluations left them stuck (11 of 453 by more than 0.5% of flow). The book nobody uses yet is `single_1178_t4_n4`; the large free tap budget is `single_1231_t4_n24`, where runs with the same chests differ by the free tap order.

## Responses

- Don't put more responses in the chain: `response_rate` 0.1 gave 87.7% hit on the singles against 90.3% for the end pass alone, 0.3 gave 80% with four times the mean gap and a worst of 25%, and a response on every normal move (rate 1) gave a mean gap of 1.6% against 0.14%.
- Don't expect `response_every` (6000 or 2000) or `response_rate` 0.03 to beat the pass at the end (90.7, 90.7 and 90.3%); the end pass is nearly all of the gain, for 1 to 2% more time. The rate at 0.03 is 16% faster only because what a response is charged is on the safe side.
- Don't drop `response_mixed`: mixing two ids of a pool took hit on the normal set from 88.9% to 92.9%, the largest single gain of that round.
- Don't expect responses to fix everything: singles with neither free taps nor advanced honing are still hit only 88% of the time, from states no single upgrade can leave.
- Don't bother with always searching pool by pool (`response_joint` 0): same results, same time on the singles.
- Don't clear the recursion's table between responses: most of it is never reached and clearing it was most of a response.

## Streak length

- Don't blame the search for the cost of a streak length: with every plan that keeps the rule tried, the best of 4 solves is the best there is at 5 and 10, and misses once at 2 by 0.02%. The rule itself costs (hit against the best known without it): 90.3% at 1, 62.3% at 2, 54.0% at 5, 45.0% at 10, mean gap 0.026% to 0.32%, worst 1.5% to 7.6%.
- Don't look for the cost in the optimizer: it is in what is owned. At 10, payloads that own no juice lose 0.07% on average, those that own some 0.3 to 0.5%; the worst are one or two upgrades with a few taps left.
- Don't let a plan from a run at another streak length be the warm start unsnapped: it won and was returned off the grid.

## Cost of a move

- Don't allocate in the accept and reject path, hash a perturbed state more than once, build a hash set per `apply_new_streak_info` call, rehash every upgrade on a restart, or use SipHash for the state hash: together they were two thirds of an iteration outside the evaluator.
- Don't read a state's back streak from the slots before `len - front_len`: `State::compute_streak_info` did, so wherever streaks were read from a payload and not kept from the last move (a copy between identical upgrades, a given state, a carried plan) the back streak was lost or one was made up. Cold runs hardly showed it.
- `fresh_streaks` on hit as often and took a tenth less time on the normal set, within the noise on the first and not on the second.

## Chained runs

- Don't leave as soon as the chain from the carried plan finds nothing better: by the last ten weeks of the weekly chains with nothing honed hit was 72% against 90%. `carry_stall` 4 (three fresh chains not bettering it) was the least that kept the hit of a cold run; 2 lost two points. A cold run with the stall rule lost a point and a half, so it is only for runs with a plan to go on from.
- Don't skip the polish of the carried plan: the same chains hit 2 to 4 points less.
- Don't start every chain from a given state: every chain agreed with it. Hit 90.8% became 94.0% once only the first went on from it.
- Don't expect a carried plan's cost to vanish as stock grows: in the last ten weeks a run still takes half of what it did in the first.
- Don't carry the other plans of the run before (`elites` 4, `warm_polish` 5) by default: it helped generated chains (0 of 18 missed against 6 of 18 at `carry_stall` 1) and not the choices stars, and bettered nothing that could be measured on the held-out set for 4% more CPU.
- Don't try a run that searches afresh every third, or one whose polish moved the carried plan searching longer: half the chained runs' polish moved nothing and those never missed, but the drift on weekly income chains came from runs whose polish did move it, and neither rule took it back.
- Don't cut the carried chain to a quarter together with `carry_stall` 1 for options: options 15 times cheaper but hit 99.3 to 96.6, every miss a large juice option on a large character (Paradise Level 10 on 61 upgrades), where the new juice has to be spread over many upgrades at once.
- Don't set `quick` on a chain of runs: weekly income chains drift under it, guard and all (hit 94.4 to 84.7, generated chains 90.7 to 83.3). On the fixture's income chains it was fine (98.4 hit, half the time), but the fixture does not stand for other accounts, so Income keeps the whole search.
- Don't use one stalled chain for every option without a guard: it missed the large juice options of held-out characters by up to 1.4% of flow. Polishing the base's plan under an option gains nothing on 229 of 288 options, and the most on exactly the large ones, hence `quick_settle`.

## Exchange

- Don't skip the streak grid in the exchange: amounts off the grid are not valid plans at streak length 10. With blocks, streak-10 singles went from 93.5 to 94.9% hit at no measurable cost.

## Marginal value

- Don't score every row as one bump in plan 0 (gross spend): for a juice or book it is about market price times the chance of running short, never over market, so it cannot say whether buying is worth it. Re-polishing under the user's plan while scoring in plan 0 also gave negative values (to -10k per book on `single_3048_serca_n37`).
- Don't average over purchases (re-polish with the row at 1000x market and divide what that costs the other rows by the units bought): with a prohibitive price the fixed plan only uses juice where even the worst case stays within stock, so purchases are credited with what flexible use of the stock saves; as stock grew the value went from about 5% of market to about 3x and back down.
- Don't search a range both ways: 16% of a solve, most of it upward searches on unused rows that can never move.
- Don't expect re-polishing at a plan's own price to move it (82 rows never did), which is why the polish is cheap.
- Full solves with buying disabled are where it was left (`crates/arena/examples/no_buy.rs`: the row's market price times 1000, a full `solve` at stock x and x + d, scored in the optimizer's plan at the reference tuning, (V(x + d) - V(x)) / d; `SWEEP=name:label:points:spacing` sweeps the stock, `ENVELOPE=...` re-solves each point carried from its neighbours'):
  - Value mostly falls smoothly with stock, with a small rise at the very start. Carrying x's plan into the x + d solve changed nothing on these scenarios.
  - Two kinds of blip: real ones, where owning more makes a better plan possible, and optimizer misses (a neighbour's plan was 511 and 915 gold better).
  - Don't use d = 1: seed spread is 15 to 45% on the larger scenarios, under 10% for most rows at d = 4. A larger d averages over later units that are worth less, which flipped buy against market for one row. Cost is one full solve per row per d, 4 to 5 s each on the large scenarios.
  - Not yet tried: d = 4 with a two-way carry (x cold, x + d from x, x again from x + d's plan, keep the better x), which removed the misses in the envelope test; and deciding whether the column means the next unit or the next few on average.

## Speed

- Don't judge speed by the old defaults: the real pages' payloads (every one at streak length 10) were hit on every run yet took 28,000 to 31,000 evaluations each, a choice's option as much as a cold run, since chains ran until 5 agreed. Leaving once the first 3 results agree and none elsewhere (`unanimous`) ended on the same plan on every run, 20% sooner; shorter chains at 32 upgrades and up took hit from 90% to 98% at the same CPU.
- Don't look for the hit criterion's error in the evaluator: the 56 runs that missed by more than 1e-4 were rescored at 8, 64 and 512 coarse cells, and all three see the miss within 3e-5 of flow.
- Don't spend effort on payloads under 8 upgrades for speed: they are 4% of CPU; payloads of 16 to 31 upgrades are half of it and 32 and up another 28%, with 80 to 90% of a solve in evaluations.

## Running an optimizer round

- Don't judge a change on the set it was developed on alone: the held-out set caught the small-payload failure above, near 98.1 against 100.
- Don't compare CPU across sessions at 16 jobs; only within one.
