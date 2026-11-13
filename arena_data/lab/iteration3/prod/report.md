# Production against the current default

**Verdict.** The live production build (9ae4f169) scores like the old 48k on today's build (within about 0.01 mlog and 1 hit point everywhere) and costs about x1.15–1.3 of it. The current default (12k, `adv_radius=1`, `coarse_shared`, `coarse_cache`, `size_power=0.5`) costs x0.20–0.33 of production and gives up about 0.06–0.11 mlog, 3–6 hit points and x1.4–2.7 the mean gap; its worst tails (p99) are 1.4–3.4x production's, worst on `real3_test` cold at streak 10.

## Setup

- `prod_a`: 9ae4f169 built from a `git archive` copy with `HF_ROOT` at this repo, registered by hand (`Optimizer lab.md`). Repeats `lab_r1:response_points=16` exactly (60 of 60 on `small`).
- Batch `a`: `prod_a` and `cold_d:max_iters=12000,adv_radius=1,coarse_shared=true,coarse_cache=true,size_power=0.5` (the tree's default), one session per `real3_*` set, 3 seeds, driven by `cold_d.exe`. logcheck passes, no deaths, no MC failures.
- Old 48k: stored `cold_d` plain runs (accuracy exact; CPU only through `ReferenceUs`, other timeframe). None on the chained test sets.
- `scripts/prod.py [db] [--streak]`. Gaps in % of flow against the best known; CPU x against production, paired by session for the default.

## Results

| set | split | variant | runs | CPU x | hit % | mean | RMS | mlog | p90 | p99 |
|---|---|---|---|---|---|---|---|---|---|---|
| cold | dev | production | 2532 | 1.00 | 93.7 | 0.0035 | 0.0187 | -2.80 | 0.0050 | 0.0796 |
| cold | dev | default (12k) | 2532 | 0.21 | 88.2 | 0.0069 | 0.0326 | -2.70 | 0.0129 | 0.1198 |
| cold | dev | old 48k (stored) | 2532 | 0.87 | 94.1 | 0.0032 | 0.0180 | -2.81 | 0.0047 | 0.0686 |
| cold | test | production | 2454 | 1.00 | 93.0 | 0.0048 | 0.0346 | -2.82 | 0.0045 | 0.0938 |
| cold | test | default (12k) | 2454 | 0.27 | 88.8 | 0.0129 | 0.0755 | -2.73 | 0.0120 | 0.3188 |
| cold | test | old 48k (stored) | 2454 | 0.83 | 93.2 | 0.0050 | 0.0353 | -2.82 | 0.0039 | 0.0938 |
| income | dev | production | 1950 | 1.00 | 90.9 | 0.0034 | 0.0116 | -2.76 | 0.0087 | 0.0543 |
| income | dev | default (12k) | 1950 | 0.20 | 85.2 | 0.0065 | 0.0191 | -2.65 | 0.0218 | 0.0821 |
| income | dev | old 48k (stored) | 1950 | 0.85 | 90.5 | 0.0033 | 0.0109 | -2.76 | 0.0094 | 0.0529 |
| income | test | production | 1950 | 1.00 | 85.3 | 0.0047 | 0.0124 | -2.69 | 0.0182 | 0.0587 |
| income | test | default (12k) | 1950 | 0.24 | 82.5 | 0.0066 | 0.0177 | -2.62 | 0.0227 | 0.0820 |
| seq | dev | production | 1896 | 1.00 | 92.5 | 0.0040 | 0.0197 | -2.78 | 0.0068 | 0.0796 |
| seq | dev | default (12k) | 1896 | 0.23 | 86.4 | 0.0077 | 0.0328 | -2.67 | 0.0164 | 0.1297 |
| seq | dev | old 48k (stored) | 1896 | 0.78 | 92.9 | 0.0041 | 0.0208 | -2.78 | 0.0059 | 0.0880 |
| seq | test | production | 1878 | 1.00 | 93.0 | 0.0044 | 0.0264 | -2.82 | 0.0039 | 0.0938 |
| seq | test | default (12k) | 1878 | 0.33 | 89.9 | 0.0077 | 0.0487 | -2.76 | 0.0104 | 0.1460 |

By streak (`--streak`): the default's loss is larger at streak 1 (-0.07 to -0.15 mlog) than at 10 (-0.05 to -0.10); cold test streak 10 has its worst tail (p99 0.53 against 0.11).
