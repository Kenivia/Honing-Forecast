# Production (prod_a, 9ae4f169) against the current default and the stored old 48k (cold_d plain) on the real3 sets.
# Gaps in % of flow against the best known per payload hash (runs without a streak_length override).
# CPU x is against prod_a: paired by session, step, seed for the default; ReferenceUs-corrected for the stored 48k.
# usage (repo root): prod.py [db] [--streak]
import sys, json, sqlite3, collections, math, glob
STREAK = '--streak' in sys.argv
DB = next((a for a in sys.argv[1:] if not a.startswith('--')), 'arena_data/lab/runs.sqlite')
DEF = 'cold_d:max_iters=12000,adv_radius=1,coarse_shared=true,coarse_cache=true,size_power=0.5'
NAMES = {'prod_a': 'production', DEF: 'default (12k)', 'cold_d': 'old 48k (stored)'}

meta = {}
for f in glob.glob('arena_data/scenarios/real3/*.json'):
    d = json.load(open(f, encoding='utf-8'))
    meta[d['name']] = d['meta']
c = sqlite3.connect(f'file:{DB}?mode=ro', uri=True)
step = {i: (sc, st, h) for i, sc, st, h in c.execute("select id, scenario, step, payload_hash from scenario_step where scenario like 'r3%'")}
snaps = dict(c.execute('select label, id from snapshot'))
want = {snaps['prod_a']: ('prod_a',), snaps['cold_d']: (DEF, 'cold_d')}
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0)
runs = {}
ids = ','.join(map(str, step))
for sid, se, v, st, seed, ch, m, fl, cpu, ref in c.execute(f"""select snapshot_id, session_id, variant, step_id, seed, chained, metric_ref, flow, cpu_s,
        json_extract(counters, '$.ReferenceUs') from run where kind='optimize' and error is null and step_id in ({ids}) order by id"""):
    if 'streak_length=' in v: continue
    h = step[st][2]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if v in want.get(sid, ()):
        k = (v, st, seed, ch)
        if v != 'cold_d' or k not in runs: runs[k] = (se, m, cpu or 0, ref)

KIND = {'r3': 'cold', 'r3inc': 'income', 'r3seq': 'seq'}
prod = {(st, seed, ch): r for (v, st, seed, ch), r in runs.items() if v == 'prod_a'}
G = collections.defaultdict(list)
for (v, st, seed, ch), (se, m, cpu, ref) in runs.items():
    p = prod.get((st, seed, ch))
    if not p: continue
    sc, k, h = step[st]
    md = meta[sc]
    if (sc.startswith('r3inc') or sc.startswith('r3seq')) != bool(ch): continue
    key = (KIND[sc.split('_')[0]], md['split']) + ((md['min_resolution'],) if STREAK else ())
    gap = max(0.0, best[h] - m) / flow[h] * 100
    if v == 'cold_d':
        if not (ref and p[3]): continue
        pair = (cpu / ref, p[2] / p[3])
    else:
        pair = (cpu, p[2]) if se == p[0] else None
    G[key + (v,)].append((gap, pair))

def stats(L):
    g = sorted(x[0] for x in L)
    q = lambda p: g[min(len(g) - 1, int(p * len(g)))]
    P = [x[1] for x in L if x[1]]
    return dict(n=len(g), mean=sum(g) / len(g), rms=math.sqrt(sum(x * x for x in g) / len(g)), mlog=sum(math.log10(x + 0.001) for x in g) / len(g),
                hit=100 * sum(x <= 0.01 for x in g) / len(g), p90=q(.9), p99=q(.99), cpu=sum(a for a, b in P) / sum(b for a, b in P) if P else float('nan'))

hdr = ['set', 'split'] + (['streak'] if STREAK else []) + ['variant', 'runs', 'CPU x', 'hit %', 'mean', 'RMS', 'mlog', 'p90', 'p99']
print('| ' + ' | '.join(hdr) + ' |'); print('|' + '---|' * len(hdr))
for key in sorted({k[:-1] for k in G}, key=lambda k: (['cold', 'income', 'seq'].index(k[0]), k[1:])):
    for v in ('prod_a', DEF, 'cold_d'):
        if key + (v,) not in G: continue
        r = stats(G[key + (v,)])
        print('| ' + ' | '.join(map(str, key)) + f" | {NAMES[v]} | {r['n']} | {r['cpu']:.2f} | {r['hit']:.1f} | {r['mean']:.4f} | {r['rms']:.4f} | {r['mlog']:.2f} | {r['p90']:.4f} | {r['p99']:.4f} |")
