# Optimizer lab

Optimizer approaches written from scratch beside `optimizer::solve`, in batches. Each idea is reviewed in a session of its own; this note is where to pick it up.

## The process

Batched and iterative. All LLM work happens in supervised sessions; the command only does CPU work.

1. **Brief.** `arena_data/lab/brief.md` is the problem as an attempt sees it: the game, the decision variables and their rules, the evaluator and the best response, what `solve` gets and returns, the CPU budget, how it is judged, how to try it. `arena_data/lab/archetypes.md` has one section per approach of the batch: the overarching algorithm, not its details.
2. **Implement.** One agent per archetype, one after another (Opus, medium effort), in the main tree, writes `crates/core/src/optimizer/lab/<name>.rs` from the brief **without reading the existing optimizer** (or its notes, the tried notes, or other attempts), and `arena_data/lab/attempts/<name>.md`: the algorithm as built, dev numbers, weaknesses. It smoke tests on `pilot` and `quick_dev` in its own `arena_data/lab/dev_<name>.sqlite`.
3. **Run (`pnpm lab`, CPU only).** `scripts/lab/lab.mjs` runs `arena_data/lab/queue.json` (`[{name, cmd, note?}]`, shell commands from the repo root) in order at below normal priority, and records each finished name in `done.json`. It reads the queue again before each job, so jobs can be added while it runs. Started and ended by hand: Ctrl-C stops the running job (the arena stops cleanly), and `pnpm lab` again goes on from it, since arena jobs carry `--resume`. `pnpm lab status` lists the queue. It is a plain trigger: queue anything that is only CPU, such as runs of the main `solve` for context. Commands run under `cmd` on Windows, so an executable's path needs backslashes (`target\arena\bin\<snapshot>.exe`); the driver is the snapshot's own exe, so a build meanwhile cannot touch it. Output goes to the terminal (`arena_data/lab/lab_run.log` when run in the background).
4. **Review, one session per idea, after the whole batch has run.** Viable or not, at matched CPU, on the held-out sets. The verdict is drop, iterate, or take pieces into `solve`; a dropped archetype goes into `Optimizer lab tried.md` with its one key number.
5. **Next batch**: another attempt at the viable ones, and new archetypes.

## How an attempt plugs in

- `Tuning::algo` names the lab algorithm (`lab::ALGOS`, dispatched by `lab::solve`); empty is `solve`. The arena's worker and `bench --solve` call `lab::solve`, so a variant is `<snapshot>:algo=<name>`.
- **CPU is the budget.** `Tuning::lab_effort` scales an attempt's work, 1 meaning about the CPU of `solve` overall; each is run at 0.5, 1 and 2, and `solve` at `max_iters` 24000, 48000 and 96000, so the review compares the two curves of quality against CPU. Attempts run on one thread and never stop on a clock.
- **The driver rejects invalid states** (`lab::check::invalid`, `invalid` in the arena's `main.rs`): ids outside the pools, two of one pool on a tap, wrong lengths, advanced indices out of range, an order that is not a permutation, chest states that do not fit, and a plan off the streak-length grid. Such a run gets `error` "invalid state: ..." and is never the best known. `solve` passed it on every run of `quick_dev` (698).
- `lab_algos_repeat_and_are_valid` (arena worker tests) runs every name in `ALGOS` twice per dummy step.

## Batch 1

Archetypes `price`, `surrogate`, `memetic`, `cem`, `lns` (`archetypes.md`). `price` and `surrogate` are written and frozen in snapshot `lab_b1a` (`arena_data/lab/lab.sqlite`), whose queue (`b1a_real`, `b1a_real_income`, `b1a_quick`) runs each at efforts 0.5, 1 and 2 beside `lab_b1a` at `max_iters` 24000, 48000 and 96000, 3 seeds. `memetic`, `cem` and `lns` are frozen in snapshot `lab_b1b`, queued as `b1b_real`, `b1b_real_income` and `b1b_quick` with the baseline curve again, since CPU compares only within a session. Queued after them, each a session of its own, with the jobs driven by `target\arena\bin\lab_driver.exe` (a copy of the driver with `--first-seed`):

- `ref_*`: `solve` at `max_iters` 384000 on the three sets, only for a firmer best known.
- `low_{a,b}_*`: `max_iters` 6000 and 12000 and efforts 0.125 and 0.25 on the three sets, with `solve` at 48000 to compare CPU against.
- `seeds_{a,b}_*`: the batch's variants at seeds 3–9 on `real_test` and 3–5 on `real_income_test`.
- `regime_*`: all five at efforts 0.5, 1 and 2 with `solve` at 24000 to 384000, in one session per set, on `advanced`, `free_taps`, `juice`, `small`, `large` (cold) and `speed_test` (both), to see where each does well. `quick_dev` holds half of each of the first five, which the attempts were developed on.

Building blocks offered: the evaluator, `shadow_prices`, and the best response (`response.rs`). The tried warnings were left out on purpose.

Judged on `real_test` (38 real characters) and `real_income_test` (22 real income chains), which no attempt sees, plus `quick_dev` for the curves. The real sets come from the analytics pulls (`Arena.md`, "Generated scenarios").
