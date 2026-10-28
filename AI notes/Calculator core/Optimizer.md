# Optimizer

Lives in `crates/core/src/optimizer`. The module header there describes the algorithm; details change often, so this note sticks to what stays true. What was tried and failed, with the key measurement, is in `Optimizer tried.md`; read it before trying something new here.

## What it does

Several short chains of simulated annealing over a `StateBundle`, each polished, the best kept, maximising the average-gold metric (see `Evaluation pipeline.md`). It runs for at most a fixed number of evaluations (`max_iters`), not a time budget, and returns the best state seen. Its constants are fields of `Tuning`. "The shape of a solve" below has the parts; the rest of this section is one chain.

Decision variables:

- each upgrade's `State` (which juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`;
- how each chest stack is opened, `chest_states`. It starts as given (the payload's, or everything on the first option) and travels with the state everywhere `special_state` does. See "Chests" below.

Things worth knowing about its shape:

- The metric has no natural scale, so there is no fixed temperature schedule. An adaptive scaler (`scaler.rs`) adjusts acceptance to track a target acceptance rate over the run.
- It restarts from the best known state very often.
- On restart it may copy one upgrade's state onto an identical upgrade, since identical upgrades often want identical plans. Identical means normal honing too (an advanced block shares its index with a low normal level, and its state is not a normal state), and the same artisan and taps done, so a state's length never changes.
- A move changes two upgrades: that is what moves juice between them.
- In the browser it posts progress at most once a second, with the best state so far on the same tick once a later chain beats the ones before. The merge, exchange and final polish after the chains post nothing, so a heavy run sits near 100% for that part.

## The shape of a solve

`solve` in `optimizer/simulated_annealing.rs`. Everything here is a `Tuning` field.

1. **A starting free tap order** (`greedy_order`): the upgrades by what a free tap saves for what it takes, whole and one piece's next level at a time, where either beats the order given.
2. **A start from shadow prices** (`shadow_start` rounds, `optimizer/shadow.rs`): each material is priced at what one more unit of it costs at the state as it is (its band thresholds moved a little, over the free tap outcomes), every normal upgrade takes its best plan under those prices alone (`Response::linear`, no evaluation), the state is evaluated and the best kept. The prices are the running average of each round's; it stops when no plan moves. Every other fresh chain starts from it, the rest from the plain start: chains that all start from it agree on its plan too readily.
3. **Chains** (`anneal`): the evaluations are shared between several chains, each from the same start with its own random numbers, each ending in the polishes. How many is by size (`chains` 0): a chain gets `chain_iters_per_upgrade` evaluations an upgrade, no less than `chain_iters_min`, and there are never fewer than 4 chains. One long chain settles into one of many plans that nothing near betters; another chain lands elsewhere.
4. **The polishes** (`polish`, `polish_rounds` rounds while any of them moves the best state):
   - the free tap order (`order_polish`): each upgrade to each place a free tap can still reach (`order_reach`);
   - the responses (`response_polish`, see "Responses");
   - the advanced upgrades (`adv_polish`): each entry of a strategy takes the best of its values;
   - the chests (`chest_polish`).
5. **Leaving early**: once `agree_exit` results have ended on the same metric, or once the first `unanimous` have and none elsewhere; or, going on from a plan (below), once `carry_stall` chains in a row have not bettered it. Too few agreeing chains leave on a plan that is reliably not the best on small payloads.
6. **Merge** (`merge`): the best chain's result takes from each other chain's, where the metric agrees, its free tap order, its plans for a whole piece type (they share juices and books), and its plan for each upgrade.
7. **Exchange** (`exchange`, in `refine`): uses of a juice or a book go from one upgrade to another that takes the same, in growing amounts: the last taps that take it of the one, the first that do not of the other. Then the polishes again, and again while it moves anything. Under a streak length (`exchange_blocks`) the amounts are blocks, and both plans are put back on the grid (`State::snap_to_block`) before the metric decides.

A chained run (`Previous`, see "Chained runs") puts the plan of the run before in front of all this.

**The free tap order is only moved while the budget can pay for a free tap** (`StateBundle::affords_a_free_tap`); otherwise those moves are wasted. `special_floor` keeps a move of the order from being drawn with nothing as its chance, which would lock the order out when the first upgrade costs more than the budget.

**What is still missed** is small and in few payloads, the hardest being one piece with many levels sharing a partial stock of juice, where every run ends on another split of it.

## Rules a neighbour move must respect

Evaluation assumes these; breaking them corrupts results without an error.

- `special_state` stays a permutation of upgrade indices. Never reorder the upgrade array.
- A state's length never changes.
- A state may be any plan: the pass of responses at the end does not keep to streaks. Reading its streaks (`compute_streak_info`) then gives the nearest streak state, which is what a perturbation goes on from.
- A chest state has one entry per option, none negative, summing to the chest's count. Change it through `set_chest_state`, or call `apply_chest_state` after, or the bands are stale.
- A book id used on an upgrade must be one that exists for that piece type and level. Id 0 means none.
- Never the same id twice on one tap. The evaluator counts it once, but a move that makes one is a bug, and the arena treats such a state as an error (`Arena.md`).
- Advanced-honing strategy indices stay within their maximum.
- Update the state's hash after changing it.
- For normal honing, taps the player has already done are fixed and must not be modified.
- Normal-honing states are kept in a restricted form: for each juice pool, one streak from the first tap and one streak from the last. This is an empirical simplification, not a proven one, and the perturbation code depends on it. It is nearly free (`State space.md`), and the responses reach the plans outside it.

## Chests

Two things change a chest state, and neither draws from the rng or touches the chain where no stack has a choice, so a payload without one runs as if there were no chests:

- **A chest move** (`chest_move`, `Tuning::chest_move_rate` of the moves): some chests of one stack go from one option to another, at most all of the option's early in the run and one at the end. It is a move like any other: evaluated, then accepted or rejected.
- **A polish** (`chest_polish`, every `Tuning::chest_polish_every` evaluations of a chain, once at its start and with the other polishes at its end): from the best state, with the upgrades as they are, every pair of options of every stack is tried in steps that halve, for as long as anything helps. Its evaluations count against `max_iters`, and the chain goes on from the result.

Opening the chests well is the easy part; what is left of the gap with chests is in the upgrades and the free tap order. Two kinds of payload stay stuck: a stack offering a book or scroll no upgrade uses yet (the chests are worth nothing until a state uses it, and the state is worse until the chests are there), and a large free tap budget with a stack that adds to it.

A move that goes on after a chest move must compute the free tap probabilities first: `neighbour` reads them for the current budget, and a chest move may have made a budget that no evaluation has seen.

## Responses

`optimizer/response.rs` works out the best plan of one normal upgrade with the rest of the state as it is, over every plan and not only the streaks (`State space.md` has why that is possible). Three things can use it, each a `Tuning` field:

- **`response_polish`, on by default**: at the end, from the best state, each normal upgrade in turn takes its best plan where the metric agrees, for up to that many passes. A solve can therefore return a state that is not a streak state. This is nearly all of what responses gain.
- **`response_every`, off**: the same single pass every so many evaluations, the chain going on from the result.
- **`response_rate`, off**: the share of normal honing moves that are a response of one upgrade in place of a random change of its streaks (`respond`). An upgrade whose state and whose rest of the state are as they were when it last responded is not asked again; where none is left the move is a random one. `response_kick` changes another upgrade at random first. More responses are worse: a response costs many evaluations' worth of time, taken from the free tap order, the advanced upgrades and the chests.

A response is charged what it took, in evaluations: its table evaluations, and the steps of its recursion at `response_steps` to one. That is about its time, within a factor of two either way.

Inside a response:

- **One id a pool, then two** (`response_mixed`, on): the search over every plan keeps to one id a pool. From what it finds, or from the plan the upgrade has where that is better, each pool then takes its best taps with any two of its ids, a tap taking either (`improve`), where the table fits. A regular book on the first taps and an enhanced one after is the usual case; without it a state with such a plan is one the responses cannot keep.
- **`response_joint`**: an upgrade with two pools is searched over both at once while that is at most this many steps, and pool by pool from its plan past that (`Response::improve`).
- **`response_points`**: where the lattice does not answer, what an amount of a material is worth is taken at that many amounts and joined by lines.
- The recursion's table is kept between responses in a thread local, stamped, because most of it is never reached and clearing it was most of a response.

## Streak length

The game can do ten taps at once, so a plan may be asked to change what it takes only every so many taps. The payload's `min_resolution` is that number (`StateBundle::block`; `Tuning::streak_length` overrides it, which is how the arena varies it). The frontend sends the profile's `min_resolution`, 1 or 10 by the "Force x10 streak length" checkbox in the optimizer control panel (saved with the profile; new profiles and V7 migrations default to 10).

- **The rule**: from an upgrade's next tap on, the taps go in blocks of that many, and every tap of a block takes the same. So every streak, of taking or of not taking, is a multiple of it but the last before the pity, which the pity cuts.
- **A random move** rounds its front streak down to a block and starts its back streak where a block does, and changes a streak by two blocks at least, or the rounding would undo it.
- **A response** chooses once a block and plays the block through, so it is the best of the plans that keep the rule, and its table is only kept where a block starts. `Model::filled` gives a plan cut at its pity its whole last block back: a pool left alone while another moves the pity out would otherwise stop inside a block.
- **A plan to go on from is put on the grid first**: `solve` snaps the given state and the carried plan (`State::snap_to_block`, each streak to the nearest whole block, the back one never crossing the front) before comparing them with the empty plan, or a plan from a run at streak length 1 would win as the warm start and be returned off the grid.
- **What the rule costs is in what is owned**: an owned amount that is not a whole number of blocks is either left over or topped up at the buy price. The search itself still finds the best plan that keeps the rule.

## What a move costs

Keep an iteration outside the evaluator cheap:

- **Accepting or rejecting copies every upgrade's state**, changed or not. `State::copy_from` does it without allocating and leaves a payload with the same hash alone.
- `clone_from_essence` skips a payload that is already the one wanted.
- **The state hash is `ahash` with fixed seeds**, in place of SipHash. The seeds are fixed because a seed has to repeat.

Of what is left, most is `neighbour` (two `WeightedIndex` built per move, and the streaks written back over the whole payload) and then the copy on accepting. Changing the first without changing what the rng is asked for takes care.

**A restart brings the streaks back** (`Tuning::fresh_streaks`, on). `clone_from_essence` restores a payload and leaves its `streak_cache`, and `perturb_normal` builds the next payload from that cache, so without it the first move of a normal upgrade after a restart goes on from where the chain was.

## Chained runs

`solve` takes an optional `Previous`: plans to go on from, the best first (`Plan`, in `optimizer/mod.rs`). Each plan is keyed by upgrade (piece, level, normal or advanced) with its states run-length coded, so it goes onto another payload's upgrades without that payload. The arena hands the last run's best plan and its blob (`Previous::of`). The wasm hands none: the frontend feeds the last result's state back in the payload, which is the same plan.

**The blob** (`Solved::blob`, `Blob`): the other plans a run's chains ended on, distinct by metric, best first, at most `elites` of them, as postcard bytes. `elites` defaults to 0, so it is empty. The frontend does not carry one.

What a run does with them (`Tuning::carry`):

- **Each is carried over** (`Plan::carry`): an upgrade that is still there keeps its state, less the taps done since (its first slots), and its place in the free tap order; a new one goes last and starts empty; a chest stack whose key and count are as they were is opened as it was. The arena's `carry::carried` is the same rule, kept apart because it reads a stored state.
- **Only those that beat the empty plan count**, the state given with the payload among them, and only one of a metric.
- **The best `warm_polish` of them are polished first** (`carry_polish`), each a result like a chain's; the best of those is the carried plan.
- **The first chain goes on from it** (for `carry_iters` of a chain's evaluations), the others start afresh as in a cold run. A chain that starts from the carried plan almost never leaves it, so only the fresh ones say whether it is still the best.
- **The run ends** as a cold one does, when enough results agree, or when `carry_stall` chains in a row have not bettered the best. That second rule is for a run with a plan to go on from only; on a cold one it costs hit rate. Leaving as soon as the chain from the carried plan finds nothing better drifts: week after week the plan is kept past its time.
- **`quick`** in the payload (the frontend sets it on a choice's option, which goes on from the run without; the wasm copies it to `Tuning::quick`) makes it one stalled chain in place of `carry_stall`, unless polishing the carried plan gained more than `quick_settle` of what the upgrades cost (`upgrade_costs`: their base materials at the last band's price). Most options leave the plan as it was, and one fresh chain finding nothing better is enough there. A large juice or leapstone option (Paradise at Level 10) moves it a lot and gets the whole search. Not for a chain of runs, where the plan drifts, so the Income page does not set it.

## Marginal value

**Shelved since 2026-10-21**: the column is commented out in `MaterialDist.vue` and `start_marginal` (`CharWorkerUtils.ts`) is not called, so nothing runs it. The Rust (`marginal.rs`) and the wasm op (`marginal_value_wrapper`) are still there. What was tried, and where to pick it up, is in `Optimizer tried.md`.

`marginal.rs` (`use_more_prices`) gives the calculator's "Est. gold value" column: for each juice or book some upgrade can take (and with a market price), the market price under which the optimizer would use more of it. Base materials are left out. The plan is polished once (`polish_only`, `anneal` with no iterations), then for each row the price is searched downwards from market over a fixed `POLISHES`, each a polish of that plan with the row's market and taxed prices scaled to the trial price:

- **Away from market**: steps of a tenth of the price's order of magnitude (`10^(floor(log10 price) - 1)`), doubling, from the last price that did not use more, until one does.
- **Then bisected** between the two for the polishes left. The answer is the middle of that bracket.
- **Never found**: the lowest price tried, shown as "<"; if that reached 0, not even for free, shown as "-".

"Uses more" is the row's expected usage (`avg_used`, exact whatever the tuning) rising past a relative 1e-6.

## Testing a change

Use the arena (`Arena.md`): snapshot the variants, run them against each other, and read the gap, hit rate and counters. Approaches written from scratch beside `solve` are in `optimizer/lab`, picked by `Tuning::algo` (`Optimizer lab.md`). All randomness must come from the solver's own `rng` so that a seed repeats. `scramble` exposes the neighbour moves for making random valid states.
