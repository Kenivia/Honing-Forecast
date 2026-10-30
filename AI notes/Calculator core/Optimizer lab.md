# Optimizer lab

Optimizer approaches tried beside `optimizer::solve`, in iterations. Each idea is reviewed in a session of its own; this note is where to pick it up.

**The aim is learning, not a better optimizer.** The point is to find out exactly what makes a result good: ablations, and single parts swapped into or out of `solve`. A strong optimizer is a side effect. Since iteration 2 an attempt may read and reuse the production optimizer; the "fresh look" rule of iteration 1 no longer applies.

## Layout

`arena_data/lab/` has a folder per iteration (`iteration1/`, `iteration2/`, ...): its brief, attempts, reviews, findings and databases. Only the runner's `queue.json` and `done.json` stay at the top; an iteration's finished queue is moved into its folder.

## The process

All LLM work happens in supervised sessions; the command only does CPU work.

1. **Brief** (iteration 1: `iteration1/brief.md`, one section per approach in `iteration1/archetypes.md`).
2. **Implement**: `crates/core/src/optimizer/lab/<name>.rs`, a write-up of the algorithm as built, smoke tested on `pilot`, `small` and `quick_dev` in a dev database of the iteration.
3. **Run (`pnpm lab`, CPU only).** `scripts/lab/lab.mjs` runs `arena_data/lab/queue.json` in order, at below normal priority, and records each finished name in `done.json`.
   - **Queue format**: `[{name, cmd, note?}]`, shell commands run from the repo root. It is read again before each job, so jobs can be added while it runs.
   - **Started and ended by hand.** Ctrl-C stops the running job (the arena stops cleanly); `pnpm lab` again goes on from it, since arena jobs carry `--resume`. `pnpm lab status` lists the queue.
   - **A plain trigger**: queue anything that is only CPU. A session waits on runs of under an hour only; anything longer, or overnight, is queued here and started by the user.
   - **Paths**: commands run under `cmd` on Windows, so an executable's path needs backslashes (`target\arena\bin\<snapshot>.exe`). The driver is the snapshot's own exe, so a build meanwhile cannot touch it.
   - Output goes to the terminal (`lab_run.log` when run in the background).
4. **Review**, one session per idea: what made it good or bad, on the held-out sets at matched CPU. The verdict is drop, iterate, or take pieces into `solve`. A dropped idea goes into `Optimizer lab tried.md` with its one key number.

## How an attempt plugs in

- `Tuning::algo` names the lab algorithm (`lab::ALGOS`, dispatched by `lab::solve`); empty is `solve`. The arena's worker and `bench --solve` call `lab::solve`, so a variant is `<snapshot>:algo=<name>`.
- **CPU is the budget.** `Tuning::lab_effort` scales an attempt's work, 1 meaning about the CPU of `solve` overall. The review compares curves of quality against CPU: efforts 0.125 to 2 against `solve` at `max_iters` 6000 to 96000. Attempts run on one thread and never stop on a clock.
- **The driver rejects invalid states** (`lab::check::invalid`, `invalid` in the arena's `main.rs`), so such a run is never the best known. It catches:
  - ids outside the pools, or two of one pool on a tap;
  - wrong lengths, or advanced indices out of range;
  - an order that is not a permutation;
  - chest states that do not fit;
  - a plan off the streak-length grid.

  Such a run gets the `error` "invalid state: ...". `solve` passed the check on every run of `quick_dev` (698).
- `lab_algos_repeat_and_are_valid` (arena worker tests) runs every name in `ALGOS` twice per dummy step.
- **Ablations**: `Tuning::lab_off` names pieces an attempt leaves out and `Tuning::lab_on` pieces it adds, joined by `+` (`<snapshot>:algo=price,lab_off=order+kicks`). Each attempt names its own, and with neither set it runs as before. `price` leaves out `order`, `kicks`, `prices`, `replan` and `ratio`, and each kick type: `kchest`, `kwindow`, `kadv`, `kexchange`, `kpiece` and `kperturb`. It adds `advall` (half its advanced kicks put every advanced upgrade of the piece type on one plan) and `chestrand` (chest moves of random amounts in place of thirds).

## Iteration 1 (batch 1)

Archetypes `price`, `surrogate`, `memetic`, `cem` and `lns`, written from scratch without reading the optimizer. The building blocks they were offered: the evaluator, `shadow_prices`, and the best response (`response.rs`).

**The first run** (`iteration1/lab.sqlite`, its queue and log beside it; `iteration1/batch1_review.md` describes its 25 sessions) used `Tuning::response_points` 16. It also kept the attempts in two snapshots, `lab_b1a` and `lab_b1b`.

- **Do not judge the attempts on it.** At 16 points the best response picks wrong plans on lone upgrades, up to 0.96% of flow (`iteration2/findings.md`). That error is in every attempt built on responses, and part of `solve`'s too.
- `iteration1/review_price.md` was written from that run.

**The re-run** (`iteration1/rerun.sqlite`) uses snapshot `lab_r1`: the fixed default of 128 points, and all five attempts in one exe, so every session has `solve` and all attempts together. Each set runs as one session, in the order of `queue.json`:

- **variants**: `solve` at 6k/12k/24k/48k/96k, `solve` 48k at 16 points (the old production), and each attempt at efforts 0.125/0.25/0.5/1/2;
- **seeds**: 10 on `real_test`, 6 on `real_income_test` (two jobs, seeds 0–2 then 3–5), 3 elsewhere;
- **modes**: as before;
- **`ref_*`**: `solve` at 384k, 3 seeds, a session per set, last, only for a firmer best known.

About 22 hours at 16 jobs; done, its queue and done list are `iteration1/rerun_queue.json` and `rerun_done.json`. Review from it.

Judged on `real_test` (38 real characters) and `real_income_test` (22 real income chains), which no attempt saw, plus `quick_dev` and the regime sets (`advanced`, `free_taps`, `juice`, `small`, `large`, `speed_test`) for where each does well. The real sets come from the analytics pulls (`Arena.md`, "Generated scenarios").

## Iteration 2

`iteration2/findings.md`, with runs in `iteration2/i2.sqlite` (snapshots `i2a`, `i2b`). So far:

- the `response_points` error, fixed in production;
- `price` on the re-run: behind `solve` at matched CPU nearly everywhere, ahead on `advanced`, and alone in finding some free tap order basins that `solve` at 384k misses;
- its ablations: the order moves and the kick-and-polish loop do the work; the shadow price trajectories, the replan on an order move and the ratio start do not;
- its losses: two parts of the state that must move together (advanced with normal plans, chest amounts off its step grid, plans with chests);
- the three experiments that followed (section 7):
  - order kicks bolted onto `solve`'s end do nothing (`order_kicks`, at up to 64);
  - in `price`, the window kick alone is what finds the order basins, and its other kicks carry the rest;
  - a joint move of every advanced upgrade of a piece type does nothing;
  - random chest amounts fix the lone-chest payloads but lose the fine steps, so they should be added to the step grid, not replace it;
- next: a stall-bounded kick-and-polish loop in `solve` over several kick types, and chest amounts from both the grid and random draws.
