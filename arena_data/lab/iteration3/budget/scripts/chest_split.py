# Of the payloads a variant misses (singles, streak 1), how many open choice chests differently from the best bud_a run:
# the start and polishes can deadlock on chest choice and the plan that uses what the chests give.
# usage (repo root): chest_split.py [variant suffix, default max_iters=0]
import sys, json, sqlite3, collections
V = 'bud_a:' + (sys.argv[1] if len(sys.argv) > 1 else 'max_iters=0')
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
sid = dict(c.execute('select label, id from snapshot'))['bud_a']
step = {i: sc for i, sc in c.execute("select id, scenario from scenario_step where scenario like 'r3\\_%\\_s1' escape '\\'")}
top = {}; mine = {}; flow = {}
for v, st, m, fl, state in c.execute(f"select variant, step_id, metric_ref, flow, state from run where snapshot_id=? and kind='optimize' and error is null and seed=0 and step_id in ({','.join(map(str, step))})", (sid,)):
    if st not in top or m > top[st][0]: top[st] = (m, state)
    flow[st] = max(flow.get(st, 1), fl or 1)
    if v == V: mine[st] = (m, state)
def choice(state):
    return {k: v for k, v in json.loads(state)['chest_states'].items() if len(v) > 1}
def arr(state): return json.loads(state)['state_arr']
out = collections.Counter()
for st, (m, s) in mine.items():
    g = (top[st][0] - m) / flow[st] * 100
    band = '>0.1' if g > 0.1 else '0.01-0.1' if g > 0.01 else 'ok'
    cm, cb = choice(s), choice(top[st][1])
    has = bool(cb)
    diff = has and any(sorted(range(len(cm[k])), key=lambda i: -cm[k][i])[0] != sorted(range(len(cb[k])), key=lambda i: -cb[k][i])[0] for k in cb if k in cm)
    out[(band, 'no choice chests' if not has else 'main option differs' if diff else 'same main option')] += 1
for k in sorted(out): print(k, out[k])
