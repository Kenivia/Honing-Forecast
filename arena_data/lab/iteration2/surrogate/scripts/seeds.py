# Seed by seed gaps on payloads, for variants matching a regex. usage: seeds.py <variant regex> <set> <scenario substring> [scenario substring...] (a name scenario:step takes one step)
import sys, collections, re
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
rx, st_ = sys.argv[1], sys.argv[2]
pats = sys.argv[3:]
c, _ = connect()
best = {}; flow = {}; runs = collections.defaultdict(lambda: collections.defaultdict(dict))
for v, sc, st, seed, m, fl, args in c.execute("""select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow, ss.args from run r join scenario_step s on s.id=r.step_id
  join session ss on ss.id=r.session_id where r.kind='optimize' and r.error is null and r.chained=0"""):
    k = (sc, st); best[k] = max(best.get(k, -1e300), m); flow[k] = max(flow.get(k, 1), fl or 1)
    if re.search(rx, v) and re.search(r'--set (\S+)', args).group(1) == st_:
        runs[k][v][seed] = m
short = lambda v: v.replace('sur_stall=', 'S').replace('sur_kinds=', '').replace('rand+chest+empty', 'RCE').replace('lab_on=', '').replace('lab_off=', 'off:')
for k in sorted(runs):
    if not any(p == f'{k[0]}:{k[1]}' or (':' not in p and p in k[0]) for p in pats): continue
    print(f'{k[0]}:{k[1]}')
    for v in sorted(runs[k]):
        d = runs[k][v]
        print(f'  {short(v):62s} ' + ' '.join(f'{(best[k]-d[s])/flow[k]*100:6.3f}' for s in sorted(d)))
