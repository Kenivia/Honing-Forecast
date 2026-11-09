# Where cheap variants miss on real3 singles: per payload over its seeds, whether misses are the payload's (every seed)
# or a seed's, and the worst payloads with their shape.
# usage (repo root): tail.py [snapshot] variant-suffix...   e.g. tail.py bud_a max_iters=0 max_iters=3000,chains=1
import sys, json, sqlite3, collections, glob, statistics
SNAP = sys.argv[1]
VARS = [SNAP] + [f'{SNAP}:{v}' for v in sys.argv[2:]]
meta = {}
for f in glob.glob('arena_data/scenarios/real3/r3_*.json'):
    d = json.load(open(f, encoding='utf-8'))
    meta[d['name']] = d['meta']
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
step = {i: (sc, h) for i, sc, h in c.execute("select id, scenario, payload_hash from scenario_step where scenario like 'r3%'")}
snap = dict(c.execute('select label, id from snapshot'))[SNAP]
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0)
runs = collections.defaultdict(list)
ids = ','.join(map(str, step))
for sid, v, st, m, fl in c.execute(f"select snapshot_id, variant, step_id, metric_ref, flow from run where kind='optimize' and error is null and step_id in ({ids})"):
    if 'streak_length=' in v: continue
    h = step[st][1]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if sid == snap and v in VARS and step[st][0].startswith('r3_'): runs[(v, st)].append(m)
gap = lambda st, m: max(0.0, best[step[st][1]] - m) / flow[step[st][1]] * 100
for v in VARS:
    print(f'== {v}')
    for res in (1, 10):
        P = [(st, sorted(gap(st, m) for m in L)) for (vv, st), L in runs.items() if vv == v and meta[step[st][0]]['min_resolution'] == res]
        for thr in (0.01, 0.1):
            every = sum(1 for _, g in P if g[0] > thr); some = sum(1 for _, g in P if g[-1] > thr)
            print(f'  x{res:<2d} >{thr}: payloads missed by every seed {every}, by some seed {some}, of {len(P)};'
                  f' mean of best seed {statistics.mean(g[0] for _, g in P):.4f}, of median {statistics.mean(g[len(g) // 2] for _, g in P):.4f}')
    P = sorted(((statistics.mean(g), st, g) for st, g in ((st, sorted(gap(st, m) for m in L)) for (vv, st), L in runs.items() if vv == v)), reverse=True)[:12]
    base = {st: statistics.mean(gap(st, m) for m in runs[(SNAP, st)]) for _, st, _ in P}
    for mg, st, g in P:
        md = meta[step[st][0]]
        print(f'    {step[st][0]:34s} tier {md["tier"]} upg {md["upgrades"]:3d} x{md["min_resolution"]:<2d} fam {md["members"]:2d} seeds {" ".join(f"{x:.3f}" for x in g)}  48k {base[st]:.4f}')
