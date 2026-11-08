# Batch a: every variant per set against a base (solve 48k by default), from the cached dump.
# usage (repo root): a.py [base suffix] [variant substring]
import sys
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
G = sys.argv[3] if len(sys.argv) > 3 else 'lns_a'
base = G + (sys.argv[1] if len(sys.argv) > 1 and sys.argv[1] else '')
pat = sys.argv[2] if len(sys.argv) > 2 else ''
for s, ch in sets_of(G):
    print(f'== {s} ch{ch} vs {base}')
    for v in sorted(runs[(G, s, ch)]):
        if pat not in v or v == base: continue
        r = cmp(G, s, ch, base, v)
        if r: print(f'  {v[6:]:52s}', fmt(r))
