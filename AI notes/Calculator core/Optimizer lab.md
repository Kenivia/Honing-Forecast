# Optimizer lab

How optimizer investigations run beside `optimizer::solve`. **The aim is learning, not a better optimizer**: find out exactly what makes a result good, by ablations and by swapping single parts into or out of `solve`. An attempt may read and reuse the production optimizer.

Each fact has one home:

| Question | Home |
| --- | --- |
| How does the lab work? | this note |
| What do we know, what is a candidate, what is open? | `Further learnings/Optimizer lab results.md` |
| What should not be tried again? | `Further learnings/Optimizer tried.md` |
| Old snapshots, iterations, changes that split the record | `Further learnings/Lab history.md` |
| The evidence | `arena_data/lab/<iteration>/<investigation>/report.md` |

## Layout

```text
arena_data/lab/
  runs.sqlite                  every result of the lab (gitignored)
  tools/                       shared scripts
  iteration<N>/
    brief.md                   aim, scope, status of each investigation
    report/                    the iteration's final report
    <investigation>/
      report.md                verdict first, then the evidence; frozen when finished
      final.patch              optional record of the code as it ended
      runs/                    <batch>_queue.json, _done.json, _run.log
      scripts/                 one-off analysis, only if needed
      scratch.sqlite           smoke tests (gitignored); deleted when finished
```

Invalid data goes to `ignored/junkyard`, never the lab.

## The process

All LLM work happens in supervised sessions; `pnpm lab` only does CPU work, started and stopped by hand.

1. **Brief**: the iteration's aim and its investigations.
2. **Implement**: a new approach is `crates/core/src/optimizer/lab/<name>.rs` with a write-up of the algorithm as built; ablation switches go in its `OFF`/`ON`, swap-ins into `solve` as off-by-default `Tuning` knobs.
3. **Smoke test** (below), then snapshot into `runs.sqlite`.
4. **Run** a batch (`tools/queue.py`, `pnpm lab`); `tools/logcheck.py <batch>` must pass.
5. **Read** with `tools/an.py` (or `cmp.py`, `pay.py`), writing findings into `report.md` as you go.
6. **Finish**: remove the knobs and switches that failed; verdict at the top of `report.md`; what is known into `Optimizer lab results.md`, what failed into `Optimizer tried.md` with its one key number; leave the code in the tree with every piece off and **stop for the user to commit** (no stash); delete `scratch.sqlite`.

## The method

1. **The build repeats its baseline**: with nothing set, 0 wins and 0 losses on `pay.py` against the run it should repeat.
2. **Read the code for bugs before judging.**
3. **Compare against `solve` at matched CPU**, set by set, then payload by payload, wins as well as losses.
4. **Ablate one piece at a time** with `lab_off`/`lab_on`.
5. **Swap good pieces into `solve`**, off by default, and test them there; a piece that works in a prototype may do nothing in `solve`, and why is worth knowing.
6. **Follow leads to the bottom**: starting a prototype from `solve`'s result (`fromsolve`) shows whether its machinery improves on `solve`'s endpoint.

## Running

A snapshot is `<investigation prefix>_<letter>` (`sur_a`), its batch the same letter.

```text
cargo build --release -p hf-arena
target/release/hf-arena.exe --db arena_data/lab/runs.sqlite snap sur_a
python arena_data/lab/tools/queue.py iteration2/surrogate/a sur_a sur_a:algo=surrogate,lab_effort=1
pnpm lab iteration2/surrogate/a
```

- **`queue.py`** writes one job per set (the standard list and its seeds are at the top of the script; about 20–25 minutes a variant). `--sets`, `--filter` (a substring) and `--seeds n` narrow it.
- **`pnpm lab`** runs the batch in order at below-normal priority. The queue (`[{name, cmd, note?}]`, commands from the repo root under `cmd`, so exe paths need backslashes) is re-read before each job. Ctrl-C stops; the same command resumes (`--resume`). `pnpm lab status <batch>` lists it. The driver is the snapshot's own exe, so a build meanwhile cannot touch it.
- **No reads of `runs.sqlite` while a batch writes**: rollback-journal mode and no busy timeout, so a long read kills the batch ("database is locked"). Analyse between batches.
- **One CPU user at a time**, or `cpu_s` means nothing. Compare `cpu_s` only within one batch against a plain baseline in it; `ref` in `cmp.py` corrects across batches where both sides have the reading (`Arena.md`, speed).
- **Compare CPU, not evaluations**: CPU per evaluation moves with the states searched.

**Tools** (`arena_data/lab/tools/`, run from the repo root, numbers in % of flow; each script's header has its arguments):

- `cmp.py`: variants per set against a base, matched by scenario, step and seed: mean diff, worst and best payload, wins and losses over 0.01%, hit, gap, evaluation and CPU ratios.
- `pay.py`: payload by payload, A against B.
- `an.py`: `cmp`, `pay`, `fmt`, `sets_of` in Python on a cached dump (`dump.py`, `ignored/lab_cache/runs.pkl`; about 110 s to rebuild after `runs.sqlite` changes, 4 s after). `sys.path.insert(0, 'arena_data/lab/tools'); from an import *`.
- `rep.py`: the repeat check of smoke runs against stored runs, seed by seed.
- `logcheck.py <batch>`: every job done, every variant summarised, no run died or failed its Monte Carlo check.
- A prototype's trace script (`attr.py`) stays in its investigation's `scripts/`.

## Smoke tests

Minutes, never in `runs.sqlite`: use the investigation's `scratch.sqlite`, which `snap` creates.

```text
target/release/hf-arena.exe --db <scratch> snap sur_a
target/arena/bin/sur_a.exe --db <scratch> --set small run sur_a --mode cold --seeds 3 --jobs 16
python arena_data/lab/tools/cmp.py <baseline> '' small 0 <scratch>
target/arena/bin/sur_a.exe --db <scratch> --set quick_dev run sur_a:<knob> --mode cold --seeds 3 --jobs 16 --filter chain_0017
```

1. **It repeats**: nothing set, on `small`, 0 wins and 0 losses against its baseline.
2. **It does what was meant**: the new knob with `--filter` on the payloads it is for.
3. **Promote**: from the same build, `snap` into `runs.sqlite` and queue the batch.

About 10 minutes; anything longer is a batch.

## How an attempt plugs in

- `Tuning::algo` names it (`lab::ALGOS`, dispatched by `lab::solve`; empty is `solve`). The arena's worker and `bench --solve` call `lab::solve`, so a variant is `<snapshot>:algo=<name>`.
- **CPU is the budget**: `Tuning::lab_effort` scales an attempt's work, 1 being about `solve`'s CPU. Attempts run on one thread and never stop on a clock.
- **Invalid states are rejected** (`lab::check::invalid`): such a run gets the error "invalid state: ..." and is never the best known.
- `lab_algos_repeat_and_are_valid` (arena worker tests) runs every name in `ALGOS` twice per dummy step.
- **Ablations**: `lab_off`/`lab_on`, joined by `+` (`<snapshot>:algo=price,lab_off=order+kicks`). Each file's `OFF` and `ON` list its switches, the `ON` ones commented there; with neither set it runs as before. `lab::check_switches` runs on every variant and rejects a name nothing reads, one in the wrong list, and unknown `kick_types`/`sur_kinds`, so a typo cannot run as the plain algorithm. `solve` reads the surrogate's switches when `sur_stall` > 0 and memetic's with `mem_polish`.

## Reading results

- **Noise**: effects under about 0.01% of flow are null unless they repeat on the same payloads across sets or seeds. The random stream shifts with any change, so check a "loss" seed by seed. The noisy payloads are listed in `Optimizer lab results.md`.
- **Judge on** the held-out real sets plus `quick_dev` and the regime sets (`advanced`, `free_taps`, `juice`, `small`, `large`, `speed_test`); since iteration 3 the `real3_*` sets (`Arena.md`, generated scenarios).
- **The best known is over all of `runs.sqlite`**, so it moves as results come in.
- **The baseline for new builds is `cut_a`**; a snapshot's plain label is the current default, not 48k (`Lab history.md`).
