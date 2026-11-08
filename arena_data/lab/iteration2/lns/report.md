# `lns` investigation

**Verdict: take one piece, not the algorithm: lns's rebuild of a piece type, as a kick type in `solve`'s kick loop (`kick_types` `rebuild`).**

- **The piece:** up to 10 normal upgrades of one piece type are emptied, or each takes its best plan at noisy shadow prices. Then responses run in turn over them, before the loop's polish.
- **What it reaches.** Added to the kick loop with `mem_polish` (KM), it takes these to 0 on every seed:
  - `b10_week_inc_0004:3` (0.064);
  - `single_0085` (0.037);
  - `single_3375` (0.047). No variant before it reached 0 on every seed.

  `single_3397` goes to 0.011 (KM 0.014). It keeps KM's order basins.
- **Cost:** x0.97–1.04 of KM's CPU on the main sets, x1.23–1.25 on the regime sets.
- **What it gives up against KM:** the noisy `single_0159` and `real_27a2bc40`, plus `chain_0001:1` (0.013 → 0.030).
- **Why it works:** the end states are a joint re-split of a shared juice across one piece type. On `single_3375` the juice tails of N0.10–N0.17 shorten together; each tail alone is already optimal with the others fixed.
- **lns itself is a start plus a loop that stalls.** The loop stops on 75–100% of runs at 0.47–0.76 of its budget, so effort does not move its losses.
- **Its loss is its free tap order.** The start builds the order place by place and drops low levels. No lns move leaves such an order; memetic's polish does.
  - The rest of its loss is the family of every prototype's own start (`fromsolve` fixes it all).
- **What carries lns is the destroy step**, mostly its noisy-price start. The ladder (iteration 1's joint repair), the record-to-record acceptance, the adaptive weights and the order-move caps carry nothing.

Scripts are in `scripts/` (listed in section 0). End states are read with `iteration2/cem/scripts/state.py` (read-only). All numbers are % of flow.

## 0. State of the work

| Round | Snapshot | Batch | Tests | Status |
|---|---|---|---|---|
| a | `lns_a` (39fb007e + switches + trace) | `iteration2/lns/a` | `fromsolve`, `mempolish`, `insert`, the order-move caps, `chestboth`; the repair starts; `rrt`, `adapt`; `solve` + `mem_polish` and KM | read, section 4 |
| b | `lns_b` | `iteration2/lns/b` | the `rebuild` kick in `solve`, alone and added to KM; `fromsolve` at effort 0.5; L in the same session | read, section 5 |

- **Repeat checks:**
  - Plain `lns_a` and `lns_b` repeat `cut_a` on all 9 sets, and KM in `lns_b` repeats KM in `lns_a` (orchestrator, `an.py`).
  - The final code (smoke, scratch label `lns_c`): plain repeats `cut_a` (66 runs) and lns 1 repeats `cut_a` (109 runs, chained included). Every kept lns switch repeats `lns_a` (36 runs each). KM, KM + `rebuild`, `rebuild` alone and `fromsolve` at 0.5 repeat `lns_b`. Evaluations equal on every run.
  - `lab_algos_repeat_and_are_valid` passes.
- **Switches in `lns.rs`:**
  - `lab_off`:
    - `ladder`, `price`, `empty`: that repair start is not used; the freed set is only swept.
    - `rrt`: nothing worse than the current state is accepted.
    - `adapt`: fixed arm weights.
  - `lab_on`:
    - `fromsolve`: `solve`'s result, unsnapped, as a candidate; its cost is outside the budget.
    - `insert`: `solve`'s single-insertion order polish after the start and before the last sweep.
    - `chestboth`: 4 random chest amounts a pair after the halving transfers.
    - `mempolish`: memetic's `polish_state` (read-only call) on the best before the last sweep.
  - Removed as null: `cutreach`, `ratio` (section 2, items 5 and 7), `allplaces`, `alltries` (section 4).
- **Knob in `solve`:** the kick type `rebuild` (`kick_types`, used only when named), `rebuild_kick` in `simulated_annealing/lab_hooks.rs`.
  - Up to 10 normal upgrades of one piece type. Half the time their plans are emptied; half the time each takes its best plan at the shadow prices times lognormal noise (0.35).
  - Then responses in turn over them, each kept where better, for up to 3 rounds; then the kick loop's polish.
- **Trace:** `lns_gain`, `lns_wins`, `lns_cost` and `lns_picks` at iter = phase (`scripts/attr.py`).
  - Phases: 0 is the memetic polish, then 1 start order, 2 start sweeps, 3 start chests, 4 start rebuild, 5 start insert, 6 end insert, 7 end sweep, 8–20 the arms in `ARMS` order.
  - Run totals at iter 0: `lns_budget`, `lns_spent`, `lns_loop`, `lns_moves`, `lns_stalled`.
- **Scripts:**
  - `ra.py`: the stored effort curve;
  - `a.py <base suffix> <filter> <group>`: batch tables;
  - `abl.py`: payloads per ablation;
  - `union.py`: FS against KM, and the offline union;
  - `sgap.py`: scratch gaps;
  - `attr.py`: the trace.

## 1. Where it stands (stored effort curve, `pr_a` batch `ra`)

Each cell is diff against `solve` 48k in the same batch, with (wins, losses over 0.01), hit %, and x CPU. Price report 11 has the whole curve.

| set | lns 1 | lns 2 |
|---|---|---|
| real_test (96.6) | -0.0335 (0, 5), 84.5, x0.79 | -0.0271 (0, 5), 87.4, x1.57 |
| quick_dev (95.5) | -0.0169 (4, 35), 89.9, x1.03 | -0.0104 (8, 27), 92.0, x1.89 |
| speed_test (96.3) | -0.0100 (3, 15), 91.0, x0.57 | -0.0097 (3, 13), 91.5, x1.07 |
| large (53.3) | -0.0909 (1, 5), 50.0, x0.95 | -0.0769 (1, 5), 50.0, x1.83 |
| income (98.0) | -0.0032 (0, 20), 91.6, x1.07 | -0.0030 (0, 18), 92.8, x2.02 |
| advanced (94.4) | +0.0025 (1, 0), 97.2, x0.99 | +0.0031 (1, 0), 100, x1.85 |
| free_taps (97.2) | -0.1636 (0, 4), 77.8, x0.52 | -0.1186 (0, 2), 88.9, x1.00 |
| juice (94.4) | -0.0014 (1, 1), 91.7, x0.75 | -0.0013 (1, 1), 91.7, x1.42 |
| small (100) | -0.0014 (0, 1), 98.3, x0.81 | -0.0014 (0, 1), 98.3, x1.48 |

**Many losses do not move with effort.** These are fixed points:
- `free_taps` `single_0071_serca_n19`: 1.236 at every effort;
- `single_1056_serca_n3`: 0.936;
- `single_1104_serca_n1`: 0.392;
- `b10_single_3174_serca_n14`: 0.222;
- `juice` `single_0034`: 0.057;
- `small` `single_0002`: 0.028;
- `real_2f76be5b`: 0.039.

**Its losses** (mean gap; `solve` 48k → lns 1 → lns 2):
- Free tap order, the level-skipping basin (section 2):
  - `chain_0017:0/1`: 0.02/0.03 → 0.83/1.01 → 0.75/0.67;
  - `single_3300`: 0 → 0.46 → 0.15;
  - `single_0087`: 0 → 0.58 → 0.19;
  - `single_0071`;
  - `single_0161`: 0.03 → 0.26;
  - `b10_chain_0017:2`: 0 → 0.19.
- Advanced with normal plans: `real_08927991`: 0.002 → 0.46 → 0.38.
- Chests: `single_1104`, `single_1056`, `real_27a2bc40` (noisy): 0.27 → 0.93.
- `large`: `single_0159`, `single_0044`, `single_0161` (0.26–0.38).
- `realinc_8f8ac564` steps 0–7: 0.04–0.18 on every step.

**Its wins are the open joint-move payloads, at effort 1 from its own start:**
- `b10_week_inc_0004:3`: 0.064 → 0;
- `single_0085`: 0.037 → 0 (advanced, juice, quick_dev);
- `single_3397_serca_n4`: 0.062 → 0.004;
- `week_hone_0009:6`: 0.023 → 0;
- `chain_0001:2`: 0.030 → 0.008;
- `single_3375`: 0.047 → 0.036.

## 2. Reading the code

1. **Budget:** effort × (18000 + 2400 n), with responses charged their work. The loop stops once 45% of the budget passes without the best moving by 1e-6 of it.
   - In the smokes 54–81% of runs stop this way, at spent/budget 0.65–0.81.
   - So effort barely moves the fixed points above. This is a stall rule, not budget.
2. **No full order polish after the start.** The order is moved only by:
   - `build_order`: place by place from the front, each place taking the later upgrade that does best there. It pulls upgrades forward only. Full in the start; 8 random candidates a place inside moves (`alltries` lifts this).
   - `order_move`: a window of 1–3 places within reach + 2, plus one from anywhere, put back at the best place. Once the region is over 8 places, only ±2 around the window plus 4 random places are tried (`allplaces` lifts this).
   - `order_tail`: `build_order` from a random reached place.
3. **`build_order` drops low levels, and it runs first on the empty plans.** Putting a higher level ahead of a lower one of its piece drops the lower one from the free taps for good, since the evaluator moves it to the back. The greedy place-by-place choice does this whenever the higher level's free tap is worth more at that place. End states (`cut_a`), the same on every seed:

   | payload | `solve` order | lns order | lns gap |
   |---|---|---|---|
   | `single_0071_serca_n19` | `[12,13,14,8,0,…]` (N5.20–22) | `[12,15,0,8,…]` (N5.20, N5.23; N5.21–22 out) | 1.234–1.246 |
   | `single_3300_t4_n23` | `[0,1,…,8,13,14,16]` | `[0,2,3,…]`, `[2,4,5,…]`, `[0,1,…,8,13,14,15]` | 0.31, 1.12, 0.13 |
   | `single_0087_t4_n9` | `[3,4,5,0,1,2,6,7,8]` | `[1,2,…,8,0]` (N0.18 out) | 0.575 |
   | `chain_0017:1` | `[0,1,5,6,8,…]` | `[4,5,6,8,…]` (N0.9, N0.10 out) | 1.09–1.18 |

   The plans then follow the order: N5.23 on `single_0071` takes a long juice run once it is free tapped. Leaving such an order is a joint move of the order and the plans.
4. **Sweeps take responses by their own score**, unchecked by the metric (`solve`'s `respond_one` checks each). The start's sweeps are guarded only against the best candidate, not against the order phase's gain. Not tested: surrogate's `checked` was identical.
5. **Reach:** a tail chance over 1e-6 on the raw outcomes. On every smoke payload it equals the evaluator's cut (mean 7.7 and 8.7 places, both rules), and `cutreach` is identical to plain on all 60 smoke runs. Null, not queued.
6. **Chests:** each option all on one stack, then halving transfers in one pass. Amounts off that grid are out of reach (`single_1104`: 9 + 5 of 14).
7. **The ratio start** (lns's greedy orders) does nothing: off, it is identical on `chain_0017` and -0.1641 against -0.1636 on `free_taps`. Not queued.
8. **No bug found** that makes a state invalid or a result wrong.

## 3. Smoke tests (scratch, 3 seeds; leads for batch `a`)

Gap sums over the payload steps named, with CPU seconds (the `cut_a` rows are stored runs).

| variant | `chain_0017` + `b10_` (8 steps) | `single_3300` + `b10_` | `week_inc_0004` + `b10_` (12 steps) | `single_33*` speed_test (9) | `free_taps` set diff |
|---|---|---|---|---|---|
| `solve` 48k | 0.387 (114 s) | 0.162 | 0.079 | 0.127 | 0 |
| lns 1 | 3.000 (39 s) | 0.779 | 0.082 | 0.089 | -0.1636 |
| `fromsolve` | 0.387 (119 s) | | **0.015** | 0.073 | +0.0004, hit 100 |
| `mempolish` | **0.108** (61 s) | **0** | 0.082 | 0.069 | |
| `insert` | 2.333 | 0.708 | | | -0.1607 |
| `allplaces+alltries` | 2.912 | 0.543 | | | -0.1606 |
| `ladder` off | 2.447 | | 0.080 | 0.095 | -0.1059 |
| `price` off | 5.440 | | 0.132 | 0.115 | -0.1448 |
| `empty` off | 2.925 | | 0.123 | 0.108 | -0.1385 |
| all three off | 5.520 | | | | -0.1067 |
| `rrt` off | 3.477 | | | | -0.1637 |
| `adapt` off | 2.422 | | 0.088 | 0.086 | -0.1359 |
| `solve` + `mem_polish` | | 0 | | | |
| kick loop + `mem_polish` | | 0 | | | |

- **From `solve`'s endpoint, the rebuilds from noisy prices and from empty plans reach joint-move payloads:** `b10_week_inc_0004:3` 0.064 → 0, `single_0085` 0.037 → 0, `single_3397` 0.062 → 0.022.
  - The trace gives them to the piece-type, material and piece arms with price or empty starts.
  - On `chain_0017` the loop gains nothing from there (stalled on 24 of 24 runs at spent/budget 0.48).
- **Memetic's polish on lns's endpoint takes the order basin** that lns's own moves cannot leave. `single_0071` stays at 1.236 under every switch but `fromsolve`.
- **`chestboth`** takes `single_1104` from 0.392 to 0, and leaves `single_1056` at 0.936.

## 4. Batch `a`: every piece at effort 1, from `solve`'s endpoint, and memetic's polish on lns's

One session per set. Each cell is diff against `lns_a` (`solve` 48k), with (wins, losses over 0.01) and hit % where useful. CPU is against `solve` 48k in the same batch. M is lns at effort 1.

| variant | real_test | quick_dev | speed_test | large | income | advanced | free_taps | juice | CPU quick_dev / large |
|---|---|---|---|---|---|---|---|---|---|
| M | -0.0335 (0, 5), 84.5 | -0.0169 (4, 35), 89.9 | -0.0100 | -0.0909, hit 50 | -0.0032 (0, 20) | +0.0025 | -0.1636 | -0.0014 | x1.03 / 0.96 |
| `ladder` off | -0.0297 | -0.0158 (3, 38) | -0.0111 | -0.0699 | -0.0030 | -0.0020 | -0.1059 | -0.0013 | x1.22 / 1.07 |
| `price` off | -0.0419 | **-0.0381** (2, 52) | -0.0129, hit 85.4 | -0.0840 | -0.0051 | +0.0031 | -0.1448 | -0.0014 | x1.00 / 0.94 |
| `empty` off | -0.0362 | -0.0215 (2, 43) | -0.0103 | -0.0745 | -0.0038 | -0.0152 | -0.1385 | -0.0014 | x0.85 / 0.85 |
| all three off (no destroy) | **-0.0718** (0, 16), 62.6 | **-0.0616** (2, 88), 75.4 | -0.0537, 72.7 | -0.1492 | **-0.0206** (0, 90) | -0.0268 | -0.1067 | -0.0256 | x0.81 / 0.53 |
| `rrt` off | -0.0358 | -0.0198 (2, 37) | -0.0105 | -0.1311 | -0.0032 | +0.0025 | -0.1637 | -0.0014 | x1.03 / 0.96 |
| `adapt` off | -0.0345 | -0.0181 (4, 36) | -0.0100 | -0.1035 | -0.0034 | +0.0031 | -0.1359 | -0.0015 | x1.04 / 0.99 |
| `allplaces+alltries` | -0.0333 | -0.0142 (4, 32) | -0.0102 | -0.0934 | -0.0033 | +0.0025 | -0.1606 | -0.0013 | x0.98 / 0.94 |
| `insert` | -0.0333 | -0.0127 (6, 30) | -0.0096 | -0.0601, hit 60 | -0.0028 (0, 11) | +0.0025 | -0.1607 | -0.0014 | x1.02 / 0.96 |
| `chestboth` | -0.0339 | -0.0169 | -0.0084 | -0.0909 | -0.0031 | +0.0025 | -0.1636 | -0.0014 | x1.03 / 0.95 |
| `mempolish` | -0.0314 (0, 4) | **-0.0005** (10, 16), 93.7 | -0.0047 | -0.0439, hit 63 | -0.0024 (0, 11) | +0.0025 | -0.1028 | -0.0014 | x1.14 / 1.12 |
| `fromsolve` (FS) | +0.0022 (1, 0), 97.1 | +0.0007 (5, 0), 96.4 | +0.0007, 96.9 | +0.0075 (3, 0), 63.3 | +0.0001 | +0.0031, 100 | +0.0004, 100 | +0.0034, 100 | x1.87 / 1.86 |
| `solve` + `mem_polish` | 0 | +0.0014 (4, 0), 96.3 | 0 | +0.0021 | 0 | 0 | 0 | 0 | x1.07 / 1.10 |
| KM (`kick_stall=32,mem_polish=true`) | +0.0058, 97.6 | +0.0019 (10, 0), 97.6 | +0.0007 | +0.0093, hit 73 | +0.0001 | 0 | 0 | +0.0003 | x1.91 / 2.59 |

- `small` is 100 for FS, KM, `mempolish`, `solve` + `mem_polish` and the all-off variant, and 98.3 (one payload, 0.028) for M and its other variants.
- FS's CPU on real_test, speed_test and income is x1.68, x1.48 and x1.97.
- FS, KM and `solve` + `mem_polish` lose no payload on any set.
- `large` rows follow `single_0159` and `single_0044`, which swing (results note).

**The repair starts.**
- **The destroy step is what lns is.** With all three repair starts off, every arm only sweeps its set, and lns loses everywhere (quick_dev 88 losses, income 90). This is iteration 1's "joint rebuild against one at a time" result, and it holds.
- **The noisy-price start carries most of it.** Without it lns loses `single_3300` (0.46 → 3.03), `b10_chain_0017:0/1` (0 → 0.95), `week_hone_0012` (5 steps, 0 → 0.05–0.10), `realinc_8f8ac564:0` (0.09 → 0.27) and `real_8f8ac564` (0.08 → 0.27).
- **The emptied start carries `chain_0001`** (0.02 → 0.40 on four steps without it) and `advanced` `single_0100` (0.008 → 0.219).
- **The ladder carries nothing.**
  - Without it, quick_dev is -0.0158 against -0.0169, `real_08927991` falls from 0.463 to 0.318, and `single_0087` from 0.575 to 0. It loses `chain_0001:0` (0.02 → 0.66).
  - It is the cheap way to spend work: without it, the same work costs x1.22 against x1.03 CPU on quick_dev, since sweeps' responses cost more than they are charged.
  - Iteration 1's "the joint rebuild is most of what earns" compared no destroy at all; the ladder itself is not that part.
- Each single start moves the order-basin payloads (`single_0087`, `chain_0017`, `single_3300`, `single_0161`) both ways. These depend on which order the start lands in (section 2), not on the repair.

**The acceptance and the weights carry nothing.** `rrt` off and `adapt` off are level on every main set.

**The order.**
- **The caps** (`allplaces+alltries`) are worth +0.0027 on quick_dev and nothing elsewhere. They are not cem's story.
- **`solve`'s insertion polish** (`insert`) fixes `single_0161` (0.26 → 0.006) and `b10_single_0161` (0.198 → 0.001), long reaches lns's moves never try. That is most of its `large` gain (-0.091 → -0.060). It does not fix the low-level orders.
- **Memetic's polish on lns's endpoint** (`mempolish`) fixes them: every step of `chain_0017` and `b10_chain_0017`, `single_3300`, `b10_single_3300`, `single_0087` and `single_3174` go to `solve`'s gap or 0.
  - It keeps lns's own wins (`single_0085`, `b10_week_inc_0004:3`, `single_3397` 0.004, `week_hone_0009:6`).
  - Its pairs (one upgrade out of the free taps, one in) with a re-plan put a dropped low level back.
  - It cannot fix `free_taps` `single_0071`, which needs N5.21 and N5.22 in and N5.23 out at once (1.236 under every variant but FS).
- **What lns + memetic's polish still loses** (quick_dev -0.0005 against `solve` + `mem_polish` +0.0014) is the family of memetic's and price's own starts:
  - `week_hone_0004` steps 0–3 (0.07–0.22);
  - `real_08927991` (0.445, advanced with normal plans);
  - `real_8f8ac564` and `realinc_8f8ac564` (0.02–0.18). lns's plans differ only on the identical upgrades N6.5 and N6.6, a fixed point of responses in turn (price report 2);
  - `single_1104` (chest grid), `juice` `single_0034`, `small` `single_0002`;
  - `large` `single_0159` / `single_0044`;
  - the bimodal `chain_1051` and `single_1208`.

  FS has none of them.

**Chests.** `chestboth` takes `single_1104` from 0.392 to 0 and changes nothing else. `single_1056` (0.936) stays.

**From `solve`'s endpoint (FS).**
- **It loses nothing**, and the loop gains little: it stalls on 98% of quick_dev runs at spent/budget 0.49.
- **What it takes that KM does not:** `b10_week_inc_0004:3` (0.064 → 0) and `single_0085` (0.037 → 0, on quick_dev, advanced and juice).
  - It also takes part of `single_3397` (0.062 → 0.022), `single_0159` (0.056 → 0.021), `single_0161`, and `chain_1051:0` (0.063 → 0, as KM does).
- **What KM takes that FS does not:** `chain_0017:2/3`, `b10_chain_0017:3`, `b10_single_3300`, `real_27a2bc40` (0.065 against 0.203).
- **Where the gains come from** (trace, quick_dev): the rebuilds from noisy prices and from empty plans of a piece type or a material's users (`ptype.empty`, `ptype.price`, `mat.price`). The order arms and the ladder add nothing there.
- **End state, `single_0085`** (one piece type, N5.7–N5.19 and four advanced): FS's best shortens the juice tails of N5.10–N5.15 together (`1,3x8 3x16 …` to `1,3x8 3x1 -x18`), where `solve`'s seeds 0 and 2 keep longer tails (gap 0.055). This is a re-split of a shared juice across one piece type: the joint move of several plans named in "Open".
- **KM and FS together (offline arithmetic, not a run):** each run's better result of the two, CPU KM + FS - `solve`.

  | set | KM ∪ FS |
  |---|---|
  | real_test | +0.0060, hit 97.9, x2.31 |
  | quick_dev | +0.0023, 98.2, x2.78 |
  | speed_test | +0.0008, 97.2, x1.95 |
  | `large` | +0.0115, 73.3, x3.45 |
  | income | +0.0002, 98.3, x2.63 |
  | advanced / juice / free_taps | +0.0031 / +0.0034 / +0.0004, all 100 |

  On quick_dev that is level with KM ∪ L (+0.0022 at x2.96) and KM ∪ cem's restarts at 0.5 (+0.0021 at x2.34), from the cem report.

**The stall** (trace, M): lns stops on 75–100% of runs (quick_dev 0.90, income 0.93, `large` 0.47) at spent/budget 0.47–0.76. The start does nearly all of the gain.

**Round b's lead, from smoke** (3 seeds): the piece-type rebuild as a kick in `solve`'s kick loop (`kick_types` `rebuild`) reaches `b10_week_inc_0004:3`, `single_0085`, `single_3375` and `single_3397` (from 0.064, 0.037, 0.047 and 0.062 to 0 alone; 0, 0, 0 and 0.011 added to KM). Added to KM it keeps KM's `chain_0017` basins (gap sum 0.025 against KM's 0.035).

## 5. Batch `b`: lns's rebuild as a kick in `solve`

One session per set. Each cell is diff against `lns_b` (`solve` 48k), with (wins, losses) and hit %. CPU is against `solve` 48k in the same batch.
- KM = `kick_stall=32,mem_polish=true`.
- KMR = KM with `kick_types=window+chest+adv+piece+rebuild`.
- R = `kick_stall=32,kick_types=rebuild`.
- FS 0.5 = `algo=lns,lab_effort=0.5,lab_on=fromsolve`.
- L = `sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach`.

| variant | real_test | quick_dev | speed_test | large | income | advanced | free_taps | juice | CPU quick_dev / large |
|---|---|---|---|---|---|---|---|---|---|
| KM | +0.0058, 97.6 | +0.0019 (10, 0), 97.6 | +0.0007, 96.9 | +0.0093, hit 73 | +0.0001 | 0 | 0 | +0.0003 | x1.92 / 2.65 |
| KMR | +0.0044, 97.9 | **+0.0021** (9, 0), 97.6 | +0.0007, 97.5 | +0.0045, hit 63 | +0.0001 | +0.0031, 100 | +0.0004, 100 | +0.0033, 100 | x1.87 / 2.49 |
| R | +0.0013, 97.1 | +0.0006 (5, 0), 96.5 | +0.0006, 97.2 | +0.0033, hit 57 | +0.0001 | +0.0031, 100 | +0.0004, 100 | +0.0031, 100 | x1.44 / 1.82 |
| FS 0.5 | +0.0015, 97.1 | +0.0006 (4, 0), 96.3 | +0.0007, 96.9 | +0.0039, hit 60 | +0.0001 | +0.0031, 100 | +0.0004, 100 | +0.0034, 100 | x1.47 / 1.43 |
| L | +0.0064, 97.9 | +0.0018 (8, 0), 97.2 | **+0.0016**, 98.3 | +0.0066, hit 63 | +0.0002 | +0.0031, 100 | +0.0004, 100 | +0.0034, 100 | x2.05 / 3.00 |

- **KMR against KM in the same batch:**
  - quick_dev +0.0002 (2, 2), real_test -0.0014 (0, 1), speed_test 0 (1, 1), `large` -0.0047 (0, 1), income 0;
  - advanced, free_taps and juice +0.003 to +0.0004 (hit 100);
  - CPU x0.94–1.04 on the main sets, x1.23–1.25 on the regime sets.
- **L against KM:** CPU x1.06–1.36 on the main sets, up to x1.58 on the regime sets.
- No variant loses a payload to `solve` anywhere (worst run -0.003).
- `small` is 100 for every variant.

**Gaps on the payloads that move** (3 seeds; real_test 10):

| payload | `solve` | KM | KMR | R | FS 0.5 | L |
|---|---|---|---|---|---|---|
| `b10_week_inc_0004:3` | 0.064 | 0.064 | **0** | **0** | 0 | 0.014 |
| `single_0085` | 0.037 | 0.037 | 0 | 0 | 0 | 0 |
| `single_3375` | 0.047 | 0.047 | **0** | **0** | 0.036 | 0.005 |
| `single_3397_serca_n4` | 0.062 | 0.014 | 0.011 | **0** | 0.029 | 0.004 |
| `week_hone_0009:6` | 0.023 | 0 | 0 | 0.023 | 0 | 0 |
| `chain_0017:2` / `:3` | 0.072 / 0.175 | 0 / 0 | 0 / 0 | 0.072 / 0.175 | 0.072 / 0.175 | 0 / 0 |
| `b10_chain_0017:3` | 0.090 | 0 | 0 | 0.090 | 0.090 | 0 |
| `b10_single_3300` | 0.162 | 0 | 0 | 0.122 | 0.122 | 0.039 |
| `chain_1051:0` / `:1` | 0.063 / 0.123 | 0 / 0.123 | 0.063 / 0.123 | 0.063 / 0.123 | 0 / 0.123 | **0 / 0** |
| `chain_0001:1` | 0.035 | 0.013 | 0.030 | 0.035 | 0.029 | 0.035 |
| `single_3116` | 0.020 | 0.018 | 0.014 | 0.017 | 0.020 | **0.002** |
| `large` `single_0159` | 0.056 | 0.008 | 0.043 | 0.054 | 0.033 | 0.053 |
| `real_27a2bc40` (10 seeds) | 0.274 | 0.065 | 0.120 | 0.236 | 0.228 | 0.044 |

**The rebuild kick takes the joint-move payloads that no kick, polish or loop before it took on every seed.**
- KMR keeps every KM order basin.
- What it gives up against KM is `single_0159` and `real_27a2bc40` (both listed as noisy) and `chain_0001:1` (0.013 → 0.030), plus `chain_1051:0`, which is bimodal.
- A likely cause of `large`, untested: the adv kick, which carries `large` (price report 9), now gets a fifth of the kicks instead of a quarter.

**End state, `single_3375`** (one piece, N0.0–N0.20):
- R ends on the best known on every seed, with N0.18 at the front of the order.
- The juice tails of N0.10–N0.17 are shortened together: N0.13 `2x14` → `2x11`, N0.15 `4x17` → `4x12`, N0.17 `4x22` → `4x16`.
- KM's seed 2 has the same order with the long tails (gap 0.033).
- Each tail is optimal with the others fixed, so no single response leaves it; emptying or re-pricing the piece type at once and responding in turn does. `single_0085` is the same (N5.10–N5.15, section 4).

**The kick alone reaches the joint-move payloads and none of the order basins.** It does as well as lns from `solve`'s endpoint at the same CPU (quick_dev +0.0006 at x1.44 against x1.47), and better on `single_3375` and `single_3397`. So lns-from-`solve` is superseded.

**`fromsolve` at effort 0.5 against effort 1** (batch a; across batches):
- the same payloads; quick_dev +0.0006 against +0.0007;
- evaluations x1.20 against x1.38, CPU x1.47 against x1.87 (each in its own batch).

**L against the rebuild kick.**
- L also reaches the joint-move payloads, a little less fully: `b10_week_inc_0004:3` 0.014, `single_3375` 0.005.
- It alone takes `chain_1051` (bimodal) and `single_3116`, and it is ahead on speed_test.
- It misses `b10_single_3300` (0.039) and costs x1.06–1.58 of KM, against KMR's x0.97–1.25.

## 6. Pieces

| Piece | Verdict | Where | Evidence | Cost | Why |
|---|---|---|---|---|---|
| Rebuild kick in `solve`, with KM (`kick_types=window+chest+adv+piece+rebuild`, `kick_stall=32,mem_polish=true`) | **Candidate** | `b10_week_inc_0004:3`, `single_0085`, `single_3375`, `single_3397` | to 0 / 0 / 0 / 0.011; against KM quick_dev +0.0002, regime sets +0.003; KM's order basins kept | x0.97–1.25 of KM | joint re-split of a shared juice across a piece type |
| Rebuild kick alone | Joint moves only | as above | quick_dev +0.0006 (5, 0) | x1.44 | no order moves |
| lns from `solve`'s endpoint (`fromsolve`) | **Superseded by the kick** | `b10_week_inc_0004:3`, `single_0085` | effort 1 +0.0007 at x1.87; 0.5 +0.0006 at x1.47 | | the same rebuilds with lns's overhead |
| Destroy step | **What lns is** | everywhere | without any: quick_dev -0.0616 (88 losses), income -0.0206 | | |
| Noisy-price repair start | Carries most | `single_3300`, `b10_chain_0017`, `week_hone_0012`, `real_8f8ac564` | without it quick_dev -0.0381 | | |
| Emptied repair start | Small | `chain_0001`, `single_0100` | without it quick_dev -0.0215 | | |
| Ladder of prices | **Nothing** | | without it quick_dev -0.0158 against -0.0169 | cheaper per unit of work | iteration 1's result was "no destroy at all" |
| Record-to-record acceptance, adaptive weights | **Nothing** | | each off: level | | |
| Order-move caps | **Nothing; removed** | | lifted: quick_dev +0.0027, nothing else | | |
| lns's reach rule against the evaluator's cut (`cutreach`) | **Nothing; removed** | | identical on every smoke run | | the two reaches agree |
| Ratio start orders (`ratio`) | **Nothing; removed** | | identical on `chain_0017`; level on `free_taps` | | |
| `solve`'s insertion polish in lns (`insert`) | Small | `single_0161` 0.26 → 0.006 | quick_dev -0.0127 against -0.0169, `large` -0.060 against -0.091 | none | long reaches lns's moves never try |
| Memetic's polish on lns's end (`mempolish`) | Fixes the order loss | `chain_0017`, `single_3300`, `single_0087`, `single_3174` | quick_dev -0.0005 (10, 16) against `solve` + `mem_polish` +0.0014 | x1.14 | its out/in pairs with a re-plan put a dropped low level back |
| lns's start (order built place by place) | **Its weakness** | `single_0071` (1.236 at every effort), `single_3300`, `single_0087`, `chain_0017:0/1` | | | drops low levels; the plans then follow the order |
| `chestboth` | Fixes `single_1104` only | | speed_test -0.0084 against -0.0100 | none | the same as `price`'s `chestboth` |

## Not yet answered

- **The rebuild kick with L in one run**, and in the final combination search. They split `chain_1051` and `single_3116` (L) against `b10_week_inc_0004:3`, `single_3375` and `b10_single_3300` (KMR).
- **Whether the `large` and real_test differences of KMR against KM are noise** (`single_0159`, `real_27a2bc40`) or the adv kick's thinner share. A 10-seed `large` run, or KMR at `kick_stall=40`, would tell.
- **Which half of the kick does the work** (emptied plans or noisy prices), and whether a material's users, which lns also rebuilds, add anything.
- **`single_0071_serca_n19`:** leaving lns's order there needs two upgrades in and one out at once; no polish tried has that move.
- The joint-move wins rest on 3 seeds per payload, every seed at 0.
