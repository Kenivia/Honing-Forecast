# best known state per step from the lab db, read only, as bench --states wants them
import json, sqlite3, sys
out = sys.argv[1]
c = sqlite3.connect('file:C:/Users/Kenivia/Documents/VSC/Honing-Forecast/arena_data/lab/runs.sqlite?mode=ro', uri=True)
best = {}
q = """select s.scenario, s.step, s.id, r.metric_ref, r.id from run r join scenario_step s on s.id = r.step_id
       where r.error is null and r.metric_ref is not null and r.state is not null and r.kind='optimize'"""
for scenario, step, sid, m, rid in c.execute(q):
    k = f'{scenario}:{step}'
    if k not in best or m > best[k][0]:
        best[k] = (m, rid)
res = {}
for k, (m, rid) in best.items():
    res[k] = [json.loads(c.execute('select state from run where id=?', (rid,)).fetchone()[0])]
json.dump(res, open(out, 'w', encoding='utf-8'))
print(len(res))
