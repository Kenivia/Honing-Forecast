# Cold real3 singles: per streak (and size), each variant's CPU against the snapshot's 48k in the same session, mean, RMS,
# mlog = mean log10(gap + 0.001), hit, quantiles, and where the time went (the us_* counters, shares of the run's wall).
# Best known per payload hash over runs.sqlite and the given db, runs without a streak_length override.
# usage (repo root): cold.py [snapshot] [db] [--size] [--phases] [--split dev|test] [--filter text] [--chests yes|no]
#   [--session id|last]
import sys, json, sqlite3, collections, math, glob
opts = sys.argv[1:]
VAL = ('--split', '--filter', '--chests', '--session')
args = [a for i, a in enumerate(opts) if not a.startswith('--') and (i == 0 or opts[i - 1] not in VAL)]
SNAP = args[0] if args else 'cold_a'
DB = args[1] if len(args) > 1 else 'arena_data/lab/runs.sqlite'
SIZE, PHASES = '--size' in opts, '--phases' in opts
only = lambda k: opts[opts.index(k) + 1] if k in opts else None
MAIN = 'arena_data/lab/runs.sqlite'

meta, choice = {}, {}
for f in glob.glob('arena_data/scenarios/real3/r3_*.json'):
    d = json.load(open(f, encoding='utf-8'))
    meta[d['name']] = d['meta']
    choice[d['name']] = any(len(c['options']) > 1 and c['count'] > 0 for c in d['steps'][0].get('chests', {}).values())
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0)
KEYS = ['start', 'chains', 'moves', 'polish', 'order', 'resp', 'adv', 'chest', 'flip', 'merge', 'exchange', 'post']
rows = []
for db in dict.fromkeys([MAIN, DB]):
    c = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    step = {i: (sc, h) for i, sc, h in c.execute("select id, scenario, payload_hash from scenario_step where scenario like 'r3%'")}
    snaps = dict(c.execute('select label, id from snapshot'))
    want = only('--session')
    want = c.execute('select max(session_id) from run').fetchone()[0] if want == 'last' else int(want) if want else None
    ext =', '.join(f"coalesce(json_extract(counters, '$.us_{k}'), 0)" for k in KEYS)
    for sid, se, v, st, seed, ch, m, fl, cpu, wall, *us in c.execute(f"""select snapshot_id, session_id, variant, step_id, seed, chained, metric_ref,
            flow, cpu_s, wall_s, {ext} from run where kind='optimize' and error is null and step_id in ({','.join(map(str, step))})"""):
        if 'streak_length=' in v: continue
        sc, h = step[st]
        best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
        if db == DB and sid == snaps.get(SNAP) and not ch and sc in meta and (want is None or se == want):
            rows.append((se, v, sc, h, seed, m, cpu or 0, wall or 0, us))

def size(n): return 'upg 1-3' if n <= 3 else 'upg 4-10' if n <= 10 else 'upg 11-25' if n <= 25 else 'upg 26+'
base = {(se, sc, seed): cpu for se, v, sc, h, seed, m, cpu, wall, us in rows if v == SNAP}
G = collections.defaultdict(list)
for se, v, sc, h, seed, m, cpu, wall, us in rows:
    md = meta[sc]
    if only('--split') and md['split'] != only('--split'): continue
    if only('--filter') and only('--filter') not in sc: continue
    if only('--chests') and choice[sc] != (only('--chests') == 'yes'): continue
    gap = max(0.0, best[h] - m) / flow[h] * 100
    G[(md['min_resolution'], size(md['upgrades']) if SIZE else 'cold', v)].append((gap, cpu, base.get((se, sc, seed)), wall, us))

order = lambda v: (v != SNAP, -int(v.split('max_iters=')[1].split(',')[0]) if 'max_iters=' in v else 0, v)
print(f"{'res':>3s} {'where':10s} {'variant':60s} {'runs':>5s} {'cpu x':>6s} {'s/run':>6s} {'mean':>7s} {'rms':>7s} {'mlog':>6s} {'hit%':>5s} {'p90':>7s} {'p99':>7s} {'worst':>7s}"
      + (' | start moves cpol  post | order  resp   adv chest  flip  exch' if PHASES else ''))
last = None
for k in sorted(G, key=lambda k: (k[0], k[1], order(k[2]))):
    L = G[k]; g = sorted(x[0] for x in L); n = len(L)
    if last and last != k[:2]: print()
    last = k[:2]
    P = [x for x in L if x[2]]
    q = lambda p: g[min(n - 1, int(p * n))]
    lab = k[2][len(SNAP) + 1:] if k[2] != SNAP else '48k'
    cx = sum(x[1] for x in P) / sum(x[2] for x in P) if P else float('nan')
    line = (f"{k[0]:3d} {k[1]:10s} {lab[:60]:60s} {n:5d} {cx:6.2f} {sum(x[1] for x in L) / n:6.2f} {sum(g) / n:7.4f} {math.sqrt(sum(x * x for x in g) / n):7.4f} "
            f"{sum(math.log10(x + 0.001) for x in g) / n:6.2f} {100 * sum(x <= 0.01 for x in g) / n:5.1f} {q(.9):7.4f} {q(.99):7.4f} {g[-1]:7.3f}")
    if PHASES:
        W = sum(x[3] for x in L) * 1e6 or 1
        s = [sum(x[4][i] for x in L) / W * 100 for i in range(len(KEYS))]
        d = dict(zip(KEYS, s))
        cpol = d['chains'] - d['moves']
        line += f" | {d['start']:5.0f} {d['moves']:5.0f} {cpol:5.0f} {d['post']:5.0f} | {d['order']:5.0f} {d['resp']:5.0f} {d['adv']:5.0f} {d['chest']:5.0f} {d['flip']:5.0f} {d['exchange']:5.0f}"
    print(line)
