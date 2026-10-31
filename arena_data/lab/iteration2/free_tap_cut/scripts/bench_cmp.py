# evaluation time per threshold, same states only (given and best, scrambles move with the reach), and the dp against it
import glob, json, os, sys
import numpy as np
S = sys.path[0] + '/out/'
def read(path):
    out = {}
    for line in open(path, encoding='utf-8'):
        a = line.rstrip('\n').split('\t')
        if a[0].endswith((' given', ' best0')):
            out[a[0]] = (float(a[1]), float(a[2]), float(a[3]))
    return out
dp = {}
for path in glob.glob(S + 'd_*.jsonl'):
    for line in open(path, encoding='utf-8'):
        r = json.loads(line)
        if r['label'] == 'best':
            dp[r['name'] + ' best0'] = (r['dp_s'], sum(1 for x in r['p'] if x >= 1e-7))
        elif r['label'] == 'given':
            dp[r['name'] + ' given'] = (r['dp_s'], sum(1 for x in r['p'] if x >= 1e-7))
for s in ['free_taps', 'real_test', 'speed_test']:
    base = read(f'{S}b_{s}_1e-7_0.tsv')
    t0 = sum(v[1] for v in base.values())
    multi = [k for k in base if dp.get(k, (0, 1))[1] > 1]
    print(s, 'states', len(base), 'with free tap outcomes', len(multi), 'total ms %.1f' % (t0 * 1e3))
    for v in ['1e-7_0', '1e-6_0', '1e-5_0', '1e-7_2', '1e-5_2', '1e-4_2', '1e-3_2']:
        b = read(f'{S}b_{s}_{v}.tsv')
        keys = [k for k in base if k in b]
        total = sum(b[k][1] for k in keys) / sum(base[k][1] for k in keys)
        m = [k for k in multi if k in b]
        ratio = sum(b[k][1] for k in m) / max(sum(base[k][1] for k in m), 1e-12)
        med = np.median([b[k][1] / base[k][1] for k in m]) if m else 0
        print(f'  {v:8} total {total:5.3f}  multi-outcome states {ratio:5.3f}  median state {med:5.3f}')
    share = [dp[k][0] / base[k][1] for k in multi if k in dp and base[k][1] > 0]
    if share:
        print('  dp as a share of one memo-off evaluation: median %.4f p90 %.4f max %.4f' % (np.median(share), np.percentile(share, 90), np.max(share)))
