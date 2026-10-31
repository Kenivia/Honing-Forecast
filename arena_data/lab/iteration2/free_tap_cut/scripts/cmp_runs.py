# variants against the baseline on the same scenario, step and seed; all scored by the same fine reference
import sqlite3, sys, json, collections
import numpy as np
db = sys.argv[1]
base = sys.argv[2] if len(sys.argv) > 2 else 'ft_a'
c = sqlite3.connect(db)
sessions = {3: 'free_taps', 4: 'real_test', 5: 'quick_dev'}
if len(sys.argv) > 3:
    sessions = dict(kv.split('=') for kv in sys.argv[3].split(','))
    sessions = {int(k): v for k, v in sessions.items()}
rows = collections.defaultdict(dict)
for sid, name in sessions.items():
    q = """select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.flow, r.cpu_s, r.counters, r.metric
           from run r join scenario_step s on s.id = r.step_id where r.session_id = ? and r.error is null and r.kind='optimize'"""
    for v, sc, st, seed, mr, fl, cpu, cnt, m in c.execute(q, (sid,)):
        rows[name].setdefault(v, {})[(sc, st, seed)] = (mr, fl, cpu, json.loads(cnt) if cnt else {}, m)
print(f"{'set':11}{'variant':38}{'runs':>6}{'mean diff%':>11}{'worst%':>9}{'best%':>9}{'wins':>6}{'losses':>7}{'=':>6}{'cpu':>7}{'evals':>7}")
for name, vs in rows.items():
    b = vs[base]
    for v in sorted(vs, key=lambda v: (v != base, v)):
        keys = [k for k in b if k in vs[v]]
        d = np.array([(vs[v][k][0] - b[k][0]) / max(b[k][1], 1) * 100 for k in keys])
        cpu = sum(vs[v][k][2] for k in keys) / sum(b[k][2] for k in keys)
        ev = [(vs[v][k][3].get('StatesEvaluated'), b[k][3].get('StatesEvaluated')) for k in keys]
        ev = sum(x[0] or 0 for x in ev) / max(sum(x[1] or 0 for x in ev), 1)
        # payload means, so seeds do not count as repeats of the same case
        pay = collections.defaultdict(list)
        for k, x in zip(keys, d):
            pay[k[:2]].append(x)
        pm = np.array([np.mean(x) for x in pay.values()])
        print(f"{name:11}{v[:37]:38}{len(keys):6}{pm.mean():11.5f}{pm.min():9.4f}{pm.max():9.4f}{(pm > .01).sum():6}{(pm < -.01).sum():7}{(np.abs(d) < 1e-9).mean() * 100:5.0f}%{cpu:7.3f}{ev:7.3f}")
    print()
# what the optimizer believed against what the reference says, per variant
print('optimizer metric less reference, / flow, of the returned state')
for name, vs in rows.items():
    for v in sorted(vs):
        e = np.array([(x[4] - x[0]) / max(x[1], 1) for x in vs[v].values()])
        print(f"{name:11}{v[:37]:38} mean {e.mean():10.2e} p99|.| {np.percentile(np.abs(e), 99):9.2e} max|.| {np.abs(e).max():9.2e}")
