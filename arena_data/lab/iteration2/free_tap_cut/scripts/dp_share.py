# the free tap dp's share of a solve, one thread
import sys
import numpy as np
S = sys.path[0] + '/out/'
for s in ['free_taps', 'real_test', 'speed_test']:
    rows = []
    for line in open(f'{S}dp_{s}.tsv', encoding='utf-8'):
        a = line.rstrip('\n').split('\t')
        rows.append((a[1], float(a[2]), float(a[3]), int(a[4]), int(a[5])))
    wall = sum(r[1] for r in rows)
    dp = sum(r[2] for r in rows)
    share = np.array([r[2] / r[1] for r in rows if r[1] > 0])
    print(f'{s}: solves {len(rows)} wall {wall:.1f}s dp {dp:.1f}s share of total {dp / wall:.3f}; per solve median {np.median(share):.3f} p90 {np.percentile(share, 90):.3f} max {share.max():.3f}; calls/solve {np.mean([r[3] for r in rows]):.0f}')
    for r in sorted(rows, key=lambda r: -r[2])[:5]:
        print(f'    {r[0]:36} wall {r[1]:7.2f} dp {r[2]:6.2f} ({r[2] / r[1]:.2f}) calls {r[3]:7} budget {r[4]} us/call {r[2] / max(r[3], 1) * 1e6:.0f}')
