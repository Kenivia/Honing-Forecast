# Per-seed gaps (% of flow) of variants on payloads matching a pattern. usage: seeds.py <group> <set> <chained> <pattern> <variant> [variant ...]
import sys
sys.path.insert(0, 'arena_data/lab/iteration2/cem/scripts')
from an import *
grp, s, ch, pat = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
vs = sys.argv[5:]
R = runs[(grp, s, ch)]
stepids = sorted({k[0] for v in vs for k in R[v] if pat in f'{steps[k[0]][0]}:{steps[k[0]][1]}'}, key=lambda st: steps[st][:2])
for st in stepids:
    sc, step, n = steps[st]
    f = flow[(st, ch)]
    print(f'{sc}:{step} n{n}')
    for v in vs:
        g = [((best[(st, ch)] - R[v][k][0]) / f * 100, R[v][k][1], R[v][k][2]) for k in sorted(R[v]) if k[0] == st]
        print(f'   {v:50s}', ' '.join(f'{x:.3f}' for x, _, _ in g), '| ev', ' '.join(f'{e/1000:.0f}k' for _, e, _ in g), '| cpu', ' '.join(f'{c:.0f}' for _, _, c in g))
