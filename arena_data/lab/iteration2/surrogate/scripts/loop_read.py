# The loop's trace per variant: rounds, time and gain by kind, and what a winning round moved. Gains in % of flow per run. usage: loop_read.py <variant regex> [scenario substring] [set] [scratch db]
import sys, collections, re, statistics
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
rx = sys.argv[1]
pat = sys.argv[2] if len(sys.argv) > 2 else ''
only = sys.argv[3] if len(sys.argv) > 3 else ''
c, dbs = connect(sys.argv[4] if len(sys.argv) > 4 else None)
d = dbs[-1]
rows = c.execute(f"""select r.id, r.variant, s.scenario, s.step, r.flow, r.cpu_s, ss.args from {d}.run r join {d}.scenario_step s on s.id=r.step_id join {d}.session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null and s.scenario like ?""", (f'%{pat}%',)).fetchall()
rows = [r for r in rows if re.search(rx, r[1]) and (not only or f'--set {only} ' in r[6])]
ids = {r[0]: r for r in rows}
kinds = collections.defaultdict(lambda: collections.defaultdict(lambda: [0, 0.0, 0.0, 0, 0.0]))
wins = collections.defaultdict(list)
for rid, key, val in c.execute(f"select run_id, key, value from {d}.trace_text where key in ('sur_kind','sur_win')"):
    if rid not in ids: continue
    r = ids[rid]; f = val.split()
    t = kinds[r[1]][f[0]]
    if key == 'sur_kind':
        t[0] += int(f[1]); t[1] += float(f[2]); t[2] += float(f[3])
    else:
        best, kicked, polished, moved, held = float(f[1]), float(f[2]), float(f[3]), int(f[4]), int(f[5])
        fl = max(r[4] or 1, 1)
        t[3] += 1; t[4] += (polished - best) / fl * 100
        wins[r[1]].append((f[0], (polished - best) / fl * 100, (best - kicked) / fl * 100, moved, held, int(f[6]), int(f[7]), int(f[8]), r[2], r[3], int(f[9])))
short = lambda v: v.split(':', 1)[-1].replace('sur_stall=', 'S').replace('sur_kinds=', '').replace('rand+chest+empty', 'RCE').replace('lab_on=', '').replace('lab_off=', 'off:')
for v in sorted(kinds):
    n = sum(1 for r in rows if r[1] == v); cpu = sum(r[5] or 0 for r in rows if r[1] == v)
    k = kinds[v]
    ks, ps = sum(t[1] for t in k.values()), sum(t[2] for t in k.values())
    print(f'{short(v)}: {n} runs, cpu {cpu:.0f}s, loop {ks+ps:.0f}s = {100*(ks+ps)/max(cpu,1e-9):.0f}% of cpu, kick+descent {ks:.0f}s, polish {ps:.0f}s')
    for name, t in sorted(k.items()):
        print(f'   {name:6s} rounds/run {t[0]/n:6.1f} kick {t[1]/max(t[0],1)*1000:7.1f}ms polish {t[2]/max(t[0],1)*1000:7.1f}ms wins/run {t[3]/n:5.2f} gain/run {t[4]/n:.4f}')
    w = [x for x in wins[v] if x[1] > 0.001]
    if w:
        print(f'   {len(w)} wins over 0.001: median gain {statistics.median(x[1] for x in w):.3f}, state below best after kick {statistics.median(x[2] for x in w):.3f}, plans moved {statistics.median(x[3] for x in w)}, of those the kick and descent had in place {statistics.median(x[4] for x in w)}, order moved {sum(x[5] for x in w)}, order was in place {sum(x[5] and x[6] for x in w)}, chest moved {sum(x[7] for x in w)}, one plan only {sum(x[3]==1 and not x[5] and not x[7] for x in w)}')
