# mlog and mean gap (% of flow) per chain step on the chained real3 sets, per streak length and variant: whether a cheap
# variant's misses build up along the chain (each variant carries its own plan).
# usage (repo root): by_step.py [snapshot] [r3inc|r3seq]
import sys, json, sqlite3, collections, math, glob
SNAP = sys.argv[1] if len(sys.argv) > 1 else 'bud_a'
KIND = sys.argv[2] if len(sys.argv) > 2 else 'r3inc'
meta = {}
for f in glob.glob('arena_data/scenarios/real3/*.json'):
    d = json.load(open(f, encoding='utf-8'))
    meta[d['name']] = d['meta']
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
step = {i: (sc, st, h) for i, sc, st, h in c.execute("select id, scenario, step, payload_hash from scenario_step where scenario like 'r3%'")}
snap = dict(c.execute('select label, id from snapshot'))[SNAP]
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0); rows = []
for sid, v, st, m, fl, cpu in c.execute(f"select snapshot_id, variant, step_id, metric_ref, flow, cpu_s from run where kind='optimize' and error is null and step_id in ({','.join(map(str, step))})"):
    if 'streak_length=' in v: continue
    h = step[st][2]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if sid == snap and step[st][0].startswith(KIND + '_'): rows.append((v, st, m, cpu or 0))
G = collections.defaultdict(list)
for v, st, m, cpu in rows:
    sc, k, h = step[st]
    G[(meta[sc]['min_resolution'], v, k)].append((max(0.0, best[h] - m) / flow[h] * 100, cpu))
steps = sorted({k[2] for k in G})
for res in (1, 10):
    print(f'streak {res}: mlog per step, then mean s/run')
    print(f"{'variant':24s} " + ' '.join(f'{s:>6d}' for s in steps))
    for v in sorted({k[1] for k in G if k[0] == res}):
        L = [G.get((res, v, s), []) for s in steps]
        lab = v[len(SNAP) + 1:][:24] if v != SNAP else '48k'
        print(f'{lab[:24]:24s} ' + ' '.join(f'{sum(math.log10(g + 0.001) for g, _ in x) / len(x):6.2f}' if x else '     -' for x in L)
              + '   ' + ' '.join(f'{sum(c for _, c in x) / len(x):5.1f}' if x else '    -' for x in L[:1] + L[1:2] + L[-1:]))
    print()
