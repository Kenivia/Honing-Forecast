# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# solve 48k (cut_a) anytime curve from trace 'best' rows: gap to best known at N evaluations, by set and streak length
import sys, json, collections, statistics, sqlite3, re, pickle
sys.path.insert(0, 'arena_data/lab/tools')
from an import *
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
mres = {i: json.loads(p).get('min_resolution', 1) for i, p in c.execute('select id, payload from scenario_step')}
sid = c.execute("select id from snapshot where label='cut_a'").fetchone()[0]
want = {'real_test', 'quick_dev', 'speed_test', 'large', 'real_income_test'}
sessions = {i: setof.get(i) for i in sess if setof.get(i) in want}
out = []
for se, s in sessions.items():
    for rid, st, seed, ch, mref, fl, ev in c.execute("select id, step_id, seed, chained, metric_ref, flow, json_extract(counters,'$.StatesEvaluated') from run where snapshot_id=? and session_id=? and variant='cut_a' and kind='optimize' and error is null and seed<3", (sid, se)):
        tr = c.execute("select iter, value from trace where run_id=? and key='best' order by iter", (rid,)).fetchall()
        out.append((s, st, seed, ch, mres[st], steps[st][2], best[(st, ch)], flow[(st, ch)], mref, ev, tr))
pickle.dump(out, open('ignored/lab_cache/anytime.pkl', 'wb'))
print(len(out))
