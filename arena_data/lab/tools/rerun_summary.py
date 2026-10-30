# Compact per set summary of the re-run on its own (snapshot lab_r1): variant x mode -> diff vs lab_r1 (same mode), hit%, worst gap, cpu x
import sys, collections, statistics
from db import connect
c = connect()[0]
SNAP = c.execute("select id from snapshot where label='lab_r1'").fetchone()[0]
SETS = {'real_test': [59], 'real_income_test': [64, 68], 'quick_dev': [65], 'small': [60], 'advanced': [61], 'juice': [62],
        'free_taps': [63], 'large': [66], 'speed_test': [67]}
like = sys.argv[1] if len(sys.argv) > 1 else ''
best = dict(c.execute('select step_id, max(metric_ref) from run where error is null and kind="optimize" and snapshot_id=? group by step_id', (SNAP,)))
def short(v):
    v = v.replace('lab_r1:', '').replace('lab_r1', 'solve 48k')
    v = v.replace('max_iters=', 'solve ').replace('000', 'k').replace('algo=', '').replace(',lab_effort=', ' ')
    return v
for name, sess in SETS.items():
    rows = c.execute(f"select variant, chained, step_id, seed, metric_ref, cpu_s from run where kind='optimize' and error is null and session_id in ({','.join(map(str, sess))})").fetchall()
    runs = collections.defaultdict(lambda: collections.defaultdict(dict))
    for v, ch, st, seed, m, cpu in rows:
        runs[(v, ch)][st][seed] = (m, cpu)
    # scale: base flow
    flow = dict(c.execute(f"select step_id, max(flow) from run where variant='lab_r1' and session_id in ({','.join(map(str, sess))}) group by step_id"))
    for ch in sorted({k[1] for k in runs}):
        base = runs[('lab_r1', ch)]
        bcpu = sum(statistics.mean(x[1] for x in d.values()) for d in base.values())
        print(f'== {name} {"chained" if ch else "cold"}  steps {len(base)}')
        for (v, c2), r in sorted(runs.items(), key=lambda kv: kv[0][0]):
            if c2 != ch or (like and not __import__("re").search(like, v) and v != "lab_r1"):
                continue
            diffs, gaps = [], []
            for st, d in r.items():
                sc = max(flow[st] or 1, 1)
                seeds = [s for s in d if s in base[st]]
                diffs.append((statistics.mean(d[s][0] for s in seeds) - statistics.mean(base[st][s][0] for s in seeds)) / sc * 100)
                gaps += [(best[st] - d[s][0]) / sc * 100 for s in d]
            cpu = sum(statistics.mean(x[1] for x in d.values()) for d in r.values())
            print(f'  {short(v):22s} diff {statistics.mean(diffs):+8.4f} worst {min(diffs):+7.3f} | hit {100*sum(g <= 0.01 for g in gaps)/len(gaps):5.1f} gap {statistics.mean(gaps):7.4f} max {max(gaps):6.3f} | cpu x{cpu/bcpu:5.2f}')
