# Iteration 2

**Aim:** learn exactly what makes each iteration 1 prototype's results good or bad, through controlled comparisons, ablations and single pieces swapped into or out of `solve`. A better optimizer is a side effect. The outcome of each review is a list of pieces with evidence: what helps, where, at what cost, and what does not. A lead that appears is followed to a reasonable depth before the review ends.

After all reviews, a final report with visualizations goes in `report/`, for deciding what goes into a replacement for production.

## Investigations

| Investigation | Status | Snapshots |
|---|---|---|
| `price/` | Done. `final.patch`; code applied to the tree after the surrogate commit, every piece off (stash "price investigation" kept as a backup) | `i2a`–`i2i` |
| `free_tap_cut/` | Done, adopted as production; `cut_a` is the baseline for new builds | `cut_a` |
| `surrogate/` | Done. Code left in the tree, every piece off; `final.patch` is the three files against 8444a44a (it carries the free tap cut's lines in them too); no stash | `sur_a`–`sur_g` |
| `memetic/` | Not started | `mem_*` |
| `cem/` | Not started | `cem_*` |
| `lns/` | Not started | `lns_*` |

## For a reviewer

Read, in this order:

1. `AI notes/Calculator core/Optimizer lab.md`: how to work.
2. `Optimizer lab results.md` beside it: what is known. Use the kick loop as a second baseline when swapping a piece into `solve`.
3. `Optimizer.md` and `Optimizer tried.md` beside it. Read the tried note before designing any experiment.
4. `iteration1/attempts/<algo>.md` and `crates/core/src/optimizer/lab/<algo>.rs`.
5. `price/report.md`, as an example of a review.

**Baselines changed.** HEAD has the new free tap cut. A new build repeats `cut_a`, not `lab_r1`, and `cut_a` holds `solve` at 48k and 96k and every prototype at effort 1. Read "The free tap cut changed" in `Optimizer lab.md` first. `price/final.patch` no longer applies cleanly at HEAD; a swap-in that needs it takes `git apply --3way`.

**How the work is split.** One agent uses the CPU at a time, so the orchestrator runs every long job. A reviewer works in rounds: write code, build, test, snapshot, smoke test for a few minutes, write the batch with `queue.py`, and end its turn saying what the batch tests. The orchestrator runs it and resumes the reviewer. Do not start `pnpm lab` or any arena run over about 10 minutes yourself.

**Keep to your files:** your algorithm's file, `tuning.rs` (new knobs), `simulated_annealing.rs` (swap-ins) and the lab `mod.rs` if needed. Do not touch another prototype's file, and never change a production default.

**At the end:**

- Remove the `solve` knobs that failed, and say which were kept and why.
- Write `<algo>/report.md` with a short verdict at the top. Write it as you go, so that nothing is lost if your context runs out.
- List the changes you suggest for `Optimizer lab results.md`, `Optimizer tried.md` and `Optimizer.md`. The orchestrator applies them, and saves the patch and stash.
- Delete your `scratch.sqlite`.

`CLAUDE.md` applies throughout: very brief top-level comments, `unwrap` freely, `rustfmt --edition 2024` on changed Rust files, never commit or merge, and never write to `docs/` or `CLAUDE.md`. Python reads and writes files with `encoding="utf-8"`. A repo-wide search is slow, so search inside `crates/` and `AI notes/`.
