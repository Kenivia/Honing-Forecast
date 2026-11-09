# What the carried plan buys: family-chain steps 1+ (r3seq, chained) against the cold single of the same payload (r3, batch
# a), same variant and seed. Gap in % of flow against the best known; CPU as raw seconds (both batches ran 16 jobs at once).
# usage (repo root): pair.py [snapshot]
import sys, sqlite3, collections, math
SNAP = sys.argv[1] if len(sys.argv) > 1 else 'bud_a'
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
step = {i: (sc, st, h) for i, sc, st, h in c.execute("select id, scenario, step, payload_hash from scenario_step where scenario like 'r3%'")}
snap = dict(c.execute('select label, id from snapshot'))[SNAP]
best = collections.defaultdict(lambda: -1e300); flow = collections.defaultdict(lambda: 1.0)
cold = {}; warm = collections.defaultdict(list)
for sid, v, st, seed, m, fl, cpu in c.execute(f"select snapshot_id, variant, step_id, seed, metric_ref, flow, cpu_s from run where kind='optimize' and error is null and step_id in ({','.join(map(str, step))})"):
    if 'streak_length=' in v: continue
    sc, k, h = step[st]
    best[h] = max(best[h], m); flow[h] = max(flow[h], fl or 1)
    if sid != snap: continue
    if sc.startswith('r3_'): cold[(v, h, seed)] = (m, cpu)
    elif sc.startswith('r3seq_') and k >= 1: warm[v].append((h, seed, m, cpu, sc.endswith('_s10')))
gap = lambda h, m: max(0.0, best[h] - m) / flow[h] * 100
print(f"{'variant':16s} {'res':>3s} {'pairs':>5s} {'cold s':>7s} {'warm s':>7s} {'cold mean':>9s} {'warm mean':>9s} {'cold mlog':>9s} {'warm mlog':>9s} {'warm better':>11s} {'worse':>6s}")
for res in (False, True):
    for v in sorted(warm, key=lambda v: (v != SNAP, -int(v.split('max_iters=')[1]) if 'max_iters=' in v else 0, v)):
        P = [(gap(h, cold[(v, h, s)][0]), gap(h, m), cold[(v, h, s)][1], cpu) for h, s, m, cpu, x10 in warm[v] if x10 == res and (v, h, s) in cold]
        if not P: continue
        n = len(P)
        ml = lambda i: sum(math.log10(p[i] + 0.001) for p in P) / n
        lab = v[len(SNAP) + 1:] if v != SNAP else '48k'
        print(f"{lab[:16]:16s} {10 if res else 1:3d} {n:5d} {sum(p[2] for p in P) / n:7.2f} {sum(p[3] for p in P) / n:7.2f} {sum(p[0] for p in P) / n:9.4f} {sum(p[1] for p in P) / n:9.4f} {ml(0):9.2f} {ml(1):9.2f}"
              f" {sum(p[1] < p[0] - 1e-6 for p in P):11d} {sum(p[1] > p[0] + 1e-6 for p in P):6d}")
