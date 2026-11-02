# Reads cut_cause.ps1's runs: gap to the best known, seed by seed, production cut against the finer one.
import sqlite3, collections
main = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
s = sqlite3.connect('file:arena_data/lab/iteration2/free_tap_cut/scratch.sqlite?mode=ro', uri=True)
q = "select s.scenario, s.step, r.metric_ref, r.flow from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null"
best, flow = collections.defaultdict(lambda: -1e300), {}
for db in (main, s):
    for sc, st, m, fl in db.execute(q):
        best[(sc, st)] = max(best[(sc, st)], m)
        flow[(sc, st)] = max(flow.get((sc, st), 1), fl or 1)
rows = collections.defaultdict(dict)
for v, sc, st, seed, m, sec in s.execute("select r.variant, s.scenario, s.step, r.seed, r.metric_ref, r.cpu_s from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null"):
    rows[(sc, st, v.replace(',special_tol=1e-7', '').replace(':special_tol=1e-7', ''))][(v.endswith('special_tol=1e-7'), seed)] = ((best[(sc, st)] - m) / flow[(sc, st)] * 100, sec)
tot = [0, 0]
for key in sorted(rows):
    r = rows[key]
    seeds = sorted({k[1] for k in r})
    for fine in (False, True):
        g = [r[(fine, x)][0] for x in seeds if (fine, x) in r]
        if not g:
            continue
        cpu = sum(r[(fine, x)][1] for x in seeds if (fine, x) in r)
        print(f'{key[0] + ":" + str(key[1]):34s} {key[2][6:] or "solve":26s} {"1e-7" if fine else "prod"} mean {sum(g) / len(g):.3f} worst {max(g):.3f} hit {sum(x <= 0.01 for x in g)}/{len(g)} cpu {cpu:.0f}  ' + ' '.join(f'{x:.3f}' for x in g))
