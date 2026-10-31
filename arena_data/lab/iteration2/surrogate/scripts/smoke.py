# Scratch variants on matching payloads: mean gap, seed gaps, errors, restart gains and rounds. usage: smoke.py <scratch db> <substring> [variant regex]
import sys, collections, statistics, re
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
s, pat = sys.argv[1], sys.argv[2]
like = sys.argv[3] if len(sys.argv) > 3 else ''
c, dbs = connect(s)
best = {}; flow = {}
for d in dbs:
    for sc, st, met, fl in c.execute(f"select s.scenario, s.step, r.metric_ref, r.flow from {d}.run r join {d}.scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null and r.chained=0 and s.scenario like ?", (f'%{pat}%',)):
        k = (sc, st); best[k] = max(best.get(k, -1e300), met); flow[k] = max(flow.get(k, 1), fl or 1)
rows = c.execute("select r.id, r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.error, r.cpu_s, r.counters from s.run r join s.scenario_step s on s.id=r.step_id where r.kind='optimize' and r.chained=0 and s.scenario like ? and r.variant like 'sur_s5%'", (f'%{pat}%',)).fetchall()
tr = collections.defaultdict(dict)
for rid, key, val in c.execute("select run_id, key, value from s.trace where key in ('sur_k_price','sur_k_empty','sur_k_rand','sur_k_piece','sur_k_last','sur_k_chest','sur_rounds')"):
    tr[rid][key] = val
by = collections.defaultdict(list)
for r in rows:
    if re.search(like, r[1]): by[r[1]].append(r)
for v, rs in sorted(by.items()):
    errs = [r[6] for r in rs if r[6]]
    good = [r for r in rs if not r[6]]
    gaps = [(best[(r[2], r[3])] - r[5]) / flow[(r[2], r[3])] * 100 for r in good]
    t = [tr[r[0]] for r in good]
    k = {n: sum(x.get('sur_k_' + n, 0) / flow[(r[2], r[3])] * 100 for x, r in zip(t, good)) / max(len(good), 1) for n in ['price', 'empty', 'rand', 'piece', 'chest', 'last']}
    rounds = statistics.mean(x.get('sur_rounds', 0) for x in t) if t else 0
    print(f'{v.replace("sur_s5:", "").replace("algo=surrogate,lab_effort=", "e"):62s} n{len(rs):2d} err {len(errs)} gap {statistics.mean(gaps) if gaps else -1:7.3f} cpu {statistics.mean(r[7] or 0 for r in good) if good else 0:6.2f} rounds {rounds:5.1f} ' + ' '.join(f'{n[:2]} {x:.3f}' for n, x in k.items()))
    if errs: print('   ', errs[0][:150])
