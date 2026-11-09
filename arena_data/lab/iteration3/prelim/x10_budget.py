# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# Same payloads at native streak 1 (cut_a solve 48k) vs forced 10 (x10_a solve 48k): x10 budget table with
# streak-10-only best, and anytime curves from traces.
import sys, json, collections, statistics, sqlite3
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
mres = {i: json.loads(p).get('min_resolution', 1) for i, p in c.execute('select id, payload from scenario_step')}
# best at streak 10: x10_a runs, plus runs of native-10 payloads anywhere
b10 = collections.defaultdict(lambda: -1e300)
for snap, se, v, st, seed, ch, m, ev, cpu, fl, ref in D['runs']:
    if snaps[snap] == 'x10_a' or (mres[st] == 10 and 'streak_length=1,' not in v + ','):
        b10[(st, ch)] = max(b10[(st, ch)], m)
base = 'x10_a:streak_length=10'
print('x10_a, best over streak-10 runs only; split by the payload native streak')
for s, ch in [('real_test', 0), ('real_income_test', 1), ('choices_test', 0), ('speed_test10', 0), ('large', 0)]:
    G = runs[('x10_a', s, ch)]
    if not G: continue
    for v in ['x10_a:streak_length=10,max_iters=12000', 'x10_a:streak_length=10,max_iters=24000', base, 'x10_a:streak_length=10,agree_exit=3,unanimous=2', 'x10_a:streak_length=10,chain_iters_per_upgrade=60,chain_iters_min=1000', 'x10_a:streak_length=10,max_iters=192000']:
        R = G[v]; B = G[base]
        for nat in (1, 10):
            ks = [k for k in R if k in B and mres[k[0]] == nat]
            if not ks: continue
            gaps = [max(0, b10[(k[0], ch)] - R[k][0]) / flow[(k[0], ch)] * 100 for k in ks]
            cpu = sum(R[k][2] for k in ks) / sum(B[k][2] for k in ks)
            print(f'{s:16s} nat{nat:<3d} {v[23:]:55s} n{len(ks):4d} hit {100*sum(g<=0.01 for g in gaps)/len(gaps):5.1f} gap {statistics.mean(gaps):.4f} max {max(gaps):.3f} cpu x{cpu:.2f}')
