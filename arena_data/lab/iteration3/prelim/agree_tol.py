# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# per-chain endpoints of solve 48k from traces: when would a flow-relative agreement tolerance have exited, and at what loss
import pickle, statistics, collections
S = 'ignored/lab_cache'
out = pickle.load(open(S + '/anytime.pkl', 'rb'))
def nchains(n):
    ci = int(250 * n)
    return max(48000 // max(1, min(max(ci, 4000), 48000 // 4)), 1)
res = collections.defaultdict(list)
for s, st, seed, ch, mr, n, b, fl, mref, ev, tr in out:
    if ch or not tr: continue
    k = nchains(n); B = 48000 // k
    ends = [max((x for i, x in tr if c * B <= i < (c + 1) * B), default=None) for c in range(k)]
    ends = [e for e in ends if e is not None]
    final = max(x for i, x in tr)
    for tol in (0.0, 1e-5, 1e-4):
        # first chain count at which >=5 agree (or first 3 unanimous) within tol*flow (tol 0: the production rule, approx)
        stop = len(ends)
        for c in range(1, len(ends) + 1):
            top = max(ends[:c]); t = max(abs(top) * 1e-7, 1.0) if tol == 0 else tol * fl
            same = sum(top - e <= t for e in ends[:c])
            if same >= 5 or (same >= 3 and same == c):
                stop = c; break
        top = max(ends[:stop])
        res[(s, mr, tol)].append((stop / len(ends), max(0, final - top) / fl * 100, max(0, b - top) / fl * 100, len(ends)))
for key in sorted(res):
    L = res[key]
    print(f'{key[0]:16s} res{key[1]:<3d} tol {key[2]:g}: runs {len(L):4d} chains used {statistics.mean(x[0] for x in L):.2f} of all; loss vs own run {statistics.mean(x[1] for x in L):.4f} (>0.01: {sum(x[1] > .01 for x in L)}); gap of best chain end (before merge/refine) {statistics.mean(x[2] for x in L):.4f}')
