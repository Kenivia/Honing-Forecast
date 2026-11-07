# Mean gap (% of flow) over seeds on chosen payloads, for variants of one group. usage: moving.py <group> <variant,variant,...> <payload:step> ...
# A payload is 'set/scenario-substring:step'.
import sys
sys.path.insert(0, 'arena_data/lab/iteration2/cem/scripts')
from an import *
grp, vs = sys.argv[1], sys.argv[2].split(',')
vs = [v.replace(';', ',') for v in vs]
print('payload'.ljust(34), ' '.join(v.split(':')[-1][:14].rjust(14) for v in vs))
for p in sys.argv[3:]:
    s, rest = p.split('/')
    pat, step = rest.rsplit(':', 1)
    ch = 1 if s == 'real_income_test' else 0
    R = runs[(grp, s, ch)]
    sts = [st for st, (sc, stp, n) in steps.items() if pat in sc and stp == int(step) and any(k[0] == st for k in R[vs[0]])]
    for st in sts:
        cells = []
        for v in vs:
            g = [(best[(st, ch)] - R[v][k][0]) / flow[(st, ch)] * 100 for k in R[v] if k[0] == st]
            cells.append(f'{statistics.mean(g):.3f}' if g else '-')
        print(f'{steps[st][0][:28]}:{step}'.ljust(34), ' '.join(c.rjust(14) for c in cells))
