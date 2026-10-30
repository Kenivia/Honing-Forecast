# Optimizer lab, batch 1: review briefing

This session reviews one batch 1 attempt: `price`. Read `AI notes/Calculator core/Optimizer lab.md` first for the process. Step 4 there is this review: is the attempt viable at matched CPU on the held-out sets? The verdict is drop, iterate, or take pieces into `solve`. A dropped attempt goes into `Optimizer lab tried.md` with its one key number. `AI notes/Calculator core/Arena.md` covers the measures and tools.

## Where things are

| What | Where |
|---|---|
| The brief the attempts were written from | `arena_data/lab/iteration1/brief.md`, `arena_data/lab/iteration1/archetypes.md` |
| Attempt code | `crates/core/src/optimizer/lab/<name>.rs`, dispatched by `lab/mod.rs` (`lab::ALGOS`) |
| Attempt write-ups | `arena_data/lab/iteration1/attempts/<name>.md`. Each was written by the agent that built the attempt: the algorithm as built, its own dev numbers, its own view of its weaknesses and what to try next |
| The attempts' dev databases | `arena_data/lab/iteration1/dev_<name>.sqlite` (`pilot` and `quick_dev`, mostly seed 0, earlier builds included) |
| Batch results | `arena_data/lab/iteration1/lab.sqlite` |
| Queue, completion times, log | `arena_data/lab/iteration1/queue.json`, `done.json`, `lab_run.log` |
| Set lists | `arena_data/sets/<set>.txt`; real scenarios in `arena_data/scenarios/real/` and `arena_data/scenarios/real_income/` |
| Frozen executables | `target/arena/bin/lab_b1a.exe`, `lab_b1b.exe`; `lab_driver.exe` is the driver with `--first-seed` |

## How the data was generated

- **Snapshots.** Both are at git `9ae4f169`, with the working-tree diff stored in `snapshot.diff`.
  - `lab_b1a` (snapshot id 1) has `price` and `surrogate`.
  - `lab_b1b` (id 2) has `memetic`, `cem` and `lns`.
- **Variant names.**
  - `<snapshot>` is `solve` at its default `max_iters` of 48000.
  - `<snapshot>:max_iters=N` is `solve` at N.
  - `<snapshot>:algo=<name>,lab_effort=E` is an attempt.
  - Each worker runs from its snapshot's own exe.
- **Runner.** `pnpm lab` (`scripts/lab/lab.mjs`) ran the commands in `queue.json` in order, at below normal priority.
  - Sessions 1–5 ran with `--jobs 14`, sessions 6–25 with `--jobs 16`.
  - The machine is a Ryzen 9 6900HX: 8 cores, 16 threads.
- **Runs.** 168,384 runs, every one of kind `optimize`.
  - None has an `error`, and none was flagged as an invalid state.
  - Every run has its returned `state` stored.
- **Repeatability.**
  - `solve` gives identical `metric_ref` in `lab_b1a` and `lab_b1b` for the same step, seed and mode: 17,866 of 17,866 pairs.
  - Every variant that ran the same step, seed and mode in two sessions gave identical results.
- **Cold and chained.** A scenario of a single step runs the same chained as cold (`Arena.md`), so the regime sets ran cold only. `quick_dev` and `speed_test` ran in both modes. `real_income_test` ran chained only.

### Sessions in `lab.sqlite`

The attempts at efforts 0.5, 1 and 2 are written as "0.5/1/2"; "solve 24k/48k/96k" is `solve` at those `max_iters`.

| Session | Set | Mode | Seeds | Variants |
|---|---|---|---|---|
| 1 | real_test | cold | 0–2 | `lab_b1a`: solve 24k/48k/96k; price, surrogate 0.5/1/2 |
| 2 | real_income_test | chained | 0–2 | as 1 |
| 3 | quick_dev | both | 0–2 | as 1 |
| 4 | real_test | cold | 0–2 | `lab_b1b`: solve 24k/48k/96k; memetic, cem, lns 0.5/1/2 |
| 5 | real_income_test | chained | 0–2 | as 4 |
| 6 | quick_dev | both | 0–2 | as 4 |
| 7 | real_test | cold | 0–2 | `lab_b1b:max_iters=384000` only |
| 8 | real_income_test | chained | 0–2 | as 7 |
| 9 | quick_dev | both | 0–2 | as 7 |
| 10 | real_test | cold | 0–2 | `lab_b1a`: solve 6k/12k/48k; price, surrogate 0.125/0.25 |
| 11 | real_test | cold | 0–2 | `lab_b1b`: solve 6k/12k/48k; memetic, cem, lns 0.125/0.25 |
| 12 | real_income_test | chained | 0–2 | as 10 |
| 13 | real_income_test | chained | 0–2 | as 11 |
| 14 | quick_dev | both | 0–2 | as 10 |
| 15 | quick_dev | both | 0–2 | as 11 |
| 16 | real_test | cold | 3–9 | as 1 |
| 17 | real_test | cold | 3–9 | as 4 |
| 18 | real_income_test | chained | 3–5 | as 2 |
| 19 | real_income_test | chained | 3–5 | as 5 |
| 20–24 | advanced, free_taps, juice, small, large | cold | 0–2 | solve 24k/48k/96k/384k (`lab_b1b`); all five attempts at 0.5/1/2 |
| 25 | speed_test | both | 0–2 | as 20–24 |

Payloads (steps) per set:

| Set | Payloads | Note |
|---|---|---|
| real_test | 38 | 38 real characters |
| real_income_test | 286 | 22 real income chains |
| quick_dev | 349 | holds half of each of advanced, free_taps, juice, small and large |
| advanced | 12 | |
| free_taps | 12 | |
| juice | 12 | |
| small | 20 | |
| large | 10 | |
| speed_test | 215 | |

`real_test` and `real_income_test` were not seen by any attempt. `quick_dev` and `pilot` are what the attempts were developed on.

Seeds per variant:

| Set | Efforts 0.5/1/2, solve 24k/48k/96k | Efforts 0.125/0.25, solve 6k/12k, solve 384k |
|---|---|---|
| real_test | 10 seeds (0–9) | 3 seeds (0–2) |
| real_income_test | 6 seeds (0–5) | 3 seeds (0–2) |
| other sets | 3 seeds (0–2) | 3 seeds (0–2) |

## Reading it

**`arena_data/diff_report.py`** compares variants against a fixed baseline, payload by payload (`Arena.md`, "Against a fixed baseline").
- A chained baseline takes the suffix ` chained`.
- `--like` filters variants, `--by kind` groups by scenario kind, and `--worst N` lists the extreme payloads.

```
python arena_data/diff_report.py 1,16 --base lab_b1a --db arena_data/lab/iteration1/lab.sqlite
python arena_data/diff_report.py 4,17 --base lab_b1b --db arena_data/lab/iteration1/lab.sqlite --like lns --by kind --worst 5
python arena_data/diff_report.py 2,18 --base "lab_b1a chained" --db arena_data/lab/iteration1/lab.sqlite
python arena_data/diff_report.py 20 --base lab_b1b --db arena_data/lab/iteration1/lab.sqlite
```

Its definitions:
- **Scale:** everything is in percent of the `flow` of the baseline's best seed.
- **Best known:** the max `metric_ref` over every error-free run of that step anywhere in the database. That includes `solve` at 384k and all the attempts, so it moves as runs are added.
- **hit:** a run within 1e-4 of flow (0.01%) of the best known.
- **diff:** the variant's mean over seeds less the baseline's, on the seeds they share.
- **cpu s / x:** `cpu_s` against the baseline's, summed over the sessions given.

**Other tools:**
- `pnpm arena compare [variants] [--session N]` prints a session's tables.
- `pnpm arena trace <run id> [key]` prints a run's trace rows. `best` is recorded for the runs that have a trace; `surrogate` also records `sur_*` keys.
- The `run` table has `state` (the returned essence as JSON), `counters` (JSON, e.g. `StatesEvaluated`), `cpu_s`, `wall_s`, `metric_ref`, `gross` and `flow`. It joins `scenario_step` on `step_id` for the scenario name, the step and the payload.

**Points about the data to keep in mind:**
- **CPU only compares within a session.**
  - Sessions ran at different times and with 14 or 16 jobs.
  - Sessions 7–9 hold only `solve` at 384k, with no 48k run in the same session.
  - In every other session, `solve` at 48k (`lab_b1a` or `lab_b1b`) is there to compare against.
- **`solve` doesn't always use its whole `max_iters`.** It can stop before reaching it, so its CPU doesn't always scale with `max_iters`.
- **The regime sets are small:** 10 to 20 payloads each.

Results already computed for these runs are not included, so the review starts from the data.
