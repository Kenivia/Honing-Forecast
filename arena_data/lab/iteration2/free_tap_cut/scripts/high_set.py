# singles where the free taps cover nearly everything at the best state
import glob, json, sys
import numpy as np
names = {}
for path in glob.glob(sys.argv[1] + '/d_*.jsonl'):
    for line in open(path, encoding='utf-8'):
        r = json.loads(line)
        if r['label'] not in ('best', 'anchor') or r['flow'] < 1000 or r['normal'] < 2:
            continue
        p = np.array(r['p'], float).clip(0)
        if (p >= 1e-7).sum() < 2:
            continue
        share = float(np.sum(p * np.arange(len(p)))) / r['normal']
        if share >= 0.8 and r['name'].endswith(':0'):
            names[r['name'][:-2]] = share
keep = sorted(names)[:: max(1, len(names) // 40)]
print(len(names), 'payloads, keeping', len(keep))
with open(sys.argv[2], 'w', encoding='utf-8', newline='\n') as f:
    f.write('# free taps cover nearly every upgrade\n' + ''.join(n + '\n' for n in keep))
