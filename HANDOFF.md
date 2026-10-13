# Handoff: evaluator crash fix (October 2026)

The fix is in commit `69fef473`. Details are in
`AI notes/Calculator core/Evaluation pipeline.md` ("Method selection") and `Arena.md`.

## What was wrong

During the explore run, one job (`chain_0180_t4_n27`, seed 2) panicked in
`saddlepoint_approximation.rs`: the probability came out as 1.14, the fallback made it 1.30,
and the range check panicked. The arena lost the evidence because the core printed to stdout
first, which the driver could not parse.

The cause was not a rounding or root-finding problem. The saddlepoint formula needs many
comparable upgrades, and that juice total was one bimodal upgrade plus noise. Looking for it
turned up four more defects:

- The formula can be 0.2 off while staying inside [0, 1], so nothing flagged it. The optimizer
  found such a state and scored it 3M gold above its true value.
- The Edgeworth fallback read its cumulants at the saddlepoint, not at zero. Right near the
  mean, meaningless away from it.
- Brute force compared with `FLOAT_TOL` (1e-9), which is below f64 resolution past about 8.4M,
  so a threshold equal to a reachable total counted as short of it.
- The lattice span used a gcd that returned the wrong value when the remainder reached 1, and
  thresholds were snapped from zero, not from the minimum.

## What changed

- **`core/lattice.rs` (new)**: the exact distribution of a material's total as an array, built
  once per material and read for every threshold. Used when lattice points x outcomes is at
  most `Tuning::max_lattice_work` (200,000), which is mostly juices and books.
- **Fallbacks**: Edgeworth takes cumulants at zero and is only used near the mean. A result
  out of range is split on the outcomes of the highest-variance upgrade, then clamped. Only a
  NaN or a root that does not converge still panics.
- **Tolerance and lattice**: `float_tol(value)` scales with the value; the span is an integer
  gcd; thresholds snap from the minimum.
- **Arena**: `pnpm arena stress` checks the evaluator's probabilities against an exact
  convolution. A panicking worker is now recorded as `died`. The crash payload is in
  `test_cases/scenarios/regressions`.

Checked with: the crash job (passes), `pnpm arena stress` on the explore set (no panics;
lattice, brute and trivial exact), `cargo test -p hf-core -p hf-arena` (passes). Run time on
the `juice` set is +5%.

## Does it affect existing run data

Slightly. Every state stored for sessions 4 and 5 (7,607 runs, 954 payloads) was re-scored
with the new evaluator and compared with its stored `metric_ref`:

- 97.5% of runs moved by less than 1e-4 of the gold flowing through the plan. None moved by
  more than 1e-3.
- 35 of 954 payloads have a run that moved by more than 1e-4.
- On 25 payloads the stored best state is no longer within 1e-4 of the best under the new
  evaluator.
- Hit rate at eps 1e-4 goes from 69.7% to 69.2%.

So conclusions drawn from the explore run stand. What does not carry over:

- **`metric_ref` in the database is the old evaluator's.** Nothing recomputes it (`rescore`
  only fills `gross` and `flow`). A new run is compared with a best known that the old
  evaluator scored, so on those 25 payloads a gap may be the evaluator, not the optimizer.
  Re-scoring the stored states, or a fresh baseline run, removes the doubt.
- **Cases where everything is owned** move the most in relative terms (`single_0504_t4_n13`
  went from about +5,000 to about -1,500), as `Arena.md` already warns.
- **Seeds do not repeat across the change**, as with any change to the evaluator.
- The `pre_fix` snapshot is the code before this work, for comparisons.

Saved frontend state is unaffected: the fix changes how a plan is scored, not how it is
stored. Displayed numbers will shift a little once the wasm is rebuilt (`pnpm run wasm`),
which has not been done.

## Still open

- The saddlepoint formula is unchanged where neither exact route applies. In the stress sweep
  it is still more than 0.01 off in about 7% of its calls, mostly sums of under 8 upgrades.
- `pnpm arena verify --set juice` fails 13 Monte Carlo rows (17 before). They are that same
  error, not the simulation: 3 or 4 upgrades, just over both exact-route limits. Building the
  lattice only up to the largest threshold would bring both worst cases under the limit.
- The performance work this started as has not begun. First candidates: `ks(0)` is recomputed
  for every threshold and for both the plain and biased call; the lattice could be reused
  across iterations and across free-tap outcomes.
