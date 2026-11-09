# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# shape of best-known plans: per normal upgrade and id, is it unused, a front streak, a back streak, both, all taps, other
import sqlite3, json, collections, re, sys
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
SETS = 'arena_data/sets'
def names(s):
    return [l.split(':')[0].strip() for l in open(f'{SETS}/{s}.txt', encoding='utf-8') if l.strip() and not l.startswith('#')]
def kind(taps, i):
    used = [i in t for t in taps]
    L = len(used); k = sum(used)
    if k == 0: return 'none'
    if k == L: return 'all'
    f = next(j for j, u in enumerate(used) if not u)  # front run length
    b = next(j for j, u in enumerate(reversed(used)) if not u)
    if f + b == k: return 'front' if b == 0 else ('back' if f == 0 else 'front+back')
    return 'other'
for sset in ('real_test', 'quick_dev', 'speed_test'):
    scen = set(names(sset))
    tally = collections.defaultdict(collections.Counter); ups = collections.Counter(); juiced = collections.Counter(); fr = collections.defaultdict(list)
    for st, sc, p in c.execute('select id, scenario, payload from scenario_step'):
        if sc not in scen: continue
        p = json.loads(p); res = p.get('min_resolution', 1)
        r = c.execute("select state from run where step_id=? and kind='optimize' and error is null and chained=0 order by metric_ref desc limit 1", (st,)).fetchone()
        if not r: continue
        s = json.loads(r[0])
        for u, taps in zip(p['upgrade_info'], s['state_arr']):
            if not u['is_normal_honing'] or not taps: continue
            taps = taps[u.get('starting_num_taps', 0) or 0:]
            if not taps: continue
            ids = sorted({i for t in taps for i in t})
            ups[res] += 1; juiced[res] += bool(ids)
            for i in ids:
                k = kind(taps, i); tally[res][k] += 1
                if k == 'front': fr[res].append(sum(i in t for t in taps) / len(taps))
            if not ids: tally[res]['(upgrade with nothing)'] += 1
    for res in sorted(tally):
        T = tally[res]; n = sum(v for k, v in T.items() if not k.startswith('('))
        fs = sorted(fr[res])
        print(f"{sset:11s} res{res:<3d} upgrades {ups[res]:5d}, with any juice/book {100*juiced[res]/ups[res]:3.0f}% | per used id: " + ', '.join(f'{k} {100*v/n:.0f}%' for k, v in T.most_common() if not k.startswith('(')) + (f" | front share of taps median {fs[len(fs)//2]:.2f}" if fs else ''))
