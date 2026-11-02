# Stored metric_ref (runs.sqlite) against the same states rescored at HEAD (scratch copy). Read-only on both.
import sqlite3, sys
import numpy as np
old = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
new = sqlite3.connect(f'file:{sys.argv[1]}?mode=ro', uri=True)
ids = [r[0] for r in new.execute('select id from picked')] if len(sys.argv) < 3 else None
q = 'select id, step_id, metric_ref, gross, flow, variant from run where error is null and state is not null and metric_ref is not null'
a = {r[0]: r for r in old.execute(q)}
b = {r[0]: r for r in new.execute(q)}
if ids is None:
    ids = list(a)
ids = [i for i in ids if i in a and i in b]
names = dict(old.execute("select id, scenario || ':' || step from scenario_step"))
da = np.array([(b[i][2] - a[i][2]) / max(abs(a[i][4]), 1.0) for i in ids])
dg = np.array([(b[i][3] - a[i][3]) / max(abs(a[i][3]), 1.0) for i in ids])
df = np.array([(b[i][4] - a[i][4]) / max(abs(a[i][4]), 1.0) for i in ids])
print('states', len(ids), 'unchanged exactly', int((da == 0).sum()))
ad = np.abs(da)
for p in (50, 90, 99, 99.9, 100):
    print(f'  |d metric_ref| / flow p{p}: {np.percentile(ad, p):.3e}')
print(f'  signed mean {da.mean():.3e}, min {da.min():.3e}, max {da.max():.3e}')
print(f'  |d gross|/gross max {np.abs(dg).max():.3e}; |d flow|/flow max {np.abs(df).max():.3e}')
print('  over 1e-6:', int((ad > 1e-6).sum()), ' over 1e-5:', int((ad > 1e-5).sum()), ' over 1e-4 (noise line):', int((ad > 1e-4).sum()))
order = np.argsort(-ad)[:12]
print('largest:')
for k in order:
    i = ids[k]
    print(f'  {names[a[i][1]]:42s} {da[k]:+.3e} metric {a[i][2]:.6g} flow {a[i][4]:.6g} {a[i][5][:40]}')
# worst per step, and whether a gap within a step moves: the step's best and each state shift together or not
step = {}
for k, i in enumerate(ids):
    step.setdefault(a[i][1], []).append((a[i][2], b[i][2], a[i][4], k))
flips, moved = 0, []
for sid, rows in step.items():
    bo = max(r[0] for r in rows)
    bn = max(r[1] for r in rows)
    for o, n, f, k in rows:
        go, gn = (bo - o) / max(abs(bo), 1), (bn - n) / max(abs(bn), 1)
        fo, fn = (bo - o) / max(abs(f), 1), (bn - n) / max(abs(f), 1)
        moved.append(abs(fn - fo))
        flips += (fo <= 1e-4) != (fn <= 1e-4)
moved = np.array(moved)
print(f'gap to the step best (% of flow basis): max change {moved.max():.3e}, p99 {np.percentile(moved, 99):.3e}; hit flips at 1e-4 of flow: {flips} of {len(moved)}')
# does the best known move: the old best's state, rescored, against the new best, in share of flow
loss = []
for sid, rows in step.items():
    old_best = max(rows, key=lambda r: r[0])
    new_best = max(rows, key=lambda r: r[1])
    loss.append(((new_best[1] - old_best[1]) / max(abs(old_best[2]), 1), sid))
loss.sort(reverse=True)
print('best known, new best above the old best rescored: max', f'{loss[0][0]:.3e}', names[loss[0][1]], '; steps over 1e-6:', sum(x[0] > 1e-6 for x in loss), 'of', len(loss))
