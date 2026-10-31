# a cut scaled by the mass outside the most likely outcome, offline
import sys, glob
sys.argv = sys.argv[:1]
src = open(sys.path[0] + '/analyse.py', encoding='utf-8').read().replace('\nmain()\n', '\n')
exec(src)
rows = [r for r in load(glob.glob(sys.path[0] + '/out/d_*.jsonl')) if r['flow'] > 1000 and (r['pa'] >= 1e-12).sum() > 1]
for r in rows:
    r['m'] = metric(np.where(r['pa'] >= 1e-12, r['pa'], 0), r['ga'])
    r['real'] = not r['name'].startswith(('single_', 'chain_', 'sweep_', 'week', 'stress', 'seed_', 'b10_'))
    r['share'] = float(np.sum(r['pa'] * np.arange(len(r['pa'])))) / max(r['normal'], 1)

def rel(p, tol):
    return cut(p, max(tol * (1 - p.max()), 1e-11), 'fold')

live0 = {id(r): (r['pa'] >= 1e-7).sum() for r in rows}
def line(name, sub, f):
    e, l = [], []
    for r in sub:
        w = f(r['pa'])
        e.append(abs(metric(w, r['ga']) - r['m']) / r['flow'])
        l.append((w != 0).sum())
    e = np.array(e)
    base = np.mean([live0[id(r)] for r in sub])
    print(f'  {name:18} p50 {np.median(e):8.1e} p99 {np.percentile(e, 99):8.1e} p99.9 {np.percentile(e, 99.9):8.1e} max {e.max():8.1e} outcomes {np.mean(l):6.2f} ratio {np.mean(l) / base:5.2f}')

for label, sub in [('all', rows), ('real', [r for r in rows if r['real']]), ('covered (share>=0.8)', [r for r in rows if r['share'] >= .8])]:
    print(label, len(sub))
    line('cur 1e-7', sub, lambda p: cut(p, 1e-7, 'cur'))
    line('fold 1e-5', sub, lambda p: cut(p, 1e-5, 'fold'))
    line('fold 1e-4', sub, lambda p: cut(p, 1e-4, 'fold'))
    for tol in [1e-4, 3e-4, 1e-3]:
        line(f'rel {tol:.0e}', sub, lambda p: rel(p, tol))

by = {}
for r in rows:
    by.setdefault(r['name'], {})[r['label']] = r
pairs = []
for states in by.values():
    a = states.get('best') or states.get('anchor')
    if a:
        pairs += [(a, r) for l, r in states.items() if l.startswith('ord')]
true = np.array([(b['m'] - a['m']) / a['flow'] for a, b in pairs])
print('order moves', len(pairs))
for name, f in [('cur 1e-7', lambda p: cut(p, 1e-7, 'cur')), ('fold 1e-5', lambda p: cut(p, 1e-5, 'fold')), ('fold 1e-4', lambda p: cut(p, 1e-4, 'fold')), ('rel 1e-4', lambda p: rel(p, 1e-4)), ('rel 3e-4', lambda p: rel(p, 3e-4)), ('rel 1e-3', lambda p: rel(p, 1e-3))]:
    d = np.array([(metric(f(b['pa']), b['ga']) - metric(f(a['pa']), a['ga'])) / a['flow'] for a, b in pairs])
    err = np.abs(d - true)
    flip = (np.sign(d) != np.sign(true)) & (np.abs(true) > 1e-12)
    print(f'  {name:12} p99 {np.percentile(err, 99):8.1e} max {err.max():8.1e} flips>1e-5 {(flip & (np.abs(true) > 1e-5)).sum():4} worst flip {(np.abs(true[flip]).max() if flip.any() else 0):8.1e}')
