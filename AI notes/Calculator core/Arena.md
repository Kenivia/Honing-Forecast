# Arena

`crates/arena` (`hf-arena`) is the native test harness for `crates/core`. It measures three things and keeps every result in a SQLite file:

- **Correctness**: the evaluator against a Monte Carlo simulation.
- **Accuracy**: how good the optimizer's result is, per test case.
- **Speed**: CPU time and work counters.

Run it with `pnpm arena <command>`, which is `cargo run --release -p hf-arena --`.

## Variants and snapshots

A variant is what gets compared. There are two ways to make one, and they combine:

- **Constants** live in `Tuning` (`crates/core/src/tuning.rs`), a serde-skipped field on `StateBundle` whose default is production. A variant overrides fields by name: `current:max_iters=6000,y_value_tol=1e-5`. A new knob needs a field there and a read where the constant used to be; nothing in the arena changes.
- **Code** is a snapshot. `pnpm arena snap <label>` copies the binary that was just built to `target/arena/bin/<label>.exe` and stores the git commit and the working tree's diff against it in the database. No commit or branch is involved. Restore a variant's code with `git apply` on that diff.

`current` is the working tree as built, without a snapshot. So the loop for several code changes is: edit, `snap a`, edit, `snap b`, then `pnpm arena run baseline a b` once.

There are no optimizer version folders or cargo features any more. `optimizer::solve` always exists.

## How a run executes

The binary is both driver and worker. The driver (always the current build) loads scenarios, starts one worker process per job, and is the only thing that writes the database. A worker is a variant's binary run as `worker`: one job as JSON on stdin, one JSON line per result on stdout. Consequences:

- A snapshot never needs to know the current schema. It does need to understand the job, so **fields added to `Job` or `StepResult` need a serde default** or old snapshots stop working.
- A panic in a variant kills one job. It is recorded with its stderr and shown as `died`; the rest of the session carries on.
- Jobs are ordered so that variants take turns, seeds outermost. With `--jobs 1` that removes drift between variants.

## Scenarios

`test_cases/scenarios/*.json`, each `{ "steps": [payload, ...] }` with every step a full `Payload`. `pnpm arena dummy` writes the placeholder one; real cases are still to come. Its payloads carry four chest stacks, keyed "0" to "3": a three-way choice, an obtain-all with gold, fractional amounts, and one that gives special leaps. A worker's `Essence` carries `chest_states` next to `special_state`; one whose keys are not the reference's (a snapshot older than the keys has none) is left out and the reference's default is kept. A variant built before chests were keyed cannot read these payloads.

Every scenario runs **cold** (each step alone) and **chained** (each step is handed a `Previous`: the last payload, the last best `StateBundle`, and the optimizer's own blob, capped at `Tuning::max_blob_bytes`). A step is seeded the same way in both, so until the optimizer reads `Previous` the two give identical results. That is the current state: the plumbing exists, `solve` ignores it.

## Generated scenarios

`pnpm arena gen <set> --singles N --chains N --steps N --sweeps --stress` (`generate.rs`) writes a set into `test_cases/scenarios/<set>/`. Loading a folder does not descend, so a set is run with `--scenarios test_cases/scenarios/<set>` and the default folder stays the dummy.

- **A case is a point on 16 axes** (`AXES`: tier, size, level band, piece mix, jitter between pieces, advanced share, progress, wealth, imbalance, band split, juice owned, juice price, free taps, plan, chests, price noise), each in [0, 1), and `build` turns the point into a payload. Drawing payload fields independently makes every case look alike; drawing these does not. The point is kept in the scenario's `meta.axes`.
- **Owned amounts are quantiles**: the wealth axis is how often the owned amount is enough under the default state, and imbalance spreads that between materials. So the same wealth means the same thing at any size. Juices, which the default state does not use, are a share of one juice per tap instead.
- **Singles and chains are points of a Sobol sequence** (seeds 1 and 2), so `--offset` gives a set that shares nothing with another, and a longer set starts with the shorter one.
- **A chain replays a player**: most steps solve the payload, do the upgrade the plan wants next (`generate::next_up` and `generate::play`, which `replan` shares) and apply one outcome of it (done, free tapped, or some failed taps in), by the rules `monte_carlo::simulate` uses. The rest are income, a price move, a goal added or removed. `meta.kinds` names what happened before each step. Unopened chests are not drawn on when a material runs out.
- **Sweeps** move one axis at a time around three anchors; **stress** is every upgrade a character has.
- **Chests are off** unless `--chests`: the axis is pinned to none and a chain's chest step becomes income, until the optimizer has chest moves.
- A case whose generation panics is left out and the panic is on stderr. A chain step the optimizer panics on is kept, with the player following the default state.

## Sets

A set is a named list of scenarios for one purpose, in `test_cases/sets/<name>.txt`: one scenario name a line, `name:4` for only its first 4 steps, `#` for a comment, the first of which is what the set is for. Names are looked up under the scenarios folder and the folders in it, so a set can mix generated folders.

- `pnpm arena sets` lists them. `--set <name>` on `run`, `verify` and `carry` uses one in place of the folder: `pnpm arena run baseline current --set pilot`.
- `pnpm arena define <name> --note "..." --kind single --where n=32..41 --where juice_owned=0.4..0.7 --steps 4 --limit 12` writes one from filters: `--kind` is a prefix of the name (single, chain, sweep, stress), a `--where` key is an axis of `meta.axes`, `n` (upgrades of the first step) or `steps`. The command is kept in the file. Or write the file by hand.
- A truncated chain's steps are the same payloads, so they share their best known with the full chain.
- `pilot` is the quick first run for a new variant. It was picked from the exploratory run as cases the optimizer of the time hit a quarter to three quarters of the time, which is where a variant can move the result either way.

## The exploratory run

`test_cases/scenarios/explore` (600 singles, 200 chains of 8, 150 sweeps) and `explore_stress` were run cold over 8 seeds in October 2026, sessions 4 and 5 of the database. `python scripts/arena/explore_report.py [session] [folder]` prints what was read off it: hit rate, time and when the best was found by size, by axis, by sweep level and by what happened before a chain step. Run it again after a full run of a new variant. Session 3 is a stopped run whose error rows are not real.

**The chains of that run are gone.** Their player skipped an upgrade whose piece was not ready instead of doing the level under it, so it did not free tap where the plan said. The 200 chain files and every row of their steps after the first were deleted; the first steps are unchanged by the fix and keep their runs. `pnpm arena gen explore --singles 600 --chains 200 --steps 8 --sweeps` writes them again under the same names, and until then the `chains` and `pilot` sets name scenarios that are not there, and `carry` has nothing to read.

## Carrying a plan over

`pnpm arena carry [--set chains] [--rows]` takes, for every chain step after the first, the best known state of the step before, puts it on the step's upgrades (`carry::carried`: a state less the taps done since, the free tap order with gone upgrades dropped and new ones last) and scores it against the default state and the step's best known. No optimizer runs: it reads the database, so the steps need runs in it. `recovered` is the share of the way from the default to the best that the carried plan already covers.

## What re-planning is worth

The metric assumes the plan made now is followed to the end. `pnpm arena replan --set replan --paths 30 --every 1` (`replan.rs`) measures what is given up by that: it plays each scenario's first step to the end many times, and after every `--every` upgrades carries the plan over and runs the optimizer again from it.

- **The gain of a re-plan is `resolved - carried`**, two evaluator outputs on the same payload. `solve` starts from the state it is given and returns the best it saw, so it is never negative. The gains of a path add up to what following the re-plans is worth over following the first plan, with no noise from the gold itself: the only sampling is which outcomes happened. This rests on carrying a plan over an upgrade keeping its value on average, which `carrying_a_plan_keeps_its_value` checks over every outcome.
- **The player does `generate::next_up`**, the lowest level left on the piece the free tap order starts with, because the evaluator's free tap order may name a level whose piece is not there yet (it means: pay for the levels under it, then free tap). Chains are generated with the same rule.
- **Three guards against a gain that is not information**: the starting plan is the best of `--seeds` cold solves and as many more from that; `control` is one more solve from it, which learns nothing, so its gain is the optimizer's slack; and on the first paths the two metrics are taken again with `max_brute_size` at its maximum, which sums every probability exactly, to see that a gain is not the optimizer finding the approximation's errors.
- Rows are in the `replan` table, one per re-plan, path -1 being the starting plan. `--session N` prints a session's tables again; `--resume` skips the paths a session has.
- **The run of October 2026** (session 8: the `replan` set, 36 singles of 5 to 14 upgrades, 30 paths, every upgrade, 24000 iterations): the gain averaged 0.12% of flow over scenarios, median 0.02%, the largest scenario 0.64% and the largest single path 3.7%. As a share of what the optimizer gains over the default state: mean 4%, median 0.3%. The control was zero everywhere and the exact gain was 0.99 of the approximate one. A third of the scenarios had no gain at all; the gain was near zero where most is owned (wealth above 0.75) and did not go to zero where little is.
- Not covered: chests, re-planning in the middle of an advanced upgrade, and choosing a plan knowing that it will be re-planned. Free taps left over when they run out on an upgrade are kept by the player, which the evaluator does not count on.

## Stopping and resuming

`run --resume` reuses the latest session started with exactly the same arguments and skips the jobs that already have a row. Ctrl-C stops cleanly: nothing a killed worker was doing is recorded. Jobs go seed by seed, so a session stopped early has covered every scenario about equally.

## The three measurements

- **Correctness** (`verify`): for every step, the given state and some scrambled ones (`optimizer::scramble`, random valid moves) are simulated and compared per material and in total. A row passes when the difference is within `z` standard errors plus `rel_tol` of the value. Failures are rows, not panics. The same command times one evaluation. `run --mc` does the check on the first seed's result instead.
- **Accuracy** (`run`): the gap to the best metric any run in the database has reached on the same payload. The driver re-scores every result with its own evaluator at default tuning (`metric_ref`), because a variant's own `metric` is not comparable once the evaluator differs. `hit %` is the share of seeds within `--eps` of the best known.
  - Next to `metric_ref` a run has `gross`, the same with every leftover worthless (the gold spent), and `flow`, the market value of what the state uses on average. `pnpm arena rescore` fills them in for older runs.
  - **What the gap is divided by is undecided.** The report divides by the best metric, which is near zero wherever credit cancels cost or everything is owned. Spent plus credit (`metric_ref - 2 * gross`) vanishes in the same cases. `flow` does not.
- **Speed**: thread CPU time around `solve` only, plus counters. The counters are exact and repeatable; CPU time is not, see below.

## Looking inside a run

`Probe` (`crates/core/src/probe.rs`) is threaded through the evaluator and the optimizer. Without the `trace` cargo feature it has no fields and its methods are empty, which is what wasm builds. With it:

- `probe.count(Counter::X)` is always kept. Add a counter by adding a name to the `counters!` list.
- `probe.put("key", value)` and `probe.put_text("key", || ...)` record a row at the iteration last given to `probe.at(..)`. They end up in the `trace` and `trace_text` tables.
- Wrap per-iteration rows in `if probe.detail()`, which is on only with `--detail`. The optimizer already writes `best` on every improvement, and `metric`, `accepted`, `scale` and move kinds under detail.

`pnpm arena trace <run id> [key]` prints a run's rows as CSV. Counters and tuning are JSON columns; query them with `json_extract`.

## Traps

- **A seed must repeat exactly.** Two things broke that before and would again: the global `rand::random_*` functions (use the solver's `rng`), and summing floats in hash map iteration order (`brute.rs` uses fixed hasher seeds for this reason; `ahash` maps are seeded per instance otherwise). The `a_seed_repeats_exactly` test guards it.
- **CPU time needs `--jobs 1`.** Running many workers at once inflated it about threefold here. Windows also accounts thread time in ticks of about 15 ms, so short measurements are repeated (`time_evaluation`).
- **The Monte Carlo samples normal honing from the upgrade's own tap distribution.** It checks everything built on that distribution, and free taps and advanced honing from the game rules, but not the tap distribution itself.
- **The Monte Carlo reads the bands and the free-tap budget of the state it is given**, so it checks the evaluator under a chest state, not `resolve_chests` itself; that has a unit test in `chest.rs`. `scramble` goes through `start_solver`, so scrambled states have random chest states.
- **The chest state is random per seed for now**, so gaps between seeds on a scenario with chests are large and say nothing about the annealer.
- **Where everything is owned the true metric is zero and the optimizer chases approximation error**: best states with a few thousand gold of negative cost were found on jobs moving millions. Gaps there are noise, whatever they are divided by.
- **Best known is per payload.** Editing a scenario's payload makes it a new case with no history. The payload is hashed with its keys sorted; hashed as serialised, its hash maps gave a new row on every driver run.
- The database (`test_cases/arena.sqlite`) is gitignored and can be deleted; snapshots in `target/arena/bin` then lose their rows and must be taken again.
