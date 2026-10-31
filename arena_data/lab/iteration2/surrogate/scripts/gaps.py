# Gap per seed (mean, min, max) of each variant on payloads matching a substring. usage: gaps.py <substring> [variant regex] [chained 0|1] [scratch db]
import sys, collections, statistics, re
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
pat = sys.argv[1]
like = sys.argv[2] if len(sys.argv) > 2 else 'lab_r1(:max_iters=(48|96|384)000)?$|surrogate|i2g:kick_stall=32,kick_types=windowq'
ch = int(sys.argv[3]) if len(sys.argv) > 3 else 0
c, dbs = connect(sys.argv[4] if len(sys.argv) > 4 else None)
best = {}; flow = {}; m = collections.defaultdict(lambda: collections.defaultdict(dict))
for d in dbs:
    for v, sc, st, seed, met, fl in c.execute(f"select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow from {d}.run r join {d}.scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null and r.chained=? and s.scenario like ?", (ch, f'%{pat}%')):
        k = (sc, st)
        best[k] = max(best.get(k, -1e300), met)
        flow[k] = max(flow.get(k, 1), fl or 1)
        if re.search(like, v): m[k][v][seed] = met
for k in sorted(m):
    print(f'{k[0]}:{k[1]}  flow {flow[k]:.0f}')
    for v in sorted(m[k]):
        g = [(best[k] - x) / flow[k] * 100 for x in m[k][v].values()]
        print(f'   {v.replace("lab_r1:", "").replace("algo=surrogate,lab_effort=", "sur "):52s} n{len(g):2d} mean {statistics.mean(g):7.3f} min {min(g):7.3f} max {max(g):7.3f}   ' + ' '.join(f'{x:.3f}' for x in g[:10]))
