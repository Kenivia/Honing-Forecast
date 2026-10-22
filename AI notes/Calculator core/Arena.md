# Arena

`crates/arena` (`hf-arena`) is the native test harness for `crates/core`. It measures three things and keeps every result in a SQLite file:

- **Correctness**: the evaluator against a Monte Carlo simulation.
- **Accuracy**: how good the optimizer's result is, per test case.
- **Speed**: CPU time and work counters.

Run it with `pnpm arena <command>`, which is `cargo run --release -p hf-arena --`.

What past sessions measured (the session list, the baselines `v0` and `baseline`, the evaluator's error against the exact one, the re-planning run) is in `Arena tried.md`. Read it only when trying something new, or to find which session holds a result.

## Variants and snapshots

A variant is what gets compared. There are two ways to make one, and they combine:

- **Constants** live in `Tuning` (`crates/core/src/tuning.rs`), a serde-skipped field on `StateBundle` whose default is production. A variant overrides fields by name: `current:max_iters=6000,y_value_tol=1e-5`. A new knob needs a field there and a read where the constant used to be; nothing in the arena changes.
- **Code** is a snapshot. `pnpm arena snap <label>` copies the binary that was just built to `target/arena/bin/<label>.exe` and stores the git commit and the working tree's diff against it in the database. No commit or branch is involved. Restore a variant's code with `git apply` on that diff.

`current` is the working tree as built, without a snapshot. So the loop for several code changes is: edit, `snap a`, edit, `snap b`, then `pnpm arena run baseline a b` once. `optimizer::solve` always exists; there are no optimizer version folders or cargo features.

**The driver checks a variant's keys against its own `Tuning`**, so the driver has to be a build that knows them. The working loop: build, `snap <label>`, copy `target/release/hf-arena.exe` somewhere (`target/arena/driver.exe`) and run that copy with the labels. The tree can then be edited and built while a run goes on, and a variant is always a snapshot.

## How a run executes

The binary is both driver and worker. The driver (always the current build) loads scenarios, starts one worker process per job, and is the only thing that writes the database. A worker is a variant's binary run as `worker`: one job as JSON on stdin, one JSON line per result on stdout. Consequences:

- A snapshot never needs to know the current schema. It does need to understand the job, so **fields added to `Job` or `StepResult` need a serde default** or old snapshots stop working.
- A panic in a variant kills one job. It is recorded with its stderr and shown as `died`; the rest of the session carries on.
- Jobs are ordered so that variants take turns, seeds outermost. With `--jobs 1` that removes drift between variants.

## The advanced honing table

`arena_data/adv_cache.sqlite` (`adv_table.rs`, gitignored) keeps the advanced honing distributions between runs, so that no solve works one out. Before a step is solved the worker puts every distribution its upgrades can ask for into `payload.adv_cache`; the core is not involved.

- **A row is one start position** (xp, balls, next free, next big, double balls, +30/+40) and holds all 361 strategies from it, about 230 kB. A start not in the file is worked out once, about half a second, and kept.
- **What it holds is what the core would compute, to the bit**, so a seed gives the same run with or without it.
- **The `AdvMiss` counter is how many distributions a solve still had to work out.** It should be 0. If it is not, the table's key and the core's `AdvConfig` have come apart.
- **A snapshot from before the table does not use it**, so its time has the distributions in it and a new build's does not.
- `bench` fills it too. `replan` does not: every upgrade played is a new start, and the file would grow by a row for each.
- Delete the file after any change to `compute_adv_dist`. Nothing notices a stale one.

## Scenarios

`arena_data/scenarios/*.json`, each `{ "steps": [payload, ...] }` with every step a full `Payload`. `pnpm arena dummy` writes the placeholder one. Its payloads carry four chest stacks, keyed "0" to "3": a three-way choice, an obtain-all with gold, fractional amounts, and one that gives special leaps. A worker's `Essence` carries `chest_states` next to `special_state`; one whose keys are not the reference's (a snapshot older than the keys has none) is left out and the reference's default is kept. A variant built before chests were keyed cannot read these payloads.

Every scenario runs **cold** (each step alone) and **chained** (each step is handed a `Previous`: the last best plan and the plans of the optimizer's blob, capped at `Tuning::max_blob_bytes`). A step is seeded the same way in both. A chained step starts from the plan of the step before (`Optimizer.md`, "Chained runs"); `carry=false` makes it a cold one again.

**A star** (`meta.star` true) hands every step the `Previous` of its first, not of the step before: the first is the Choices page's run without a choice, the others its options. Chained mode is what the page does; cold mode solves each option alone. A snapshot from before stars runs one as a chain.

A Serca payload's materials have to be the Serca labels of the frontend, the four Serca books included; `python scripts/arena/add_serca_books.py` gave every older one the books (none owned, 15,000 gold), and does nothing to a payload that has them.

`arena_data/scenarios/regressions` holds payloads that broke the evaluator. `pnpm arena run --scenarios arena_data/scenarios/regressions --mode cold --seeds 3` is the check.

## Generated scenarios

`pnpm arena gen <set> --singles N --chains N --steps N --sweeps --stress` (`generate.rs`) writes a set into `arena_data/scenarios/<set>/`. Loading a folder does not descend, so a set is run with `--scenarios arena_data/scenarios/<set>` and the default folder stays the dummy.

- **A case is a point on 18 axes** (`AXES`: tier, size, level band, piece mix, jitter between pieces, advanced share, progress, wealth, imbalance, band split, juice owned, juice price, free taps, plan, chests, price noise, chest base share, chest optional share), each in [0, 1), and `build` turns the point into a payload. Drawing payload fields independently makes every case look alike; drawing these does not. The point is kept in the scenario's `meta.axes`.
- **Owned amounts are quantiles**: the wealth axis is how often the owned amount is enough under the default state, and imbalance spreads that between materials. So the same wealth means the same thing at any size. Juices, which the default state does not use, are a share of one juice per tap instead.
- **Generating a set again gives other payloads once the evaluator has changed**, since the quantiles are read off the evaluator. A payload that moved is a new case with no history.
- **Singles and chains are points of a Sobol sequence** (seeds 1 and 2), so `--offset` gives a set that shares nothing with another, and a longer set starts with the shorter one.
- **A chain replays a player**: most steps solve the payload, do the upgrade the plan wants next (`generate::next_up` and `generate::play`, which `replan` shares) and apply one outcome of it (done, free tapped, or some failed taps in), by the rules `monte_carlo::simulate` uses. The rest are income, a price move, a goal added or removed. `meta.kinds` names what happened before each step. Unopened chests are not drawn on when a material runs out.
- **Sweeps** move one axis at a time around three anchors; **stress** is every upgrade a character has.
- **Chests are off** unless `--chests`: the axis is pinned to none and a chain's chest step becomes income. See "Chests in a generated case".
- **`--no-adv`** pins the advanced share to none.
- **Advanced honing is generated with the rule of before**: double balls only under the express event (`Frontend/Wasm contract.md`).
- **`--weekly N`** writes 2N chains of `--steps` weeks (`generate::weekly`, Sobol seed 3): a player with the same income every week who plans once a week. No chests, no advanced honing. A step is the payload at the start of a week.
  - The income is what the default state uses on average over about thirty weeks (0.7 to 1.6 of it), each material with its own constant noise; juices, books and free taps are a share of that or none.
  - `week_inc_*`: nothing is honed, only the income moves. All steps have the same upgrades, and the cost goes to nothing as the stock grows.
  - `week_hone_*`: the player starts with little, and each week follows a short solve's plan (`next_up`, as chains do) for as long as what is owned covers the base materials of the next tap, buying only juices; a tap short leaves the upgrade in progress. The chain ends when the upgrades do.
- A case whose generation panics is left out and the panic is on stderr. A chain step the optimizer panics on is kept, with the player following the default state.

The generated folders, and the commands that made them:

| Folder | Command |
| --- | --- |
| `explore` (600 singles, 200 chains of 8, 150 sweeps: 2316 payloads), `explore_stress` | generated without `--chests` (the exact command is not recorded) |
| `normal` | `gen normal --singles 500 --offset 3000 --no-adv` |
| `weekly` (48 chains, 24 of each kind) | `gen weekly --weekly 24 --steps 30` |
| `chests` (402 scenarios, 822 payloads) | `gen chests --singles 300 --chains 60 --steps 8 --sweeps --chests --offset 1000`, less the sweeps that have no chests. The offset keeps its names apart from `explore`'s, which sets look scenarios up by |
| `seeded` | `nh_dev` with `v0`'s state from seed 0 given in each payload, as the frontend gives the plan it shows when the user optimizes again. Names start with `seed_`; compare with the same payload unseeded by name |
| `tiny` | `pnpm arena tiny` ("The state space") |
| `choices` (22 stars, 289 payloads), `income` (8 chains of 24) | `pnpm exec vitest run --config scripts/arena/vitest.export.config.ts`: the real save of `tests/income-fixture.json` through the pages' own job builders (`scripts/arena/export_pages.test.ts`). Choices: per character with upgrades, the bonus chests of the three raids shown and Paradise (Destiny at Base and Level 10, Netherworld at Base), as stars. Income: every snapshot, now and 12 weeks ahead. Every payload is at a streak length of 10, as the site sends |
| `b10` | the scenarios of `speed_dev` and `speed_test` with `min_resolution` 10, as the site sends by default (names start with `b10_`) |

`python arena_data/explore_report.py [session] [folder]` prints what a run of `explore` says: hit rate, time and when the best was found by size, by axis, by sweep level and by what happened before a chain step.

## Chests in a generated case

The cases are meant to cover what the optimizer can be handed, not what a player usually has.

- **The `chests` axis is how many stacks**: none under 0.12, then 1 to 8.
- **A stack is sized against the average used with every tap taking all it can** (`max_used`: a juice and a book on every tap, both on every advanced tap, the most over which id of a pool it is), not against what the default state uses, which is no juice at all. Free taps are sized against what it takes on average to free tap every upgrade.
- **Base materials from a stack are mostly small**: `chest_base` gives 1 to 15% of that reference four times in five and 15 to 150% otherwise. **Optional ones (juices, books, scrolls, free taps) are anything**: `chest_optional` is 2% to 300%. Each stack has its own noise on top, and stacks add up.
- **What a stack gives is one of 14 families** (`chest_family`), after the game's chests: red or blue; fusion, free taps or shards; free taps, red or blue; all five of those; free taps or leapstones; lava or glacier, alone or with books; books only; bundles of several materials an option; fusion, lava or free taps; gold or a juice; and obtain-alls of base materials, of optional ones and of free taps. A book no upgrade of the case can use is a legitimate option.
- **A stack is 1 to 300 chests**, fewer where the whole stack gives only a few units. Four Serca stacks in ten are converted T4 materials: fifths of a unit a chest, roster bound at best.
- `python arena_data/chest_report.py [session] [--by choice|stacks|kind]` puts a session's variants side by side with the gap as a share of `flow`.

## Sets

A set is a named list of scenarios for one purpose, in `arena_data/sets/<name>.txt`: one scenario name a line, `name:4` for only its first 4 steps, `#` for a comment, the first of which is what the set is for. Names are looked up under the scenarios folder and the folders in it, so a set can mix generated folders.

- `pnpm arena sets` lists them. `--set <name>` on `run`, `verify` and `carry` uses one in place of the folder: `pnpm arena run baseline current --set pilot`.
- `pnpm arena define <name> --note "..." --kind single --where n=32..41 --where juice_owned=0.4..0.7 --steps 4 --limit 12` writes one from filters: `--kind` is a prefix of the name (single, chain, sweep, stress), a `--where` key is an axis of `meta.axes`, `n` (upgrades of the first step) or `steps`. The command is kept in the file. Or write the file by hand.
- A truncated chain's steps are the same payloads, so they share their best known with the full chain.
- `pilot` is the quick first run for a new variant: 16 payloads, about 8 CPU seconds a seed. Payloads picked to be hit a quarter to three quarters of the time, which is where a variant can move the result either way; seven have chests.
- `nh_dev` and `nh_test` are the singles of the `normal` folder with 4 upgrades or more, the even ones (158) and the odd ones (157): normal honing only. The first is what the optimizer of October 2026 was developed on, the second was kept out of it. `nh_quick` is every fourth of `nh_dev`.
- `weekly` is the 48 weekly chains, `weekly_dev` the first six of each kind, which the chained runs were tuned on, and `weekly_test` the other 36.
- `chests` is every case of the `chests` folder that has a choice in it, chains cut to three steps: 349 scenarios, 453 payloads, about 15 minutes for three variants and three seeds at 16 jobs.
- `speed_dev` mixes every kind (`nh_quick`, the small named sets, every eighth of `chests`, ten weeks of `weekly_dev`, `chains`); `speed_test` is held out of it. `speed_dev10` and `speed_test10` are the same at a streak length of 10 (`b10`). `choices_dev`/`choices_test` and `income_dev`/`income_test` split the real pages' characters in two.
- `opt_dev` and `opt_test` are those four together; `quick_dev` a quick cut of `opt_dev` (every other single, short weekly and chains, both streak lengths, half the choices, eight income weeks), about 380 payloads, 2 minutes a variant and seed at 16 jobs. `warm_dev` is its runs that go on from a plan, with the Paradise stars added.
- `cpu_check` (pilot, three choice stars, two income chains) is for CPU time at `--jobs 1`, a few minutes a variant; `income_all` every income chain at full length; `fail_small` seven small payloads the shadow start alone used to miss.
- The rest (`advanced`, `chains`, `free_taps`, `juice`, `large`, `small`, `stress`, `replan`) say what they are for in their first comment.

## Against a fixed baseline

The best known moves every time a variant finds something new, so a hit rate read today is not the one read tomorrow. A frozen snapshot (`v0`, `Arena tried.md`) is the fixed point instead: run it next to a new variant, or in a session of its own, and compare with `python arena_data/diff_report.py <sessions> [--base v0] [--by n|kind|week] [--like text] [--strip text] [--worst 10]`.

- **`diff` is signed**: a variant's mean over seeds less the baseline's on the same seeds, per payload, as a share of the flow of the baseline's best seed. `+` and `-` count the payloads that moved by more than `--eps`. Nothing in it depends on what else is in the database.
- `hit`, `x10` and `gap` are still against the best any run has reached (within `eps`, within ten times it, and the mean), and so still move.
- `spread` is the best seed less the worst of a payload, and `same` the share of payloads whose seeds all agree: the run to run variance.
- The database keeps every run's metric and state, so all of it can be read another way later.

**A run with an error is never the best known.** A state that puts the same id twice on a tap is scored well above any real plan by the evaluator, which does not check; such runs are marked as errors by hand (`error` starts with `invalid state`). Check a new kind of move for that before believing a new best.

## Carrying a plan over

`pnpm arena carry [--set chains] [--rows]` takes, for every chain step after the first, the best known state of the step before, puts it on the step's upgrades (`carry::carried`: a state less the taps done since, the free tap order with gone upgrades dropped and new ones last) and scores it against the default state and the step's best known, all under `Tuning::reference()` as `metric_ref` is. No optimizer runs: it reads the database, so the steps need runs in it. `recovered` is the share of the way from the default to the best that the carried plan already covers.

## What re-planning is worth

The metric assumes the plan made now is followed to the end. `pnpm arena replan --set replan --paths 30 --every 1` (`replan.rs`) measures what is given up by that: it plays each scenario's first step to the end many times, and after every `--every` upgrades carries the plan over and runs the optimizer again from it.

- **The gain of a re-plan is `resolved - carried`**, two evaluator outputs on the same payload. `solve` starts from the state it is given and returns the best it saw, so it is never negative. The gains of a path add up to what following the re-plans is worth over following the first plan, with no noise from the gold itself: the only sampling is which outcomes happened. This rests on carrying a plan over an upgrade keeping its value on average, which `carrying_a_plan_keeps_its_value` checks over every outcome.
- **The player does `generate::next_up`**, the lowest level left on the piece the free tap order starts with, because the evaluator's free tap order may name a level whose piece is not there yet (it means: pay for the levels under it, then free tap). Chains are generated with the same rule.
- **Three guards against a gain that is not information**: the starting plan is the best of `--seeds` cold solves and as many more from that; `control` is one more solve from it, which learns nothing, so its gain is the optimizer's slack; and on the first paths the two metrics are taken again with `max_brute_size` at its maximum, which sums every probability exactly, to see that a gain is not the optimizer finding the approximation's errors.
- Rows are in the `replan` table, one per re-plan, path -1 being the starting plan. `--session N` prints a session's tables again; `--resume` skips the paths a session has.
- Not covered: chests, re-planning in the middle of an advanced upgrade, and choosing a plan knowing that it will be re-planned. Free taps left over when they run out on an upgrade are kept by the player, which the evaluator does not count on.

## Stopping and resuming

`run --resume` reuses the latest session started with exactly the same arguments and skips the jobs that already have a row. Ctrl-C stops cleanly: nothing a killed worker was doing is recorded. Jobs go seed by seed, so a session stopped early has covered every scenario about equally.

## The three measurements

- **Correctness** (`verify`): for every step, the given state and some scrambled ones (`optimizer::scramble`, random valid moves) are simulated and compared per material and in total. The evaluator's side is `Tuning::reference()` (`monte_carlo::check`), whatever the variant ran with. A row passes when the difference is within `z` standard errors plus `rel_tol` of the value. Failures are rows, not panics. The same command times one evaluation. `run --mc` does the check on the first seed's result instead.
  - **A failing row is usually the saddlepoint formula, not the simulation.** Look at which method the material got before suspecting the supports. `max_lattice_work=2000000000` answers every material exactly and is how a doubt is settled.
- **Accuracy** (`run`): the gap to the best metric any run in the database has reached on the same payload. The driver re-scores every result with `Tuning::reference()` (`metric_ref`, see `Evaluation pipeline.md`), because a variant's own `metric` is not comparable once the evaluator differs and the optimizer's own is up to 1e-4 off. `hit %` is the share of seeds within `--eps` (1e-4) of the best known, which is ten times the reference's worst error.
  - `metric_ref` is the coarse lattice at 64 points, not the evaluator the optimizer ran with, so a state that exploits the optimizer's error does not score for it. It is still an approximation, 1e-5 of flow at worst. Compare against `max_brute_size` at its maximum (`replan` does) when that is the question.
  - Next to `metric_ref` a run has `gross`, the same with every leftover worthless (the gold spent), and `flow`, the market value of what the state uses on average. `pnpm arena rescore` scores runs recorded before the reference (`ref_reference` null) again under it and fills these in.
  - **What the gap is divided by is undecided.** The report divides by the best metric, which is near zero wherever credit cancels cost or everything is owned. Spent plus credit (`metric_ref - 2 * gross`) vanishes in the same cases. `flow` does not.
- **Probabilities** (`stress`, `stress.rs`): `pnpm arena stress [--scenarios dir | --set name] [--scrambles 3] [--tuning key=value]`. For the given state and scrambles 20, 200 and 2000 moves away, every material's probability is asked for at some 70 thresholds (packed around the mean, at quantiles, at the ends of the range, at random), plain, and compared with an exact convolution done here. A third kind, `leftover`, is t F - mean F' = E[(t - S)+], what the metric takes of the two, by the route the metric would take (`expected_leftover`), its error in standard deviations of the sum; `mean signed` below 0 there is optimistic. The `leftover` rows are the metric's route (lattice or coarse lattice), the `plain` rows the probability's (lattice, saddlepoint or brute force), whatever the tuning. It reads nothing from the database and writes nothing. The table is by method, region and kind; lattice, brute and trivial must be exact, and a panic is counted by where it was. `verify` cannot see what this sees: it compares the metric at the payload's own thresholds, and within Monte Carlo noise. Materials whose convolution would be too big are left out and counted.
- **Speed**: thread CPU time around `solve` only, plus counters. The counters are exact and repeatable; CPU time is not, see "Traps".
  - **`eval_s` and `decide_s` split a solve's `wall_s`** into the evaluations (`Probe::clock` and `Probe::evaluated` around `metric_router`) and everything else the optimizer does: choosing a move, copying states, the best list. They are wall time, because thread CPU time comes in 15 ms ticks, and are empty for a snapshot that does not send them.
- **The evaluator alone** (`bench`, `bench.rs`): `pnpm arena bench [--scenarios dir | --set name] [--filter text] [--tuning key=value] [--states file] [--out file] [--jobs 1]` evaluates each step's given state, two scrambles of it and any states handed in (`--states`, a json of `"scenario:step"` to a list of essences) once for the counters, then repeatedly for the time. `--out` writes one line a state with its metric, the seconds of an evaluation and its flow (at the default tuning, which it does not depend on), which is how two builds of the evaluator are compared state by state. It reads and writes no results database. The repeats are of one state, so pass `max_memo_size=0` or they time the memo. `--solve <seed>` times one solve a step instead. `python arena_data/bench_diff.py before.tsv after.tsv` puts two `--out` files side by side.

**After any change to the evaluator**, compare against the build before: CPU time with `--jobs 1` on `pilot` and `stress`; the same search with the memo off (most `pilot` payloads should find their best at the same iteration); the metric state by state (`bench --out` and `bench_diff.py`); `stress`; `verify`; and a full pass of `explore`. `Arena tried.md` has how the speed-up of October 2026 did on each.

## The state space

`space.rs`, for what `State space.md` reports. None of it writes the database.

- `pnpm arena tiny [--count 60] [--lone 30] [--lone-taps 20] [--cap 3e6]` writes `arena_data/scenarios/tiny`: cases of one to three normal upgrades with taps failed until every plan can be tried, and lone upgrades with 20 taps left. No chests, no advanced honing, free taps on half.
- `pnpm arena exhaust --scenarios arena_data/scenarios/tiny [--block 10] [--out file.jsonl]` tries every plan and every free tap order of each case on the exact metric, and reports the best, the best the streaks can make, what the annealer finds and what `ascend` reaches. A lone upgrade with more plans than `--cap` is not enumerated: its best plan comes from `Response::best`. `--block` is the streak length (`Optimizer.md`), under which only the plans that keep to it are tried. About 10 minutes at 1, seconds at 10.
- `cargo test --release -p hf-arena lone_exact -- --ignored --nocapture` scores the response's plan, the best streak plan and four solves of `lone_022` and `lone_029` (single upgrades whose best plan is not a streak plan) with the evaluator and with `monte_carlo::exact_single`, a plain sum over the tap of success. `lone_dump` and `lone_moves` (same flags, plus `DUMP_OUT=<file>`) write their per-tap data and every placement of the juices, by material, as JSON.
- `pnpm arena ascend [--set name] [--filter text] [--tuning key=value] [--out file.jsonl]` starts from each step's best known state, and from that state with nothing on its normal upgrades, and lets each upgrade in turn take its best plan, of the streaks only and of every plan. The gain is a share of `flow`.
- `pnpm arena landscape --filter text --x 0f --y 0b --out file.tsv` writes the metric over two streak lengths of each normal upgrade, the rest as best known: under the default tuning, under `--tuning`, and as the response has it. An axis is a pool and `f` or `b`. `python arena_data/landscape_plot.py file.tsv out_dir` draws them.

## Looking inside a run

`Probe` (`crates/core/src/probe.rs`) is threaded through the evaluator and the optimizer. Without the `trace` cargo feature it has no fields and its methods are empty, which is what wasm builds. With it:

- `probe.count(Counter::X)` is always kept. Add a counter by adding a name to the `counters!` list.
- `probe.put("key", value)` and `probe.put_text("key", || ...)` record a row at the iteration last given to `probe.at(..)`. They end up in the `trace` and `trace_text` tables.
- Wrap per-iteration rows in `if probe.detail()`, which is on only with `--detail`. The optimizer already writes `best` on every improvement, and `metric`, `accepted`, `scale` and move kinds under detail.

`pnpm arena trace <run id> [key]` prints a run's rows as CSV. Counters and tuning are JSON columns; query them with `json_extract`.

## Traps

- **A seed must repeat exactly.** Two things break that: the global `rand::random_*` functions (use the solver's `rng`), and summing floats in hash map iteration order (`brute.rs` uses fixed hasher seeds for this reason; `ahash` maps are seeded per instance otherwise). The `a_seed_repeats_exactly` test guards it.
- **CPU time needs `--jobs 1`.** Running many workers at once inflated it about threefold here. Windows also accounts thread time in ticks of about 15 ms, so short measurements are repeated (`time_evaluation`).
- **The Monte Carlo samples normal honing from the upgrade's own tap distribution.** It checks everything built on that distribution, and free taps and advanced honing from the game rules, but not the tap distribution itself. Its advanced honing simulation, `tap_map_generator` and `gold` live in core, which the gold distribution shares them with; its free taps stay its own, so it still checks `special_probs`.
- **The Monte Carlo reads the bands and the free-tap budget of the state it is given**, so it checks the evaluator under a chest state, not `resolve_chests` itself; that has a unit test in `chest.rs`. `scramble` opens the chests at random first (`randomize_chest_state`), so scrambled states have random chest states.
- **A snapshot from before chest moves opens the chests at random**, once, so its gaps on a scenario with chests are tens of percent and say nothing about its annealer.
- **`--jobs` defaults to half the logical cores.** Pass the number of cores for a long run, and 1 when the time is what is measured.
- **A mean of gaps over flow is ruled by a few payloads with almost no flow** (millions of percent). `chest_report.py` prints it, but read the median, the 90th percentile and the hit rate.
- **Where everything is owned the true metric is zero and the optimizer chases approximation error**: best states with a few thousand gold of negative cost have been found on jobs moving millions. Gaps there are noise, whatever they are divided by.
- **Best known is per payload.** Editing a scenario's payload makes it a new case with no history. The payload is hashed with its keys sorted, since its hash maps serialise in no fixed order.
- The database (`arena_data/arena.sqlite`) is gitignored and can be deleted; snapshots in `target/arena/bin` then lose their rows and must be taken again.
