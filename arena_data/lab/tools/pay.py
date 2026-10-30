# Per payload: variant A vs B, mean over common seeds, % of flow; gaps to best known over everything read
# usage: pay.py <A> <B> [k shown each side] [scenario substring] [chained 0|1] [scratch db]
import sys, collections, statistics, json
from db import connect
A, B = sys.argv[1:3]
k = int(sys.argv[3]) if len(sys.argv) > 3 else 8
pat = sys.argv[4] if len(sys.argv) > 4 else ''
ch = int(sys.argv[5]) if len(sys.argv) > 5 else 0
c, dbs = connect(sys.argv[6] if len(sys.argv) > 6 else None)
best = collections.defaultdict(lambda: -1e300); flow = {}
m = collections.defaultdict(lambda: collections.defaultdict(dict)); nup = {}
for d in dbs:
    for v, sc, st, seed, met, fl, p in c.execute(f"select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow, s.payload from {d}.run r join {d}.scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null and r.chained=?", (ch,)):
        key = (sc, st)
        best[key] = max(best[key], met)
        if fl: flow[key] = max(flow.get(key, 1), fl)
        if v in (A, B):
            m[key][v][seed] = met
            if key not in nup: nup[key] = len(json.loads(p)['upgrade_info'])
out = []
for key, d in m.items():
    if A not in d or B not in d or pat not in key[0]: continue
    seeds = [s for s in d[A] if s in d[B]]
    if not seeds: continue
    f = max(flow.get(key, 1), 1)
    a = statistics.mean(d[A][s] for s in seeds); b = statistics.mean(d[B][s] for s in seeds)
    out.append(((a - b) / f * 100, (best[key] - a) / f * 100, (best[key] - b) / f * 100, nup[key], len(seeds), f'{key[0]}:{key[1]}'))
out.sort()
wins = [o for o in out if o[0] > 0.01]; losses = [o for o in out if o[0] < -0.01]
print(f'{A} vs {B}: {len(out)} payloads, {len(wins)} wins, {len(losses)} losses (>0.01%)')
for o in losses[:k] + [None] + wins[-k:]:
    print('  ---' if o is None else f'  {o[0]:+7.3f}  gapA {o[1]:6.3f} gapB {o[2]:6.3f}  n={o[3]:2d} s={o[4]:2d} {o[5]}')
