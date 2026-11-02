# Two post-solve loops from one solve endpoint: A, B and the better of the two per seed, against a base. usage: union.py <base> <A> <B> [chained 0|1]
import sys, collections, statistics, re
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
base, A, B = sys.argv[1:4]
ch = int(sys.argv[4]) if len(sys.argv) > 4 else 0
c, _ = connect()
best = {}; flow = {}; runs = collections.defaultdict(dict); cpu = collections.defaultdict(dict)
for v, sc, st, seed, m, fl, args, sec in c.execute("""select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow, ss.args, r.cpu_s from run r join scenario_step s on s.id=r.step_id
  join session ss on ss.id=r.session_id where r.kind='optimize' and r.error is null and r.chained=?""", (ch,)):
    k = (sc, st); best[k] = max(best.get(k, -1e300), m); flow[k] = max(flow.get(k, 1), fl or 1)
    if v in (base, A, B):
        s = re.search(r'--set (\S+)', args).group(1)
        runs[(v, s)][(k, seed)] = m; cpu[(v, s)][(k, seed)] = sec or 0
for s in sorted({k[1] for k in runs}):
    if not all((v, s) in runs for v in (base, A, B)): continue
    b, a, d = runs[(base, s)], runs[(A, s)], runs[(B, s)]
    keys = [k for k in b if k in a and k in d]
    if not keys: continue
    per = collections.defaultdict(list)
    for k in keys: per[k[0]].append(k)
    row = {}
    for name, f in (('A', lambda k: a[k]), ('B', lambda k: d[k]), ('either', lambda k: max(a[k], d[k]))):
        ds = [statistics.mean(f(k) - b[k] for k in ks) / flow[p] * 100 for p, ks in per.items()]
        gaps = [(best[k[0]] - f(k)) / flow[k[0]] * 100 for k in keys]
        row[name] = (statistics.mean(ds), sum(x > .01 for x in ds), sum(x < -.01 for x in ds), 100 * sum(g <= .01 for g in gaps) / len(gaps), statistics.mean(gaps))
    bg = [(best[k[0]] - b[k]) / flow[k[0]] * 100 for k in keys]
    print(f'{s:16s} n{len(per):4d} base hit {100*sum(g<=.01 for g in bg)/len(bg):5.1f} | ' + ' | '.join(f'{n} {r[0]:+.4f} +{r[1]} -{r[2]} hit {r[3]:5.1f} gap {r[4]:.4f}' for n, r in row.items()) + f' | cpu A x{sum(cpu[(A,s)][k] for k in keys)/max(sum(cpu[(base,s)][k] for k in keys),1e-9):.2f} B x{sum(cpu[(B,s)][k] for k in keys)/max(sum(cpu[(base,s)][k] for k in keys),1e-9):.2f}')
