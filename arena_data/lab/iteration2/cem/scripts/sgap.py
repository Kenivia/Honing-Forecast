# Mean gap (% of flow) per payload of every variant in a scratch db, against the best known over runs.pkl and the scratch.
# usage: sgap.py <scratch db> [scenario substring]
import sys, sqlite3, collections, statistics
sys.path.insert(0, 'arena_data/lab/iteration2/cem/scripts')
from an import best, flow, steps
db, pat = sys.argv[1], (sys.argv[2] if len(sys.argv) > 2 else '')
c = sqlite3.connect(db)
key = {(sc, st): i for i, (sc, st, n) in steps.items()}
rows = c.execute("""select r.variant, s.scenario, s.step, r.seed, r.chained, r.metric_ref, r.cpu_s from run r join scenario_step s on s.id=r.step_id
 where r.kind='optimize' and r.error is null""").fetchall()
b = dict(best)
for v, sc, st, seed, ch, m, cpu in rows:
    k = (key.get((sc, st)), ch)
    b[k] = max(b.get(k, -1e300), m)
g = collections.defaultdict(lambda: collections.defaultdict(list)); cpus = collections.defaultdict(float)
for v, sc, st, seed, ch, m, cpu in rows:
    if pat not in sc: continue
    k = (key.get((sc, st)), ch)
    g[f'{sc[:26]}:{st}'][v].append((b[k] - m) / flow.get(k, 1) * 100)
    cpus[v] += cpu or 0
vs = sorted(cpus)
for i, v in enumerate(vs): print(f'{i}: {v}  cpu {cpus[v]:.0f}')
print('payload'.ljust(30), ' '.join(f'{i:>6d}' for i in range(len(vs))))
for p in sorted(g):
    print(p.ljust(30), ' '.join(f'{statistics.mean(g[p][v]):6.3f}' if g[p][v] else '     -' for v in vs))
