# A variant per category (streak x size, optionally x choice chests), picked to trade mlog against CPU: for each weight
# lam on CPU, every cell takes the variant that minimises sum(log10(gap + 0.001)) + lam * sum(cpu / mean 48k cpu of the set).
# Prints the frontier: CPU against 48k (ratio of sums, per streak), mlog per streak, and each cell's pick.
# usage (repo root): mix.py [snapshot] [--chests] [--split dev|test] [--only a,b,...]  (--only: variant suffixes allowed)
import sys, json, sqlite3, collections, math, glob
opts = sys.argv[1:]
args = [a for i, a in enumerate(opts) if not a.startswith('--') and (i == 0 or opts[i - 1] not in ('--split', '--only'))]
SNAP = args[0] if args else 'cold_a'
CHESTS = '--chests' in opts
only = lambda k: opts[opts.index(k) + 1] if k in opts else None
meta, choice = {}, {}
for f in glob.glob('arena_data/scenarios/real3/r3_*.json'):
    d = json.load(open(f, encoding='utf-8'))
    meta[d['name']] = d['meta']
    choice[d['name']] = any(len(c['options']) > 1 and c['count'] > 0 for c in d['steps'][0].get('chests', {}).values())
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
step = {i: (sc, h) for i, sc, h in c.execute("select id, scenario, payload_hash from scenario_step where scenario like 'r3%'")}
snap = dict(c.execute('select label, id from snapshot'))[SNAP]
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0); rows = []
for sid, v, st, seed, ch, m, fl, cpu in c.execute(f"select snapshot_id, variant, step_id, seed, chained, metric_ref, flow, cpu_s from run where kind='optimize' and error is null and step_id in ({','.join(map(str, step))})"):
    if 'streak_length=' in v: continue
    sc, h = step[st]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if sid == snap and not ch and sc in meta: rows.append((v, sc, h, seed, m, cpu or 0))
size = lambda n: '1-3' if n <= 3 else '4-10' if n <= 10 else '11-25' if n <= 25 else '26+'
allowed = set(only('--only').split(',')) if only('--only') else None
lab = lambda v: v[len(SNAP) + 1:] if v != SNAP else '48k'
cell = {}
data = collections.defaultdict(dict)  # (cell) -> variant -> {(sc, seed): (log, cpu)}
for v, sc, h, seed, m, cpu in rows:
    md = meta[sc]
    if only('--split') and md['split'] != only('--split'): continue
    if allowed and lab(v) not in allowed and v != SNAP: continue
    k = (md['min_resolution'], size(md['upgrades'])) + ((choice[sc],) if CHESTS else ())
    gap = max(0.0, best[h] - m) / flow[h] * 100
    data[k].setdefault(v, {})[(sc, seed)] = (math.log10(gap + 0.001), cpu)
# only runs every variant of the cell has
for k, V in data.items():
    common = set.intersection(*(set(x) for x in V.values()))
    data[k] = {v: {r: x[r] for r in common} for v, x in V.items()}
base = {res: sum(x[1] for k, V in data.items() if k[0] == res for x in V[SNAP].values()) for res in (1, 10)}
nrun = {res: sum(len(V[SNAP]) for k, V in data.items() if k[0] == res) for res in (1, 10)}
seen = set()
print(f"{'lam':>8s} {'cpu x 1':>7s} {'mlog 1':>6s} {'cpu x 10':>8s} {'mlog 10':>7s}  picks")
for lam in [0] + [10 ** (e / 4) for e in range(-8, 17)] + [1e9]:
    pick = {}
    for k, V in data.items():
        scale = base[k[0]] / nrun[k[0]]
        pick[k] = min(V, key=lambda v: sum(l for l, _ in V[v].values()) + lam * sum(cp for _, cp in V[v].values()) / scale)
    out = []
    for res in (1, 10):
        ks = [k for k in data if k[0] == res]
        cx = sum(cp for k in ks for _, cp in data[k][pick[k]].values()) / base[res]
        ml = sum(l for k in ks for l, _ in data[k][pick[k]].values()) / nrun[res]
        out += [cx, ml]
    key = tuple(sorted(pick.items()))
    if key in seen: continue
    seen.add(key)
    print(f"{lam:8.3g} {out[0]:7.2f} {out[1]:6.2f} {out[2]:8.2f} {out[3]:7.2f}  " + '; '.join(f"{k[0]}/{k[1]}{'/c' if CHESTS and k[2] else ('/-' if CHESTS else '')}: {lab(pick[k])}" for k in sorted(pick)))
