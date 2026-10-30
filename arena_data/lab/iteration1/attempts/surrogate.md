# surrogate: moment surrogate, verified by the true metric

Code: `crates/core/src/optimizer/lab/surrogate.rs` (registered in `lab/mod.rs`). Dev db: `arena_data/lab/iteration1/dev_surrogate.sqlite`.

## Algorithm as built

- **Surrogate.** Per upgrade and row: mean, variance, k3, k4, min and max of its honed support, plus its skipped value. Per live free tap outcome (`special_probs`), sum them over the order (skipped prefix, honed suffix). A row's gold is `gold()` with each E[(t-S)+] from an Edgeworth leftover (k3, k4), clamped to [max(t-m,0), t-min] and exact off the range. Rows with linear bands are exact. Calibration: after a true evaluation, `corr[row] = -shadow_price - surrogate slope`, a linear term in each row's mean. The surrogate's raw slopes already match the shadow prices to 1e-4 on most rows.
- **Surrogate best response.** For a normal upgrade, tables of what each amount of each row is worth (the other upgrades as the summed moments, its own amount exact), taken at 24 points with straight lines between. Then the best response's recursion (`best`/`improve`/`better` copied from `response.rs`, since its tables are private) over every plan, keeping the block rule.
- **Moves the surrogate ranks**: advanced upgrades (every index of each slot), free tap order (move any affordable upgrade to any place a free tap reaches, to the end, or to a greedy order: what it saves at shadow prices per leapstone a success takes, levels kept in order per piece), chests (move all, half, an eighth or one chest between two options).
- **Descent** (surrogate only): sweeps of surrogate responses, advanced, order and chest moves until nothing moves, then **kick and repair**: one upgrade is put on a plan improved from an id combination on every tap, every other normal upgrade answers it, then it answers back; kept if the surrogate total is better. This is the joint move a single response cannot make.
- **Restarts** from the best: odd ones kick the prices (half the rows, price x e^U(-0.7,0.7)) for one descent and then descend unkicked; even ones kick the state (everything empty plus random chests; random streaks on 30% of upgrades plus order moves; one piece type emptied plus one random chest). Each descent result is evaluated once by truth, then **polished**.
- **Polish** (truth-checked): per normal upgrade, the surrogate response (1 eval), else the true `Response::of` + `better` (1 eval), else **blends** between the current and the rejected plan (the proposal on one pool only, or up to 1/4, 1/2, 3/4 of the taps). Advanced, order and chest moves: the surrogate's top 3 checked by truth. Every evaluation is checked against the best, so the result is never worse than the given or carried plans (snapped with `Model::filled`, invalid parts reset).
- **Last phase** (25% of the budget): from the best, polish with wider responses (`improve` also from every id combination on every tap) and single-block flips (every block of the live taps given another choice of one pool, ranked by the true response's score, top 4 checked by truth).
- **Budget**: work = true evaluations + `Response::work` + surrogate cost (2000 recursion steps or 6000 gold functions count as one evaluation); budget = effort x (75000 + 3000 n); stop also after 40 x effort restarts without a better best.

## Constants

SKEW true, CALIBRATE true, POINTS 24, DESCENT 6 rounds, TOP 3, KICK 0.7, START_COMBOS 12, KICK_TRIALS 24 x effort, FLIP_TOP 4, FLIP_SHARE 0.25, STALL 40, BASE_EVALS 75000, PER_UPGRADE 3000, GOLDS_PER_EVAL 6000, STEPS_PER_EVAL 2000. None tuned beyond the smoke sets.

## Dev numbers (quick_dev, 1 seed, base `current` of session 4, surrogate session 56)

| effort | diff cold / chained | hit % cold | x (CPU) |
| --- | --- | --- | --- |
| 0.5 | -0.0034 / -0.0028 | 90.5 | 0.38 |
| 1 | -0.0030 / -0.0022 | 91.4 | 0.74 |
| 2 | -0.0018 / -0.0014 | 93.7 | 1.52 |

`current` hits 97.1%. No invalid states, no panics, 349 payloads at each effort. By kind (effort 1, cold): b10 -0.0012, single -0.0029, chain -0.0081, week_hone -0.0198 (4 payloads near -0.1), week_inc -0.0022. Pilot sessions 1 to 3 were the first versions.

## How the surrogate tracked the truth (session 56, effort 1, runs with 10+ checked predictions: 228)

- Predicted against true change of each checked move: Pearson r median 0.9999, p10 0.23: big moves are predicted well, small ones are noise.
- Sign agreement 61% (median); of the moves the surrogate called better, 41% (median 32%) were truly better.
- Level |surrogate - truth| / |truth| at the end, uncorrected: median 0.08%, p90 5%.
- Where it fails: few upgrades. On `single_0002_t4_n3` (3 upgrades, long pity tails) the surrogate preferred the worse of two states by 1,800 gold where the truth said 520 the other way, and the linear correction made it worse (10,000): the slope at one state does not carry over. The true response's own tables (16 points on a coarse lattice) were off there too: it predicted +400 and the truth gave -290, hence the blends and flips.

## Where it is weak

- Hit rate trails `current` by about 6 points at matched CPU. Most misses are Nash points: every upgrade at its best response, while the best state differs in several upgrades together (back streaks on several upgrades, book streaks running to the pity on scarce stock, `week_hone`).
- The free tap order moves one upgrade at a time. Swaps were tried and dropped: they led `chain_0017` into a basin (free tapping three more upgrades, juice taken off them) it never left, -9%.
- Stalls early on easy payloads, so x at effort 1 is 0.74, not 1.
- The surrogate's cost has to be counted in the budget (an uncounted version used 5 to 10 times the CPU on some payloads), and the conversion constants are rough.

## Next

- A trust region for the correction (damp it with distance from the calibration state), or fall back to the raw surrogate on payloads with few upgrades.
- Kick and repair with the true response checking the repaired state, plus order moves that repair the upgrades they move.
- Moments that know the pity mass (a two-part model: a lattice for the pity outcome, Edgeworth for the rest).
