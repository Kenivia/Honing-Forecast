# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# solve's budget curve (pr_a batch ra) and x10_a, per set, split by streak length of the payload
import sys, json, collections, statistics, sqlite3
sys.path.insert(0, 'arena_data/lab/tools')
from an import *

c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
mres = {}
for i, p in c.execute('select id, payload from scenario_step'):
    mres[i] = json.loads(p).get('min_resolution', 1)

def table(grp, s, ch, base, variants, split=True):
    B = runs[(grp, s, ch)][base]
    if not B: return
    for v in variants:
        R = runs[(grp, s, ch)].get(v)
        if not R: continue
        for res in ((1, 10) if split else (None,)):
            ks = [k for k in R if k in B and (res is None or mres[k[0]] == res)]
            if not ks: continue
            gaps = [(best[(k[0], ch)] - R[k][0]) / flow[(k[0], ch)] * 100 for k in ks]
            cpu = sum(R[k][2] for k in ks) / max(sum(B[k][2] for k in ks), 1e-9)
            ev = statistics.mean(R[k][1] for k in ks)
            gs = sorted(gaps)
            print(f'{s:17s} ch{ch} res{res!s:4s} {v[-40:]:40s} n{len(ks):4d} hit {100*sum(g<=0.01 for g in gaps)/len(gaps):5.1f} gap {statistics.mean(gaps):.4f} p90 {gs[int(.9*(len(gs)-1))]:.3f} max {gs[-1]:.3f} cpu x{cpu:4.2f} ev {ev:7.0f}')

print('== pr_a ra: solve 6k..96k')
vs = ['pr_a:max_iters=6000', 'pr_a:max_iters=12000', 'pr_a:max_iters=24000', 'pr_a', 'pr_a:max_iters=96000']
for s, ch in sets_of('ra'):
    if s in ('real_test', 'real_income_test', 'quick_dev', 'speed_test'):
        table('ra', s, ch, 'pr_a', vs)
print('== x10_a (all at streak 10)')
x = sorted(runs[('x10_a', 'real_test', 0)].keys())
print(x)
for s, ch in sets_of('x10_a') + [('choices_test', 0), ('speed_test10', 0), ('choices_test', 1)]:
    if runs.get(('x10_a', s, ch)):
        table('x10_a', s, ch, 'x10_a:streak_length=10', sorted(runs[('x10_a', s, ch)].keys()), split=False)
