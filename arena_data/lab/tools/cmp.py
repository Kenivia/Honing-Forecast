# Variants against a base variant, per set, matched by scenario/step/seed. With a scratch db, its variants against a base in either.
# Best known is over everything read. diff/gap in % of flow; evals = StatesEvaluated ratio, cpu = cpu_s ratio (runs of 16 jobs).
# The re-run's other variants show only when the regex asks for them.
# usage: cmp.py <base> [variant regex] [set,set] [chained 0|1] [scratch db]
import sys, collections, statistics, re
from db import connect
base_name = sys.argv[1]
like = sys.argv[2] if len(sys.argv) > 2 else ''
sets = sys.argv[3].split(',') if len(sys.argv) > 3 and sys.argv[3] else None
ch = int(sys.argv[4]) if len(sys.argv) > 4 else 0
c, dbs = connect(sys.argv[5] if len(sys.argv) > 5 else None)
def load(d):
    return c.execute(f"""select r.variant, s.scenario, s.step, r.seed, r.metric_ref, coalesce(json_extract(r.counters, '$.StatesEvaluated'), 0), r.cpu_s, r.flow, ss.args from {d}.run r join {d}.scenario_step s on s.id=r.step_id
        join {d}.session ss on ss.id=r.session_id where r.kind='optimize' and r.error is null and r.chained=?""", (ch,)).fetchall()
setof = lambda args: re.search(r'--set (\S+)', args).group(1)
runs = collections.defaultdict(dict); best = collections.defaultdict(lambda: -1e300); flow = {}
for d in dbs:
    for v, sc, st, seed, m, cpu, sec, fl, args in load(d):
        key = (sc, st)
        best[key] = max(best[key], m)
        flow[key] = max(flow.get(key, 1), fl or 1)
        if d == dbs[-1] or v == base_name:
            runs[(v, setof(args))][(key, seed)] = (m, cpu, sec or 0)
by_set = collections.defaultdict(list)
for (v, s) in runs: by_set[s].append(v)
for s in sorted(by_set):
    if sets and s not in sets: continue
    if (base_name, s) not in runs: continue
    base = runs[(base_name, s)]
    print(f'== {s} vs {base_name}')
    for v in sorted(by_set[s]):
        if like and not re.search(like, v): continue
        if not like and v.startswith('lab_r1') and v != base_name: continue
        r = runs[(v, s)]
        common = [k for k in r if k in base]
        if not common: continue
        steps = collections.defaultdict(list)
        for k in common: steps[k[0]].append(k)
        diffs = [statistics.mean(r[k][0] - base[k][0] for k in ks) / flow[st] * 100 for st, ks in steps.items()]
        gaps = [(best[k[0]] - r[k][0]) / flow[k[0]] * 100 for k in common]
        cpu = sum(r[k][1] for k in common) / max(sum(base[k][1] for k in common), 1)
        sec = sum(r[k][2] for k in common) / max(sum(base[k][2] for k in common), 1e-9)
        print(f'  {v:44s} n{len(steps):4d} diff {statistics.mean(diffs):+8.4f} worst {min(diffs):+7.3f} best {max(diffs):+7.3f} +{sum(d > 0.01 for d in diffs):3d} -{sum(d < -0.01 for d in diffs):3d} | hit {100*sum(g <= 0.01 for g in gaps)/len(gaps):5.1f} gap {statistics.mean(gaps):7.4f} | evals x{cpu:5.2f} cpu x{sec:5.2f}')
