# Batch a: FS (lns from solve's endpoint) against KM (kick loop + mem_polish), payloads where they differ, and the offline better-of-two
# per run (not a run): diff against solve 48k, hit, CPU KM + FS - solve.
# usage (repo root): union.py
import sys, statistics, collections
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
G, B = 'lns_a', 'lns_a'
FS = 'lns_a:algo=lns,lab_effort=1,lab_on=fromsolve'
KM = 'lns_a:kick_stall=32,mem_polish=true'
MP = 'lns_a:algo=lns,lab_effort=1,lab_on=mempolish'
for s, ch in sets_of(G):
    R = runs[(G, s, ch)]
    b, f, k = R[B], R[FS], R[KM]
    keys = [x for x in b if x in f and x in k]
    by = collections.defaultdict(list)
    for x in keys: by[x[0]].append(x)
    diffs, gaps = [], []
    for st, ks in by.items():
        fl = flow[(st, ch)]
        diffs.append(statistics.mean(max(f[x][0], k[x][0]) - b[x][0] for x in ks) / fl * 100)
        gaps += [(best[(st, ch)] - max(f[x][0], k[x][0])) / fl * 100 for x in ks]
    cpu = (sum(f[x][2] + k[x][2] - b[x][2] for x in keys)) / sum(b[x][2] for x in keys)
    print(f'== {s}: KM u FS diff {statistics.mean(diffs):+.4f} hit {100 * sum(g <= 0.01 for g in gaps) / len(gaps):5.1f} cpu x{cpu:.2f}')
    pf = {r[5]: r for r in pay(G, s, ch, FS, B, 999)}
    pk = {r[5]: r for r in pay(G, s, ch, KM, B, 999)}
    pm = {r[5]: r for r in pay(G, s, ch, MP, B, 999)}
    for name in sorted(pf):
        a, c = pf[name], pk.get(name)
        if c and (abs(a[1] - c[1]) > 0.01 or a[0] > 0.01 or c[0] > 0.01):
            m = pm.get(name)
            print(f'   {name:38s} solve {a[2]:6.3f} FS {a[1]:6.3f} KM {c[1]:6.3f} lns+mem {m[1] if m else float("nan"):6.3f}')
