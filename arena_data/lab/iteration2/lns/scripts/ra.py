# lns in pr_a batch ra: per set against solve 48k, and the payloads that move at effort 1 and 2 (gaps, mean over seeds).
# usage (repo root): python arena_data/lab/iteration2/lns/scripts/ra.py [k]
import sys
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
k = int(sys.argv[1]) if len(sys.argv) > 1 else 10
B = 'pr_a'
L = {e: f'pr_a:algo=lns,lab_effort={e}' for e in ('0.25', '0.5', '1', '2')}
for s, ch in sets_of('ra'):
    print(f'== {s} ch{ch}')
    for e, v in L.items():
        r = cmp('ra', s, ch, B, v)
        if r: print(f'  lns {e:4s}', fmt(r))
    for v in ('pr_a:max_iters=96000', 'pr_a:kick_stall=32'):
        r = cmp('ra', s, ch, B, v)
        if r: print(f'  {v[5:]:9s}', fmt(r)[:60])
    rows = pay('ra', s, ch, L['1'], B, 999)
    two = {r[5]: r for r in pay('ra', s, ch, L['2'], B, 999)}
    kick = {r[5]: r for r in pay('ra', s, ch, 'pr_a:kick_stall=32', B, 999)}
    sel = [r for r in rows if abs(r[0]) > 0.01]
    sel = sel[:k] + [r for r in sel[k:] if r[0] > 0][-k:]
    for d, ga, gb, n, ns, name in sel:
        t = two.get(name); kk = kick.get(name)
        print(f'    {name:40s} n{n:3d} diff1 {d:+7.3f} gap solve {gb:6.3f} lns1 {ga:6.3f} lns2 {t[1] if t else float("nan"):6.3f} kick {kk[1] if kk else float("nan"):6.3f}')
