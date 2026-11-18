# The Maxroll override's owned stock variants (algo=maxroll) against solve on real3na (real3 cold singles at streak 1, advanced
# honing removed): gap to the best known in % of flow, by owned juice and chests, and payloads better/worse than the first variant.
# Best known per payload hash over every run without a streak_length override, as simple.py.
# usage (repo root): noadv.py [--best] variant ...   (first variant is the base for the paired diff; --best: gap in % of |best known| not flow)
import sys, json, sqlite3, collections, math, glob
DB = 'arena_data/lab/runs.sqlite'
FLOOR = 0.001
by_best = '--best' in sys.argv
variants = [v for v in sys.argv[1:] if v != '--best']

info = {}
for f in glob.glob('arena_data/scenarios/real3_noadv/r3na_*.json'):
    d = json.load(open(f, encoding='utf-8'))
    p = d['steps'][0]
    jl = set(p['material_labels'][7:])
    owned = lambda m: m['bound'] + m['roster']
    juice = sum(owned(m) * m['market_price'] for k, m in p['materials'].items() if k in jl)
    tradable = sum(m['tradable'] * m['market_price'] for k, m in p['materials'].items() if k in jl and m['tradable'] > 0)
    info[d['name']] = dict(juice=juice, tradable=tradable, chests=bool(p['chests']), stripped=d['meta']['advanced_removed'] > 0)

c = sqlite3.connect(f'file:{DB}?mode=ro', uri=True)
step = {i: (sc, h) for i, sc, st, h in c.execute("select id, scenario, step, payload_hash from scenario_step where scenario like 'r3na\\_%' escape '\\'")}
hashes = set(h for _, h in step.values())
# best known over every run of the same payload, the real3 copies included
allsteps = {i: h for i, h in c.execute('select id, payload_hash from scenario_step') if h in hashes}
ids = ','.join(map(str, allsteps))
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0)
runs = collections.defaultdict(list)
for v, st, m, fl in c.execute(f"select variant, step_id, metric_ref, flow from run where kind='optimize' and error is null and chained=0 and step_id in ({ids})"):
    h = allsteps[st]
    if st in step: runs[v].append((st, m))
    if 'streak_length=' in v: continue
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)

def gap(st, m):
    h = step[st][1]
    return max(0.0, best[h] - m) / (abs(best[h]) if by_best else flow[h]) * 100 if not by_best or best[h] else 0.0

def jbin(st):
    sc, h = step[st]
    share = info[sc]['juice'] / flow[h]
    return 'bound juice 0' if share <= 0 else 'bound juice <10%' if share < 0.1 else 'bound juice >=10%'

groups = [('all', lambda st: True)]
for j in ('bound juice 0', 'bound juice <10%', 'bound juice >=10%'): groups.append((j, lambda st, j=j: jbin(st) == j))
groups += [('tradable juice', lambda st: info[step[st][0]]['tradable'] > 0)]
groups += [('has chests', lambda st: info[step[st][0]]['chests']), ('adv removed', lambda st: info[step[st][0]]['stripped'])]

per = {}
for v in variants:
    acc = collections.defaultdict(list)
    for st, m in runs[v]:
        acc[st].append(gap(st, m))
    per[v] = {st: sum(L) / len(L) for st, L in acc.items()}
common = set.intersection(*(set(p) for p in per.values()))
# the metric is net gold, so it can be 0: those payloads have no % of best
if by_best: common = {st for st in common if best[step[st][1]] != 0}

print(f"{'group':18s} {'variant':44s} {'n':>4s} {'mean':>7s} {'rms':>7s} {'mlog':>6s} {'hit%':>5s} {'p50':>7s} {'p90':>7s} {'worst':>7s} {'diff':>8s} {'+':>4s} {'-':>4s}")
for name, pred in groups:
    sts = sorted(st for st in common if pred(st))
    if not sts: continue
    for v in variants:
        g = sorted(per[v][st] for st in sts)
        q = lambda p: g[min(len(g) - 1, int(p * len(g)))]
        d = [per[variants[0]][st] - per[v][st] for st in sts]
        print(f"{name:18s} {v[:44]:44s} {len(g):4d} {sum(g) / len(g):7.4f} {math.sqrt(sum(x * x for x in g) / len(g)):7.4f} "
              f"{sum(math.log10(x + FLOOR) for x in g) / len(g):6.2f} {100 * sum(x <= 0.01 for x in g) / len(g):5.1f} {q(.5):7.4f} {q(.9):7.4f} "
              f"{g[-1]:7.3f} {sum(d) / len(d):+8.4f} {sum(x > 0.01 for x in d):4d} {sum(x < -0.01 for x in d):4d}")
    print()
