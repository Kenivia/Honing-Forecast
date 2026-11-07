# Scratch runs against stored runs of the same variant under another label: same metric and evals, seed for seed.
# usage (repo root): rep.py <scratch db> <scratch label> <stored label> [variant substring]
import sys, sqlite3, re
sys.path.insert(0, 'arena_data/lab/iteration2/cem/scripts')
from an import D, runs, steps
db, lab, old = sys.argv[1:4]
pat = sys.argv[4] if len(sys.argv) > 4 else ''
c = sqlite3.connect(db)
key = {(sc, st): i for i, (sc, st, n) in steps.items()}
stored = {}
for snap, se, v, st, seed, ch, m, ev, cpu, fl, ref in D['runs']:
    if D['snaps'][snap] == old:
        stored[(v, st, seed, ch)] = (m, ev)
q = """select r.variant, s.scenario, s.step, r.seed, r.chained, r.metric_ref, coalesce(json_extract(r.counters, '$.StatesEvaluated'), 0), r.flow
 from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null"""
tot = {}
for v, sc, st, seed, ch, m, ev, fl in c.execute(q):
    if pat not in v or not v.startswith(lab): continue
    o = old + v[len(lab):]
    t = tot.setdefault(v, [0, 0, 0, 0, 0])
    k = (o, key.get((sc, st)), seed, ch)
    if k not in stored:
        t[4] += 1; continue
    sm, sev = stored[k]
    t[0] += 1
    d = (m - sm) / max(fl or 1, 1) * 100
    t[1] += d > 1e-9
    t[2] += d < -1e-9
    t[3] += sev == ev
for v, (n, w, l, same, miss) in sorted(tot.items()):
    print(f'{v}: {n} paired, {w} better, {l} worse, evals equal on {same}, {miss} unpaired')
