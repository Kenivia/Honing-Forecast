# every cut of the free tap outcomes, offline, from the dump: error against the sum over all outcomes
import json, sys, glob, collections
import numpy as np

TOLS = [1e-8, 1e-7, 1e-6, 1e-5, 3e-5, 1e-4, 3e-4, 1e-3, 3e-3, 1e-2]


def load(paths):
    rows, seen = [], set()
    for path in paths:
        for line in open(path, encoding='utf-8'):
            r = json.loads(line)
            key = (r['name'], r['label'])
            if key in seen:
                continue
            seen.add(key)
            r['pa'] = np.array(r['p'], float).clip(0)
            g = np.full(len(r['p']), np.nan)
            d = np.full(len(r['p']), np.nan)
            g[r['k']] = r['g_ref']
            d[r['k']] = r['g_def']
            r['ga'], r['da'] = g, d
            rows.append(r)
    return rows


def cut(p, tol, policy):
    """weights over the outcomes kept: same length as p, zero where not kept"""
    w = np.zeros_like(p)
    if policy == 'cur':
        q = p.copy()
        cs, ci = 0.0, 0.0
        for i in range(len(q)):
            t = q[i]
            if t > tol:
                if cs > 0:
                    q[int(round(ci / cs))] = cs
                break
            ci += t * i
            cs += t
            if cs > tol:
                q[int(round(ci / cs))] = cs
                break
            q[i] = 0
        w = np.where(q >= tol, q, 0.0)
        return w
    if policy.startswith('cum'):
        # as much of each tail as adds to tol
        c = np.cumsum(p)
        lo = int(np.searchsorted(c, tol, side='right'))
        r = np.cumsum(p[::-1])
        hi = len(p) - 1 - int(np.searchsorted(r, tol, side='right'))
        if lo > hi:
            lo = hi = int(np.argmax(p))
    else:
        live = np.nonzero(p >= tol)[0]
        lo, hi = live[0], live[-1]
    w[lo:hi + 1] = p[lo:hi + 1]
    below, above = p[:lo].sum(), p[hi + 1:].sum()
    if policy == 'renorm':
        return w / w.sum()
    w[lo] += below
    w[hi] += above
    if policy in ('mean', 'cummean') and hi > lo:
        lm = sum(p[k] * (lo - k) for k in range(lo))
        hm = sum(p[k] * (k - hi) for k in range(hi + 1, len(p)))
        w[lo] += lm
        w[lo + 1] -= lm
        w[hi] += hm
        w[hi - 1] -= hm
    return w


def metric(w, g):
    idx = np.nonzero(w)[0]
    return float(np.sum(w[idx] * np.nan_to_num(g[idx])))


def main():
    rows = load(sys.argv[1:])
    rows = [r for r in rows if r['flow'] > 1000 and (r['pa'] >= 1e-12).sum() > 1]
    print('states with more than one live outcome:', len(rows), 'payloads', len({r['name'] for r in rows}))
    by = collections.defaultdict(dict)
    for r in rows:
        r['m'] = metric(np.where(r['pa'] >= 1e-12, r['pa'], 0), r['ga'])
        r['mdef'] = metric(np.where(r['pa'] >= 1e-12, r['pa'], 0), r['da'])
        by[r['name']][r['label']] = r
    coarse = np.array([abs(r['mdef'] - r['m']) / r['flow'] for r in rows])
    live0 = np.array([(r['pa'] >= 1e-7).sum() for r in rows], float)
    print('coarse (8 cells) against reference, share of flow: median %.2e p99 %.2e max %.2e' % (np.median(coarse), np.percentile(coarse, 99), coarse.max()))
    print('outcomes at 1e-7: mean %.2f median %.0f max %.0f' % (live0.mean(), np.median(live0), live0.max()))
    print()
    print('ERROR of the metric, |cut - all| / flow, all state kinds')
    print(f"{'policy':8}{'tol':>8}{'median':>10}{'p99':>10}{'max':>10}{'mean signed':>13}{'outcomes':>10}{'ratio':>7}")
    for policy in ['cur', 'renorm', 'fold', 'mean', 'cum', 'cummean']:
        for tol in TOLS:
            e, l = [], []
            for r in rows:
                w = cut(r['pa'], tol, policy)
                e.append((metric(w, r['ga']) - r['m']) / r['flow'])
                l.append((w != 0).sum())
            e, l = np.array(e), np.array(l, float)
            a = np.abs(e)
            print(f"{policy:8}{tol:8.0e}{np.median(a):10.1e}{np.percentile(a, 99):10.1e}{a.max():10.1e}{e.mean():13.1e}{l.mean():10.2f}{l.mean() / live0.mean():7.2f}")
        print()
    print('ORDER MOVES: error of the difference between a state and an order move of it, / flow')
    print(f"{'policy':8}{'tol':>8}{'pairs':>7}{'median':>10}{'p99':>10}{'max':>10}{'flips':>7}{'flip1e-5':>9}{'worstflip':>11}")
    pairs = []
    for name, states in by.items():
        anchor = states.get('best') or states.get('anchor')
        if anchor is None:
            continue
        for label, r in states.items():
            if label.startswith('ord') and r['flow'] > 1000:
                pairs.append((anchor, r))
    true = np.array([(b['m'] - a['m']) / a['flow'] for a, b in pairs])
    print('pairs', len(pairs), 'true delta: median %.2e, zero %d' % (np.median(np.abs(true)), (np.abs(true) < 1e-13).sum()))
    cd = np.array([(b['mdef'] - a['mdef']) / a['flow'] for a, b in pairs])
    a_ = np.abs(cd - true)
    moving = np.abs(true) > 1e-12
    print(f"{'coarse8':8}{'':8}{len(pairs):7}{np.median(a_):10.1e}{np.percentile(a_, 99):10.1e}{a_.max():10.1e}{(np.sign(cd[moving]) != np.sign(true[moving])).sum():7}{((np.sign(cd) != np.sign(true)) & (np.abs(true) > 1e-5)).sum():9}{np.abs(true[(np.sign(cd) != np.sign(true)) & moving]).max():11.1e}")
    for policy in ['cur', 'fold', 'mean']:
        for tol in TOLS:
            d = []
            for a, b in pairs:
                d.append((metric(cut(b['pa'], tol, policy), b['ga']) - metric(cut(a['pa'], tol, policy), a['ga'])) / a['flow'])
            d = np.array(d)
            err = np.abs(d - true)
            flip = (np.sign(d) != np.sign(true)) & moving
            print(f"{policy:8}{tol:8.0e}{len(pairs):7}{np.median(err):10.1e}{np.percentile(err, 99):10.1e}{err.max():10.1e}{flip.sum():7}{(flip & (np.abs(true) > 1e-5)).sum():9}{(np.abs(true[flip]).max() if flip.any() else 0):11.1e}")
        print()


main()
