# Batch d: per-payload table of gaps for each variant, the variants' disagreements, and CPU ratios. usage: d_read.py <cpu|table|diff A B [k]> [sets]
import sys, collections, re, statistics
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
SHORT = {
 'sur_d': 'base',
 'sur_d:sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach': 'RCE',
 'sur_d:sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach+truedescent': 'true',
 'sur_d:sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach,lab_off=descent': 'none',
 'sur_d:sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach,lab_off=skew+calib': 'normal',
 'sur_d:sur_stall=32,sur_kinds=price+rand+chest+empty,lab_on=randreach': 'PRCE',
 'sur_d:sur_stall=32,lab_on=randboth': 'both',
 'sur_d:sur_stall=16,sur_kinds=price+rand+chest+empty,lab_on=randreach': 'P16',
}
NAMES = list(SHORT.values())
SETS = ['real_test', 'quick_dev', 'speed_test', 'large', 'advanced', 'free_taps', 'juice']
c, _ = connect()
best = {}; flow = {}; runs = collections.defaultdict(dict)
for v, sc, st, seed, m, fl, args, sec, ev, rid in c.execute("""select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow, ss.args, r.cpu_s,
  json_extract(r.counters,'$.StatesEvaluated'), r.id from run r join scenario_step s on s.id=r.step_id join session ss on ss.id=r.session_id
  where r.kind='optimize' and r.error is null and r.chained=0"""):
    k = (sc, st); best[k] = max(best.get(k, -1e300), m); flow[k] = max(flow.get(k, 1), fl or 1)
    if v in SHORT:
        s = re.search(r'--set (\S+)', args).group(1)
        runs[s, SHORT[v]][k, seed] = (m, sec or 0, ev or 0, rid)
mode = sys.argv[1]
if mode == 'table':
    sets = sys.argv[2].split(',') if len(sys.argv) > 2 else SETS
    print(f'{"payload":44s}' + ''.join(f'{n:>8s}' for n in NAMES))
    for s in sets:
        pays = collections.defaultdict(lambda: collections.defaultdict(list))
        for n in NAMES:
            for (k, seed), row in runs[s, n].items():
                pays[k][n].append((best[k] - row[0]) / flow[k] * 100)
        rows = []
        for k, d in pays.items():
            means = {n: statistics.mean(d[n]) for n in NAMES}
            if max(means.values()) - min(means.values()) > 0.01:
                rows.append((k, means))
        rows.sort(key=lambda r: -(r[1]['base'] + max(r[1].values())))
        print('--', s, len(rows), 'of', len(pays))
        for k, m in rows:
            print(f'{k[0][:40]+":"+str(k[1]):44s}' + ''.join(f'{m[n]:8.3f}' for n in NAMES))
elif mode == 'diff':
    A, B = sys.argv[2:4]
    k_ = int(sys.argv[4]) if len(sys.argv) > 4 else 15
    sets = sys.argv[5].split(',') if len(sys.argv) > 5 else SETS
    for s in sets:
        a, b = runs[s, A], runs[s, B]
        per = collections.defaultdict(list)
        for key in a:
            if key in b: per[key[0]].append(key)
        out = []
        for k, keys in per.items():
            out.append((statistics.mean(a[x][0] - b[x][0] for x in keys) / flow[k] * 100, k, sum(a[x][0] > b[x][0] + 1e-4 * flow[k] / 100 for x in keys), sum(a[x][0] < b[x][0] - 1e-4 * flow[k] / 100 for x in keys), len(keys)))
        out.sort(key=lambda o: o[0])
        print('--', s, 'mean', f'{statistics.mean(o[0] for o in out):+.4f}', 'wins', sum(o[0] > .01 for o in out), 'losses', sum(o[0] < -.01 for o in out))
        for o in out[:k_]:
            if o[0] < -0.005: print(f'  {o[0]:+.3f} seeds +{o[2]} -{o[3]} of {o[4]} {o[1]}')
        for o in out[-k_:]:
            if o[0] > 0.005: print(f'  {o[0]:+.3f} seeds +{o[2]} -{o[3]} of {o[4]} {o[1]}')
elif mode == 'cpu':
    log = open('arena_data/lab/iteration2/surrogate/runs/d_run.log', encoding='utf-8', errors='replace').read().splitlines()
    idx = [i for i, l in enumerate(log) if l.startswith('== d_quick_dev')]
    early = {(m.group(1), int(m.group(2))) for m in (re.search(r' (\S+) seed (\d+) cold', x) for x in log[idx[0]:idx[1]]) if m}
    print(len(early), 'qd scenario+seed pairs touch the time before the stop;', sorted({k[1] for k in early}))
    for s in SETS:
        keep = (lambda key: (key[0][0], key[1]) not in early) if s == 'quick_dev' else (lambda key: True)
        b = runs[s, 'base']
        line = f'{s:12s}'
        for n in NAMES[1:]:
            r = runs[s, n]
            keys = [x for x in r if x in b and keep(x)]
            line += f' {n} x{sum(r[x][1] for x in keys)/sum(b[x][1] for x in keys):.2f}/{sum(r[x][2] for x in keys)/sum(b[x][2] for x in keys):.2f}'
        print(line, f'[{len(keys)} runs]')
