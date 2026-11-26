# Optimizer

Lives in `crates/core/src/optimizer`; `solve` is in `simulated_annealing.rs`, and each step of it has a doc comment there. Every constant is a `Tuning` field (`tuning.rs`, documented per field). This note is what stays true and what the code cannot say. Measurements and dead ends are in `Further learnings/` (`Optimizer tried.md` before trying something new, `Optimizer lab results.md` for what the lab found).

## What it does

Several short chains of simulated annealing over a `StateBundle`, each polished, the best kept, maximising the average-gold metric (`Evaluation pipeline.md`). The budget is a number of evaluations (`max_iters`, scaled by size), never a clock, and it returns the best state seen.

Decision variables:

- each upgrade's `State` (juices and books on which taps; or the advanced-honing strategy indices);
- the free-tap order, `special_state`;
- how each chest stack is opened, `chest_states`, which travels everywhere `special_state` does.

Its shape:

- The metric has no natural scale, so an adaptive scaler (`scaler.rs`) tracks a target acceptance rate in place of a temperature schedule.
- It restarts from the best state very often, and on a restart may copy one upgrade's state onto an identical one (normal honing, same artisan and taps done, so a state's length never changes).
- A move changes two upgrades: that is what moves juice between them.
- Progress goes through the hook in `progress.rs` (set by `hf-wasm`; native has none and skips it). How often and what is sent is on `Report` in `simulated_annealing.rs`.

## The shape of a solve

1. **Start order** (`greedy_order`): free taps by what they save for what they take, where that beats the given order.
2. **Shadow-price start** (`shadow.rs`): each material priced at what one more unit costs, every normal upgrade takes its best plan at those prices (`Response::linear`), averaged over rounds. Every other fresh chain starts from it, the rest from the plain start: chains that all start from one place agree on its plan too readily.
3. **Chains** (`anneal`): `max_iters` shared between at least 4 chains, each ending in the polishes.
4. **Polishes** (`polish`, rounds while anything moves): free-tap order (`order_polish`, each upgrade to each place a free tap reaches), responses (`response_polish`), advanced strategies (`adv_polish`), chests (`chest_polish`).
5. **Early exit**: enough chains agree on one metric (`agree_exit`, `unanimous`), or, going on from a plan, `carry_stall` chains in a row do not better it.
6. **Merge**: the best chain takes from the others, where the metric agrees, their order, their plans for a whole piece type (shared juices) and single plans.
7. **Exchange** (`refine`): uses of a juice or book move between two upgrades in growing amounts (blocks under a streak length, both put back on the grid), then the polishes again.
8. **Lab tails, off by default**: `mem_polish`, the kick loop (`kick_stall`, `kick_types`) and the surrogate loop (`sur_stall`); see their `Tuning` fields. They are the candidates of `Optimizer lab results.md`.

**The free-tap order is only moved while the budget can pay for a free tap** (`StateBundle::affords_a_free_tap`); `special_floor` keeps the order from being locked out when the first upgrade costs more than the budget.

## Rules a neighbour move must respect

Evaluation assumes these; breaking them corrupts results without an error. `optimizer::lab::check::invalid` checks most of them, and the arena runs it on every result.

- `special_state` stays a permutation of upgrade indices. Never reorder the upgrade array.
- A state's length never changes. For normal honing, taps already done are fixed.
- A chest state has one entry per option, none negative, summing to the count. Change it through `set_chest_state`, or call `apply_chest_state` after, or the bands are stale.
- A book id used on an upgrade must exist for that piece type and level. Id 0 means none.
- Never the same id twice on one tap: the evaluator scores it well above any real plan.
- Advanced-honing strategy indices stay within `MAX_ADV_STATE`.
- Update the state's hash after changing it (`update_payload` does).
- Random moves keep normal-honing states in the streak form: per juice pool, one streak from the first tap and one from the last. This is empirical (`State space.md`) and the perturbation code depends on it. Responses may leave any plan; `compute_streak_info` reads the nearest streak state back.
- All randomness comes from the solver's `rng`, and nothing sums floats in hash-map order, so a seed repeats (`a_seed_repeats_exactly`).

## Chests

A chest move (`chest_move_rate`) shifts some chests of one stack between options and is accepted or rejected like any move; `chest_polish` tries every pair of options in halving steps from the best state. Neither touches the rng where no stack has a choice, so such a payload runs as if there were no chests. A move after a chest move must recompute the free-tap probabilities first, since the budget may be new.

## Responses

`response.rs` works out the best plan of one normal upgrade with the rest fixed, over every plan, not only streaks (`State space.md`). Used by `response_polish` (on: nearly all of what responses gain), `response_every` and `response_rate` (off: more responses are worse).

- `response_mixed` (on): after the one-id-per-pool search, each pool may take two of its ids on different taps (a regular book then an enhanced one).
- `response_points` where the lattice does not answer; low values pick wrong plans (`Optimizer tried.md`).
- A response is charged its table evaluations plus its recursion's steps at `response_steps` to one.
- The recursion's table is kept between responses in a stamped thread local; clearing it was most of a response.

## Streak length

The game does ten taps at once, so a plan may be asked to change only every so many taps: `StateBundle::block`, from the payload's `min_resolution` (the "Force x10 streak length" checkbox, default 10) or `Tuning::streak_length`.

- From an upgrade's next tap on, taps go in blocks and every tap of a block takes the same; only the last streak before the pity may be cut.
- Random moves round to blocks; a response chooses once a block (`Model::filled` gives a pity-cut plan its last block back).
- A plan to go on from is snapped first (`snap`), or a streak-1 plan would win as the warm start and come back off the grid. `snap` leaves a plan whose live taps it would not change. `State::snap_to_block` is lab only: it drops the middle blocks the responses leave.
- What the rule costs is in what is owned (stock not a whole number of blocks), not in the search.

## Chained runs

`solve` takes an optional `Previous` (`optimizer/mod.rs`): plans keyed by upgrade (piece, level, normal or advanced), states run-length coded, so they go onto another payload. The arena hands the last run's best plan and its blob; the wasm hands none, since the frontend puts the last state in the payload.

- `Plan::carry`: an upgrade still there keeps its state less the taps done since, and its place in the order; a new one goes last, empty; a chest stack with the same key and count is opened as before.
- Only plans that beat the empty plan count; the best `warm_polish` are polished (`carry_polish`). The first chain goes on from the best of them, the others start fresh: a chain from the carried plan almost never leaves it.
- `carry_stall` ends a run with a plan to go on from; on a cold run it costs hit rate.
- `quick` (the frontend sets it on a choice's option): one stalled chain is enough, unless polishing the carried plan moved it by more than `quick_settle` of the upgrades' cost. Not for chains of runs (the plan drifts), so Income does not set it.
- The blob (`Blob`, `elites`, off) holds other plans the chains ended on.

## Marginal value

Shelved: `ui/marginal.rs` and its wasm op exist, nothing calls them. How it worked and where to pick it up: `Further learnings/Marginal value.md`.

## Testing a change

Use the arena (`Arena.md`): snapshot the variants, run them against each other, read gap, hit rate and counters. A refactor must repeat seed for seed: `pnpm arena bench --set pilot --solve 0 --out x.tsv` before and after, under the tunings it touches, must give the same metrics. New approaches go in `optimizer/lab` (`Optimizer lab.md`); `simulated_annealing/lab_hooks.rs` holds the lab-only pieces of `solve`. `scramble` makes random valid states from the neighbour moves.
