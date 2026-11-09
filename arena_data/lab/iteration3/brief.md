# Iteration 3: good results per CPU on real cases

**Aim.** An optimizer that gives good results per CPU second on real cases. x0.5 of `solve` was a first guess, not a target: the first part of this iteration finds which part of the CPU–accuracy curve to aim for, per kind of run (cold singles likely need more, small income diffs later in a chain less), including far lower CPU than x0.5.

**Data.** Real cases from the analytics pulls up to **2026-11-09** (`scripts/arena/export_real3.test.ts`, sets `real3_*`). Pull again and rebuild if that is more than a week old when a new investigation starts.

## Investigations

| Investigation | Status | Snapshots |
|---|---|---|
| `budget/` | Batches `a`, `a_inc`, `a_seq`, `a_gen` done, `budget/report.md`: no knee; later chain steps cost and score like cold runs; below x0.3 the per-chain polishes are the CPU; the cheap end's real-case misses are mostly a chest-choice deadlock. Suggested aim x0.2–0.25 at 12k–24k accuracy; batch `b` (cheaper polishes, fewer chains) proposed, not run | `bud_a` |

## Decisions (2026-11-09)

1. **No fixed budget.** Map the curve per kind of run and pick a trade-off point for each.
2. **Data date:** the pull of 2026-11-09. Prompt for a new pull when it is more than a week old.
3. **Families** as proposed (same grids, stock within 10% of its gold value). The old `real` sets are not held out any more; **rosters split 50/50 at random** (seeded).
4. **Every case at both streak lengths** (paired, `_s1` and `_s10`).
5. **Artificial cases deferred** until more data is needed.
6. Go ahead with the sets, the reference runs and investigations. One subagent at most at a time, at a fitting effort.
7. **Metrics:** report how badly runs miss, not only a hit rate at an arbitrary threshold: RMS gap and the mean of log10(gap + 0.001), with log-scale quantiles, beside the existing mean gap and hit.

## The sets (`real3_*`, in `arena_data/scenarios/real3`)

| Set | What | Scenarios | Steps |
|---|---|---|---|
| `real3_dev` / `real3_test` | Cold singles: each family's latest snapshot (`r3_<roster>_<char>_f<k>_s<1,10>`) | 844 / 818 | 1 each |
| `real3_income_dev` / `_test` | Per roster, the character whose latest family wants the most: now and 12 weeks ahead at Auto income, chained (`r3inc_`) | 50 / 50 | 13 each |
| `real3_seq_dev` / `_test` | Each character with 3 or more families: its families in order, chained (`r3seq_`) | 142 / 150 | 3–10 |

831 families of 262 characters in 50 rosters, 25 rosters a side, 4,220 payload steps. A family's single and its step in a `r3seq_` chain are the same payload, so the best known is taken per payload hash across scenarios and modes.

---

# Preliminary report (2026-11-09, before the decisions)

Every number below comes from runs already in `runs.sqlite` (snapshots `pr_a`, `cut_a`, `x10_a`) or from the analytics pulls in `ignored/analytics`. The scripts are in `prelim/`, run from the repo root (`prelim/traces.py` first for the trace-based ones). All numbers are in % of flow; **hit** is within 0.01 of the best known. Sections 3, 5 and 7 are superseded by the decisions above.

## 0. Summary

**Aim, as first put.** Cut CPU to about half of production `solve` (48k) or less, losing as little accuracy as possible, judged on real cases. Three questions:

1. How good are very cheap heuristics (a few hundred evaluations), on real cases and on generated ones, and why?
2. Can a case's category be detected and given its own heuristic?
3. How accurate can a run at x0.5 CPU be?

**What the stored runs already say.**

- **Halving CPU is cheap on the real pages and costs something on cold singles.**
  - Real income at streak 10: `solve` 12k is hit 98.1% (48k: 99.8%) at x0.48 CPU.
  - Choices: 12k is hit 97.7% (48k: 98.7%) at x0.35.
  - Cold real singles: 12k loses 3–8 points of hit (89–91% against 94–99%) at x0.36–0.48. The loss is a tail of a few payloads.
- **Difficulty is very uneven.** On real singles at streak 1, `solve`'s start is already the best known on 35% of runs, and 60% are there by 500 evaluations. The median run has its final answer by 261 evaluations, yet most of these runs use the full 48k. **Spending the budget unevenly is the biggest lever.** Every experiment so far gave each case the same budget.
- **Streak 10 does need its own heuristics at the cheap end, but not at x0.5.** On the same payloads, forced to 10:
  - the start is worse (hit 21% against 35%);
  - the median run takes 861 evaluations to its answer, against 261;
  - the two curves meet by 2–4k evaluations.
- **Real best plans are simple.** 71–80% of their juice and book uses are one streak from the first tap (generated sets: 60–65%). Streak 10 is the most regular, at 80%. A small family of front-streak plans could be a real heuristic, and at streak 10 it is small enough to enumerate.
- **Real payloads are unlike the generated ones:**
  - 32 of 38 have a chest stack with a choice, against 17% of quick_dev runs;
  - most are Serca;
  - most own much of what they use;
  - streak 10 is the site's default.

  `solve`'s early search, though, finds them about as hard as generated ones.
- **An adaptive stop looks nearly free, offline.** Stop when the first two chains end within 0.01% of flow of each other. Simulated on stored traces, that cuts real singles to x0.66 of their chains at streak 1 and x0.84 at streak 10, with no measured loss.
  - It must be checked on the sets where early agreement failed before: small payloads and weekly chains (`Optimizer tried.md`).

**New data.** The pulls up to 2026-11-09 hold 2,092 distinct real cases. Grouped as near-duplicates (same character, same grids, materials within 10% of their gold value), they make 831 families: 262 characters, 50 rosters. 530 of the families came after the current `real` set was built.

**Proposed route.**

1. Build real dev and test sets from those families, split by roster, each case at both streak lengths.
2. Map the cheap end of `solve`'s curve with existing knobs.
3. Measure cheap heuristics, and categories, against that floor.
4. Put the x0.5 effort into allocation (an adaptive stop, a probe that decides the budget) and short tails, and compare at matched CPU against `solve` at about 14–16k.
5. Generate artificial "real" cases only if the real families turn out too few for some category.

## 1. A fresh look at iterations 1 and 2

Both iterations asked **"what does more CPU buy?"** and answered: a tail after `solve` (x1.35–3.7), never more iterations. The gains were small (at most 0.01 on any set's mean) and lay on a few payloads per set. This iteration asks the opposite: **what can be given up for half the CPU?** Four things from 1 and 2 bear on that:

- **Starts were every prototype's weakness**, and `solve`'s averaged shadow-price start is hard to beat (`Optimizer tried.md`: prices taken from the solved plan are 5.3% off, against 0.63%). A heuristic-only answer starts from that known weakness. Expect it to land near the start's numbers (section 2.2) unless the plan structure (2.4) gives a much better construction. Its main value is as the first stage of an adaptive run, and for telling categories apart.
- **Tails beat iterations at the high end.** Whether a short `solve` plus a cheap tail beats a plain short `solve` at the low end is untested, and worth testing.
- **Early agreement failed before.** Leaving after 3 agreeing chains lost 2 points on weekly chains of 1–7 upgrades: chains from one start agree on a plan that is not the best. Any adaptive stop has to be checked on those sets as well as on real ones.
- **Both iterations judged on 38 real singles**, and real_test's ranking was mostly one payload (`real_27a2bc40`). The new pulls give about 20 times as many real cases, which allows real held-out numbers with confidence intervals.

**Is the generated data too theoretical?** Real payloads differ in structure (2.5), but not obviously in how hard `solve` finds them (2.2). So judge on real cases, and keep the generated sets as guard rails: payloads a real user can still hit, such as small payloads and weekly chains.

## 2. What the stored runs say

### 2.1 Halving CPU with existing knobs

`solve` at lower `max_iters`, and the `x10_a` variants. CPU is against `solve` 48k in the same batch. At streak 10, the best known is over streak-10 runs only. `prelim/budget_curves.py`, `prelim/x10_budget.py`.

| Set (payloads at their own streak) | 48k hit / gap | 24k | 12k | other |
|---|---|---|---|---|
| real singles, streak 1 | 98.9 / 0.0006 | x0.63: 96.3 / 0.0011 | x0.42: 91.1 / 0.0022 | |
| real singles, streak 10 | 94.2 / 0.0153 | x0.66: 91.6 / 0.0243 | x0.48: 89.5 / 0.0216 (one payload 1.08) | |
| real singles, streak 1 payloads forced to 10 | 94.2 / 0.0016 | x0.62: 93.7 / 0.0021 | x0.36: 91.1 / 0.0030 | `agree_exit=3,unanimous=2` x0.87: 93.7 |
| real income, streak 1 (chained) | 95.9 / 0.0018 | x0.77: 94.9 / 0.0021 | x0.53: 94.4 / 0.0028 | |
| real income, streak 10 (chained) | 99.8 / 0.0001 | x0.71: 99.8 / 0.0001 | x0.48: 98.1 / 0.0006 | `agree_exit=3,unanimous=2` x0.40: 98.9 / 0.0005 |
| choices (streak 10) | 98.7 / 0.0003 | x0.60: 98.0 / 0.0008 | x0.35: 97.7 / 0.0017 | `agree_exit=3,unanimous=2` x0.75: 98.0 |
| quick_dev, streak 1 / 10 (contrast) | 93.4 / 96.9 | x0.64: 89.3 / 94.7 | x0.42: 87.8 / 91.8 | |

So the plain baseline for question 3 is `solve` at about 14–16k. On the real pages it gives up almost nothing; on cold real singles it gives up 3–8 points of hit, from a tail.

### 2.2 How early `solve` has its answer

`solve` 48k (`cut_a`), seeds 0–2, from the `best` trace rows (`prelim/traces.py`, `prelim/anytime.py`). Each cell is mean gap / hit at that many evaluations, counted cumulatively across chains. "Start" means greedy order, shadow prices, the first chest polish and the first move (trace iteration 1).

| Set | Start | 500 | 4,000 | End | Evaluations to the run's own answer (median / p90) |
|---|---|---|---|---|---|
| real singles, streak 1 | 1.29 / 35% | 0.18 / 60% | 0.013 / 75% | 0.001 / 98% | 261 / 8,000 |
| the same payloads, forced to 10 | 1.62 / 21% | 0.25 / 42% | 0.013 / 81% | 0.002 / 93% | 861 / 5,333 |
| real singles, streak 10 | 1.91 / 26% | 1.14 / 28% | 0.14 / 58% | 0.015 / 95% | 3,317 / 12,000 |
| quick_dev, streak 1 | 3.22 / 41% | 0.36 / 52% | 0.030 / 79% | 0.004 / 93% | 238 / 8,000 |
| quick_dev, streak 10 | 4.04 / 40% | 0.91 / 52% | 0.044 / 80% | 0.002 / 97% | 202 / 12,000 |
| speed_test | 1.46 / 44% | 0.22 / 51% | 0.032 / 78% | 0.002 / 96% | 155 / 6,857 |

- On real income chains the carried plan alone, before any evaluation, is hit 86% (streak 1) and 90% (streak 10).
- **Real singles are no easier than generated ones for the start and the first 500 evaluations.** If anything they are harder at streak 10.
- **The median run is done within a few hundred evaluations, and the tail decides the budget.** The tail payloads are the known basins of iteration 2 (free tap order, joint re-splits).

### 2.3 Streak 1 against streak 10

On the same 19 real payloads (`prelim/x10_anytime.py`, table above), streak 10 starts worse and takes about three times the evaluations to its answer. By 2–4k evaluations the two are level. The plans also differ (2.4): at 10 they are more regular, and the grid is a handful of blocks an upgrade.

**Verdict so far:** heuristics in the few-hundred-evaluation range should be designed and judged separately per streak length. At x0.5 the streak length changes little. That rests on 19 payloads; the new data will settle it, with every case run at both lengths.

### 2.4 The shape of real best plans

Each normal upgrade of each best-known state, juice and book ids taken one at a time (`prelim/plan_shapes.py`). "Front" means taps from the first one on; "back" means taps up to the last.

| Set | Upgrades with any juice/book | Front | All taps | Front+back | Back | Other |
|---|---|---|---|---|---|---|
| real singles, streak 1 | 100% | 71% | 14% | 4% | 2% | 9% |
| real singles, streak 10 | 87% | 80% | 4% | 8% | 4% | 5% |
| quick_dev, streak 1 | 80% | 65% | 8% | 9% | 4% | 14% |
| quick_dev, streak 10 | 66% | 60% | 7% | 18% | 12% | 3% |
| speed_test | 84% | 62% | 8% | 9% | 7% | 14% |

A front streak covers a median of about 60% of the taps to pity. So a heuristic family suggests itself: per piece type (which share a juice), how far the front streak runs. At streak 10 that is a block count, small enough to enumerate in a few hundred evaluations; at streak 1 it needs a line search. This is a second reason to expect different heuristics per streak.

### 2.5 Real payloads against generated ones

- **Chest stacks with a choice:** 32 of 38 real singles have at least one; 83% of quick_dev runs have none. The first chest polish (with the first move) gains 0.6 on average on real singles that have one. Generated sets under-represent this.
- **Ownership:** 76% of real runs have a net cost under 0.3 of flow; many are negative, meaning credit for what is left over exceeds what is spent.
- **Mix:** 72% of the families are Serca; 26% want advanced honing.
- **Streak length:** the site defaults to 10, and snapshots do not record the user's setting, so the real mix of 1 and 10 is unknown. It would take one field added to the analytics upload to find out.

### 2.6 Payload features against difficulty, first cut

On the 38 real singles (`prelim/features.py`), no single payload feature separates easy cases from hard ones:

- net cost share, owned juice share, size and number of choice stacks show no clean pattern;
- owning free taps comes closest (start hit 39% with, 19% without; at 4k evaluations 77% against 52%).

Thirty-eight payloads are too few to conclude anything. With the new families this becomes a proper question. If payload features stay weak, the detector should be a cheap probe rather than the payload: run the start and one polish, and read how much moved and where.

### 2.7 An adaptive stop, offline

Simulated on the same traces (`prelim/adaptive_stop.py`): stop when the first K chains end within a tolerance of flow of each other, and otherwise run on as `solve` did. CPU is read as the share of chains used, which are of equal length.

| Set | K=2, tol 1e-4: stopped early | Chains used | Hit (production) |
|---|---|---|---|
| real singles, streak 1 | 63% | x0.66 | 98 (98) |
| real singles, streak 10 | 51% | x0.84 | 95 (95) |
| quick_dev, streak 1 | 63% | x0.72 | 91 (93) |
| quick_dev, streak 10 | 72% | x0.72 | 96 (97) |
| speed_test | 51% | x0.70 | 96 (96) |
| `large` | 30% | x0.84 | 53 (53) |

- **Why the plain exit rarely fires on real singles:** `solve` counts chains as agreeing within 1e-7 of the metric, not of flow. On real payloads the metric is often near zero, since what is owned covers most of the cost, so chains a chest apart never agree. Real streak-1 singles ran their full 48k (median 48,600 evaluations).
- **Caveats:** this is offline, with chain ends read back from the traces. The trap in `Optimizer tried.md` (weekly chains of 1–7 upgrades) is not among these sets. A knob is needed before believing it.

### 2.8 Where the CPU goes

The evaluator takes 79–85% of a solve's time (real_test 79%, quick_dev and `large` 85%; `fin_a`). A cut has to come from fewer evaluations, not from cheaper bookkeeping.

## 3. New real data

`prelim/inventory.py`, over every pull in `ignored/analytics` (2026-10-22 to 11-09; dev and follow-up uploads left out, typed placeholders left out).

- **2,092 distinct cases**: a character's snapshot with something wanted, distinct by bound, roster and tradable stock, special leaps, grids and tier. They cover 262 characters in 50 rosters.
- **Families:** a case is a near-duplicate of an earlier one of the same character when its grids (done and wanted cells) are the same and its materials moved by less than 10% of their gold value. That gives **831 families**: 973 at a 5% threshold, 719 at 20%.
  - Grids, not the lostark.bible gear, because the payload is built from the grids. Users do not always keep grids current, but a payload with the same grids and stock is the same case.
- **Size:** wanted cells per family have a median of 10 (p90 24, max 82). 600 of the 831 are Serca, 215 want advanced honing, and 599 have chests.
- **Rosters are uneven:** the largest has 93 families, and half the rosters have under 10. Weight by roster as well as by family (section 5).
- **Real re-optimization sequences:** 146 characters have 3 or more families (629 in all). In order, they form real chained sequences of the same character as stock moves, which is what a warm re-solve on the Calculator page sees. That is a more realistic warm-run set than the 12 weeks of auto income.

**Proposed split, by roster:**

- the 23 rosters of the current `real_test`/`real_income_test` (502 families) become **test**;
- the 27 new rosters (329 families) become **dev**.

That is 60/40 by family and 46/54 by roster. It keeps the old held-out sets held out, and no roster sits on both sides. Splitting by family instead would leak a character's near-duplicates across the split.

## 4. Artificial "real" cases

**Option A, as asked: find each real case's axes and sample from a restricted range.** About half of the 18 axes can be read straight off a payload:

- tier, size, level band, piece mix, advanced share, progress, free taps and the chest counts;
- wealth, imbalance and juice owned need the evaluator (owned against the default state's use), which is a small arena addition;
- `plan`, `jitter` and the band split need definitions.

Two caveats:

- **Sample around real points, not inside a box.** Taking a real dev case's axis vector and jittering it keeps the correlations: Serca with vambrace focus, wealth with level. Independent per-axis ranges make combinations nobody has.
- **The generator cannot reproduce real chests.** Real payloads carry about 20 stacks, mostly single-option income plus a few choices; the generator's sizing differs. `build_with` can take a real case's upgrades but not its chests.

**Option B: perturb real dev snapshots before building the payload.** Scale stock per band, move goals by a level at the ends, draw prices from market history, flip the streak length, and build through the same `job_for` as `export_real.test.ts`. Everything about the structure stays real by construction.

**Recommendation.** Defer both until phase 5. 831 families may be enough, and the real sets give a yardstick for whatever is generated. Fidelity test: a generated set is good if methods rank on it as they do on real dev. Only dev rosters may seed it.

## 5. Procedure and metrics

**Sets:**

- `real3_dev` and `real3_test`: one payload per family, the family's latest snapshot, each at streak 1 and 10 (paired), cold;
- `real3_income_dev`/`_test`: per roster, the first pick as now plus 12 weeks, both streaks;
- optionally `real3_seq`: the families of each character in order, chained;
- the guard rails stay: `quick_dev`, `speed_test`, `weekly_dev` (the early-agreement trap), `small`, `large`.

**Best known:** as before, over all runs, plus a reference batch per case (`solve` 48k and KMR+Ls, 3 seeds). Each run's starting metric (trace iteration 0) is kept for the captured share.

**Metrics, per streak length and per kind** (cold single, chained, sequence), and per category in phase 3:

| Metric | Why |
|---|---|
| CPU against `solve` 48k in the same batch: summed, and the per-run p90 | The target, and the worst wait a user sees |
| Mean gap, hit, p95 gap, worst gap | As in iterations 1–2, plus the tail, which is where a cheap run loses |
| Captured share: (metric − starting metric) / (best known − starting metric), median and p10 | For heuristics: how much of what optimizing is worth they get. Not distorted by flow |
| Gold lost (gap × flow), median and p90 | What a user loses, in gold |

**Weighting:**

- each family counts once (primary);
- each roster counts once, split over its families (secondary);
- per snapshot, shown once, for how often a case comes up.

Confidence intervals come from resampling rosters, which fits the clustering better than the fixed 0.01 noise rule. Where the weightings disagree, say so.

**The deciding comparison for question 3:** on `real3_test`, the best method at x0.5 summed CPU against `solve` at the `max_iters` that costs x0.5, as curves of gap and hit against CPU per streak length. Guard rail: on the guard sets, no payload more than 0.1 worse than that `solve`.

## 6. Plan

| Phase | What | Code | CPU |
|---|---|---|---|
| 0. Data | Export the families (a sibling of `export_real.test.ts`: families, both streaks, income chains, the split), sets, a reference batch | export script, sets | about 3 h |
| 1. Knob map | `solve` at 3k/6k/12k/16k/24k; `chains=1` and `2` at small budgets; start and polish only. The cheap end of the curve and the floor for heuristics | none | about 3–4 h |
| 2. Cheap heuristics | Lab algo `heur`, a switch per piece: each polish in the start on or off; front-streak plans per piece type (enumerated at 10, line search at 1); chests by shadow price; greedy order. Real against generated, streak 1 against 10, and why | `optimizer/lab/heur.rs` | 1–2 h a round |
| 3. Categories | Payload features plus a probe (per material, how often the owned amount is crossed; what the first polish moved). A selection rule fitted on dev, tested on test, against the single best heuristic and the oracle (best per case) | an arena feature dump | small |
| 4. x0.5 | An adaptive stop (`agree_first`, flow-relative tolerance); probe-then-allocate; short `solve` plus a cheap tail (12k + `mem_polish`, 12k + a short kick loop), at matched CPU | `solve` knobs, off by default | 3–5 h a round |
| 5. Artificial cases (if needed) | Option A or B (section 4), with the fidelity test | generator or export | small |

## 7. Decisions needed

1. **Which workloads count toward x0.5?** Cold Calculator solves, warm re-solves (the plan shown is given back), Income chains, Choices stars. Default: report all; target cold singles and income chains.
2. **Pull analytics again first?** `pnpm pull-analytics` deletes the remote rows it saves, as usual. Default: use the local pulls up to 11-09 01:49.
3. **Family rule and split:** 10% of gold with the same grids; old rosters as test, new as dev (60/40). Or a random roster split nearer 50/50, which gives up the old sets' held-out status.
4. **Both streak lengths for every case?** That doubles each batch, and it is what settles the x10 question. Default: yes.
5. **Artificial cases deferred to phase 5?** Default: yes.
6. **Go-ahead for phase 0**, which includes the reference batch of about 3 hours.

Also open: `x10/report.md` still waits for batch `a`'s write-up. Sections 2.1 and 2.3 here hold its numbers; it could be closed into this iteration.
