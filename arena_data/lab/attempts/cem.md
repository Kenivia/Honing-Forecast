# cem: cross-entropy method (batch 1)

`crates/core/src/optimizer/lab/cem.rs`, one file, about 960 lines.

## The algorithm as built

- **Candidates**: the given plan and the carried ones, all snapped to the block grid (carried chest states that do not fit go back to the given ones), the given plan with nothing on its normal upgrades, and a plan built from scratch with nothing used. Each is scored if `check::invalid` passes. Then every normal upgrade takes `Response::linear(...).best()` at the `shadow_prices` of the best. The best of these is the start.
- **Model** (`Cem::around`, centred on one plan):
  - per normal upgrade and pool: a categorical over the ids (0 = none), a Gaussian front streak length in taps, and a Gaussian back streak length counted back from the pity of the front-only plan (anchored to the last slot). Lengths are rounded to the block grid; a plan is `canonical(filled(..))`, so it is always valid. The centre comes from `derive` (most used id, leading run, trailing run).
  - a joint factor: one Gaussian shift per piece type, in shares of the upgrade's length, added to every front of that type (the shared juice and book stock).
  - per advanced slot: a categorical over `0..=MAX_ADV_STATE`, peaked at the centre.
  - free tap order (only where `affords_a_free_tap`): Plackett-Luce weights, sampled with Gumbel keys. The refit puts weight `-sharp * mean elite position`, where `sharp` grows as the elites agree on positions. The raw permutation goes to the evaluator; its own cleaning decides what is free tapped. At first I forced lower levels of a piece first, which ruled out orders the evaluator rates far better (pilot chain_0011: -0.4%).
  - per chest stack with a choice: logistic-normal logits (centre share mixed 90/10 with uniform, sigma 1.5 * spread), split by largest remainder.
- **Sampling is partial**: a sample is the current plan with a few parts redrawn (1 to 25% of the parts at random, or with p 0.2 every normal of one piece type). When the order is redrawn, the normals it moves most are redrawn with it. Each part is refit only on the top 25% of the batch samples that redrew it (at least 2). Smoothing 0.7 on means and probabilities, 0.5 on sigmas, sigma floor 0.3 blocks, probability floor 0.02. The batch is sized so that each part is redrawn about 4 times (12 to 48 samples).
  Redrawing every part at once was useless: after one polish no joint sample ever beat the incumbent.
- **Cycles**: run a model until 10 rounds pass with no new best, then polish what it found. Even cycles restart from the start plan with a wide model (spreads 1, 2, 0.5 in turn); odd cycles go on from the best with spread 0.5. A plan already polished is not polished again. Stops when the budget is spent.
- **Polish** (up to 12 passes, stops when a pass gains less than 1e-7): `Response::of` + `better(.., response_joint)` on every normal upgrade in random order; advanced indices moved by ±1 and ±2; the order rebuilt greedily one place at a time over the reach of the free taps (+2, at most 10), then every move of one upgrade into or out of that reach; per chest stack, every chest on one option, then moves between two options repeated while they gain, with the step halving down to 1.
- **Budget**: `lab_effort * (10000 + 1300 * upgrades)` evaluations. A response counts as `response.work(response_steps)`.

## Dev numbers (`arena_data/lab/dev_cem.sqlite`)

quick_dev, 1 seed, session 7, against `current` (349 payloads, cold / chained):

| effort | diff % | hit % | x | invalid / panics |
|---|---|---|---|---|
| 0.5 | -0.0196 / -0.0173 | 84.5 / 88.5 | 0.52 / 0.56 | 0 |
| 1 | -0.0139 / -0.0121 | 87.4 / 91.1 | 1.03 / 1.09 | 0 |
| 2 | -0.0085 / -0.0092 | 92.0 / 93.1 | 1.98 / 2.13 | 0 |

(current: hit 97.4.) Pilot, 2 seeds, session 5 (an earlier budget, x 0.70): diff -0.0002, hit 93.8. The losses are on 8 to 40 upgrades: chain_0017_t4_n14 (-0.6%), single_0031_serca_n36 and single_0161_t4_n39 (-0.4%). There, the coordinate-wise responses of the polish keep gaining for many passes and still stop short. The fact that its losses shrink at effort 2 shows the extra work pays.

**Does the learned model earn?** At effort 1, cold, quick_dev:

- with no refit (session 8: the same cycles, partial samples and polish, but each model stays as built): diff -0.0137, hit 89.4, x 1.09;
- with refit: diff -0.0139, hit 87.4;
- polish only, from the start (session 9, x 0.04): diff -0.0763, hit 71.9.

So what earns is the sampling and restart structure: random partial redraws about the incumbent and wide restarts, each polished. The refit itself adds nothing measurable at this budget. In single traces the first model does carry plans far (pilot chain_1033 seed 2: 85.9M to 132.4M before any polish), but a model that is never refit does as well overall.

## Where it is weak

- The refit does not beat no refit. The model collapses within a few rounds onto the incumbent, and the stall rule restarts it. It then amounts to an adaptive perturbation, not a learned distribution.
- Coupled payloads with many upgrades, where the result depends on polish passes.
- The order polish costs about n * reach evaluations per pass, a lot on 65 upgrades with free taps.
- Streak model: one id per pool and front/back only. Middle blocks and mixed books come only from the polish.

## Next

- Keep elites across rounds (a sliding window) and refit fewer, larger batches, so the model has data before it narrows. Or adapt the smoothing to what the elites agree on.
- Make the joint factor carry more: a price-like latent per material used by `Response::linear` inside each sample, so one draw moves all upgrades consistently.
- Polish the elite samples of a batch, not only the end of a cycle.
