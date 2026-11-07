# End states part by part. usage: state.py <snapshot label> <scenario substring> <step> <variant> [variant ...] [--db path]
# Prints per run: gap (% of flow), the order (upgrade ids, '|' after the first 12), chests, and each upgrade's plan where variants differ.
import sys, sqlite3, json, collections
args = sys.argv[1:]
db = 'arena_data/lab/runs.sqlite'
if '--db' in args:
    i = args.index('--db'); db = args[i + 1]; del args[i:i + 2]
label, pat, step, vs = args[0], args[1], int(args[2]), args[3:]
c = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
snap = c.execute('select id from snapshot where label=?', (label,)).fetchone()[0]
sid, payload = [r for r in c.execute('select id, payload from scenario_step where scenario like ? and step=?', (pat + '%', step))][0]
up = json.loads(payload)['upgrade_info']
best = c.execute("select max(metric_ref) from run where step_id=? and kind='optimize' and error is null", (sid,)).fetchone()[0]
rows = c.execute("select variant, seed, metric_ref, flow, state, chained from run where step_id=? and snapshot_id=? and kind='optimize' and error is null order by variant, seed", (sid, snap)).fetchall()
desc = [f"{'N' if u['is_normal_honing'] else 'A'}{u['piece_index']}.{u['upgrade_index']}" for u in up]
print('upgrades:', ' '.join(f'{i}={d}' for i, d in enumerate(desc)))
def rle(slots):
    out, prev, n = [], None, 0
    for s in slots:
        k = ','.join(map(str, s)) or '-'
        if k == prev: n += 1
        else:
            if prev is not None: out.append(f'{prev}x{n}')
            prev, n = k, 1
    if prev is not None: out.append(f'{prev}x{n}')
    return ' '.join(out)
got = collections.defaultdict(list)
for v, seed, m, fl, st, ch in rows:
    if v in vs and all(x[0] != seed for x in got[v]): got[v].append((seed, ch, (best - m) / max(fl, 1) * 100, json.loads(st)))
plans = collections.defaultdict(set)
for v in vs:
    for seed, ch, g, st in got[v]:
        for i, s in enumerate(st['state_arr']): plans[i].add(rle(s))
for v in vs:
    print('==', v)
    for seed, ch, g, st in got[v]:
        o = st['special_state']
        print(f'  seed {seed} ch{ch} gap {g:.3f} order {o[:12]} chests {st["chest_states"]}')
        for i, s in enumerate(st['state_arr']):
            if len(plans[i]) > 1: print(f'     {i:2d} {desc[i]:6s} {rle(s)}')
