# Iteration 3 metrics on the real3 sets: per kind (single, inc, seq), streak, split and chain position, each variant's
# mean gap, RMS gap, mean log10(gap + 0.001), quantiles, hit and CPU against solve 48k in the same session.
# Best known per payload hash over every run (any scenario, mode or snapshot) without a streak_length override.
# --size splits singles by upgrade count, --roster weighs every roster equally, --members weighs a single by its family's size (how often it was re-run).
# usage (repo root): curve.py [snapshot] [db] [--roster] [--members] [--size] [--split dev|test] [--kind r3|r3inc|r3seq]
import sys, json, sqlite3, collections, math, glob, os
opts = sys.argv[1:]
args = [a for i, a in enumerate(opts) if not a.startswith('--') and (i == 0 or opts[i - 1] not in ('--split', '--kind'))]
SNAP = args[0] if args else 'bud_a'
DB = args[1] if len(args) > 1 else 'arena_data/lab/runs.sqlite'
ROSTER = '--roster' in opts
MEMBERS = '--members' in opts
SIZE = '--size' in opts
only = lambda k: opts[opts.index(k) + 1] if k in opts else None
FLOOR = 0.001

meta = {}
for f in glob.glob('arena_data/scenarios/real3/*.json'):
    d = json.load(open(f, encoding='utf-8'))
    meta[d['name']] = d['meta']
c = sqlite3.connect(f'file:{DB}?mode=ro', uri=True)
step = {i: (sc, st, h) for i, sc, st, h in c.execute("select id, scenario, step, payload_hash from scenario_step where scenario like 'r3%'")}
snap = dict(c.execute('select label, id from snapshot'))[SNAP]
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0); rows = []
ids = ','.join(map(str, step))
for sid, se, v, st, seed, ch, m, fl, cpu, ev in c.execute(f"""select snapshot_id, session_id, variant, step_id, seed, chained, metric_ref, flow, cpu_s,
        coalesce(json_extract(counters, '$.StatesEvaluated'), 0) from run where kind='optimize' and error is null and step_id in ({ids})"""):
    if 'streak_length=' in v: continue
    h = step[st][2]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if sid == snap: rows.append((se, v, st, seed, ch, m, cpu or 0, ev))

def kind(sc): return sc.split('_')[0]
def size(n): return 'upg 1-3' if n <= 3 else 'upg 4-10' if n <= 10 else 'upg 11-25' if n <= 25 else 'upg 26+'
def pos(sc, st):
    if kind(sc) == 'r3': return size(meta[sc]['upgrades']) if SIZE else 'cold'
    return 'step 0' if st == 0 else ('steps 1-3' if st <= 3 else 'steps 4+')
base = {(se, st, seed, ch): cpu for se, v, st, seed, ch, m, cpu, ev in rows if v == SNAP}
G = collections.defaultdict(list)
for se, v, st, seed, ch, m, cpu, ev in rows:
    sc, k, h = step[st]
    md = meta[sc]
    if only('--split') and md['split'] != only('--split'): continue
    if only('--kind') and kind(sc) != only('--kind'): continue
    gap = max(0.0, best[h] - m) / flow[h] * 100
    b = base.get((se, st, seed, ch))
    for p in (pos(sc, k), 'all') if kind(sc) != 'r3' else (pos(sc, k),):
        G[(kind(sc), md['min_resolution'], p, v)].append((gap, cpu, b, ev, md['roster'], md.get('members', 1)))

def stats(L):
    w = collections.Counter(x[4] for x in L)
    L = sorted(L)
    wt = [(1 / w[x[4]] if ROSTER else 1.0) * (x[5] if MEMBERS else 1) for x in L]
    W = sum(wt)
    cum = [0.0]
    for a in wt: cum.append(cum[-1] + a)
    q = lambda p: L[min(len(L) - 1, next(i for i in range(len(L)) if cum[i + 1] > p * W))][0]
    avg = lambda f: sum(f(x[0]) * a for x, a in zip(L, wt)) / W
    paired = [x for x in L if x[2]]
    return dict(n=len(L), mean=avg(lambda g: g), rms=math.sqrt(avg(lambda g: g * g)), mlog=avg(lambda g: math.log10(g + FLOOR)),
                hit=100 * avg(lambda g: g <= 0.01), p50=q(.5), p90=q(.9), p99=q(.99), worst=L[-1][0],
                cpu=sum(x[1] for x in paired) / max(sum(x[2] for x in paired), 1e-9), ev=sum(x[3] for x in L) / len(L), cps=sum(x[1] for x in L) / len(L))

def label(v):
    s = v[len(SNAP) + 1:] if v != SNAP else '48k'
    s = s.replace('max_iters=', 'm').replace('kick_stall=32,mem_polish=true,kick_types=window+chest+adv+piece+rebuildempty,sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach,sur_screen=true', 'KMRe+Ls')
    return s

print(f"{'kind':6s} {'res':>3s} {'where':10s} {'variant':22s} {'runs':>5s} {'cpu x':>6s} {'s/run':>6s} {'evals':>7s} {'mean':>7s} {'rms':>7s} {'mlog':>6s} {'hit%':>5s} {'p50':>7s} {'p90':>7s} {'p99':>7s} {'worst':>7s}" + ('   (roster weighted)' if ROSTER else '') + ('   (family size weighted)' if MEMBERS else ''))
order = lambda v: (0 if v == SNAP else 1, -float(v.split('max_iters=')[1].split(',')[0]) if 'max_iters=' in v else 1e9, v)
for key in sorted({k[:3] for k in G}, key=lambda k: (k[0], k[1], ['cold', 'all', 'step 0', 'steps 1-3', 'steps 4+', 'upg 1-3', 'upg 4-10', 'upg 11-25', 'upg 26+'].index(k[2]))):
    for v in sorted({k[3] for k in G if k[:3] == key}, key=order):
        r = stats(G[key + (v,)])
        print(f"{key[0]:6s} {key[1]:3d} {key[2]:10s} {label(v)[:22]:22s} {r['n']:5d} {r['cpu']:6.2f} {r['cps']:6.2f} {r['ev']:7.0f} {r['mean']:7.4f} {r['rms']:7.4f} {r['mlog']:6.2f} {r['hit']:5.1f} {r['p50']:7.4f} {r['p90']:7.4f} {r['p99']:7.4f} {r['worst']:7.3f}")
    print()
