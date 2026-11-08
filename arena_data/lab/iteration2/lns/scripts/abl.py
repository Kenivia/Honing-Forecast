# Batch a: the payloads each lns variant moves against lns 1 (mean over seeds), with both gaps.
# usage (repo root): abl.py <variant suffix after lab_effort=1> [k] [set]
import sys
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
G = 'lns_a'
M = 'lns_a:algo=lns,lab_effort=1'
v = M + sys.argv[1]
k = int(sys.argv[2]) if len(sys.argv) > 2 else 6
only = sys.argv[3] if len(sys.argv) > 3 else None
for s, ch in sets_of(G):
    if only and s != only: continue
    rows = [r for r in pay(G, s, ch, v, M, 999) if abs(r[0]) > 0.01]
    if not rows: continue
    print(f'== {s}: {sum(r[0] > 0 for r in rows)} better, {sum(r[0] < 0 for r in rows)} worse')
    for d, ga, gb, n, ns, name in rows[:k] + [r for r in rows[k:] if r[0] > 0][-k:]:
        print(f'   {name:38s} n{n:3d} {d:+7.3f}  gap {gb:6.3f} -> {ga:6.3f}')
