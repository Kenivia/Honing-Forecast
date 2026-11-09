# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
import pickle, collections, statistics, bisect
out = pickle.load(open('ignored/lab_cache/anytime.pkl', 'rb'))
NS = [0, 1, 50, 200, 500, 1000, 2000, 4000, 8000, 16000, 10**9]
def gap_at(tr, N, b, fl):
    v = max((x for i, x in tr if i <= N), default=None)
    return None if v is None else max(0.0, (b - v) / fl * 100)
groups = collections.defaultdict(list)
for s, st, seed, ch, res, n, b, fl, mref, ev, tr in out:
    if not tr: continue
    groups[(s, ch, res)].append((st, b, fl, mref, ev, tr, n))
print('gap (% flow) mean / hit<=0.01 at N evaluations; first row is iter 0 (given or empty), 1 the start')
print(f"{'set':22s} {'n':>5s} " + ' '.join(f'{("end" if N > 10**8 else N):>12}' for N in NS))
for key in sorted(groups):
    L = groups[key]
    cells = []
    for N in NS:
        g = [gap_at(tr, N, b, fl) for st, b, fl, mref, ev, tr, n in L]
        g = [x for x in g if x is not None]
        cells.append(f'{statistics.mean(g):6.3f}/{100*sum(x<=0.01 for x in g)/len(g):4.0f}%')
    print(f'{key[0][:12]:12s} ch{key[1]} r{key[2]:<3d} {len(L):5d} ' + ' '.join(f'{c:>12s}' for c in cells))
# evaluations until within 0.01 of the run's own final, as share of the run's evaluations
print('\nevaluations to reach within 0.01 of own final (median, p90), and share of run')
for key in sorted(groups):
    L = groups[key]
    first = []; share = []
    for st, b, fl, mref, ev, tr, n in L:
        fin = max(x for i, x in tr)
        it = next(i for i, x in tr if (fin - x) / fl * 100 <= 0.01)
        first.append(it); share.append(it / max(ev, 1))
    first.sort(); share.sort()
    print(f'{key[0][:12]:12s} ch{key[1]} r{key[2]:<3d} median {first[len(first)//2]:6d} p90 {first[int(.9*(len(first)-1))]:6d}  share median {share[len(share)//2]:.2f} p90 {share[int(.9*(len(share)-1))]:.2f}; evals median {sorted(x[4] for x in L)[len(L)//2]}')
