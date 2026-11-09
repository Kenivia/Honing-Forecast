# The real3 metrics on a generated set (default quick_dev), each step run cold: per streak length and variant, CPU against
# the snapshot's 48k on the same steps and seeds, mean, RMS, mlog, quantiles. Best known per payload hash over every run.
# usage (repo root): gen.py [snapshot] [set]
import sys, json, sqlite3, collections, math
SNAP = sys.argv[1] if len(sys.argv) > 1 else 'bud_a'
SET = sys.argv[2] if len(sys.argv) > 2 else 'quick_dev'
names = [l.strip() for l in open(f'arena_data/sets/{SET}.txt', encoding='utf-8') if l.strip() and not l.startswith('#')]
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
step = {}
for i, sc, h, p in c.execute(f"select id, scenario, payload_hash, payload from scenario_step where scenario in ({','.join('?' * len(names))})", names):
    step[i] = (sc, h, json.loads(p).get('min_resolution', 1))
snap = dict(c.execute('select label, id from snapshot'))[SNAP]
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0); rows = []
for sid, v, st, seed, ch, m, fl, cpu in c.execute(f"select snapshot_id, variant, step_id, seed, chained, metric_ref, flow, cpu_s from run where kind='optimize' and error is null and step_id in ({','.join(map(str, step))})"):
    if 'streak_length=' in v: continue
    h = step[st][1]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if sid == snap and not ch: rows.append((v, st, seed, m, cpu or 0))
base = {(st, seed): cpu for v, st, seed, m, cpu in rows if v == SNAP}
G = collections.defaultdict(list)
for v, st, seed, m, cpu in rows:
    h = step[st][1]
    G[(step[st][2], v)].append((max(0.0, best[h] - m) / flow[h] * 100, cpu, base.get((st, seed))))
print(f"{'res':>3s} {'variant':22s} {'runs':>5s} {'cpu x':>6s} {'mean':>7s} {'rms':>7s} {'mlog':>6s} {'hit%':>5s} {'p50':>7s} {'p90':>7s} {'p99':>7s} {'worst':>7s}")
for res, v in sorted(G, key=lambda k: (k[0], k[1] != SNAP, -int(k[1].split('max_iters=')[1].split(',')[0]) if 'max_iters=' in k[1] else 0)):
    L = G[(res, v)]; g = sorted(x[0] for x in L); n = len(L)
    P = [x for x in L if x[2]]
    q = lambda p: g[min(n - 1, int(p * n))]
    lab = v[len(SNAP) + 1:] if v != SNAP else '48k'
    print(f"{res:3d} {lab[:22]:22s} {n:5d} {sum(x[1] for x in P) / sum(x[2] for x in P):6.2f} {sum(g) / n:7.4f} {math.sqrt(sum(x * x for x in g) / n):7.4f} "
          f"{sum(math.log10(x + 0.001) for x in g) / n:6.2f} {100 * sum(x <= 0.01 for x in g) / n:5.1f} {q(.5):7.4f} {q(.9):7.4f} {q(.99):7.4f} {g[-1]:7.3f}")
