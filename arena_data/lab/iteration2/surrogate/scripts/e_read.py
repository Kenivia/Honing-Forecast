# Batch e by session: each variant against plain sur_e of the same sessions; and payload by payload, A against B. usage: e_read.py sum <session ids a-b or a,b> [chained] | pay <ids> <A> <B> [chained]
import sys, collections, statistics
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
R = 'sur_stall=32,sur_kinds=rand+chest+empty'
T = 'lab_on=randreach+truedescent'
O = 'lab_off=dorder+dadv+dchest'
K = 'sur_stall=32,sur_kinds=lprice+rand+chest+empty'
NAMES = {'': 'base', f'{R},lab_on=randreach': 'L', f'{R},lab_on=randreach,lab_off=descent': 'none', f'{R},lab_on=randreach,lab_off=dplan': 'noplan',
         f'{R},lab_on=randreach,{O}': 'surplans', f'{R},{T},{O}': 'T', f'{R},{T}+checked,{O}': 'Tcheck', f'{R},{T}+d1,{O}': 'T1',
         f'{R},{T}+randorder,{O}': 'Torder', f'{R},{T}+skipchest,{O}': 'Tskip', f'{K},{T},{O}': 'K', 'sur_stall=32,sur_kinds=price+rand+chest+empty,lab_on=randreach': 'P',
         f'{K},{T}+d1+skipchest,{O}': 'C32', f'{K.replace("32", "16")},{T}+d1+skipchest,{O}': 'C16', f'{R},lab_on=randreach,lab_off=skew+calib': 'normal',
         'sur_stall=16,sur_kinds=price+rand+chest+empty,lab_on=randreach': 'P16', 'sur_stall=32,lab_on=randreach': 'PRC', 'sur_stall=32,sur_kinds=price+rand+empty,lab_on=randreach': 'PRE',
         'sur_stall=32,lab_on=randboth': 'both'}
short = lambda v: NAMES.get(v.split(':', 1)[1] if ':' in v else '', v)
mode, ids = sys.argv[1], sys.argv[2]
ids = list(range(int(ids.split('-')[0]), int(ids.split('-')[1]) + 1)) if '-' in ids else [int(x) for x in ids.split(',')]
rest = sys.argv[3:]
ch = int(rest[-1]) if rest and rest[-1] in '01' else 0
c, _ = connect()
best = {}; flow = {}
for sc, st, m, fl in c.execute('select s.scenario, s.step, r.metric_ref, r.flow from run r join scenario_step s on s.id=r.step_id where r.kind=\'optimize\' and r.error is null and r.chained=?', (ch,)):
    k = (sc, st); best[k] = max(best.get(k, -1e300), m); flow[k] = max(flow.get(k, 1), fl or 1)
runs = collections.defaultdict(dict)
q = f"select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.cpu_s, json_extract(r.counters,'$.StatesEvaluated') from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null and r.session_id in ({','.join(map(str, ids))})"
for v, sc, st, seed, m, sec, ev in c.execute(q):
    runs[short(v)][(sc, st), seed] = (m, sec or 0, ev or 0)
def per(a, b):
    p = collections.defaultdict(list)
    for k in a:
        if k in b: p[k[0]].append(k)
    return p
if mode == 'sum':
    b = runs['base']
    order = [n for n in NAMES.values() if n in runs]
    for n in order:
        r = runs[n]; p = per(r, b)
        keys = [k for ks in p.values() for k in ks]
        d = [statistics.mean(r[k][0] - b[k][0] for k in ks) / flow[s] * 100 for s, ks in p.items()]
        gaps = [(best[k[0]] - r[k][0]) / flow[k[0]] * 100 for k in keys]
        print(f'  {n:9s} n{len(p):4d} diff {statistics.mean(d):+.4f} worst {min(d):+.3f} +{sum(x > .01 for x in d):2d} -{sum(x < -.01 for x in d):2d} hit {100*sum(g <= .01 for g in gaps)/len(gaps):5.1f} gap {statistics.mean(gaps):.4f} evals x{sum(r[k][2] for k in keys)/sum(b[k][2] for k in keys):.2f} cpu x{sum(r[k][1] for k in keys)/sum(b[k][1] for k in keys):.2f}')
else:
    A, B = rest[0], rest[1]
    a, b = runs[A], runs[B]; p = per(a, b)
    out = sorted((statistics.mean(a[k][0] - b[k][0] for k in ks) / flow[s] * 100, s, len(ks), statistics.mean((best[s] - a[k][0]) / flow[s] * 100 for k in ks), statistics.mean((best[s] - b[k][0]) / flow[s] * 100 for k in ks),
                  sum(a[k][0] > b[k][0] + 1e-4 * flow[s] / 100 for k in ks), sum(a[k][0] < b[k][0] - 1e-4 * flow[s] / 100 for k in ks)) for s, ks in p.items())
    print(f'{A} vs {B}: {len(out)} payloads, mean {statistics.mean(o[0] for o in out):+.4f}')
    for o in out:
        if abs(o[0]) > 0.005: print(f'  {o[0]:+.3f} gapA {o[3]:.3f} gapB {o[4]:.3f} seeds +{o[5]} -{o[6]} of {o[2]} {o[1][0]}:{o[1][1]}')
