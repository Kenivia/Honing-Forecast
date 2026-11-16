# The simple strategy (Maxroll's rule) against solve on the cold real3 singles: gap to the best known in % of flow,
# split by streak length, juice owned (gold value at market, chests at their best juice option, as a share of flow),
# chests and advanced honing.
# Best known per payload hash over every run without a streak_length override, as curve.py.
# usage (repo root): simple.py [--list] [variant ...]   (first variant is the base for the paired diff)
import sys, json, sqlite3, collections, math, glob
DB = 'arena_data/lab/runs.sqlite'
FLOOR = 0.001
opts = sys.argv[1:]

info = {}
for f in glob.glob('arena_data/scenarios/real3/r3_*.json'):
    d = json.load(open(f, encoding='utf-8'))
    p = d['steps'][0]
    jl = set(p['material_labels'][7:])
    juice = sum((m['bound'] + m['roster'] + m['tradable']) * m['market_price'] for k, m in p['materials'].items() if k in jl)
    # a chest stack counted at its best juice option
    juice += sum(ch['count'] * max(sum(a * p['materials'][k]['market_price'] for k, a in o.items() if k in jl) for o in ch['options'])
                 for ch in p['chests'].values())
    adv = any(not u['is_normal_honing'] for u in p['upgrade_info'])
    info[d['name']] = dict(res=d['meta']['min_resolution'], juice=juice, adv=adv, split=d['meta']['split'], n=d['meta']['upgrades'],
                           chests=bool(p['chests']))

c = sqlite3.connect(f'file:{DB}?mode=ro', uri=True)
step = {i: (sc, h) for i, sc, st, h in c.execute("select id, scenario, step, payload_hash from scenario_step where scenario like 'r3\\_%' escape '\\'")}
ids = ','.join(map(str, step))
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0)
runs = collections.defaultdict(list)
for v, st, m, fl, cpu in c.execute(f"select variant, step_id, metric_ref, flow, cpu_s from run where kind='optimize' and error is null and chained=0 and step_id in ({ids})"):
    if 'streak_length=' in v: continue
    h = step[st][1]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    runs[v].append((st, m, cpu or 0))

if '--list' in opts:
    for v, L in sorted(runs.items(), key=lambda x: -len(x[1])):
        print(f'{len(L):6d}  {v}')
    sys.exit()

def gap(st, m):
    h = step[st][1]
    return max(0.0, best[h] - m) / flow[h] * 100

def jbin(st):
    sc, h = step[st]
    share = info[sc]['juice'] / flow[h]
    return 'juice 0' if share <= 0 else 'juice <10%' if share < 0.1 else 'juice >=10%'

groups = [('all', lambda st: True)]
for r in (1, 10): groups.append((f'streak {r}', lambda st, r=r: info[step[st][0]]['res'] == r))
for j in ('juice 0', 'juice <10%', 'juice >=10%'): groups.append((j, lambda st, j=j: jbin(st) == j))
groups += [('no chests', lambda st: not info[step[st][0]]['chests']), ('has chests', lambda st: info[step[st][0]]['chests'])]
groups += [('no advanced', lambda st: not info[step[st][0]]['adv']), ('has advanced', lambda st: info[step[st][0]]['adv'])]
groups += [('no adv/chests', lambda st: not info[step[st][0]]['adv'] and not info[step[st][0]]['chests'])]

variants = [v for v in opts if not v.startswith('--')]
# mean over seeds per payload
per = {v: {} for v in variants}
for v in variants:
    acc = collections.defaultdict(list)
    for st, m, cpu in runs[v]:
        acc[st].append((gap(st, m), cpu))
    per[v] = {st: (sum(g for g, _ in L) / len(L), sum(c for _, c in L) / len(L)) for st, L in acc.items()}
common = set.intersection(*(set(p) for p in per.values()))

print(f"{'group':14s} {'variant':48s} {'n':>5s} {'s/run':>7s} {'mean':>7s} {'rms':>7s} {'mlog':>6s} {'hit%':>5s} {'p50':>7s} {'p90':>7s} {'p99':>7s} {'worst':>7s} {'diff':>7s}")
for name, pred in groups:
    sts = sorted(st for st in common if pred(st))
    if not sts: continue
    for v in variants:
        g = sorted(per[v][st][0] for st in sts)
        q = lambda p: g[min(len(g) - 1, int(p * len(g)))]
        diff = sum(per[v][st][0] - per[variants[0]][st][0] for st in sts) / len(sts)
        print(f"{name:14s} {v[:48]:48s} {len(g):5d} {sum(per[v][st][1] for st in sts) / len(sts):7.3f} {sum(g) / len(g):7.4f} "
              f"{math.sqrt(sum(x * x for x in g) / len(g)):7.4f} {sum(math.log10(x + FLOOR) for x in g) / len(g):6.2f} "
              f"{100 * sum(x <= 0.01 for x in g) / len(g):5.1f} {q(.5):7.4f} {q(.9):7.4f} {q(.99):7.4f} {g[-1]:7.3f} {diff:+7.4f}")
    print()
