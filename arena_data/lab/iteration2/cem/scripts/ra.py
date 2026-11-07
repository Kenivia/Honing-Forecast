# A batch's variants against its plain baseline, per set. usage: ra.py <snapshot label> [variant substring]
import sys
sys.path.insert(0, 'arena_data/lab/iteration2/cem/scripts')
from an import *
lab = sys.argv[1]
pat = sys.argv[2] if len(sys.argv) > 2 else ''
for s, ch in sets_of(lab):
    print('==', s, ch)
    for v in sorted(runs[(lab, s, ch)]):
        if pat not in v: continue
        r = cmp(lab, s, ch, lab, v)
        if r: print(f'  {v[len(lab) + 1:] or "solve":52s} {fmt(r)}')
