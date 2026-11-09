# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# offline: stop after the first k chain endpoints agree within tol*flow (else run on as production did); chains used and loss
import pickle, statistics, collections
S = 'ignored/lab_cache'
out = pickle.load(open(S + '/anytime.pkl', 'rb'))
def nchains(n):
    return max(48000 // max(1, min(max(int(250 * n), 4000), 12000)), 1)
res = collections.defaultdict(list)
for s, st, seed, ch, mr, n, b, fl, mref, ev, tr in out:
    if ch or not tr: continue
    k = nchains(n); B = 48000 // k
    ends = [max((x for i, x in tr if c * B <= i < (c + 1) * B), default=None) for c in range(k)]
    ends = [e for e in ends if e is not None]
    final = max(x for i, x in tr)
    for K in (2, 3):
        for tol in (1e-6, 1e-5, 1e-4):
            stop = len(ends) if len(ends) < K or max(ends[:K]) - min(ends[:K]) > tol * fl else K
            top = max(ends[:stop]) if stop < len(ends) else final
            res[(s, mr, K, tol)].append((stop, len(ends), max(0, b - top) / fl * 100, max(0, b - final) / fl * 100))
print('stop after the first K chains agree within tol of flow; chains = used/production; gap vs best known (production gap)')
for key in sorted(res):
    L = res[key]
    print(f'{key[0]:11s} res{key[1]:<3d} K{key[2]} tol {key[3]:g}: early {100*sum(x[0]<x[1] for x in L)/len(L):3.0f}% chains x{sum(x[0] for x in L)/sum(x[1] for x in L):.2f} gap {statistics.mean(x[2] for x in L):.4f} ({statistics.mean(x[3] for x in L):.4f}) hit {100*sum(x[2]<=.01 for x in L)/len(L):3.0f}% ({100*sum(x[3]<=.01 for x in L)/len(L):3.0f}%)')
