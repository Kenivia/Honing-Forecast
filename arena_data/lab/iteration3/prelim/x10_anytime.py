# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# anytime curve of solve 48k on the same native-1 payloads: at streak 1 (cut_a) and forced 10 (x10_a), each vs its own streak's best
import sys, json, collections, statistics, sqlite3, pickle
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
mres = {i: json.loads(p).get('min_resolution', 1) for i, p in c.execute('select id, payload from scenario_step')}
b10 = collections.defaultdict(lambda: -1e300)
for snap, se, v, st, seed, ch, m, ev, cpu, fl, ref in D['runs']:
    if snaps[snap] == 'x10_a': b10[(st, ch)] = max(b10[(st, ch)], m)
x10 = c.execute("select id from snapshot where label='x10_a'").fetchone()[0]
cut = c.execute("select id from snapshot where label='cut_a'").fetchone()[0]
NS = [1, 200, 500, 1000, 2000, 4000, 8000, 16000, 10**9]
def curve(snap, var, sset, ch, bestmap, nat):
    rows = []
    for se in [i for i in sess if setof.get(i) == sset]:
        for rid, st, seed in c.execute("select id, step_id, seed from run where snapshot_id=? and session_id=? and variant=? and kind='optimize' and error is null and seed<3 and chained=?", (snap, se, var, ch)):
            if mres[st] != nat: continue
            tr = c.execute("select iter, value from trace where run_id=? and key='best'", (rid,)).fetchall()
            if tr: rows.append((st, sorted(tr)))
    cells = []
    for N in NS:
        g = [max(0, bestmap[(st, ch)] - max(x for i, x in tr if i <= N)) / flow[(st, ch)] * 100 for st, tr in rows if any(i <= N for i, x in tr)]
        cells.append(f'{statistics.mean(g):.3f}/{100*sum(x<=0.01 for x in g)/len(g):3.0f}%')
    firsts = sorted(next(i for i, x in tr if (max(y for _, y in tr) - x) / flow[(st, ch)] * 100 <= 0.01) for st, tr in rows)
    return len(rows), cells, firsts[len(firsts)//2], firsts[int(.9*(len(firsts)-1))]
print('N:', NS)
for sset, ch in [('real_test', 0), ('real_income_test', 1), ('large', 0)]:
    for lab, snap, var, bm in [('streak 1 ', cut, 'cut_a', best), ('forced 10', x10, 'x10_a:streak_length=10', b10)]:
        n, cells, med, p90 = curve(snap, var, sset, ch, bm, 1)
        print(f'{sset:16s} {lab} n{n:4d} ' + ' '.join(f'{x:>11s}' for x in cells) + f'  to-final median {med} p90 {p90}')
