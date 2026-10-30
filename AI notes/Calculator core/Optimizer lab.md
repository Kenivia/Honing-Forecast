# Optimizer lab

Optimizer approaches tried beside `optimizer::solve`, in iterations. This note is how the lab works; what it has found is in `Optimizer lab results.md`.

**The aim is learning, not a better optimizer.** The point is to find out exactly what makes a result good: ablations, and single parts swapped into or out of `solve`. A strong optimizer is a side effect. Since iteration 2 an attempt may read and reuse the production optimizer; the "fresh look" rule of iteration 1 no longer applies.

## Layout

```
arena_data/lab/
  runs.sqlite                  every result of the lab (gitignored)
  tools/                       shared scripts
  iteration<N>/
    brief.md                   aim, scope, status of each investigation
    report/                    the iteration's final report
    <investigation>/
      report.md                verdict first, then the evidence; frozen when finished
      final.patch              the code as it ended
      runs/                    <batch>_queue.json, _done.json, _run.log
      scripts/                 one-off analysis, only if needed
      scratch.sqlite           smoke tests (gitignored); deleted when finished
```

Iteration 1 predates this: `iteration1/brief.md`, `archetypes.md`, `attempts/` and `runs/`.

Each fact has one home:

| Question | Home |
|---|---|
| How does the lab work? | this note |
| What do we know, what is a candidate, what is open? | `Optimizer lab results.md` |
| What should not be tried again? | `Optimizer tried.md` |
| What is the evidence? | `<investigation>/report.md` |

Invalid data goes to `ignored/junkyard`, not the lab.

## The process

All LLM work happens in supervised sessions; the command only does CPU work.

1. **Brief.** The iteration's aim, and which investigations it holds.
2. **Implement.** A new approach is `crates/core/src/optimizer/lab/<name>.rs`, with a write-up of the algorithm as built. An investigation adds `lab_off`/`lab_on` switches in that file, and off-by-default knobs in `solve` for swap-ins.
3. **Smoke test** (below), then snapshot into `runs.sqlite`.
4. **Run.** Write a batch with `tools/queue.py`, and run it with `pnpm lab`.
5. **Read** with `tools/cmp.py` and `pay.py`, and write the findings into `report.md` as you go, so that nothing is lost.
6. **Finish.**
   - Remove the knobs that failed, so that `solve` stays clean.
   - Put the verdict at the top of `report.md`.
   - Move what is now known into `Optimizer lab results.md`, and what failed into `Optimizer tried.md` with its one key number.
   - Save the code: `git diff --output=<investigation>/final.patch -- crates scripts`, then `git stash push -m "<investigation> investigation" -- crates scripts`. The patch is the record, since stash positions shift; the stash is what clears the tree.
   - Delete `scratch.sqlite`.

## The method

1. **Check that the build repeats its baseline.** The snapshot with nothing set must show 0 wins and 0 losses on `pay.py` against the run it should repeat.
2. **Read the code for bugs before judging.** The `response_points` error sat in every attempt built on responses.
3. **Compare against `solve` at matched CPU**, set by set, then payload by payload. Look at where it wins as well as where it loses.
4. **Ablate each piece** with `lab_off` and `lab_on`. Change one part at a time.
5. **Swap pieces that look good into `solve`**, off by default, and test them there. A piece that works in a prototype may do nothing in `solve`, and why is worth finding out.
6. **Follow leads to the bottom** before finishing. Starting a prototype from `solve`'s finished result (`fromsolve` in `price`) shows whether its machinery can improve on `solve`'s endpoint.

## Running

**Snapshots and batches.** A snapshot is `<investigation prefix>_<letter>` (`sur_a`, `mem_b`), and its batch has the same letter. `i2a`–`i2i` and `lab_r1` are older names.

```
cargo build --release -p hf-arena
target/release/hf-arena.exe --db arena_data/lab/runs.sqlite snap sur_a
python arena_data/lab/tools/queue.py iteration2/surrogate/a sur_a sur_a:algo=surrogate,lab_effort=1
pnpm lab iteration2/surrogate/a
```

- **`queue.py`** writes one job per set: `real_test` cold, 10 seeds; `real_income_test` chained, 3 seeds; `quick_dev`, `speed_test`, `advanced`, `free_taps`, `juice`, `small`, `large` cold, 3 seeds. `--sets` and `--filter` narrow it. The whole list takes about 20–25 minutes a variant.
- **`pnpm lab`** runs the batch in order at below normal priority, and writes `<batch>_done.json` and `<batch>_run.log`.
  - **Queue format**: `[{name, cmd, note?}]`, shell commands run from the repo root. It is read again before each job, so jobs can be added while it runs.
  - **Started and ended by hand.** Ctrl-C stops the running job (the arena stops cleanly); the same command again goes on from it, since arena jobs carry `--resume`. `pnpm lab status <batch>` lists it.
  - **A plain trigger**: queue anything that is only CPU.
  - **Paths**: commands run under `cmd` on Windows, so an executable's path needs backslashes (`target\arena\bin\<snapshot>.exe`). The driver is the snapshot's own exe, so a build meanwhile cannot touch it.
- **A variant** is `<label>:key=val,...`; any `Tuning` field can be set by name. `--filter` is a plain substring, not a regex.
- **Only one job at a time uses the CPU**, or the `cpu_s` of a session means nothing.

**Tools**, run from the repo root. Every number is in % of flow.

- `tools/cmp.py <base> [variant regex] [set,set] [chained 0|1] [scratch db]`: variants per set against a base, matched by scenario, step and seed. Columns: mean diff per payload, worst and best payload, wins and losses over 0.01%, hit (within 0.01% of the best known), mean gap, evaluations and CPU as ratios. The re-run's other variants show only when the regex names them. Sets are matched by name, so a base must have run on the same set.
- `tools/pay.py <A> <B> [k] [scenario substring] [chained 0|1] [scratch db]`: payload by payload, A against B, with both gaps, the number of upgrades and the number of seeds.
- `tools/rerun_summary.py [variant regex]`: the re-run on its own, every variant against `solve` 48k, with hit, gap and CPU.

## Smoke tests

A smoke test shows that a build runs and does what was meant, in minutes, before a batch is spent on it. It never touches `runs.sqlite`, so a broken build leaves nothing among the results. Use the investigation's `scratch.sqlite`, which `snap` creates.

```
target/release/hf-arena.exe --db <scratch> snap sur_a
target/arena/bin/sur_a.exe --db <scratch> --set small run sur_a --mode cold --seeds 3 --jobs 16
python arena_data/lab/tools/cmp.py <baseline> '' small 0 <scratch>
target/arena/bin/sur_a.exe --db <scratch> --set quick_dev run sur_a:<knob> --mode cold --seeds 3 --jobs 16 --filter chain_0017
```

1. **It repeats.** With nothing set, on `small`, the snapshot must show 0 wins and 0 losses against its baseline. This is what caught the one bad cleaned build of the kick loop.
2. **It does what was meant.** Run the new knob with `--filter` on the payloads it is for, against the same baseline.
3. **Promote.** From the same build, `snap` the label into `runs.sqlite` and queue the batch.

Keep it to about 10 minutes. Anything longer is a batch.

## How an attempt plugs in

- `Tuning::algo` names the lab algorithm (`lab::ALGOS`, dispatched by `lab::solve`); empty is `solve`. The arena's worker and `bench --solve` call `lab::solve`, so a variant is `<snapshot>:algo=<name>`.
- **CPU is the budget.** `Tuning::lab_effort` scales an attempt's work, 1 meaning about the CPU of `solve` overall. Reviews compare curves of quality against CPU: efforts 0.125 to 2 against `solve` at `max_iters` 6000 to 96000. Attempts run on one thread and never stop on a clock.
- **The driver rejects invalid states** (`lab::check::invalid`, `invalid` in the arena's `main.rs`), so such a run is never the best known. It catches:
  - ids outside the pools, or two of one pool on a tap;
  - wrong lengths, or advanced indices out of range;
  - an order that is not a permutation;
  - chest states that do not fit;
  - a plan off the streak-length grid.

  Such a run gets the `error` "invalid state: ...". `solve` passed the check on every run of `quick_dev` (698).
- `lab_algos_repeat_and_are_valid` (arena worker tests) runs every name in `ALGOS` twice per dummy step.
- **Ablations**: `Tuning::lab_off` names pieces an attempt leaves out and `Tuning::lab_on` pieces it adds, joined by `+` (`<snapshot>:algo=price,lab_off=order+kicks`). Each attempt names its own, and with neither set it runs as before. `price` leaves out `order`, `kicks`, `prices`, `replan` and `ratio`, and each kick type: `kchest`, `kwindow`, `kadv`, `kexchange`, `kpiece` and `kperturb`. It adds `advall` (half its advanced kicks put every advanced upgrade of the piece type on one plan), `chestrand` (chest moves of random amounts in place of thirds), `chestboth` (thirds plus 4 random amounts) and `fromsolve` (`solve`'s result as one more carried start, its cost outside the budget).

## Reading results

- **Noise.** Treat effects under about 0.01% of flow as null, unless they repeat on the same payloads across sets or seeds. The random stream shifts with any change, so check a "loss" seed by seed before believing it. `Optimizer lab results.md` lists the noisy payloads.
- **Judge on** `real_test` (38 real characters) and `real_income_test` (22 real income chains), which no attempt saw, plus `quick_dev` and the regime sets (`advanced`, `free_taps`, `juice`, `small`, `large`, `speed_test`) for where each does well. The real sets come from the analytics pulls (`Arena.md`, "Generated scenarios").
- **The best known is over all of `runs.sqlite`**, so it moves as results come in.

## What is in `runs.sqlite`

| Snapshots | What |
|---|---|
| `lab_r1` | The re-run of iteration 1. Default of 128 `response_points`, all five attempts in one exe, so that each session has `solve` and all attempts together. **The baseline**; `lab_r1` alone is `solve` 48k. |
| `i2a`–`i2g` | The `price` investigation. |
| `i2h`, `i2i` | Registered, no stored runs. `i2i` is the final kick loop's code, and repeats `i2g` exactly (57 of 57 runs). |

The re-run in detail, about 22 hours at 16 jobs, each set one session:

- **variants**: `solve` at 6k/12k/24k/48k/96k, `solve` 48k at 16 points (the old production, kept for comparison), and each attempt at efforts 0.125/0.25/0.5/1/2;
- **seeds**: 10 on `real_test`, 6 on `real_income_test` (two jobs, seeds 0–2 then 3–5), 3 elsewhere;
- **`ref_*`**: `solve` at 384k, 3 seeds, a session per set, last, only for a firmer best known.

Iteration 1's first run was at 16 `response_points`, where the best response picks wrong plans on lone upgrades. It is in `ignored/junkyard/lab_iteration1_16pt` with its reviews, and nothing should be judged on it.

## Iterations

- **Iteration 1 (batch 1):** archetypes `price`, `surrogate`, `memetic`, `cem` and `lns`, written from scratch without reading the optimizer. The building blocks offered: the evaluator, `shadow_prices`, and the best response (`response.rs`). Brief in `iteration1/brief.md`, write-ups in `iteration1/attempts/`.
- **Iteration 2:** reviews of each prototype in turn, one investigation each. Status in `iteration2/brief.md`.
