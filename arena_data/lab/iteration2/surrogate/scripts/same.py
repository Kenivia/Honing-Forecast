# Run by run, two scratch variants: equal metric and evaluations? usage: same.py <scratch db> <A> <B> [scenario substring]
import sys, json
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
s, A, B = sys.argv[1:4]
pat = sys.argv[4] if len(sys.argv) > 4 else ''
c, _ = connect(s)
def get(v):
    return {(sc, st, seed): (m, e) for sc, st, seed, m, e in c.execute("select s.scenario, s.step, r.seed, r.metric_ref, json_extract(r.counters,'$.StatesEvaluated') from s.run r join s.scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null and r.variant=? and s.scenario like ?", (v, f'%{pat}%'))}
a, b = get(A), get(B)
keys = [k for k in a if k in b]
print(f'{len(keys)} runs in common; same metric {sum(a[k][0]==b[k][0] for k in keys)}, same evals {sum(a[k][1]==b[k][1] for k in keys)}, evals x{sum(b[k][1] for k in keys)/max(sum(a[k][1] for k in keys),1):.3f}')
