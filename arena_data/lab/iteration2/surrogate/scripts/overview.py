# Surrogate at each effort against solve 48k / 96k and kick loop, per set and per mode, and the top losses / wins at effort 1 and 2.
import sys, collections, statistics, re, json
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
c, _ = connect()
rows = c.execute("""select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.cpu_s, r.flow, ss.args, r.chained
 from run r join scenario_step s on s.id=r.step_id join session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null""").fetchall()
setof = lambda a: re.search(r'--set (\S+)', a).group(1)
best = {}; flow = {}
runs = collections.defaultdict(dict)
for v, sc, st, seed, m, cpu, fl, args, ch in rows:
    k = (sc, st)
    best[k] = max(best.get(k, -1e300), m)
    flow[k] = max(flow.get(k, 1), fl or 1)
    runs[(v, setof(args), ch)][(k, seed)] = (m, cpu or 0)
print(sorted({k[0] for k in runs if 'surrogate' in k[0] or k[0].startswith('lab_r1')})[:60])
base = 'lab_r1'
for (v, s, ch) in sorted(runs):
    if not re.search(r'surrogate,lab_effort=(1|2)$', v): continue
    r = runs[(v, s, ch)]; b = runs[(base, s, ch)]
    common = [k for k in r if k in b]
    per = collections.defaultdict(list)
    for k in common: per[k[0]].append(k)
    ds = sorted(((statistics.mean(r[k][0] - b[k][0] for k in ks) / flow[p] * 100, p) for p, ks in per.items()))
    print(f'== {v} {s} ch={ch}: n={len(ds)} mean {statistics.mean(d[0] for d in ds):+.4f}, wins {sum(d[0] > 0.01 for d in ds)} losses {sum(d[0] < -0.01 for d in ds)}')
    for d, p in ds[:6]:
        if d < -0.01: print(f'   {d:+.3f} {p[0]}:{p[1]}  gap {(best[p]-statistics.mean(r[k][0] for k in per[p]))/flow[p]*100:.3f}')
    for d, p in ds[-6:]:
        if d > 0.01: print(f'   {d:+.3f} {p[0]}:{p[1]}  gap {(best[p]-statistics.mean(r[k][0] for k in per[p]))/flow[p]*100:.3f}')
