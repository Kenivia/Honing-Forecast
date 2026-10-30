# Batch 1 review: `price`

> **Superseded in part.** This review was written from the first run, at `response_points` 16. Its losses on lone upgrades (`small`, `single_1235_t4_n1`) were errors of the best response, not back-streak basins (`arena_data/lab/iteration2/findings.md`). Re-read it against `rerun.sqlite`.

Verdict: **iterate.** Not viable as built: on the held-out sets it needs about twice `solve`'s CPU for the same result. Nearly all of the loss comes from one mechanism that can be found and fixed (below). It is at or ahead of `solve` at matched CPU at streak length 10 on `quick_dev` and on `advanced`. Nothing goes into `solve` yet.

All numbers are `diff_report.py` against `solve` at 48k of the same snapshot, in % of flow; x is CPU over that baseline, within a session.

## Matched CPU

`real_test` (cold; sessions 1+16 at 10 seeds, efforts 0.125/0.25 and 6k/12k from session 10 at 3 seeds):

| solve | x | diff | price | x | diff |
|---|---|---|---|---|---|
| 6k | 0.35 | -0.0122 | 0.125 | 0.16 | -0.0316 |
| 12k | 0.46 | -0.0069 | 0.25 | 0.27 | -0.0230 |
| 24k | 0.64 | -0.0050 | 0.5 | 0.47 | -0.0192 |
| 48k | 1.00 | 0 | 1 | 0.79 | -0.0093 |
| 96k | 1.72 | +0.0042 | 2 | 1.42 | -0.0051 |

`real_income_test` (chained; sessions 2+18 at 6 seeds, low from 12):

| solve | x | diff | price | x | diff |
|---|---|---|---|---|---|
| 6k | 0.41 | -0.0013 | 0.125 | 0.22 | -0.0032 |
| 12k | 0.54 | -0.0007 | 0.25 | 0.36 | -0.0019 |
| 24k | 0.76 | -0.0002 | 0.5 | 0.58 | -0.0012 |
| 96k | 1.11 | +0.0000 | 1 | 0.96 | -0.0008 |
| | | | 2 | 1.68 | -0.0003 |

**Key number: on `real_test`, price at effort 2 (x1.42, -0.0051) equals `solve` at 24k (x0.64, -0.0050), about 2.2x the CPU for the same result.** It is about 2x on `real_income_test`.

Elsewhere (effort 2 against the nearest `solve`):

| set | price 2 | solve |
|---|---|---|
| quick_dev cold | -0.0000 at x1.68 | 96k +0.0014 at x1.64 |
| speed_test cold | -0.0059 at x1.21 | 24k -0.0006 at x0.61 |
| advanced | +0.0027 at x1.96; at 0.5: +0.0010 at x0.57 | 96k 0 at x1.80; 24k -0.0014 at x0.59 |
| juice | +0.0014 at x1.29 | 96k 0 at x1.78 |
| free_taps | -0.0071 at x1.10; at 0.5: -0.069 | 24k -0.0005 at x0.55 |
| large | -0.0033 at x1.82 | 96k +0.0054 at x1.83 |
| small | -0.028 at every effort, spread 0 (4 of 20 payloads) | 24k 0 |

### By streak length

The work counter is calibrated very differently at the two streak lengths. At streak length 1, effort 2 is x0.97 on `real_test`; at 10 it is x2.15.

| set, streak | price 0.5 | price 1 | price 2 | solve 24k | solve 96k |
|---|---|---|---|---|---|
| real_test, 1 | -0.0156 x0.29 | -0.0100 x0.52 | -0.0076 x0.97 | -0.0004 x0.62 | +0.0002 x1.72 |
| real_test, 10 | -0.0221 x0.76 | -0.0083 x1.23 | -0.0026 x2.15 | -0.0094 x0.67 | +0.0079 x1.73 |
| real_income, 1 | -0.0024 x0.34 | -0.0017 x0.62 | -0.0007 x1.19 | -0.0002 x0.77 | +0.0000 x1.09 |
| real_income, 10 | -0.0002 x0.80 | -0.0001 x1.28 | +0.0000 x2.15 | -0.0001 x0.75 | +0.0000 x1.12 |
| quick_dev, 1 | -0.0094 x0.44 | -0.0077 x0.79 | -0.0032 x1.43 | -0.0021 x0.63 | +0.0016 x1.71 |
| quick_dev, 10 | +0.0008 x0.65 | +0.0011 x1.10 | +0.0020 x1.91 | -0.0026 x0.67 | +0.0012 x1.57 |

At streak length 10, `price` is at matched CPU or ahead on `quick_dev`, and level on `real_income`. At streak length 1 it is both underspent and behind.

## Where it loses: fixed points of the responses

Everything `price` returns comes out of true best responses (`Response::of(..).better`), polished from its starts. These starts are the empty plan, the given plan, and linear responses at shadow prices. Responses taken in turn have fixed points that depend on where they start. The plan `solve` finds uses a juice or book again on the last taps before the pity: a back streak. That plan is a fixed point when responses start from it, but they do not reach it from the empty plan.

`pnpm arena ascend` (default tuning, from the best known state and from nothing) shows it:

- `small`: from nothing, the responses end at -0.424 on `single_0049_t4_n1`, -0.085 on `single_0002_t4_n3` and -0.032 on `single_0039_t4_n1`. These are exactly `price`'s losses there, at every effort, with seed spread 0. From the best known they do not move. Two of them are payloads with one upgrade.
- `real_test`: from nothing, -0.395 (`real_08927991_4514f9`), -0.404 (`real_27a2bc40_2991b9`), -0.273 (`real_8f8ac564_8bb1fc`) and -0.020 (`real_46b6a0d3_1ac5d5`). These are `price`'s four worst payloads (means -0.131, -0.062, -0.008, -0.006), 98% of its total loss on the set. It sometimes escapes through its price trajectories and kicks, never reliably. It always escapes on four more payloads with bad cold fixed points (-0.14 to -0.36).
- `speed_test`'s worst are also lone or small upgrades (`single_1235_t4_n1` -0.730, `single_1104_serca_n1` -0.131); not checked with `ascend`.
- `solve` gets out because its moves are random streak moves judged by the true metric, and its state keeps a back streak per pool.

None of `price`'s kicks changes a back streak of a single upgrade, and the price phase cannot express "juice on the late taps": the linear responses near the empty plan stay near empty, which the attempt's own write-up noted.

## Where it is ahead

- It beats every `solve` run, the 384k ones included, on 14 of 286 `real_income_test` steps, by 0.012% of flow in total (largest 0.0044, `realinc_8f8ac564_8bb1fc`). On none of `real_test`, 4 of `quick_dev`, and 8 of `speed_test` (0.025% in total).
- `advanced` at every effort, `chain` and `b10` on `quick_dev` (+0.0039 and +0.0032 at effort 2). The write-up's claims fit this: pricing advanced plans jointly, and the exchange kick.

These finds are too small and too scattered to justify moving a piece into `solve` as it is.

## What to change (price v2)

In order of what the data says they are worth:

1. **Reach the back-streak basin.** Add starts and a move:
   - starts: each normal upgrade with each pool's cheapest id on its last k blocks, for a few k, plus the full plan, each followed by responses;
   - a move in the polish and the kicks: per upgrade and pool, the back streak set to k blocks from the pity on a short grid, judged by the true metric, then that upgrade's response.

   The test is `ascend`'s "cold a" column going to 0 on `small` and the four `real_test` payloads above. That alone should close most of the -0.005 on `real_test`.
2. **Calibrate the work counter by streak length**, so that an effort costs the same CPU at 1 and at 10. At streak length 1 it now spends a third of what it spends at 10. Without this, any matched-CPU reading is skewed by the mix of streak lengths.
3. **Ablate the price phase** (`PRICE_SHARE` 0, its work given to the polish and kicks). Nothing here shows that the coordination by prices earns its 25% rather than the polish and kicks. If it does not, `price` is a response-polish local search and should be judged as one.
4. The attempt's own next step: price trajectories from the polished best, whose shadow prices are a far better guide than those of the empty plan.

Develop on `pilot`, `quick_dev`, `small` and `speed_dev`; keep `real_test` and `real_income_test` for the final reading, or they stop being held out.
