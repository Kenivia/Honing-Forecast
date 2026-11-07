# Run from the repo root (about a minute). One pass over runs.sqlite: light columns of every optimize run, plus steps and sessions, to a pickle.
import sqlite3, pickle, json, re, time
t = time.time()
c = sqlite3.connect('file:C:/Users/Kenivia/Documents/VSC/Honing-Forecast/arena_data/lab/runs.sqlite?mode=ro', uri=True)
snaps = dict(c.execute('select id, label from snapshot'))
sess = {i: a for i, a in c.execute('select id, args from session')}
steps = {}
for i, sc, st, p in c.execute('select id, scenario, step, payload from scenario_step'):
    steps[i] = (sc, st, len(json.loads(p)['upgrade_info']))
print('steps', len(steps), time.time() - t, flush=True)
runs = c.execute("""select snapshot_id, session_id, variant, step_id, seed, chained, metric_ref, coalesce(json_extract(counters, '$.StatesEvaluated'), 0),
    cpu_s, flow, json_extract(counters, '$.ReferenceUs') from run where kind='optimize' and error is null""").fetchall()
print('runs', len(runs), time.time() - t, flush=True)
pickle.dump({'snaps': snaps, 'sess': sess, 'steps': steps, 'runs': runs},
            open('arena_data/lab/iteration2/cem/scripts/runs.pkl', 'wb'))
print('done', time.time() - t)
