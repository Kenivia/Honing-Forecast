# cem's stored effort curve (pr_a batch ra): per set, and its worst and best payloads against solve 48k. Run from the repo root.
import sys
sys.path.insert(0, 'arena_data/lab/iteration2/cem/scripts')
from an import *
B = 'pr_a'
for s, ch in sets_of('ra'):
    for v in ['pr_a:algo=cem,lab_effort=1', 'pr_a:algo=cem,lab_effort=2', 'pr_a:max_iters=96000', 'pr_a:kick_stall=32']:
        r = cmp('ra', s, ch, B, v)
        if r: print(f'{s:17s} {ch} {v[5:]:28s} {fmt(r)}')
    for v in ['pr_a:algo=cem,lab_effort=1', 'pr_a:algo=cem,lab_effort=2']:
        p = pay('ra', s, ch, v, B)
        print('   worst', v[5:], [(f'{d:+.3f}', f'{ga:.3f}', f'{gb:.3f}', n, sc) for d, ga, gb, n, k, sc in p[:8]])
        print('   best ', v[5:], [(f'{d:+.3f}', f'{ga:.3f}', f'{gb:.3f}', n, sc) for d, ga, gb, n, k, sc in p[::-1][:5] if d > 0.005])
