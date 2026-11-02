# Hits over 10 seeds per step: production cut, old reach with the production cut, and the finer cut.
import sqlite3, collections
main = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
s = sqlite3.connect('file:arena_data/lab/iteration2/free_tap_cut/scratch.sqlite?mode=ro', uri=True)
q = "select s.scenario, s.step, r.metric_ref, r.flow from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null"
best, flow = collections.defaultdict(lambda: -1e300), {}
for db in (main, s):
    for sc, st, m, fl in db.execute(q):
        best[(sc, st)] = max(best[(sc, st)], m)
        flow[(sc, st)] = max(flow.get((sc, st), 1), fl or 1)
cols = {'cut_a': 'prod', 'cut_x': 'old reach', 'cut_a:special_tol=1e-7': 'fine',
        'cut_a:algo=lns,lab_effort=1': 'prod', 'cut_x:algo=lns,lab_effort=1': 'old reach', 'cut_a:algo=lns,lab_effort=1,special_tol=1e-7': 'fine'}
rows = collections.defaultdict(lambda: collections.defaultdict(list))
for v, sc, st, m in s.execute("select r.variant, s.scenario, s.step, r.metric_ref from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null"):
    if v in cols:
        rows[(sc, st, 'lns' if 'lns' in v else 'solve')][cols[v]].append((best[(sc, st)] - m) / flow[(sc, st)] * 100)
print(f'{"payload":32s} {"algo":6s} ' + ' '.join(f'{c:>16s}' for c in ('prod', 'old reach', 'fine')))
tot = collections.Counter()
for key in sorted(rows):
    if key[0].find('3300') < 0 and key[0].find('0017') < 0 and key[0].find('1033') < 0:
        continue
    cells = []
    for c in ('prod', 'old reach', 'fine'):
        g = rows[key][c]
        h = sum(x <= 0.01 for x in g)
        tot[c] += h
        cells.append(f'{h:2d}/{len(g):2d} {sum(g) / max(len(g), 1):.3f}')
    print(f'{key[0] + ":" + str(key[1]):32s} {key[2]:6s} ' + ' '.join(f'{c:>16s}' for c in cells))
print(dict(tot))
