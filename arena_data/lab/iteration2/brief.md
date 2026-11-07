# Iteration 2

**Aim:** learn exactly what makes each iteration 1 prototype's results good or bad, through controlled comparisons, ablations and single pieces swapped into or out of `solve`. A better optimizer is a side effect. The outcome of each review is a list of pieces with evidence: what helps, where, at what cost, and what does not. A lead that appears is followed to a reasonable depth before the review ends.

After all reviews, a final report with visualizations goes in `report/`, for deciding what goes into a replacement for production.

## Investigations

| Investigation | Status | Snapshots |
|---|---|---|
| `price/` | Done. Code in the tree, every piece off; re-run at the new cut in `pr_a` (batches `ra`–`rd`), report current | `i2a`–`i2i`, `pr_a` |
| `free_tap_cut/` | Done, adopted as production; `cut_a` is the baseline for new builds | `cut_a` |
| `surrogate/` | Done. Code left in the tree, every piece off; `final.patch` is the three files against 8444a44a (it carries the free tap cut's lines in them too); no stash | `sur_a`–`sur_g` |
| `memetic/` | Done. Code in the tree, every piece off (`mem_polish` in `solve`); report current | `mem_a`, `mem_b` |
| `cem/` | Done. Code in the tree, every piece off (no `solve` knob); report current | `cem_a`, `cem_b` |
| `lns/` | Not started | `lns_*` |

## For a reviewer

Read, in this order:

1. `AI notes/Calculator core/Optimizer lab.md`: how to work.
2. `Optimizer lab results.md` beside it: what is known. The candidates in `solve` (`kick_stall=32`; `kick_stall=32,mem_polish=true`; `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach`) are second baselines when swapping a piece in. Run them in your own batch; `pr_a:kick_stall=32` has the kick loop's stored runs at the new cut.
3. `Optimizer.md` and `Optimizer tried.md` beside it. Read the tried note before designing any experiment.
4. `iteration1/attempts/<algo>.md` and `crates/core/src/optimizer/lab/<algo>.rs`.
5. `price/report.md`, as an example of a review.

**Baselines changed.** HEAD has the new free tap cut. A new build repeats `cut_a`, not `lab_r1`, and `cut_a` holds `solve` at 48k and 96k and every prototype at effort 1. Read "The free tap cut changed" in `Optimizer lab.md` first. The effort curves at the new cut (efforts 0.25–2 against `solve` 6k–96k, in one batch) are in `pr_a` and in `price/report.md` section 11; start from them rather than re-running.

**How the work is split.** One agent uses the CPU at a time, so the orchestrator runs every long job. A reviewer works in rounds: write code, build, test, snapshot, smoke test for a few minutes, write the batch with `queue.py`, and end its turn saying what the batch tests. The orchestrator runs it and resumes the reviewer. Do not start `pnpm lab` or any arena run over about 10 minutes yourself.

**Rules learned the hard way:**

- Subagents cannot write `report.md`. Send changed sections to the orchestrator at the end of each round, marked ready to paste.
- Keep a plain baseline in every batch, and compare `cpu_s` only inside one batch. `ref x` in `cmp.py` crosses batches, but trust it only on sessions of 10 minutes or more.
- `--resume` reuses a session only with identical arguments, so a narrower batch does not skip runs of a wider one.
- Smoke every new switch in your `scratch.sqlite`, and show plain repeats `cut_a` before a batch.
- Do not touch `crates/arena`.

**Keep to your files:** your algorithm's file, `tuning.rs` (new knobs), `simulated_annealing.rs` (swap-ins) and the lab `mod.rs` if needed. Do not touch another prototype's file, and never change a production default.

**At the end:**

- Remove the `solve` knobs that failed, and say which were kept and why.
- Write `<algo>/report.md` with a short verdict at the top. Write it as you go, so that nothing is lost if your context runs out.
- List the changes you suggest for `Optimizer lab results.md`, `Optimizer tried.md` and `Optimizer.md`. The orchestrator applies them and stops for the user to commit; nothing is stashed.
- Delete your `scratch.sqlite`.

`CLAUDE.md` applies throughout: very brief top-level comments, `unwrap` freely, `rustfmt --edition 2024` on changed Rust files, never commit or merge, and never write to `docs/` or `CLAUDE.md`. Python reads and writes files with `encoding="utf-8"`. A repo-wide search is slow, so search inside `crates/` and `AI notes/`.
