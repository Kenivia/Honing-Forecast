# Arena

`crates/arena` (`hf-arena`) is the native test harness for `crates/core`, run as `pnpm arena <command>` (`cargo run --release -p hf-arena --`; `--help` lists the commands and their flags). It measures **correctness** (the evaluator against Monte Carlo), **accuracy** (the optimizer's result against the best known) and **speed**, and keeps every result in `arena_data/arena.sqlite` (gitignored, deletable; snapshots then need taking again).

What was ruled out, and the measurements behind the traps below, are in `Further learnings/Arena tried.md`.

## Variants and snapshots

- **Constants** are `Tuning` fields (`crates/core/src/tuning.rs`). A variant overrides them by name: `current:max_iters=6000,y_value_tol=1e-5`. A new knob needs only a field and a read.
- **Code** is a snapshot: `pnpm arena snap <label>` copies the binary just built to `target/arena/bin/<label>.exe` and stores the commit and the working tree's diff (`git apply` restores it). `current` is the working tree as built.
- **The driver checks a variant's keys against its own `Tuning`**, so it must be a build that knows them. Loop: build, `snap <label>`, run a copy of `target/release/hf-arena.exe` (`target/arena/driver.exe`) so the tree can be rebuilt meanwhile.

## How a run executes

The binary is driver and worker. The driver loads scenarios, starts one worker process per job (a variant's exe run as `worker`, one job as JSON on stdin, one JSON line a result on stdout) and alone writes the database.

- **Fields added to `Job` or `StepResult` need a serde default**, or old snapshots stop working. The stored state is `StateEssence` (JSON field names are API for the same reason).
- A panic kills one job; it is recorded with its stderr as `died`.
- Jobs go seed by seed, variants taking turns, so `--jobs 1` has no drift between variants and a session stopped early covers every scenario about equally.
- **A run with an invalid state is never the best known**: the driver checks every state (`optimizer::lab::check::invalid`) and records an `error` starting `invalid state`.
- `run --resume` reuses the latest session started with exactly the same arguments; Ctrl-C stops cleanly; `--first-seed N` adds seeds in a new session. A one-step scenario only needs `--mode cold`.

## The advanced honing table

`arena_data/adv_cache.sqlite` (`adv_table.rs`, gitignored) holds advanced-honing distributions between runs: one row per start position with all 361 strategies, computed once (about half a second) and bit-identical to what the core would compute. The worker fills `payload.adv_cache` before a solve. **`AdvMiss` should be 0**; if not, the table's key and `AdvConfig` have come apart. `bench` fills it, `replan` does not. **Delete the file after any change to `compute_adv_dist`**: nothing notices a stale one.

## Scenarios

`arena_data/scenarios/*.json`, each `{ "steps": [payload, ...] }`. Folders are not descended, so a generated folder is run with `--scenarios arena_data/scenarios/<set>` or through a set. `pnpm arena dummy` writes the placeholder (four chest stacks of each kind).

- Every scenario runs **cold** (each step alone) and **chained** (each step handed a `Previous` of the step before; `Optimizer.md`, chained runs), seeded the same in both. **A star** (`meta.star`) hands every step its first step's `Previous`: the Choices page's run without a choice, then its options.
- **Best known is per payload**, hashed from the serialised `Payload` with sorted keys. Editing a payload, or adding or removing a `Payload` field, makes new cases with no history.
- A Serca payload needs the four Serca books among its materials (`python scripts/arena/add_serca_books.py`).
- `arena_data/scenarios/regressions` holds payloads that broke the evaluator: `pnpm arena run --scenarios arena_data/scenarios/regressions --mode cold --seeds 3`.

## Generated scenarios

`pnpm arena gen <set> --singles N --chains N --steps N [--sweeps] [--stress] [--chests] [--no-adv] [--weekly N] [--offset N]` (`generate.rs`).

- **A case is a point on 18 axes** (`AXES`, documented there; kept in `meta.axes`), which `build` turns into a payload. Owned amounts are quantiles read off the evaluator, so **regenerating after an evaluator change gives other payloads**.
- Singles and chains are Sobol points (`--offset` for a disjoint set). **A chain replays a player**: solve, do the upgrade the plan wants next (`next_up`, `play`), apply one outcome; other steps are income, a price move, a goal change (`meta.kinds`). **Sweeps** move one axis around three anchors; **stress** is every upgrade a character has; `--weekly` writes constant-income chains (`generate::weekly`, `week_inc_*` and `week_hone_*`). Chests are off unless `--chests`.
- A case whose generation panics is left out; a chain step the optimizer panics on is kept, the player following the default state.

The folders:

| Folder | Made by |
| --- | --- |
| `explore`, `explore_stress` | generated without `--chests` (command not recorded) |
| `normal` | `gen normal --singles 500 --offset 3000 --no-adv` |
| `weekly` | `gen weekly --weekly 24 --steps 30` |
| `chests` | `gen chests --singles 300 --chains 60 --steps 8 --sweeps --chests --offset 1000`, less sweeps without chests |
| `seeded` | `nh_dev` with `v0`'s seed-0 state given in each payload (`seed_` names) |
| `tiny` | `pnpm arena tiny` |
| `choices`, `income` | `pnpm exec vitest run --config scripts/arena/vitest.export.config.ts`: the real save `tests/income-fixture.json` through the pages' own job builders (`export_pages.test.ts`), all at streak 10 |
| `b10` | `speed_dev` and `speed_test` at `min_resolution` 10 (`b10_` names) |
| `real`, `real_income` | `vitest.export_real.config.ts` (`export_real.test.ts`): real characters from `ignored/analytics` pulls, half at streak 1 |
| `real3` | `vitest.export_real3.config.ts` (`export_real3.test.ts`): every pull, snapshots grouped into families; singles `r3_`, income chains `r3inc_`, family chains `r3seq_`, each at `_s1` and `_s10`; rosters split 50/50 into `_dev`/`_test` sets. Rebuild after a new pull |

How each export picks and builds its payloads is in the test file's header. `python arena_data/explore_report.py [session] [folder]` and `chest_report.py [session] [--by choice|stacks|kind]` summarise a run by axis or chest shape.

## Sets

`arena_data/sets/<name>.txt`: a scenario name a line (`name:4` for its first 4 steps, `#` comments, the first saying what it is for). Names are found in every scenario folder, so a set can mix them. `pnpm arena sets` lists them with that comment; `--set <name>` works on `run`, `verify`, `carry`, `bench`, `stress`. `pnpm arena define <name> --note ... --kind single --where n=32..41 --where juice_owned=0.4..0.7 --steps 4 --limit 12` writes one from filters and keeps the command in the file.

The ones to know: `pilot` (a quick first look), `small`, `quick_dev` (about 380 payloads, 2 minutes a variant and seed at 16 jobs), `opt_dev`/`opt_test`, `real3_*` (iteration 3's judges), `cpu_check` (CPU time at `--jobs 1`), and the regime sets `advanced`, `free_taps`, `juice`, `large`.

## Against a fixed baseline

The best known moves whenever any run finds something new, so compare against a frozen snapshot instead: `python arena_data/diff_report.py <sessions> [--base <label>] [--by n|kind|week] [--like text] [--strip text] [--worst 10] [--db file]`.

- `diff` is signed: a variant's mean over seeds less the baseline's on the same seeds, per payload, as a share of the baseline's flow. It depends on nothing else in the database.
- `hit`, `x10`, `gap` are against the best known (within `eps`, within ten times it, mean) and still move. `spread` and `same` are the run-to-run variance.

## Carrying a plan over, and re-planning

- `pnpm arena carry [--set chains] [--rows]`: each chain step's best known state carried onto the next step (`carry::carried`, the same rule as `Plan::carry`) and scored against its default and best under `Tuning::reference()`. `recovered` is the share of the way from default to best. Needs runs in the database.
- `pnpm arena replan --set replan --paths 30 --every 1` (`replan.rs`): plays each scenario's first step to the end many times, re-solving from the carried plan every `--every` upgrades. The gain of a re-plan is `resolved - carried`; a `control` solve that learns nothing measures the optimizer's slack, and the first paths are re-scored exactly. Rows in the `replan` table (path -1 the starting plan). Not covered: chests, re-planning inside an advanced upgrade, planning knowing it will be re-planned.

## The measurements

- **Correctness** (`verify`): the given state and scrambles (`optimizer::scramble`) simulated and compared per material and in total, the evaluator at `Tuning::reference()`. A row passes within `z` standard errors plus `rel_tol`. `run --mc` checks the first seed's result. A failing row is usually the method, not the simulation: look at which method the material got; `max_lattice_work=2000000000` answers exactly.
- **Accuracy** (`run`): the gap to the best metric any run reached on the payload, every result re-scored at `Tuning::reference()` (`metric_ref`, 1e-5 of flow at worst), so a state exploiting the optimizer's own approximation does not score. `hit %` is within `--eps` (1e-4). Runs also store `gross` (every leftover worthless) and `flow` (market value of what the state uses), which gaps are divided by; dividing by the metric is meaningless near zero. `pnpm arena rescore` fills these for old runs.
- **Probabilities** (`stress`, `stress.rs`): every material's probability at some 70 thresholds, for the given state and scrambles 20, 200 and 2000 moves away, against an exact convolution; plus `leftover` rows, E[(t - S)+] by the metric's own route. Lattice, brute and trivial must be exact. Reads and writes no database. It sees what `verify` cannot (thresholds other than the payload's, no Monte Carlo noise).
- **Speed**: thread CPU time around `solve`, plus exact counters (`Probe`). `eval_s` (inside `StateBundle::evaluate`) and the rest of `wall_s` split evaluating from deciding; `us_<key>` counters are `Probe::time` phases. **`ReferenceUs`** is the machine's speed: each driver thread times fixed work (`worker::reference_s`, free of hf-core on purpose, never to be changed) before its first job and every 5 minutes. `cpu_s / ReferenceUs` compares across batches, roughly; trust it on sessions of 10 minutes or more, and keep a plain baseline in each batch anyway.
- **The evaluator alone** (`bench`): `pnpm arena bench [--set name] [--filter text] [--tuning key=value] [--states file] [--out file] [--jobs 1]` evaluates each step's given state, two scrambles and any `--states` (`"scenario:step"` to essences) for counters and time; `--out` writes metric, seconds and flow a state (`python arena_data/bench_diff.py before.tsv after.tsv`). Pass `max_memo_size=0` or the repeats time the memo. `--solve <seed>` runs whole solves: their `--out` metrics before and after a refactor must be identical.

**After any change to the evaluator**, against the build before: CPU at `--jobs 1` on `pilot` and `stress`; the same search with the memo off; `bench --out` state by state; `stress`; `verify`; a full pass of `explore`.

## The state space

`space.rs`, for `State space.md`; none of it writes the database.

- `pnpm arena tiny [--count 60] [--lone 30] [--lone-taps 20] [--cap 3e6]` writes `scenarios/tiny`: one to three normal upgrades with every plan triable, and lone upgrades with 20 taps left.
- `pnpm arena exhaust --scenarios arena_data/scenarios/tiny [--block 10] [--out file.jsonl]`: every plan and free-tap order on the exact metric, against the best streak plan, the annealer and `ascend`. About 10 minutes at block 1.
- `cargo test --release -p hf-arena lone_exact -- --ignored --nocapture` (and `lone_dump`, `lone_moves` with `DUMP_OUT=<file>`): lone upgrades whose best plan is not a streak plan, scored with the evaluator and `monte_carlo::exact_single`.
- `pnpm arena ascend [--set name] [--tuning ...] [--out file.jsonl]`: from each best known state, and from it emptied, each upgrade takes its best plan in turn, of streaks only and of all plans.
- `pnpm arena landscape --filter text --x 0f --y 0b --out file.tsv`: the metric over two streak lengths of each normal upgrade; `python arena_data/landscape_plot.py file.tsv out_dir` draws it.

## Looking inside a run

`Probe` (`crates/core/src/probe.rs`, filled only with the `trace` feature): `count(Counter::X)` (add a name to `counters!`), `put`/`put_text` rows at the iteration of the last `at(..)` (into the `trace` and `trace_text` tables), per-iteration rows only under `if probe.detail()` (`--detail`). `pnpm arena trace <run id> [key]` prints a run's rows as CSV; `pnpm arena compare [variants] [--session N]` a session's tables. Counters and tuning are JSON columns (`json_extract`).

## Traps

- **A seed must repeat exactly**: no global `rand::random_*`, no float sums in hash-map order (`brute.rs` uses fixed hasher seeds). `a_seed_repeats_exactly` guards it.
- **CPU time needs `--jobs 1`**; many workers inflate it about threefold, and Windows counts thread time in 15 ms ticks. `--jobs` defaults to half the logical cores; pass the core count for a long run.
- **Monte Carlo samples normal honing from the upgrade's own tap distribution**, so it does not check that distribution itself; and it reads the bands and free-tap budget of the given chest state, so `resolve_chests` has its own unit test.
- **Read medians, the 90th percentile and hit rate, not a mean gap over flow**: a few near-zero-flow payloads rule the mean. Where everything is owned the true metric is zero and gaps are noise.
