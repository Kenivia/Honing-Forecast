# cmp.py/pay.py on the cached dump of runs.sqlite (dump.py), far faster than the database. Best known is over every run.
# cmp(group, set, chained, base, variant[, base group]) and pay(...) follow cmp.py and pay.py; a group is a snapshot label,
# or pr_a's batch (ra..rd). Use: sys.path.insert(0, 'arena_data/lab/tools'); from an import *  (or python -i an.py)
import re, collections, statistics
from dump import load
D = load()
steps, sess, snaps = D['steps'], D['sess'], D['snaps']
best = collections.defaultdict(lambda: -1e300); flow = {}
for snap, se, v, st, seed, ch, m, ev, cpu, fl, ref in D['runs']:
    k = (st, ch)
    if m > best[k]: best[k] = m
    flow[k] = max(flow.get(k, 1), fl or 1)
setof = {i: re.search(r'--set (\S+)', a).group(1) for i, a in sess.items() if '--set' in a}
def batch(a):
    for b, mark in [('ra', 'max_iters=6000 '), ('rb', 'lab_off=order'), ('rc', 'prices+kicks'), ('rd', '--set pilot')]:
        if mark in a: return b
    return '?'
# runs[(group, set, chained)][variant][(step, seed)] = (metric, evals, cpu, ref); the group is the snapshot label,
# or pr_a's batch (ra..rd), so that cpu compares inside one batch
runs = collections.defaultdict(lambda: collections.defaultdict(dict))
for snap, se, v, st, seed, ch, m, ev, cpu, fl, ref in D['runs']:
    lab = snaps[snap]
    grp = batch(sess[se]) if lab == 'pr_a' else lab
    runs[(grp, setof.get(se), ch)][v][(st, seed)] = (m, ev, cpu or 0, ref)

def cmp(grp, s, ch, base, v, bgrp=None):
    B = runs[(bgrp or grp, s, ch)][base]; R = runs[(grp, s, ch)][v]
    common = [k for k in R if k in B]
    if not common: return None
    by = collections.defaultdict(list)
    for k in common: by[k[0]].append(k)
    diffs = [statistics.mean(R[k][0] - B[k][0] for k in ks) / flow[(st, ch)] * 100 for st, ks in by.items()]
    gaps = [(best[(k[0], ch)] - R[k][0]) / flow[(k[0], ch)] * 100 for k in common]
    bgaps = [(best[(k[0], ch)] - B[k][0]) / flow[(k[0], ch)] * 100 for k in common]
    ev = sum(R[k][1] for k in common) / max(sum(B[k][1] for k in common), 1)
    cpu = sum(R[k][2] for k in common) / max(sum(B[k][2] for k in common), 1e-9)
    return dict(n=len(by), diff=statistics.mean(diffs), worst=min(diffs), best=max(diffs), win=sum(d > 0.01 for d in diffs), loss=sum(d < -0.01 for d in diffs),
                hit=100 * sum(g <= 0.01 for g in gaps) / len(gaps), bhit=100 * sum(g <= 0.01 for g in bgaps) / len(bgaps), gap=statistics.mean(gaps), ev=ev, cpu=cpu, runs=len(common))

def fmt(r):
    return f"n{r['n']:4d} diff {r['diff']:+8.4f} worst {r['worst']:+7.3f} best {r['best']:+7.3f} +{r['win']:3d} -{r['loss']:3d} | hit {r['hit']:5.1f} (base {r['bhit']:5.1f}) gap {r['gap']:7.4f} | ev x{r['ev']:5.2f} cpu x{r['cpu']:5.2f}"

def pay(grp, s, ch, A, Bv, k=8, pat=''):
    a_, b_ = runs[(grp, s, ch)][A], runs[(grp, s, ch)][Bv]
    by = collections.defaultdict(list)
    for key in a_:
        if key in b_: by[key[0]].append(key)
    out = []
    for st, ks in by.items():
        sc, step, n = steps[st]
        if pat not in sc: continue
        f = flow[(st, ch)]
        a = statistics.mean(a_[x][0] for x in ks); b = statistics.mean(b_[x][0] for x in ks)
        out.append(((a - b) / f * 100, (best[(st, ch)] - a) / f * 100, (best[(st, ch)] - b) / f * 100, n, len(ks), f'{sc}:{step}'))
    out.sort()
    return out

SETS = ['real_test', 'quick_dev', 'speed_test', 'advanced', 'free_taps', 'juice', 'small', 'large', 'real_income_test']
def sets_of(grp):
    return [(s, ch) for s in SETS + ['pilot'] for ch in (0, 1) if runs.get((grp, s, ch))]
