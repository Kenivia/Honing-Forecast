# where the worst errors of a cut come from, and the real sets alone
import sys, glob
sys.argv = sys.argv[:1]
src = open(sys.path[0] + '/analyse.py', encoding='utf-8').read().replace('\nmain()\n', '\n')
exec(src)
paths = glob.glob(sys.path[0] + '/out/d_*.jsonl')
rows = [r for r in load(paths) if r['flow'] > 1000 and (r['pa'] >= 1e-12).sum() > 1]
for r in rows:
    r['m'] = metric(np.where(r['pa'] >= 1e-12, r['pa'], 0), r['ga'])

print('worst by fold at 1e-4')
e = sorted(rows, key=lambda r: -abs(metric(cut(r['pa'], 1e-4, 'fold'), r['ga']) - r['m']) / r['flow'])
seen = set()
for r in e:
    if r['name'] in seen:
        continue
    seen.add(r['name'])
    err = (metric(cut(r['pa'], 1e-4, 'fold'), r['ga']) - r['m']) / r['flow']
    errc = (metric(cut(r['pa'], 1e-7, 'cur'), r['ga']) - r['m']) / r['flow']
    live = np.nonzero(r['pa'] >= 1e-12)[0]
    print(f"{r['name']:34}{r['label']:7} n{r['n']:3} budget {r['budget']:7} flow {r['flow']:11.0f} metric {r['m']:13.0f} fold1e-4 {err:9.1e} cur1e-7 {errc:9.1e}")
    print('    p', ' '.join(f'{r["pa"][k]:.1e}' for k in live[:12]))
    print('    G', ' '.join(f'{r["ga"][k]:.3g}' for k in live[:12]))
    if len(seen) >= 6:
        break

print()
real = [r for r in rows if not r['name'].startswith(('single_', 'chain_', 'sweep_', 'week', 'stress', 'seed_', 'b10_'))]
print('real and page cases', len(real), 'payloads', len({r['name'] for r in real}))
print('names', sorted({r['name'].split(':')[0] for r in real})[:12])
for label, sub in [('real', real), ('generated', [r for r in rows if r not in real] if len(rows) < 3000 else [r for r in rows if r['name'].startswith(('single_', 'chain_', 'sweep_', 'week', 'stress', 'seed_', 'b10_'))])]:
    live = np.array([(r['pa'] >= 1e-7).sum() for r in sub], float)
    print(label, 'states', len(sub), 'outcomes at 1e-7 mean %.2f median %.0f p90 %.0f max %.0f' % (live.mean(), np.median(live), np.percentile(live, 90), live.max()))
    for policy, tol in [('cur', 1e-7), ('cur', 1e-6), ('cur', 1e-5), ('fold', 1e-5), ('fold', 1e-4), ('fold', 1e-3), ('mean', 1e-4), ('mean', 1e-3)]:
        er, l = [], []
        for r in sub:
            w = cut(r['pa'], tol, policy)
            er.append(abs(metric(w, r['ga']) - r['m']) / r['flow'])
            l.append((w != 0).sum())
        er = np.array(er)
        print(f"  {policy:5}{tol:7.0e} p50 {np.median(er):8.1e} p99 {np.percentile(er, 99):8.1e} p99.9 {np.percentile(er, 99.9):8.1e} max {er.max():8.1e} outcomes {np.mean(l):6.2f} ratio {np.mean(l) / live.mean():5.2f}")

# how the cut compares by how much of the upgrades the free taps cover
print()
print('budget regime: mean outcomes and error by the chance no upgrade is left (p at the top outcome) ')
exp = [(float(np.sum(r['pa'] * np.arange(len(r['pa'])))) / max(r['normal'], 1), r) for r in rows]
for lo, hi in [(0, .1), (.1, .3), (.3, .6), (.6, .9), (.9, 1.01)]:
    sub = [r for f, r in exp if lo <= f < hi]
    if not sub:
        continue
    live = np.array([(r['pa'] >= 1e-7).sum() for r in sub], float)
    l4 = np.array([(cut(r['pa'], 1e-4, 'fold') != 0).sum() for r in sub], float)
    e4 = np.array([abs(metric(cut(r['pa'], 1e-4, 'fold'), r['ga']) - r['m']) / r['flow'] for r in sub])
    e0 = np.array([abs(metric(cut(r['pa'], 1e-7, 'cur'), r['ga']) - r['m']) / r['flow'] for r in sub])
    print(f'  skipped share {lo:.1f}-{hi:.1f}: states {len(sub):6} outcomes {live.mean():6.2f} -> {l4.mean():6.2f} at fold 1e-4; p99 err {np.percentile(e0, 99):8.1e} -> {np.percentile(e4, 99):8.1e}; max {e0.max():8.1e} -> {e4.max():8.1e}')
