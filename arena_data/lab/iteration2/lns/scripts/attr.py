# What found lns's new bests (gain to the best so far, % of flow, a run), new bests, picks and each phase's share of the work; with run totals.
# usage (repo root): attr.py <db> <variant> [set] [scenario substring] [snapshot label]
import sys, sqlite3, re, collections
db, var = sys.argv[1:3]
only = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] else None
pat = sys.argv[4] if len(sys.argv) > 4 else ''
lab = sys.argv[5] if len(sys.argv) > 5 else None
PH = ['e.mem', 's.order', 's.sweep', 's.chest', 's.rebuild', 's.insert', 'e.insert', 'e.sweep',
      'adv.ladder', 'otail', 'pair.ladder', 'ptype.ladder', 'mat.ladder', 'ptype.price', 'ptype.empty',
      'piece.price', 'piece.empty', 'mat.price', 'mat.empty', 'order', 'chest']
c = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
q = """select r.id, s.scenario, s.step, r.flow, ss.args, r.snapshot_id from run r join scenario_step s on s.id=r.step_id join session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null and r.variant=?"""
snap = c.execute('select id from snapshot where label=?', (lab,)).fetchone()[0] if lab else None
meta = {}
for rid, sc, st, fl, args, sn in c.execute(q, (var,)):
    s = re.search(r'--set (\S+)', args).group(1)
    if (only and s != only) or pat not in f'{sc}:{st}' or (snap and sn != snap): continue
    meta[rid] = (s, max(fl or 1, 1))
K = ('lns_gain', 'lns_wins', 'lns_cost', 'lns_picks')
by = collections.defaultdict(lambda: collections.defaultdict(lambda: [0.0] * 4))
tot = collections.defaultdict(lambda: collections.defaultdict(float))
nrun = collections.Counter(s for s, _ in meta.values())
ids = list(meta)
for at in range(0, len(ids), 500):
    part = ids[at:at + 500]
    for rid, it, key, val in c.execute(f"select run_id, iter, key, value from trace where key like 'lns_%' and run_id in ({','.join(map(str, part))})"):
        s, fl = meta[rid]
        if key in K:
            by[s][it][K.index(key)] += val / fl * 100 if key == 'lns_gain' else val
        else:
            tot[s][key] += val
for s in sorted(by):
    n = nrun[s]
    t = tot[s]
    work = sum(v[2] for v in by[s].values()) or 1
    print(f'== {s}: {n} runs, spent/budget {t["lns_spent"] / max(t["lns_budget"], 1):.2f}, loop share {t["lns_loop"] / max(t["lns_spent"], 1):.2f}, '
          f'moves {t["lns_moves"] / n:.0f}, stalled {t["lns_stalled"] / n:.2f}')
    for code, (g, w, k, p) in sorted(by[s].items()):
        print(f'   {PH[code]:12s} gain {g / n:9.5f}  wins {w / n:6.2f}  picks {p / n:6.1f}  work {k / work:5.3f}')
