# What found cem's new bests (gain to the best so far, % of flow, a run, in all and after the first polish) and what each phase cost (share of charged work).
# usage (repo root): attr.py <db> <variant> [set] [scenario substring] [snapshot label]
import sys, sqlite3, re, collections
db, var = sys.argv[1:3]
only = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] else None
pat = sys.argv[4] if len(sys.argv) > 4 else ''
lab = sys.argv[5] if len(sys.argv) > 5 else None
PH = ['cand', 'shadow', 'fresh', 'narrow', 'p.resp', 'p.adv', 'p.oprefix', 'p.oinsert', 'p.chest', 'solvepolish', 'exchange']
c = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
q = """select r.id, s.scenario, s.step, r.flow, ss.args, r.snapshot_id from run r join scenario_step s on s.id=r.step_id join session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null and r.variant=?"""
snap = c.execute('select id from snapshot where label=?', (lab,)).fetchone()[0] if lab else None
meta = {}
for rid, sc, st, fl, args, sn in c.execute(q, (var,)):
    s = re.search(r'--set (\S+)', args).group(1)
    if (only and s != only) or pat not in f'{sc}:{st}' or (snap and sn != snap): continue
    meta[rid] = (s, max(fl or 1, 1))
by = collections.defaultdict(lambda: collections.defaultdict(lambda: [0.0, 0.0, 0.0, 0.0]))
tot = collections.defaultdict(lambda: collections.defaultdict(float))
nrun = collections.Counter(s for s, _ in meta.values())
ids = list(meta)
for at in range(0, len(ids), 500):
    part = ids[at:at + 500]
    for rid, it, key, val in c.execute(f"select run_id, iter, key, value from trace where key like 'cem_%' and run_id in ({','.join(map(str, part))})"):
        s, fl = meta[rid]
        K = ('cem_gain', 'cem_wins', 'cem_cost', 'cem_late')
        if key in K:
            by[s][it][K.index(key)] += val / fl * 100 if key in ('cem_gain', 'cem_late') else val
        else:
            tot[s][key] += val
for s in sorted(by):
    n = nrun[s]
    cost = sum(v[2] for v in by[s].values()) or 1
    print(f'== {s}: {n} runs, spent/budget {cost / max(tot[s]["cem_budget"], 1):.2f}, cycles {tot[s]["cem_cycles"] / n:.1f}, polishes {tot[s]["cem_polishes"] / n:.1f} a run')
    for code, (g, w, k, l) in sorted(by[s].items()):
        print(f'   {PH[code]:11s} gain {g / n:9.5f}  after first polish {l / n:9.5f}  wins {w / n:7.2f}  cost {k / cost:5.3f}')
