# Scratch runs: mean gap per payload step (over seeds) for every variant, beside stored cut_a's solve 48k and lns 1. Best known over both dbs.
# usage (repo root): sgap.py <scratch db> [scenario substring] [chained 0|1]
import sys, sqlite3, collections, statistics
db = sys.argv[1]
pat = sys.argv[2] if len(sys.argv) > 2 else ''
ch = int(sys.argv[3]) if len(sys.argv) > 3 else 0
q = """select sn.label, r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow, r.cpu_s from run r join scenario_step s on s.id=r.step_id
 join snapshot sn on sn.id=r.snapshot_id where r.kind='optimize' and r.error is null and r.chained=? and s.scenario like ?"""
rows = []
for path in ('arena_data/lab/runs.sqlite', db):
    c = sqlite3.connect(f'file:{path}?mode=ro', uri=True)
    rows += [(path == db,) + r for r in c.execute(q, (ch, f'%{pat}%'))]
best = collections.defaultdict(lambda: -1e300); flow = {}
for scr, lab, v, sc, st, seed, m, fl, cpu in rows:
    best[(sc, st)] = max(best[(sc, st)], m); flow[(sc, st)] = max(flow.get((sc, st), 1), fl or 1)
mine = {k for scr, lab, v, sc, st, *_ in rows if scr for k in [(sc, st)]}
g = collections.defaultdict(lambda: collections.defaultdict(list)); cpu = collections.defaultdict(float)
for scr, lab, v, sc, st, seed, m, fl, c_ in rows:
    k = (sc, st)
    if k not in mine: continue
    if not scr and v not in ('cut_a', 'cut_a:algo=lns,lab_effort=1'): continue
    name = v if not scr else v.split(':', 1)[1] if ':' in v else 'plain'
    if not scr: name = 'cut solve' if v == 'cut_a' else 'cut lns1'
    g[name][k].append((best[k] - m) / flow[k] * 100); cpu[name] += c_ or 0
names = sorted(g, key=lambda n: (not n.startswith('cut'), n))
keys = sorted(mine)
print(f'{"variant":55s} ' + ' '.join(f'{sc[-12:]}:{st}'[-15:].rjust(15) for sc, st in keys) + '      sum    cpu')
for n in names:
    vals = [statistics.mean(g[n][k]) if g[n][k] else float('nan') for k in keys]
    print(f'{n[:55]:55s} ' + ' '.join(f'{x:15.3f}' for x in vals) + f' {sum(x for x in vals if x == x):8.3f} {cpu[n]:6.0f}')
