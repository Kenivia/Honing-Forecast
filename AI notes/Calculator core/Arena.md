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

## The advanced honing table

`test_cases/adv_cache.sqlite` (`adv_table.rs`, gitignored) keeps the advanced honing distributions between runs, so that no solve works one out. Before a step is solved the worker puts every distribution its upgrades can ask for into `payload.adv_cache`; the core is not involved.

- **A row is one start position** (xp, balls, next free, next big, double balls, +30/+40) and holds all 361 strategies from it, about 230 kB. A start not in the file is worked out once, about half a second, and kept. `explore` has 121 starts, 3 of them fresh.
- **What it holds is what the core would compute, to the bit**, so a seed gives the same run with or without it: the `pilot` runs of session 21 have the metrics of session 18, all 33.
- **The `AdvMiss` counter is how many distributions a solve still had to work out.** It should be 0. If it is not, the table's key and the core's `AdvConfig` have come apart.
- **A snapshot from before the table does not use it**, so its time has the distributions in it and a new build's does not: on payloads that are mostly advanced honing that was three quarters of a solve or more.
- `bench` fills it too. `replan` does not: every upgrade played is a new start, and the file would grow by a row for each.
- Delete the file after any change to `compute_adv_dist`. Nothing notices a stale one.

## Scenarios

`test_cases/scenarios/*.json`, each `{ "steps": [payload, ...] }` with every step a full `Payload`. `pnpm arena dummy` writes the placeholder one; real cases are still to come. Its payloads carry four chest stacks, keyed "0" to "3": a three-way choice, an obtain-all with gold, fractional amounts, and one that gives special leaps. A worker's `Essence` carries `chest_states` next to `special_state`; one whose keys are not the reference's (a snapshot older than the keys has none) is left out and the reference's default is kept. A variant built before chests were keyed cannot read these payloads.

Every scenario runs **cold** (each step alone) and **chained** (each step is handed a `Previous`: the last payload, the last best `StateBundle`, and the optimizer's own blob, capped at `Tuning::max_blob_bytes`). A step is seeded the same way in both. A chained step starts from the plan of the step before (`Optimizer.md`, "Chained runs"); `carry=false` makes it a cold one again.

## Generated scenarios

`pnpm arena gen <set> --singles N --chains N --steps N --sweeps --stress` (`generate.rs`) writes a set into `test_cases/scenarios/<set>/`. Loading a folder does not descend, so a set is run with `--scenarios test_cases/scenarios/<set>` and the default folder stays the dummy.

- **A case is a point on 18 axes** (`AXES`: tier, size, level band, piece mix, jitter between pieces, advanced share, progress, wealth, imbalance, band split, juice owned, juice price, free taps, plan, chests, price noise, chest base share, chest optional share), each in [0, 1), and `build` turns the point into a payload. Drawing payload fields independently makes every case look alike; drawing these does not. The point is kept in the scenario's `meta.axes`.
- **Owned amounts are quantiles**: the wealth axis is how often the owned amount is enough under the default state, and imbalance spreads that between materials. So the same wealth means the same thing at any size. Juices, which the default state does not use, are a share of one juice per tap instead.
- **Singles and chains are points of a Sobol sequence** (seeds 1 and 2), so `--offset` gives a set that shares nothing with another, and a longer set starts with the shorter one.
- **A chain replays a player**: most steps solve the payload, do the upgrade the plan wants next (`generate::next_up` and `generate::play`, which `replan` shares) and apply one outcome of it (done, free tapped, or some failed taps in), by the rules `monte_carlo::simulate` uses. The rest are income, a price move, a goal added or removed. `meta.kinds` names what happened before each step. Unopened chests are not drawn on when a material runs out.
- **Sweeps** move one axis at a time around three anchors; **stress** is every upgrade a character has.
- **Chests are off** unless `--chests`: the axis is pinned to none and a chain's chest step becomes income. `explore` was generated without. See "Chests in a generated case".
- **`--no-adv`** pins the advanced share to none. `test_cases/scenarios/normal` is `gen normal --singles 500 --offset 3000 --no-adv`.
- **`--weekly N`** writes 2N chains of `--steps` weeks (`generate::weekly`, Sobol seed 3): a player with the same income every week who plans once a week. No chests, no advanced honing. A step is the payload at the start of a week.
  - The income is what the default state uses on average over about thirty weeks (0.7 to 1.6 of it), each material with its own constant noise; juices, books and free taps are a share of that or none.
  - `week_inc_*`: nothing is honed, only the income moves. All 30 steps have the same upgrades, and the cost goes to nothing as the stock grows.
  - `week_hone_*`: the player starts with little, and each week follows a short solve's plan (`next_up`, as chains do) for as long as what is owned covers the base materials of the next tap, buying only juices; a tap short leaves the upgrade in progress. The chain ends when the upgrades do.
  - `test_cases/scenarios/weekly` is `gen weekly --weekly 24 --steps 30`: 48 chains, 24 of each.
- A case whose generation panics is left out and the panic is on stderr. A chain step the optimizer panics on is kept, with the player following the default state.

## Chests in a generated case

The cases are meant to cover what the optimizer can be handed, not what a player usually has.

- **The `chests` axis is how many stacks**: none under 0.12, then 1 to 8.
- **A stack is sized against the average used with every tap taking all it can** (`max_used`: a juice and a book on every tap, both on every advanced tap, the most over which id of a pool it is), not against what the default state uses, which is no juice at all. Free taps are sized against what it takes on average to free tap every upgrade.
- **Base materials from a stack are mostly small**: `chest_base` gives 1 to 15% of that reference four times in five and 15 to 150% otherwise. **Optional ones (juices, books, scrolls, free taps) are anything**: `chest_optional` is 2% to 300%. Each stack has its own noise on top, and stacks add up.
- **What a stack gives is one of 14 families** (`chest_family`), after the game's chests: red or blue; fusion, free taps or shards; free taps, red or blue; all five of those; free taps or leapstones; lava or glacier, alone or with books; books only; bundles of several materials an option; fusion, lava or free taps; gold or a juice; and obtain-alls of base materials, of optional ones and of free taps. A book no upgrade of the case can use is a legitimate option.
- **A stack is 1 to 300 chests**, fewer where the whole stack gives only a few units. Four Serca stacks in ten are converted T4 materials: fifths of a unit a chest, roster bound at best.
- **`test_cases/scenarios/chests`** is `gen chests --singles 300 --chains 60 --steps 8 --sweeps --chests --offset 1000`, less the sweeps that have no chests: 402 scenarios, 822 payloads, 43 of them without a chest and 260 with a choice that includes free taps. The offset keeps its names apart from `explore`'s, which sets look scenarios up by.
- `python scripts/arena/chest_report.py [session] [--by choice|stacks|kind]` puts a session's variants side by side with the gap as a share of `flow`.

## Sets

A set is a named list of scenarios for one purpose, in `test_cases/sets/<name>.txt`: one scenario name a line, `name:4` for only its first 4 steps, `#` for a comment, the first of which is what the set is for. Names are looked up under the scenarios folder and the folders in it, so a set can mix generated folders.

- `pnpm arena sets` lists them. `--set <name>` on `run`, `verify` and `carry` uses one in place of the folder: `pnpm arena run baseline current --set pilot`.
- `pnpm arena define <name> --note "..." --kind single --where n=32..41 --where juice_owned=0.4..0.7 --steps 4 --limit 12` writes one from filters: `--kind` is a prefix of the name (single, chain, sweep, stress), a `--where` key is an axis of `meta.axes`, `n` (upgrades of the first step) or `steps`. The command is kept in the file. Or write the file by hand.
- A truncated chain's steps are the same payloads, so they share their best known with the full chain.
- `pilot` is the quick first run for a new variant: 16 payloads, 8 CPU seconds a seed. Two scenarios without chests are from the pilot of before (a chain cut to four steps and a single, picked from session 14), and their runs match sessions 18 to 25 seed for seed. Seven have chests (picked from session 29): optional materials only, almost no base materials over 8 stacks, mostly base materials, Serca fifths with a free tap choice and without, and two chains cut to three steps, one with 8 stacks and one where a chest step adds a stack. All were hit a quarter to three quarters of the time, which is where a variant can move the result either way. The three that were dropped (`chain_0057_serca_n8`, `single_0466_t4_n10`, `single_0085_t4_n17`) are still in `explore`.
- `nh_dev` and `nh_test` are the singles of the `normal` folder with 4 upgrades or more, the even ones (158) and the odd ones (157): normal honing only. The first is what the optimizer of October 2026 was developed on, the second was kept out of it. `nh_quick` is every fourth of `nh_dev`.
- `test_cases/scenarios/seeded` is `nh_dev` with the state `v0` found on seed 0 (session 50) given in each payload, as the frontend gives the plan it shows when the user optimizes again. Its names start with `seed_`; compare with the same payload unseeded by name.
- `weekly` is the 48 weekly chains, `weekly_dev` the first six of each kind, which the chained runs were tuned on, and `weekly_test` the other 36.
- `chests` is every case of the `chests` folder that has a choice in it, chains cut to three steps: 349 scenarios, 453 payloads, about 15 minutes for three variants and three seeds at 16 jobs.

## Against a fixed baseline

The best known moves every time a variant finds something new, so a hit rate read today is not the one read tomorrow. For the optimizer work of October 2026 one variant was frozen instead:

- **`v0` is a snapshot of the code at `2dacbf9a`**, the optimizer before that work: one chain of 24,000 evaluations and a pass of responses at the end. Run it next to a new variant, or in a session of its own, and compare with `python scripts/arena/diff_report.py <sessions> [--base v0] [--by n|kind|week] [--like text] [--strip text] [--worst 10]`.
- **`diff` is signed**: a variant's mean over seeds less the baseline's on the same seeds, per payload, as a share of the flow of the baseline's best seed. `+` and `-` count the payloads that moved by more than `--eps`. Nothing in it depends on what else is in the database.
- `hit`, `x10` and `gap` are still against the best any run has reached (within `eps`, within ten times it, and the mean), and so still move.
- `spread` is the best seed less the worst of a payload, and `same` the share of payloads whose seeds all agree: the run to run variance.
- The database keeps every run's metric and state, so all of it can be read another way later.

**A run with an error is never the best known.** An experiment that could put the same id twice on a tap was scored well above any real plan by the evaluator, which does not check; its 6 runs were marked as errors by hand (`error` starts with `invalid state`). Check a new kind of move for that before believing a new best.

**The driver checks a variant's keys against its own `Tuning`**, so the driver has to be a build that knows them. The working loop: build, `snap <label>`, copy `target/release/hf-arena.exe` somewhere (`target/arena/driver.exe`) and run that copy with the labels. The tree can then be edited and built while a run goes on, and a variant is always a snapshot.

## The exploratory run

`test_cases/scenarios/explore` (600 singles, 200 chains of 8, 150 sweeps: 2316 payloads) and `explore_stress` were run cold in October 2026. `python scripts/arena/explore_report.py [session] [folder]` prints what was read off a run: hit rate, time and when the best was found by size, by axis, by sweep level and by what happened before a chain step. Run it again after a full run of a new variant.

| session | what |
| --- | --- |
| 14 | `explore`, 8 seeds, the code at `69fef473` (the `baseline` snapshot) |
| 20 | `explore`, 8 seeds, the evaluator sped up (`Evaluation pipeline.md`) |
| 18 | `pilot`, 3 seeds, `--jobs 1`: `baseline`, `current`, and `current` with the memo off and with the old root finding too |
| 16, 19 | `stress`, 1 seed, `--jobs 2`: `baseline` and `current` |
| 21 | `pilot`, 3 seeds, `--jobs 1`, with the advanced honing table: 13.0 CPU seconds a seed against 21.0 |
| 23 | `explore`, 8 seeds, the `adv_table` snapshot: the faster distributions and the table. Every run has the state and metric of session 20; 13468 CPU seconds became 5867, in 59 minutes |
| 25 | `pilot`, 3 seeds, `--jobs 1`, after the optimizer's own work was cut (`Optimizer.md`): 11.0 CPU seconds a seed |
| 28 | the `chests` folder, 1 seed: the random chest state (`chest_base` snapshot) against chest moves and polishes |
| 29, 31, 32 | the `chests` set, 3 seeds: variants of the chest moves (`Optimizer.md`, "Chests"). In 31 the compound variants died, a bug of theirs |
| 30 | the `chests` set, 1 seed, 96,000 evaluations: the best known to measure against |
| 33 | the new `pilot`, 3 seeds, `--jobs 1`: `chest_base` 7.6 CPU seconds a seed, `chest_moves` 7.9 |

The optimizer round of October 2026 (`Optimizer.md`, "The shape of a solve"), all cold at 16 jobs unless said:

| session | what |
| --- | --- |
| 50 | `nh_dev`, 8 seeds: `v0` at 24,000 and at 48,000 evaluations |
| 51, 52, 54 | `nh_dev`, 4 seeds: single knobs, the free tap order's fixes, chains |
| 55, 57, 61 to 64, 67, 68 | `nh_dev`, 4 seeds: mixed responses, leaving early, merge, exchange, chains by size, the streak fix |
| 70 | `nh_dev`, 4 seeds: `v1`, the new defaults, and other chain lengths |
| 73 to 76 | `v0` against `v1`: `nh_test` (8 seeds), `explore` singles 100 to 199 (3), the `chests` set (1), `nh_quick` at a streak length of 10 (4) |
| 59, 60, 65, 66, 71, 72, 77 | `weekly_dev`, 2 seeds, cold and chained: what a carried plan is worth and when to stop |

`v1` and `v2` are snapshots of the defaults at the end of it; `v2` differs only in `carry_stall`. `v6` is the defaults as handed over, `v7` that with an id counted once a tap and the crossover kept to plans of one length, `v8` that with the cumulants fixed for hard tilts (sessions 97 and 98 have `v7`). The `k*` labels in those sessions are the steps on the way, with their diffs in the `snapshot` table.

Sessions 3 to 5 are of the payloads before the folder was generated again and match nothing in it now. Session 17 is session 20 with a root finder bug that killed 3 of its jobs; 15 is 18 before that fix.

**Generating a set again gives other payloads once the evaluator has changed.** The owned amounts are quantiles read off the evaluator, so generating `explore` again after the fix of October 2026 moved the amounts of two thirds of the singles and sweeps by a unit or so. A payload that moved is a new case with no history, which is why the whole folder was run again and not only its chains.

## The evaluator before and after its speed-up

`baseline` is a snapshot of the code at `69fef473`. What was measured against it, to be repeated after any change to the evaluator:

- **CPU time with one job at a time.** `pilot`: 58.0 seconds to 21.0 (29.6 with `max_memo_size=0`, 33.8 with `taylor_skip=0,seed_biased=false` as well). `stress`: 1517 to 274. Payloads that are mostly advanced honing gain least (`single_0085_t4_n17`: 5.8 to 4.6), because half of what is left of a solve is `compute_adv_dist_wrapper` on a cache miss.
- **The same search.** With the memo off, most `pilot` payloads find their best at the same iteration with the same gap as `baseline`, seed for seed: the metric is the same to the digits the annealer's decisions hang on. The memo changes the 9th digit and the search with it.
- **The metric state by state** (`bench --out` and `scripts/arena/bench_diff.py`), over the given state, two scrambles and the best and worst found of every `explore` payload, 11465 states: 99.6% within 1e-6 of the baseline's metric, 10 further than 1e-5, none further than 1e-4. The 3 furthest move as far with the old root finding, on rounding alone: they are thresholds deep in a tail, where the root finder's tolerance on the derivative leaves the root loose. On all of them the saddlepoint's own error against `max_lattice_work` at 2e9 is 10 to 1000 times the movement.
- **`stress`** on 100 singles: the table of calls by error bucket is the same but for a handful of calls on two rows. **`verify --set juice`**: the same 13 failing rows, largest 66 standard errors, no row changing side.
- **A full pass.** Over all of `explore`, 8 seeds at 16 jobs (sessions 14 and 20): hit rate 77.5% and 77.5% (the difference by payload is +0.04 points, within 0.5 either way), the new best better by more than 1e-4 on 86 payloads and worse on 75, no job died in either, and 39147 CPU seconds became 13468.

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

`test_cases/scenarios/regressions` holds payloads that broke the evaluator. `pnpm arena run --scenarios test_cases/scenarios/regressions --mode cold --seeds 3` is the check: seed 2 of the first one used to panic, and every seed of `no_root_week_inc_0022` (the second week of a weekly chain) did until the root finder's failure stopped being one.

**`verify` on that weekly chain is off by 1%**, on its first week too, where nothing ever panicked, and as much before that change as after: Red, Fusion, Shards and the total of the given state and of scrambles, the evaluator cheaper than the simulation by 5 to 6 standard errors. It is the saddlepoint formula: with the free taps taken out it is still there (2% on one scramble), and with `max_lattice_work=2000000000`, which answers every material exactly, every row passes within 0.4 standard errors. The payload is one weapon from +8 to +24, so a few high levels make most of the spread of the sum. As a share of flow it is 0.12 to 0.28%; the 1 to 2% is of the total, which credit makes small.

**How far the default evaluator is from the exact one** (October 2026, `max_lattice_work=2000000000` as the reference, which agreed with 100k-sample Monte Carlo on every total of `nh_dev` and `weekly_dev` but for rare events under one gold). Error as a share of the step's best known `flow`:

| states | over 1e-4 | over 1e-3 | worst | optimistic, of those over 1e-4 |
| --- | --- | --- | --- | --- |
| `nh_dev`, given and 3 scrambles (632) | 18.5% | 2.5% | 0.49% | 82% |
| `nh_dev`, best known plan (158) | 12.0% | 1.3% | 0.27% | 95% |
| `weekly_dev`, given and 1 scramble (634) | 14.4% | 1.1% | 0.16% | 100% |
| `weekly_dev`, best known plan (317) | 20.2% | 3.2% | 0.22% | 97% |

- On `nh_dev` optimized plans are no worse than random ones; on `weekly_dev` somewhat worse, so the optimizer leans a little towards the error there. Where best known and v0's seed 0 differ (54 and 159 steps), the exact evaluator orders them the other way on 3 and 4, each by under 2e-5 of flow.
- In a weekly chain it grows as the chain goes on: 9% of states over 1e-4 in weeks 0 to 4, 27% in weeks 25 to 29.
- Not a matter of the number of upgrades: 13% over 1e-4 at 4 to 7 upgrades, 23% at 25 and up. Red, Fusion and Serca Red carry it.
- The exact evaluator is slow: 790 states took 40 minutes on 8 threads and 1585 weekly ones 87 minutes on 12, against seconds.
- Scripts in the session scratchpad only; `verify` with the two variants and `bench --states --out` under each tuning reproduce it.

**`verify` on the `juice` set fails 13 rows, and they are the saddlepoint formula, not the simulation.** The worst are juice rows (`Glacier's Breath`, up to 66 standard errors). On those states the simulation agrees with an exact sum over the evaluator's own supports, and the evaluator does not: the material has 3 or 4 upgrades with more than one outcome, just over both `max_brute_size` (65,400 against 50,000) and `max_lattice_work` (234,000 against 200,000), so it gets the saddlepoint formula, which is 0.01 to 0.06 off there. That is multiplied by the threshold and the price. A failing juice row means look at which method the material got before suspecting the supports.

**`metric_ref` cannot see an evaluator that is wrong.** It is the driver's evaluator, so a state that exploits its error scores well under every variant. Compare against `max_brute_size` at its maximum (`replan` does) when that is the question.

## Stopping and resuming

`run --resume` reuses the latest session started with exactly the same arguments and skips the jobs that already have a row. Ctrl-C stops cleanly: nothing a killed worker was doing is recorded. Jobs go seed by seed, so a session stopped early has covered every scenario about equally.

## The three measurements

- **Correctness** (`verify`): for every step, the given state and some scrambled ones (`optimizer::scramble`, random valid moves) are simulated and compared per material and in total. A row passes when the difference is within `z` standard errors plus `rel_tol` of the value. Failures are rows, not panics. The same command times one evaluation. `run --mc` does the check on the first seed's result instead.
- **Accuracy** (`run`): the gap to the best metric any run in the database has reached on the same payload. The driver re-scores every result with its own evaluator at default tuning (`metric_ref`), because a variant's own `metric` is not comparable once the evaluator differs. `hit %` is the share of seeds within `--eps` of the best known.
  - Next to `metric_ref` a run has `gross`, the same with every leftover worthless (the gold spent), and `flow`, the market value of what the state uses on average. `pnpm arena rescore` fills them in for older runs.
  - **What the gap is divided by is undecided.** The report divides by the best metric, which is near zero wherever credit cancels cost or everything is owned. Spent plus credit (`metric_ref - 2 * gross`) vanishes in the same cases. `flow` does not.
- **Probabilities** (`stress`, `stress.rs`): `pnpm arena stress [--scenarios dir | --set name] [--scrambles 3] [--tuning key=value]`. For the given state and scrambles 20, 200 and 2000 moves away, every material's probability is asked for at some 70 thresholds (packed around the mean, at quantiles, at the ends of the range, at random), plain and biased, and compared with an exact convolution done here. It reads nothing from the database and writes nothing. The table is by method, region and kind; lattice, brute and trivial must be exact, and a panic is counted by where it was. `verify` cannot see what this sees: it compares the metric at the payload's own thresholds, and within Monte Carlo noise. Materials whose convolution would be too big are left out and counted.
- **Speed**: thread CPU time around `solve` only, plus counters. The counters are exact and repeatable; CPU time is not, see below.
  - **`eval_s` and `decide_s` split a solve's `wall_s`** into the evaluations (`Probe::clock` and `Probe::evaluated` around `metric_router`) and everything else the optimizer does: choosing a move, copying states, the best list. They are wall time, because thread CPU time comes in 15 ms ticks, and are empty for runs from before October 2026 and for a snapshot that does not send them. It was 29% of `pilot` at one job and 19% of `explore` in session 23, and is 12% of `pilot` since (`Optimizer.md`, "What a move costs").
- **The evaluator alone** (`bench`, `bench.rs`): `pnpm arena bench [--scenarios dir | --set name] [--filter text] [--tuning key=value] [--states file] [--out file] [--jobs 1]` evaluates each step's given state, two scrambles of it and any states handed in (`--states`, a json of `"scenario:step"` to a list of essences) once for the counters, then repeatedly for the time. `--out` writes one line a state with its metric, which is how two builds of the evaluator are compared state by state. It reads and writes no results database. The repeats are of one state, so pass `max_memo_size=0` or they time the memo. `--solve <seed>` times one solve a step instead. `python scripts/arena/bench_diff.py before.tsv after.tsv` puts two `--out` files side by side.

## The state space

`space.rs`, for what `State space.md` reports. None of it writes the database.

- `pnpm arena tiny [--count 60] [--lone 30] [--lone-taps 20] [--cap 3e6]` writes `test_cases/scenarios/tiny`: cases of one to three normal upgrades with taps failed until every plan can be tried, and lone upgrades with 20 taps left. No chests, no advanced honing, free taps on half.
- `pnpm arena exhaust --scenarios test_cases/scenarios/tiny [--block 10] [--out file.jsonl]` tries every plan and every free tap order of each case on the exact metric, and reports the best, the best the streaks can make, what the annealer finds and what `ascend` reaches. A lone upgrade with more plans than `--cap` is not enumerated: its best plan comes from `Response::best`. `--block` is the streak length (`Optimizer.md`), under which only the plans that keep to it are tried. About 10 minutes at 1, seconds at 10.
- `pnpm arena ascend [--set name] [--filter text] [--tuning key=value] [--out file.jsonl]` starts from each step's best known state, and from that state with nothing on its normal upgrades, and lets each upgrade in turn take its best plan, of the streaks only and of every plan. The gain is a share of `flow`.
- `pnpm arena landscape --filter text --x 0f --y 0b --out file.tsv` writes the metric over two streak lengths of each normal upgrade, the rest as best known: under the default tuning, under `--tuning`, and as the response has it. An axis is a pool and `f` or `b`. `python scripts/arena/landscape_plot.py file.tsv out_dir` draws them.

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
- **The Monte Carlo reads the bands and the free-tap budget of the state it is given**, so it checks the evaluator under a chest state, not `resolve_chests` itself; that has a unit test in `chest.rs`. `scramble` opens the chests at random first (`randomize_chest_state`), so scrambled states have random chest states.
- **A snapshot from before `chest_moves` opens the chests at random**, once, so its gaps on a scenario with chests are tens of percent and say nothing about its annealer.
- **`--jobs` defaults to half the logical cores.** Pass the number of cores for a long run, and 1 when the time is what is measured.
- **A mean of gaps over flow is ruled by a few payloads with almost no flow** (millions of percent). `chest_report.py` prints it, but read the median, the 90th percentile and the hit rate.
- **Where everything is owned the true metric is zero and the optimizer chases approximation error**: best states with a few thousand gold of negative cost were found on jobs moving millions. Gaps there are noise, whatever they are divided by.
- **Best known is per payload.** Editing a scenario's payload makes it a new case with no history. The payload is hashed with its keys sorted; hashed as serialised, its hash maps gave a new row on every driver run.
- The database (`test_cases/arena.sqlite`) is gitignored and can be deleted; snapshots in `target/arena/bin` then lose their rows and must be taken again.
