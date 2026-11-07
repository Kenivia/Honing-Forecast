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
      final.patch              the code as it ended (a record; the code stays in the tree)
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
2. **Implement.** A new approach is `crates/core/src/optimizer/lab/<name>.rs`, with a write-up of the algorithm as built. An investigation adds `lab_off`/`lab_on` switches in that file, listed in its `OFF` and `ON`, and off-by-default knobs in `solve` for swap-ins.
3. **Smoke test** (below), then snapshot into `runs.sqlite`.
4. **Run.** Write a batch with `tools/queue.py`, and run it with `pnpm lab`. Then `tools/logcheck.py <batch>` must pass.
5. **Read** with `tools/an.py` (or `cmp.py` and `pay.py`), and write the findings into `report.md` as you go, so that nothing is lost.
6. **Finish.**
   - Remove the knobs and switches that failed (and their names from `OFF`/`ON`), so that `solve` stays clean.
   - Put the verdict at the top of `report.md`.
   - Move what is now known into `Optimizer lab results.md`, and what failed into `Optimizer tried.md` with its one key number.
   - Leave the code in the tree with every piece off, and stop for the user to commit. No stash: stashes went stale across commits. `final.patch` (`git diff --output=<investigation>/final.patch <base> -- <files>`) is optional, as a record.
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
- **Compare `cpu_s` only inside one timeframe**, against a plain baseline in the same batch; a restart or a stopped fan moves it by 10%. Keep the baseline in every batch. `ref` in `cmp.py` compares across batches, for sessions that have the reading.

**Tools**, run from the repo root. Every number is in % of flow.

- `tools/cmp.py <base> [variant regex] [set,set] [chained 0|1] [scratch db]`: variants per set against a base, matched by scenario, step and seed. Columns: mean diff per payload, worst and best payload, wins and losses over 0.01%, hit (within 0.01% of the best known), mean gap, evaluations and CPU as ratios, and `ref`, the CPU ratio corrected for the machine's speed, when both sides have the reading (`Arena.md`, "The three measurements"). The re-run's other variants show only when the regex names them. Sets are matched by name, so a base must have run on the same set.
- `tools/pay.py <A> <B> [k] [scenario substring] [chained 0|1] [scratch db]`: payload by payload, A against B, with both gaps, the number of upgrades and the number of seeds.
- `tools/rerun_summary.py [variant regex]`: the re-run on its own, every variant against `solve` 48k, with hit, gap and CPU.
- `tools/an.py`: `cmp`, `pay`, `fmt` and `sets_of` in Python on a cached dump of the light columns of every run (`tools/dump.py`, in `ignored/lab_cache/runs.pkl`). The first load after `runs.sqlite` changes rebuilds it, about 110 s; later loads take about 4 s. Groups are snapshot labels, and `pr_a`'s batches `ra`–`rd`. Use it from a script with `sys.path.insert(0, 'arena_data/lab/tools'); from an import *`.
- `tools/rep.py <scratch db> <scratch label> <stored label> [variant substring]`: the repeat check, seed by seed, of smoke runs against stored runs of the same variant.
- `tools/logcheck.py <batch>` (`iteration2/cem/b`): every queued job done, its variant summary naming every variant it ran, and no run died or failed its Monte Carlo check. It fails if it parses no rows.
- A prototype's trace (`attr.py`) stays in its investigation's `scripts/`, since the keys are its own.

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
- **Ablations**: `Tuning::lab_off` names pieces an attempt leaves out and `Tuning::lab_on` pieces it adds, joined by `+` (`<snapshot>:algo=price,lab_off=order+kicks`). Each attempt names its own in its `OFF` and `ON`, and with neither set it runs as before.
  - **Unknown names fail.** The arena runs `lab::check_switches` on every variant before it starts (`worker::tuning_with`). It rejects a name nothing reads, a name in the wrong list, and an unknown `sur_kinds` kind, so a typo cannot run as the plain algorithm. `solve` reads the surrogate's switches when `sur_stall` > 0 and memetic's with `mem_polish`, and so does a prototype run with `fromsolve`.
  - The switches of each prototype: `price` leaves out `order`, `kicks`, `prices`, `replan` and `ratio`, and each kick type: `kchest`, `kwindow`, `kadv`, `kexchange`, `kpiece` and `kperturb`. It adds `advall` (half its advanced kicks put every advanced upgrade of the piece type on one plan), `chestrand` (chest moves of random amounts in place of thirds), `chestboth` (thirds plus 4 random amounts) and `fromsolve` (`solve`'s result as one more carried start, its cost outside the budget). `memetic` leaves out `cross`, `gens`, `restart`, `improve`, `pseed`, `oreplan`, `toggle` and `runs`, and adds `fromsolve` (starts from `solve`'s result, its cost included); each run writes `mem_gain`/`mem_wins`/`mem_cost` trace rows by phase and move (`iteration2/memetic/scripts/attr.py`). `cem` leaves out `refit`, `fresh`, `group` and `oprefix`, and adds `fullreach` (no cap of 10 places on its order polish), `fromsolve` (`solve`'s result as a candidate, raw and snapped, its cost included), `ownstart` (with `fromsolve`, every cycle restarts from cem's own start, `solve`'s result the best to beat), `solvepolish` and `exchange`; each run writes `cem_gain`/`cem_late`/`cem_wins`/`cem_cost` trace rows by phase (`iteration2/cem/scripts/attr.py`).

## Reading results

- **Noise.** Treat effects under about 0.01% of flow as null, unless they repeat on the same payloads across sets or seeds. The random stream shifts with any change, so check a "loss" seed by seed before believing it. `Optimizer lab results.md` lists the noisy payloads.
- **Judge on** `real_test` (38 real characters) and `real_income_test` (22 real income chains), which no attempt saw, plus `quick_dev` and the regime sets (`advanced`, `free_taps`, `juice`, `small`, `large`, `speed_test`) for where each does well. The real sets come from the analytics pulls (`Arena.md`, "Generated scenarios").
- **The best known is over all of `runs.sqlite`**, so it moves as results come in.
- **Compare CPU, not evaluations.** CPU per evaluation moves with the states searched: cem with `refit` off costs x1.11 at x0.88 the evaluations.
- **The free tap cut changed on 2026-10-31** (`Evaluation pipeline.md`). Snapshots before `cut_a` used the old cut.
  - **Stored `metric_ref` values stay valid.** All 414,624 were rescored at the new reference: at most 2.2e-6 of flow, and no hit changes.
  - **Builds since do not repeat older snapshots seed by seed.** Use `cut_a` for the repeat check, and put `solve` in the batch for matched CPU, which is about 5 to 15% cheaper.
  - **Gains on order-basin payloads against `lab_r1` run high**, since the new cut alone lifts `solve` there. The detail is in `iteration2/free_tap_cut/report.md`.
  - **CPU ratios inside a batch carry over.** Against `solve` 48k in the same session, every prototype at effort 1 and `solve` 96k cost the same at both cuts within about ±0.05, up to +0.09 on `free_taps`. Absolute `cpu_s` does not carry over: `solve` 48k is x0.78–1.03 per run at the same evaluations, mixed with machine state.
  - **The effort curves are re-run at the new cut** in `pr_a` (batch `iteration2/price/runs/ra`): `price`, `memetic`, `cem` and `lns` at efforts 0.25–2 (`price` also 0.125) against `solve` 6k–96k; table in `price` report 11. `surrogate`'s is at the old cut only.

## What is in `runs.sqlite`

| Snapshots | What |
|---|---|
| `lab_r1` | The re-run of iteration 1. Default of 128 `response_points`, all five attempts in one exe, so that each session has `solve` and all attempts together. **The baseline**; `lab_r1` alone is `solve` 48k. |
| `i2a`–`i2g` | The `price` investigation. |
| `i2h`, `i2i` | Registered, no stored runs. `i2i` is the final kick loop's code, and repeats `i2g` exactly (57 of 57 runs). |
| `sur_a`, `sur_b` | The `surrogate` investigation, old cut. |
| `sur_c`–`sur_g` | The `surrogate` investigation, new cut. From `sur_e` on, runs carry the `ReferenceUs` reading. |
| `cut_a` | `solve` at 48k and 96k and each attempt at effort 1, at the new cut (a2d2c0f2). **The baseline for new builds**; `cut_a` alone is `solve` 48k. |
| `pr_a` | The `price` re-run at the new cut (code of 97e7115d), batches `ra`–`rd`: `solve` 6k–96k, every prototype but `surrogate` at efforts 0.25–2, the kick loop and its variants, `price`'s ablations, and `ascend` by `response_points`. Repeats `cut_a` exactly. |
| `mem_a`, `mem_b` | The `memetic` investigation (new cut): its ablations, `fromsolve`, `mem_polish` in `solve`, with the kick loop and the surrogate loop in the same sessions. Both repeat `cut_a`. |
| `cem_a`, `cem_b` | The `cem` investigation (new cut): its ablations, the order-polish cap, `solve`'s start, polish and exchanges inside it, `fromsolve` and `fromsolve+ownstart`; in `cem_b` the kick loop with `mem_polish` and the surrogate loop in the same sessions. Both repeat `cut_a`. |

The re-run in detail, about 22 hours at 16 jobs, each set one session:

- **variants**: `solve` at 6k/12k/24k/48k/96k, `solve` 48k at 16 points (the old production, kept for comparison), and each attempt at efforts 0.125/0.25/0.5/1/2;
- **seeds**: 10 on `real_test`, 6 on `real_income_test` (two jobs, seeds 0–2 then 3–5), 3 elsewhere;
- **`ref_*`**: `solve` at 384k, 3 seeds, a session per set, last, only for a firmer best known.

Iteration 1's first run was at 16 `response_points`, where the best response picks wrong plans on lone upgrades. It is in `ignored/junkyard/lab_iteration1_16pt` with its reviews, and nothing should be judged on it.

## Iterations

- **Iteration 1 (batch 1):** archetypes `price`, `surrogate`, `memetic`, `cem` and `lns`, written from scratch without reading the optimizer. The building blocks offered: the evaluator, `shadow_prices`, and the best response (`response.rs`). Brief in `iteration1/brief.md`, write-ups in `iteration1/attempts/`.
- **Iteration 2:** reviews of each prototype in turn, one investigation each. Status in `iteration2/brief.md`.
