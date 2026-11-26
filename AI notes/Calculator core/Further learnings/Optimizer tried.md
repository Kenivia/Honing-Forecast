# Optimizer: what not to try

Warnings behind `../Optimizer.md`: what failed or was ruled out, with the one measurement that says so. "Hit" is within 1e-4 of the best known as a share of flow, "near" within 1e-3. `../Arena.md` has the sets, `Arena tried.md` the traps of running them, `Optimizer lab results.md` what the lab found works. Lab numbers are 10-seed gap sums on a prototype's moving payloads, or set means in % of flow, as noted.

## Evaluations and chains

- Don't spend more evaluations in one chain: 48,000 in one chain gave 81.5% hit for twice the time, 48,000 in 4 chains 88.9%, and two chains of 24,000 beat one of 48,000. 64,000 in total gave 94.8% against 94.3% for a fifth more time.
- Don't make chains short for small payloads or long for large ones: 4 to 7 upgrades hit most with many short chains, 32 and up with a few long ones, and the opposite was worst (71% against 79%). `chain_iters_per_upgrade` 150 to 500 are within a point of each other.
- Don't lower `chain_iters_min` for small payloads or start every chain from the shadow start: held-out payloads of 1 to 9 upgrades stopped at some 6,000 evaluations where the old search took 20,000 to 48,000, since three short chains from one start agree on a plan that is not the best (hit on seven such payloads 25%, against 86%). Small payloads are 4% of the time, so longer chains cost nothing.
- Don't leave at fewer than 5 agreeing results without `unanimous`: on weekly chains payloads of 1 to 7 upgrades lost 2 points (`week_hone_0014` lost 0.3 to 1.3% of flow some weeks). `agree_exit` 4 is no better than 5 with `unanimous`.
- Don't use `max_iters` 32,000 or 40,000 with the shadow start: 93.0% and 93.8% hit against 95.0% (at the 48k-era defaults).
- Don't stop a stalled chain early: a chain's last improvement is at a median of 43% of it, 58 to 71% from 8 upgrades up.
- Don't add, to the annealing: going back to the best state without an evaluation (a fifth more time); longer between going back; one upgrade a move in place of two (73% against 80%); a polish in which upgrades sharing a material agree on its price; exchanges on every chain's result (a seventh more time).
- Don't use iterated local search (kick, polish, keep if better) in place of chains: 83.3% hit against 91.7% at 0.83 of the CPU; it stalls near 85% whatever the budget.

## A cheaper cold run (lab iteration 3, `cold/report.md`, `real3_dev`, mlog streak 1 / 10)

- Don't look in the search's knobs: screened chain polishes (12k x0.35 -2.59 against x0.41 -2.62), none, `exchange_stop`, fewer exchange amounts or rounds, `refine_rounds=1`, `magic_number=0.25`, two chains on large payloads (`min_chains=2`, removed: x0.20 -2.59 against x0.22 -2.61) and a size power of 1 over 0.5 all land on the curve of `max_iters` within about 0.02. `chest_flip=final` only pays without chains (at 6k with 2 chains +3% CPU for nothing).
- Don't spend a cold 12k run's CPU on the kick loop (§8): after 12k, `kick_stall=4` with window, chest, adv, piece and `rebuildempty` is -2.64 / -2.76 at x0.27 / 0.22 against 16k's -2.66 / -2.77 at x0.26 / 0.24; in place of chains (8k, `kick_stall=6`) it is below the curve. A one-seed smoke's smaller tails did not hold.
- Don't raise the refine's caps: `exchange_powers=9`, `exchange_rounds=4` and `refine_rounds=6` leave every 12k run identical; `response_polish=5` with or without `polish_rounds=5` is on the curve (§9).
- Don't expect the evaluator's cuts to help: `max_lattice_work` 20k, `special_tol=3e-4` and `max_memo_size=262144` left CPU and mlog where they were. `coarse_cells=6` is on the curve; 4 loses hit (79 to 76% at 6k).

## Starts

- Don't start every chain from one polished start (`start_polish`): 11% fewer evaluations but hit 95.3 to 93.4.
- Don't start from the prices of the solved plan itself: mean 5.3% of flow off, 90th percentile 20%. Where stock is owned, its price at the best plan is the leftover's, and every upgrade takes too much. So don't learn a predictor of the prices either; the averaged fixed point (4 rounds, mean 0.63% off, from 23% for the empty plan) is hard to beat, and a running average beats a fixed 0.5 damping.
- Don't expect the shadow start to fix the order: with the identity order it is 3.2% off against 0.63%.
- Don't try a lost free-tap order again with the moved upgrades re-planned, for single insertions: in the lab (the moved upgrade's best response in `order_polish`, up to 16 a polish) it moved nothing on `quick_dev`, `speed_test`, `real_test` or `large`. Only with paired in/out moves does it pay (`mem_polish` without it +0.0001 against +0.0014).

## The free-tap order

- Don't draw a move of the order with the chance that the first upgrade is free tapped: that is nothing when it costs more than the budget, and it locked the order out (the old optimizer's worst case, 12% of flow).
- Don't allow order moves while the budget cannot pay for a free tap: they were a fifth of all moves.

## Kicks and loops after the chains (lab iteration 2)

- Don't kick a window of the order without re-planning the moved upgrades before the polish (`order_kicks`: at 64, 5 wins of at most 0.1% over 632 payloads, for up to 2.4x the evaluations; `order_polish` puts the window back).
- Don't re-plan every normal upgrade after a kick by true responses alone (lab `replan`: only `single_3300`, partly); in the surrogate loop it needs order and advanced moves among the plans (0.303–0.313 with them against 0.424 without).
- Don't use a kick that moves one upgrade from anywhere into reach (`insertr`): no better than the window kick.
- Don't run the kick loop at a stall of 96 (little over 32 for twice the CPU), or give half the annealing to kicks (`max_iters=24000,kick_stall=32`: losses on quick_dev at the same CPU).
- Don't use `solve` at 96k in place of a tail after it (`fin_a`): x1.6–1.9 for less gain than a cheaper tail on every main set but `large`, where it ties memetic's generations at x1.32.

## From the lab prototypes

`price` (effort 1):

- One plan for every advanced upgrade of a piece type at once (`advall`): nothing on `real_08927991`.
- Shadow-price trajectories in place of a fixed price (`lab_off=prices`): quick_dev +0.0002, 10 better and 7 worse. A start order by leapstone efficiency (`lab_off=ratio`): +0.0001.

`surrogate` (gap sums, its loop 0.276):

- A kick polished with no re-plan between (`lab_off=descent`): quick_dev +0.0008 against +0.0018. A re-plan without order moves (`lab_off=dorder+dadv+dchest`): 0.399. The order alone (`lab_off=dplan`): 0.656.
- True order moves without true advanced moves (`trueorder`, `lab_off=dadv+dchest`): `single_0161` lost on 5 of 10 seeds. Each true response checked by a full evaluation (`checked`): identical. One round of true responses in place of six (`d1`): 0.390 against 0.431.
- A price kick without a surrogate (`lprice`, linear responses under kicked shadow prices): 0.559 against 0.289. Edgeworth terms and a linear correction in the moment surrogate: plain normal 0.300 against 0.289. Price noise on top of its re-plan: 0.274 against 0.276 at x1.27 the CPU on `large`.
- Dropping the chest kick on payloads with no stack (`skipchest`): same gain, more CPU. Mixed insertion places for the random restart (`randboth`): 0.396 against 0.406.
- A surrogate descent as a chain start (`sur_start`): quick_dev +0.0001, 6 wins and 9 losses. Its loop at a stall of 16: 0.291 against 0.176 (without `chain_1051`). Skipping the polish of a restart more than 0.1% below the best (`sur_skip`): saves 30% only on `large`, loses `single_3116` on 5 of 10 seeds.

`memetic` (effort 1 unless noted):

- Responses in chunks of 4 decided by one evaluation (`chunk`), shadow prices afresh at each restart (`freshprice`): level.
- Starts by perturbed prices (`pseed` off, mutated copies instead: quick_dev -0.0054 against -0.0045; from `solve`'s endpoint level or better, `large` +0.0096 against +0.0035).
- Runs of 2–3 upgrades to the front or end of the order: `mem_polish` identical without them.
- The re-plan of failed order moves inside a population search: level at x0.56 the CPU in the prototype; from `solve`'s endpoint it only loses `chain_0017:3`, which the polish takes.
- Memetic from its own start (quick_dev -0.0045 at effort 1; loses `real_08927991`, `chain_0001`, `single_1104`), below effort 1 (budget-starved: `single_1000` is -0.30 of `speed_test`'s -0.36 at 0.25), or from `solve`'s endpoint at effort 0.5 (behind effort 1 with the re-plan off at about the same CPU).

`cem` (effort 1, against its own base):

- A cross-entropy model refit to the elites (refit and piece-type factor off: -0.0071 against -0.0078); a shared front shift per piece type (`group` off: -0.0131 against -0.0121).
- Partial redraws about a polished state (from `solve`'s endpoint +0.0001 at x2.9).
- An order polish capped at 10 places (`large` -0.130 against -0.029 uncapped); a full scan of each advanced slot in place of ±1, ±2 (`advanced` -0.0025 against -0.0011).
- `solve`'s greedy order and shadow rounds as its start (real_test -0.021 against -0.008); carried plans kept unsnapped on chained runs (income -0.0021 at x1.58 against -0.0017 at x1.35); cem from its own start (quick_dev -0.0078 at effort 1, -0.0042 at 2).
- Keep its greedy rebuild of the order's front: without it quick_dev -0.0056 against -0.0078, open payloads kept.

`lns` (effort 1, against `solve` 48k):

- lns from its own start (quick_dev -0.0169; its order build drops low levels, `single_0071` 1.236 at every effort).
- Its ladder of price-scaled candidates (off: -0.0158 against -0.0169), record-to-record acceptance, adaptive arm weights (each off: level), lifting its order-move caps (+0.0027, nothing else), its reach rule in place of the evaluator's cut (identical).
- lns from `solve`'s endpoint as its own phase (effort 0.5: +0.0006 at x1.47); the rebuild kick alone does the same at x1.44 and also takes `single_3375`.
- The rebuild kick in place of the kick loop's other kicks (alone +0.0006 against +0.0019 for the loop with `mem_polish`); its noisy-price half alone (`rebuildprice`: `single_0085` only to 0.009, `single_0012` not at all, x3.1–3.6 on advanced and juice).

## Same id twice on a tap

- Don't let a move put the same id twice on a tap: the evaluator scored it well above any real plan (chance counted twice, cost once), and such states looked like new bests.
- Don't let the crossover copy between upgrades whose progress differs: a state took the other's length (a level 19 armour came out at 48 to 77 slots), the kept streaks overran it, and that made such states.

## Chests

- Don't tune the chest move rate: 0.05 to 0.3 cannot be told apart. Any way of moving chests takes hit from 28% (random chest state) to about 80%; the polish on top trims the tail (mean gap 0.08% of flow against 0.16%).
- Don't drop the random-amount chest moves: without them (`chest_move_rate=0`) real_test -0.079 (up to -1.0 a payload), `speed_test` -0.022.
- Don't add a fading rate, a polish at the start only, or a chest move with an ordinary move in the same step: no gain, the last 8 to 16% more time.
- Don't expect more evaluations to free the stuck payloads: four times as many left 11 of 453 stuck by over 0.5% of flow. They are a stack offering a book no upgrade uses yet (`single_1178_t4_n4`: worth nothing until a state uses it, and the state is worse until the chests are there) and a large free-tap budget with a stack adding to it (`single_1231_t4_n24`).

## Responses

- Don't put more responses in the chain: `response_rate` 0.1 gave 87.7% hit against 90.3% for the end pass alone, 0.3 gave 80% with four times the mean gap, rate 1 a mean gap of 1.6% against 0.14%.
- Don't expect `response_every` (6000 or 2000) or `response_rate` 0.03 to beat the end pass (90.7, 90.7, 90.3%).
- Don't drop `response_mixed`: it took hit on the normal set from 88.9% to 92.9%.
- Don't lower `response_points` to 16: the lines picked the wrong plan for some lone upgrades, by up to 0.96% of flow. 128 costs no measurable CPU.
- Don't expect responses to fix everything: singles with neither free taps nor advanced honing are hit 88%, from states no single upgrade can leave.
- `response_joint` 0 (always pool by pool): same results, same time.
- Don't clear the recursion's table between responses.

## Streak length

- Don't blame the search for a streak length's cost: with every plan that keeps the rule tried, the best of 4 solves is the best there is at 5 and 10, and misses once at 2 by 0.02%. The rule itself costs (hit against the best without it) 90.3% at 1, 62.3% at 2, 54.0% at 5, 45.0% at 10.
- The cost is in what is owned: at 10, payloads owning no juice lose 0.07% on average, those owning some 0.3 to 0.5%; the worst are one or two upgrades with a few taps left.
- Don't let a plan from another streak length be the warm start unsnapped: it won and was returned off the grid.

## Cost of a move

- Don't allocate in the accept and reject path, hash a perturbed state more than once, build a hash set per `apply_new_streak_info`, rehash every upgrade on a restart, or use SipHash: together they were two thirds of an iteration outside the evaluator. What is left is mostly `neighbour` (two `WeightedIndex` a move, streaks written back over the payload); changing it without changing what the rng is asked for takes care.
- Don't read a state's back streak from the slots before `len - front_len`: wherever streaks were read from a payload (a crossover copy, a given state, a carried plan) the back streak was lost or made up.
- `fresh_streaks` on: same hit, a tenth less time on the normal set.

## Chained runs

- Don't leave as soon as the chain from the carried plan finds nothing better: in the last ten weeks of weekly chains with nothing honed hit was 72% against 90%. `carry_stall` 4 was the least that kept a cold run's hit; 2 lost two points. On a cold run the stall rule lost a point and a half.
- Don't skip the polish of the carried plan: 2 to 4 points less.
- Don't start every chain from a given state: every chain agreed with it (90.8% against 94.0% with only the first).
- Don't expect a carried plan's cost to vanish as stock grows: in the last ten weeks a run still takes half of the first.
- Don't carry the run's other plans by default (`elites` 4, `warm_polish` 5): it helped generated chains (0 of 18 missed against 6) and not the choices stars, and nothing on the held-out set, for 4% more CPU.
- Don't search afresh every third run, or longer where the polish moved the carried plan: the drift on weekly income chains came from runs whose polish moved it, and neither rule took it back.
- Don't cut the carried chain to a quarter with `carry_stall` 1 for options: 15x cheaper but hit 99.3 to 96.6, every miss a large juice option on a large character (Paradise Level 10 on 61 upgrades).
- Don't set `quick` on a chain of runs: weekly income chains drift under it (hit 94.4 to 84.7, generated chains 90.7 to 83.3).
- Don't use one stalled chain for every option without a guard: it missed large juice options by up to 1.4% of flow; polishing the base's plan gains nothing on 229 of 288 options and most on the large ones, hence `quick_settle`.

## Exchange

- Don't skip the streak grid in the exchange: with blocks, streak-10 singles went from 93.5 to 94.9% hit at no cost.
- Don't rank pairs by what one block gains alone (`exchange_rank=12`, removed: 12k -2.61 / -2.73 at x0.38 against `exchange_stop`'s -2.60 / -2.75 at x0.32), or leave out ids at one price throughout (`exchange_flat=false`, removed: no real case has one).

## Speed

- Don't judge speed by the old defaults: real pages' payloads (all at streak 10) were hit on every run yet took 28,000 to 31,000 evaluations, a choice's option as much as a cold run. `unanimous` ended on the same plan 20% sooner; shorter chains at 32+ upgrades took hit from 90% to 98% at the same CPU.
- Don't look for the hit criterion's error in the evaluator: 56 runs missing by over 1e-4, rescored at 8, 64 and 512 coarse cells, all see the miss within 3e-5 of flow.
- Don't spend effort on payloads under 8 upgrades for speed: 4% of CPU; 16 to 31 upgrades are half, 32 and up another 28%, with 80 to 90% of a solve in evaluations.
- Don't give each free-tap outcome its own step in the shared coarse pass (steps within 1.09–1.15 of each other, worth 3–10% of the pass), or key kept coarse cells on the exact step (the same as last time only 12–39% of the time, hence the ladder). `target-cpu=x86-64-v2` changes nothing (within 1%). Reusing the lattices' buffers or reading the shared pass without copying: identical, 136.5 against 137.5 CPU s.

## Maxroll override

- Don't count tradable juice and books as free in the owned stock rule: the metric values a used tradable unit at its sell price, so 70 of 724 `real3na` cases got worse than ignoring owned stock (worst 52% of flow).

## Running an optimizer round

- Don't judge a change on its development set alone: the held-out set caught the small-payload failure above (98.1 against 100).
- Don't compare CPU across sessions at 16 jobs; only within one.
