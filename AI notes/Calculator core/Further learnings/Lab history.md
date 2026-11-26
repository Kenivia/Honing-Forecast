# Optimizer lab: history and what is in `runs.sqlite`

Background for reading old lab results; how the lab runs is `../Optimizer lab.md`, what it found `Optimizer lab results.md`.

## Iterations

- **1 (batch 1):** archetypes `price`, `surrogate`, `memetic`, `cem` and `lns`, written from scratch without reading the optimizer, from the evaluator, `shadow_prices` and `response.rs`. `iteration1/brief.md`, write-ups in `iteration1/attempts/`. Its first run was at 16 `response_points`, where responses pick wrong plans on lone upgrades; it is in `ignored/junkyard/lab_iteration1_16pt` and nothing should be judged on it.
- **2:** a review of each prototype in turn, one investigation each (`iteration2/brief.md`), and a final batch (`fin_a`, `iteration2/report/`). Since then an attempt may reuse the production optimizer.
- **3:** good results per CPU second on real cases, judged on the `real3_*` sets: the accuracy-against-CPU curve per kind of run far below 48k, then cheap heuristics (`iteration3/brief.md`). Reports RMS gap and mlog, the mean of log10(gap + 0.001).
- **x10 (2026-11-08, between 2 and 3):** can `solve` be faster at a forced streak of 10 (`x10/report.md`). §1, from stored runs: the real pages hit 100% / 99.8% already at 24k; streak-10 misses are the same order and joint re-split basins as at 1. Batch `x10/a` ran (`a_done.json`) but was **never analysed**; iteration 3's lower default superseded the question.
- **4 (`simple`):** Maxroll's rule as `algo=simple`, then `algo=maxroll`, the calculator's override (`iteration4/simple/report.md`, `Maxroll override.md`).

## Changes that split the record

- **The free-tap cut, 2026-10-31** (`Evaluation pipeline.md`). Snapshots before `cut_a` used the old cut.
  - Stored `metric_ref` values stay valid: all 414,624 were rescored at the new reference, at most 2.2e-6 of flow, no hit changes.
  - Builds since do not repeat older snapshots seed by seed. Gains on order-basin payloads against `lab_r1` run high, since the new cut alone lifts `solve` there (`iteration2/free_tap_cut/report.md`).
  - CPU ratios inside a batch carry over (within about ±0.05, +0.09 on `free_taps`); absolute `cpu_s` does not.
  - The effort curves were re-run at the new cut in `pr_a` (batch `iteration2/price/runs/ra`; table in `price` report 11). `surrogate`'s is at the old cut only.
- **The defaults, 2026-11-12**: `max_iters` 12000 scaled by `size_power` 0.5, `adv_radius` 1, `coarse_shared`, `coarse_cache`. A snapshot's plain label is `solve` at those, not at 48k; every snapshot up to `cold_d` is the old 48k, which the lab's scripts call "48k". The old production on a newer build is `max_iters=48000,adv_radius=0,coarse_shared=false,coarse_cache=false,size_power=0`, which repeats `cold_d` exactly (444 of 444 runs).

## Snapshots

| Snapshots | What |
| --- | --- |
| `lab_r1` | The re-run of iteration 1 (128 `response_points`, all five attempts in one exe). Alone it is `solve` 48k; the old-cut baseline. |
| `i2a`–`i2g` | The `price` investigation. `i2h`, `i2i` registered without runs; `i2i` is the final kick loop's code and repeats `i2g` (57 of 57). |
| `sur_a`, `sur_b` / `sur_c`–`sur_g` | `surrogate`, old cut / new cut. From `sur_e` on, runs carry `ReferenceUs`. |
| `cut_a` | `solve` 48k and 96k and each attempt at effort 1, new cut (a2d2c0f2). **The baseline for new builds**. |
| `pr_a` | The `price` re-run at the new cut (97e7115d), batches `ra`–`rd`: `solve` 6k–96k, every prototype but `surrogate` at efforts 0.25–2, the kick loop and variants, `price`'s ablations, `ascend` by `response_points`. Repeats `cut_a`. |
| `mem_a`, `mem_b` | `memetic`: ablations, `fromsolve`, `mem_polish` in `solve`, with the kick and surrogate loops. |
| `cem_a`, `cem_b` | `cem`: ablations, the order-polish cap, `solve`'s pieces inside it, `fromsolve(+ownstart)`; `cem_b` with the kick loop + `mem_polish` and the surrogate loop. |
| `lns_a`, `lns_b` | `lns`: ablations, `fromsolve`, `mempolish`, `insert`; the rebuild kick in `solve` alone and with the kick loop + `mem_polish`. |
| `x10_a` | Every variant at `streak_length=10`; its best known must be taken over streak-10 runs only. |
| `fin_a` | Iteration 2's final batch: every candidate, one session per set, `large` at 10 seeds. The one batch where the candidates' CPU compares directly (`iteration2/report/report.md` 5). |
| `bud_a` | Iteration 3's `budget` (530bcd35): `solve` from 0 to 48k on `real3_*`. |
| `cold_a`–`cold_d` | Iteration 3's `cold` rounds on cold `real3_dev` singles: chain counts, polishes, `chest_flip`, exchange loops (a); `adv_radius`, `exchange_rank`, evaluator knobs (b); `coarse_shared` (c); `coarse_cache`, size rules, `min_chains` (d, the tree repeats it). Runs carry `us_*` phase timers. |
| `prod_a` | The live production build of then (9ae4f169: 48k at 16 points, old cut and evaluator), from a `git archive` copy with `HF_ROOT` set to this repo; row's sha set by hand. Repeats `lab_r1:response_points=16`. Batch `iteration3/prod/a`. |
| `sim_a`, `sim_b` | `algo=simple` on cold `real3` singles; `algo=maxroll` with its stock switches on `real3na_dev`/`_test` (real3 at streak 1, advanced removed). |

Each later snapshot repeats the one before it on every shared run.

## The iteration-1 re-run

About 22 hours at 16 jobs, each set one session: `solve` at 6k–96k, `solve` 48k at 16 points (the old production), each attempt at efforts 0.125–2; 10 seeds on `real_test`, 6 on `real_income_test`, 3 elsewhere; `ref_*` sessions of `solve` at 384k, 3 seeds, for a firmer best known.

## Old tools

`tools/rerun_summary.py [variant regex]` prints that re-run alone, every variant against `solve` 48k.
