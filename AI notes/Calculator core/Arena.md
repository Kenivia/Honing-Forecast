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
- Jobs are ordered so that variants take turns. With `--jobs 1` that removes drift between variants.

## Scenarios

`test_cases/scenarios/*.json`, each `{ "steps": [payload, ...] }` with every step a full `Payload`. `pnpm arena dummy` writes the placeholder one; real cases are still to come. Its payloads carry four chest stacks: a three-way choice, an obtain-all with gold and a label of no tier, fractional amounts, and one that gives special leaps. A worker's `Essence` carries `chest_state_arr` next to `special_state`; one from a snapshot older than chests has none and the reference's default is kept.

Every scenario runs **cold** (each step alone) and **chained** (each step is handed a `Previous`: the last payload, the last best `StateBundle`, and the optimizer's own blob, capped at `Tuning::max_blob_bytes`). A step is seeded the same way in both, so until the optimizer reads `Previous` the two give identical results. That is the current state: the plumbing exists, `solve` ignores it.

## The three measurements

- **Correctness** (`verify`): for every step, the given state and some scrambled ones (`optimizer::scramble`, random valid moves) are simulated and compared per material and in total. A row passes when the difference is within `z` standard errors plus `rel_tol` of the value. Failures are rows, not panics. The same command times one evaluation. `run --mc` does the check on the first seed's result instead.
- **Accuracy** (`run`): the gap to the best metric any run in the database has reached on the same payload. The driver re-scores every result with its own evaluator at default tuning (`metric_ref`), because a variant's own `metric` is not comparable once the evaluator differs. `hit %` is the share of seeds within `--eps` of the best known.
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
- **Best known is per payload.** Editing a scenario's payload makes it a new case with no history.
- The database (`test_cases/arena.sqlite`) is gitignored and can be deleted; snapshots in `target/arena/bin` then lose their rows and must be taken again.
