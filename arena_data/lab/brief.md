# Optimizer lab: the problem

You are writing an optimizer for Honing Forecast, a Lost Ark honing calculator, from scratch. One overarching approach is given to you in `arena_data/lab/archetypes.md`; the details are yours. Make it a decent attempt at that approach: something that works on every kind of payload, keeps every rule below, and spends its budget well. It will be judged later, against the existing optimizer, on payloads you never see.

## Ground rules

- **Do not read the existing optimizer**: nothing under `crates/core/src/optimizer/` except `response.rs`, `lab/mod.rs` and `lab/check.rs`; not `AI notes/Calculator core/Optimizer.md` or any `* tried.md` note; not other attempts under `lab/` or `arena_data/lab/attempts/`; not the git history of those files. The point is a fresh look.
- Worth reading: `AI notes/Overview.md`, `AI notes/Calculator core/Domain model.md`, `Evaluation pipeline.md`, `State space.md`, and `Arena.md` for running things. Any code outside the optimizer folder.
- **Write only your own module**: `crates/core/src/optimizer/lab/<name>.rs` (or a `lab/<name>/` folder), plus three lines in `lab/mod.rs`: the `mod`, its name in `ALGOS`, and its arm in `solve`'s match. Nothing else in the repo. If you need something private elsewhere, copy what you need into your module or say so in your note.
- Repo conventions: keep comments brief and top level; `unwrap` freely, this is not a library; run `rustfmt --edition 2024` on your files. Never commit. Never write to `docs/` or `CLAUDE.md`.

## The game

A character wants a list of **upgrades**. Each is one piece going up one level (normal honing, `+N` to `+N+1`) or one block of ten levels of advanced honing.

**Normal honing.** Each tap (attempt) costs fixed base materials (`upgrade.costs[row]` for the first `NUM_BASE_MATS` rows; `unlock_costs` once if not `unlocked`) and succeeds with a chance that grows with each failure (`Model::chance`). Failures also fill an artisan meter; once full, the next tap cannot fail (the pity). On any tap the player may take **juices and books**, each adding a fixed chance for a fixed amount of its material (`juice_info.access(id, piece_type, level)`: `normal_chance`, `normal_amt_used`). Ids are grouped in **pools** (`state::get_pool_ids`, usually one pool of juices and one of books); a tap takes at most one id of each pool. Using them makes success come sooner and also pulls the pity in.

**Advanced honing** uses XP and grace taps; its decision is two strategy indices (juice and scroll), each meaning "use it on the first a grace taps / first b non-grace taps" (`advanced_honing/utils.rs`, `index_to_tuple`). Its distributions are precomputed; changing an index is cheap.

**Free taps.** Special leapstones (`prep_output.special_budget`) pay for free taps: an attempt on an upgrade costs `upgrade.special_cost` and succeeds with `upgrade.base_chance`; a success completes the upgrade for nothing else. Free taps go through the upgrades in the order `special_state`: on one upgrade until it succeeds or the budget cannot pay, then the next. Only normal upgrades can be free tapped, and on one piece a lower level must come before a higher one; the evaluator moves entries that break that to the end of the order, where free tapping stops (`clean_special_state`). If `affords_a_free_tap()` is false, the order does not matter.

**Chests.** Stacks of unopened chests (`prep_output.chests`), each chest giving one of its options (a bag of materials; free-tap leapstones too). How a stack is opened is a choice where it has more than one option.

**Gold.** Every material is owned in three bands (bound, roster-bound, tradable). Using it costs nothing while owned stock lasts and the market price past it; stock left over is credited at what the user says a leftover unit of that band is worth (often nothing for bound). So the gold of a material is a concave piecewise linear function of how much of it is used in total (`prep_output.bands[prep_output.optimizer_plan][row]`, `Domain model.md`, "Materials and prices"). **The metric is the expected gold over all the randomness** (taps, free taps), usually negative, and is maximised. Because the function is not linear, the whole distribution of each material's total matters, not just its mean: a plan that uses owned stock efficiently on most outcomes beats one with the same mean that often runs out.

Structure worth knowing:

- **Upgrades are independent** once the plan is fixed, so each material's total is a sum of independent per-upgrade amounts. The metric is a sum over materials, averaged over how many upgrades the free taps skip (a prefix of the cleaned order, `special_probs()`).
- **The coupling is all through the shared stocks**: an upgrade's best plan depends on what the others use. Juices and books are shared by every upgrade of a piece type.
- The landscape of one upgrade's plan is a sawtooth (`State space.md`).

## The decision variables

Everything lives in a `StateBundle`:

1. **Normal upgrade plan**: `upgrade.state.payload: Vec<OneState>`, one slot per remaining tap, slot 0 the next tap, its length fixed (`upgrade.clean_prob_dist_len`, the pity with nothing used). A slot is the ids taken on that tap. Slots past the pity are never reached. `response::Model` converts: `Model::of(upgrade, juice_info, block)`, `model.indices(payload)` gives an option index per tap (`model.options[k]` is its `OneState`), `model.plan(indices)` goes back, `pity`, `canonical` (empties unreached slots), `filled` (puts a plan on the block grid).
2. **Advanced upgrade plan**: payload of `juice_info.adv_uindex_to_id[piece_type][level].len()` slots (usually 2), each `smallvec![k]`, `k` in `0..=MAX_ADV_STATE`.
3. **Free tap order** `special_state`: a permutation of the upgrade indices.
4. **Chest states** `chest_states`: per chest key, one count per option, none negative, summing to `chest.count`. Change them only with `state_bundle.set_chest_state(&states)`, which rebuilds the stocks and the free-tap budget.

**The rules** a returned state must keep (`lab::check::invalid` tests all of them; the arena counts a state that breaks one as a failed run, because the evaluator scores some such states far above any real plan):

- the lengths above; only ids of the upgrade's pools, at most one per pool on a tap, never one twice;
- advanced indices in range; the order a permutation; chest states as above;
- **the streak length**: `state_bundle.block()` (1, or usually 10): from the next tap on, taps go in blocks of that many and every tap of a block takes what its first takes (taps past the pity excepted). `model.filled` and `State::snap_to_block` put a plan on the grid.
- After changing a payload, `state.update_payload(new)`, or `update_hash()` after changing it in place: the evaluator caches by that hash, and a stale one gives wrong numbers silently.

## Evaluating

- `lab::evaluate(&mut bundle, probe)` returns the metric of the state as it is (store it in `bundle.metric` yourself). It recomputes only the upgrades whose hash moved and memoises per material, so changing one or two upgrades and evaluating again is much cheaper than a fresh state. One evaluation is tens of microseconds to some milliseconds by size.
- After an evaluation, `bundle.special_probs()[k]` is the chance that exactly the first `k` upgrades of the cleaned order are free tapped.
- `bundle.shadow_prices(probe)`: per material row (`prep_output.table`: base materials, then `table.juice_row(id)`), what one more unit used costs at the state last evaluated, in gold, over the free-tap outcomes.
- A `StateBundle` clone copies its caches: fine, not free on large payloads.

## The best response (offered)

`optimizer::response` gives one normal upgrade its best plan with everything else fixed, over every plan there is (not only streaks), keeping the block rule:

- `Response::of(&mut bundle, index, probe)`: tables of what each amount of each material is worth to upgrade `index`, the rest as it is. About one to a few evaluations of work.
- `Response::linear(&bundle, index, &prices)`: the same with each material at a fixed price a unit, by row, and no evaluation.
- `response.best()` -> `(score, indices)`: the best plan with one id a pool. `response.improve(&indices)`: from a plan, each pool in turn, allowing two ids of a pool on different taps. `response.better(&indices, tuning.response_joint)`: `best` where affordable, `improve` past that. `response.score(&indices)` scores a plan within this response (up to a constant).
- `response.work(tuning.response_steps)`: what it took, in evaluations.
- It does not choose the free tap order, chests or advanced plans, and every upgrade taking its best response in turn can stop at a state no single upgrade can leave.

## What `solve` gets and returns

```rust
pub fn solve<R: Rng>(rng: &mut R, state_bundle: StateBundle, previous: Option<&Previous>, probe: &mut Probe) -> Solved
```

- `state_bundle` comes with a state: often all empty, but the website feeds back the plan of its last run, so it may be a good plan already.
- `previous` is set in chained runs: the plans of the run before (a step later: some income came in, a tap was done, an option of a chest taken). `previous.carried(&state_bundle)` puts them on these upgrades. A carried plan is often near the best.
- Treat the given and carried plans as candidates (put normal ones on the block grid first: `upgrade.state.snap_to_block(block, piece_type_usize, upgrade_index, juice_info)`); never return worse than the best of them.
- Return `Solved { best, blob: Vec::new() }`: `best.metric` its evaluated metric, `best.tuning` as given. With no upgrades, evaluate and return.
- `tuning` (`state_bundle.tuning`) carries `lab_effort` (below); ignore the other optimizer fields, which belong to the existing optimizer. Add no fields to `Tuning`: constants of yours are constants in your module.
- Pass `probe` to every evaluation; `probe.count`/`put` are optional instrumentation (`Arena.md`, "Looking inside a run").

## Budget, determinism

- **The budget is CPU time**, the thread CPU of `solve`. One thread only: no rayon, no threads. Never stop on a clock: a seed must give the same run every time. Instead scale your work (evaluations, iterations, population, restarts) by `tuning.lab_effort`: at 1.0 aim at about the CPU the existing optimizer takes on the same payloads overall (the `x` column of `diff_report.py` near 1). Your algorithm will also run at 0.5 and 2, and should use about half and twice that and do better with more. The existing optimizer's CPU grows with the number of upgrades and with the cost of an evaluation, and it stops early when its results agree; matching it overall matters, not on each payload.
- **All randomness from `rng`** (seed a `SmallRng` from it if you like). No `rand::random`, no `thread_rng`, no float sums in `HashMap` iteration order (use `Vec`, `BTreeMap`, or `ahash` with fixed seeds).

## How it will be judged

After the whole batch is written, everything runs against the existing optimizer at matched CPU on held-out payloads: the signed difference of the metric per payload as a share of the gold value the plan moves, the share of runs within 1e-4 of the best known, and the CPU ratio. An invalid state or a panic is a failed run. Payloads have 1 to 65 upgrades, T4 and Serca, with and without chests, free taps and advanced honing, streak length 1 and 10, solved cold and chained. The arena scores every state with a finer evaluator than the one you optimise with (`Tuning::reference()`), so chasing the approximation error does not pay.

## Working

- Build `cargo build --release -p hf-arena`; test `cargo test --release -p hf-arena lab_` (every name in `ALGOS` repeats a seed and returns a valid state on the dummy scenario).
- Try it against the existing optimizer (`current` with no `algo` is the existing one), in a database of your own:
  `./target/release/hf-arena.exe --db arena_data/lab/dev_<name>.sqlite --set pilot run current current:algo=<name> --seeds 2 --jobs 12`
  then `python arena_data/diff_report.py <session> --db arena_data/lab/dev_<name>.sqlite --base current`. Add `--mode cold` for a quicker look; `lab_effort=0.5` etc. go after the algo, comma separated (`current:algo=<name>,lab_effort=2`). Before you finish, one seed of `--set quick_dev` (about 380 payloads, a few minutes) at each of 0.5, 1 and 2.
- `pilot` and `quick_dev` are smoke tests, not targets: the judging is on other payloads.
- Stop when it is a decent attempt: valid everywhere, sensible at the three budgets, the idea given a fair shot. Then write `arena_data/lab/attempts/<name>.md` (60 lines at most): the algorithm as built, its constants, the dev numbers with their session ids, where it is weak, and what you would try next.
