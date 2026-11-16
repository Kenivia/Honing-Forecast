# Copies of real3 singles without advanced honing, with nothing owned, no chests and no free taps, into a scratch folder:
# there the metric is gold at market prices, where Maxroll's rule should be the best plan.
# usage (repo root): zeroed.py <out dir> [count]
import sys, json, glob, os
out, count = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 30
os.makedirs(out, exist_ok=True)
n = 0
for f in sorted(glob.glob('arena_data/scenarios/real3/r3_*.json')):
    d = json.load(open(f, encoding='utf-8'))
    p = d['steps'][0]
    if any(not u['is_normal_honing'] for u in p['upgrade_info']) or d['meta']['upgrades'] < 4:
        continue
    for m in p['materials'].values():
        m['bound'] = m['roster'] = m['tradable'] = 0
    p['special_budget'] = 0
    p['chests'] = {}
    d['name'] = 'zero_' + d['name']
    json.dump(d, open(os.path.join(out, d['name'] + '.json'), 'w', encoding='utf-8', newline='\n'))
    n += 1
    if n >= count:
        break
print(n, 'written to', out)
