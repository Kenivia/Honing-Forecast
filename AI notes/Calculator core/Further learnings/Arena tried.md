# Arena: what not to try

Warnings about running the arena and reading its results, behind `../Arena.md`. The optimizer's own are in `Optimizer tried.md`, the evaluator's in `Evaluation pipeline tried.md`.

## Running it

- Don't time anything at more than one job: many workers inflated CPU time about threefold. At 16 jobs only counters compare across sessions.
- Don't iterate on a set that takes too long: a full `opt_dev` pass was too slow, hence `quick_dev` and `pilot`. A full `explore` pass at 8 seeds and 16 jobs takes about 2.5 hours; benchmarks beside it slow both (about 2.5x).
- Don't judge a snapshot's speed against a build with a different advanced honing table: one from before the table works the distributions out inside its time (21.0 CPU s a seed on `pilot` against 13.0; mostly-advanced payloads were three quarters distributions).
- Don't copy a new driver over the one a run is using.
- Don't trust `ReferenceUs` on a session under 5 minutes: it has only its opening readings, all in one second, so a moment's disturbance skews it (`sur_e` on `advanced` read 2.24 s against about 0.9 s elsewhere while its solves ran at `cut_a`'s speed). It is a correction, not a measure: from 8 to 16 jobs solves slowed x1.70 and the reading x1.83. Heat and boost move plain `cpu_s` by 10% or more within a run.

## Reading results

- Don't read a hit rate across days: best known moves. Compare with a frozen snapshot (`diff_report.py`).
- Don't trust runs recorded before the reference scoring (16 October 2026) without `pnpm arena rescore`, or runs before 2026-10-27 to carry the automatic invalid-state error (earlier ones were marked by hand).
- Don't keep results across an evaluator change for generated payloads: regenerating `explore` after the evaluator fix moved the owned amounts of two thirds of the singles and sweeps by a unit or so; it was all run again.
- Don't compare a re-export of `real` with older ones where materials are disabled: since 2026-11-17 a disabled material's bound stock is `MAX_AMOUNT`, as on the calculator.

## Checking the evaluator

- Don't chase a failing `verify` row as a simulation bug when the material got the saddlepoint formula: on the `juice` set 13 rows failed (up to 66 standard errors, Glacier's Breath) for materials just over both `max_brute_size` and `max_lattice_work`, 0.01 to 0.06 off; all passed at `max_lattice_work=2000000000`. Measured while the saddlepoint was in the metric.
- Don't use the exact evaluator for broad checks: 790 states took 40 minutes on 8 threads, 1585 weekly ones 87 minutes on 12.
- Don't assume the old default evaluator was close: against the exact one, 12 to 20% of states had an error over 1e-4 of flow (worst 0.49%), mostly optimistic, growing along a weekly chain, tied to few effective upgrades rather than many. That is why the metric dropped the saddlepoint (`Evaluation pipeline tried.md`).

## Scenarios

- Don't load a Serca payload without the four Serca books: `MaterialTable::new` asserts one label per juice. `add_serca_books.py` adds them (none owned, market 15,000, taxed 14,250), making each a new case.

## Re-planning

- Don't expect re-planning to be worth much: on 36 singles of 5 to 14 upgrades the gain averaged 0.12% of flow (median 0.02%), 4% of what the optimizer gains over the default state. The control was zero everywhere and the exact gain 0.99 of the approximate one. It was near zero where most is owned and did not go to zero where little is. Leftover free taps are kept by the player, which the evaluator does not count on.
