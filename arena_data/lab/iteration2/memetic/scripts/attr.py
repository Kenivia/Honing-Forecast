# What found memetic's new bests (gain to the best so far, % of flow) and what each phase and move cost (share of charged work).
# usage (repo root): attr.py <db> <variant> [set] [scenario substring]
import sys, sqlite3, re, collections
db, var = sys.argv[1:3]
only = sys.argv[3] if len(sys.argv) > 3 else None
pat = sys.argv[4] if len(sys.argv) > 4 else ''
PH = ['seed', 'pseed', 'anchor', 'porder', 'child', 'restart', 'rseed', 'final']
MV = ['-', 'sweep', 'adv', 'owin', 'owin+r', 'tog', 'tog+r', 'run', 'run+r', 'chest', 'pairs', 'i.resp', 'i.adv', 'i.order', 'i.chest', 'r.pass']
c = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
q = """select r.id, s.scenario, s.step, r.flow, ss.args from run r join scenario_step s on s.id=r.step_id join session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null and r.variant=?"""
meta = {}
for rid, sc, st, fl, args in c.execute(q, (var,)):
    s = re.search(r'--set (\S+)', args).group(1)
    if (only and s != only) or pat not in f'{sc}:{st}': continue
    meta[rid] = (s, max(fl or 1, 1))
by = collections.defaultdict(lambda: collections.defaultdict(lambda: [0.0, 0.0, 0.0]))
tot = collections.defaultdict(lambda: collections.defaultdict(float))
nrun = collections.Counter(s for s, _ in meta.values())
ids = list(meta)
for at in range(0, len(ids), 500):
    part = ids[at:at + 500]
    for rid, it, key, val in c.execute(f"select run_id, iter, key, value from trace where key like 'mem_%' and run_id in ({','.join(map(str, part))})"):
        s, fl = meta[rid]
        if key in ('mem_gain', 'mem_wins', 'mem_cost'):
            cell = by[s][it]
            cell[('mem_gain', 'mem_wins', 'mem_cost').index(key)] += val / fl * 100 if key == 'mem_gain' else val
        else:
            tot[s][key] += val
for s in sorted(by):
    n = nrun[s]
    cost = sum(v[2] for v in by[s].values()) or 1
    print(f'== {s}: {n} runs, spent/budget {cost / max(tot[s]["mem_budget"], 1):.2f}, children {tot[s]["mem_children"] / n:.0f}, restarts {tot[s]["mem_restarts"] / n:.1f} a run')
    rows = sorted(by[s].items(), key=lambda kv: -kv[1][0])
    for code, (g, w, k) in rows:
        if g == 0 and k / cost < 0.01: continue
        print(f'   {PH[code // 16]:8s} {MV[code % 16]:8s} gain {g / n:9.5f}  wins {w / n:6.2f}  cost {k / cost:5.3f}')
