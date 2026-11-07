# The light columns of every optimize run in runs.sqlite, plus steps, sessions and snapshots, cached in
# ignored/lab_cache/runs.pkl. load() rebuilds the cache (about 80 s) only when runs.sqlite is newer.
# usage: dump.py rebuilds now; other scripts call load().
import sqlite3, pickle, json, os, time
ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../..'))
DB = os.path.join(ROOT, 'arena_data/lab/runs.sqlite')
CACHE = os.path.join(ROOT, 'ignored/lab_cache/runs.pkl')

def db_time():
    return max(os.path.getmtime(p) for p in (DB, DB + '-wal') if os.path.exists(p))

def build():
    t = time.time()
    c = sqlite3.connect(f'file:{DB}?mode=ro', uri=True)
    snaps = dict(c.execute('select id, label from snapshot'))
    sess = dict(c.execute('select id, args from session'))
    steps = {i: (sc, st, len(json.loads(p)['upgrade_info'])) for i, sc, st, p in c.execute('select id, scenario, step, payload from scenario_step')}
    runs = c.execute("""select snapshot_id, session_id, variant, step_id, seed, chained, metric_ref, coalesce(json_extract(counters, '$.StatesEvaluated'), 0),
        cpu_s, flow, json_extract(counters, '$.ReferenceUs') from run where kind='optimize' and error is null""").fetchall()
    D = {'snaps': snaps, 'sess': sess, 'steps': steps, 'runs': runs}
    os.makedirs(os.path.dirname(CACHE), exist_ok=True)
    pickle.dump(D, open(CACHE, 'wb'))
    print(f'dumped {len(runs)} runs in {time.time() - t:.0f} s', flush=True)
    return D

def load():
    if os.path.exists(CACHE) and os.path.getmtime(CACHE) > db_time():
        return pickle.load(open(CACHE, 'rb'))
    return build()

if __name__ == '__main__':
    build()
