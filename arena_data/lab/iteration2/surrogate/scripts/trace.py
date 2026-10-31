# Budget use of the surrogate per set: spent vs budget, evaluations, cpu, sur_* medians. usage: trace.py [variant] [scratch db]
import sys, collections, statistics, re, json
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
var = sys.argv[1] if len(sys.argv) > 1 else 'lab_r1:algo=surrogate,lab_effort=1'
c, dbs = connect(sys.argv[2] if len(sys.argv) > 2 else None)
d = dbs[-1]
rows = c.execute(f"""select r.id, s.scenario, s.payload, r.cpu_s, r.wall_s, json_extract(r.counters,'$.StatesEvaluated'), ss.args, r.chained, length(s.payload)
 from {d}.run r join {d}.scenario_step s on s.id=r.step_id join {d}.session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null and r.variant=?""", (var,)).fetchall()
tr = collections.defaultdict(dict)
for rid, key, val in c.execute(f"select run_id, key, value from {d}.trace where key like 'sur_%' and run_id in (select id from {d}.run where variant=?)", (var,)):
    tr[rid][key] = val
agg = collections.defaultdict(list)
for rid, sc, p, cpu, wall, ev, args, ch, _ in rows:
    s = re.search(r'--set (\S+)', args).group(1)
    n = len(json.loads(p)['upgrade_info'])
    agg[(s, ch)].append((n, cpu or 0, ev or 0, tr[rid].get('sur_work', 0), tr[rid].get('sur_spent', 0), tr[rid].get('sur_n', 0), tr[rid].get('sur_sign', 0), tr[rid].get('sur_confirmed', 0), abs(tr[rid].get('sur_level', 0)), wall or 0))
print('set ch runs | n med | cpu_s mean | wall sum | evals med | work/spent | spent/budget med | checks med | sign med | confirmed med | level med')
for k, v in sorted(agg.items()):
    bud = [75000 + 3000 * x[0] for x in v]
    print(f'{k[0]:18s} {k[1]} {len(v):5d} | {statistics.median(x[0] for x in v):3.0f} | {statistics.mean(x[1] for x in v):7.2f} | {sum(x[9] for x in v):8.0f} | {statistics.median(x[2] for x in v):8.0f} | {sum(x[3] for x in v)/max(sum(x[4] for x in v),1):.2f} | {statistics.median(x[4]/b for x, b in zip(v, bud)):.2f} | {statistics.median(x[5] for x in v):6.0f} | {statistics.median(x[6] for x in v):.2f} | {statistics.median(x[7] for x in v):.2f} | {statistics.median(x[8] for x in v):.5f}')
