# real3 cold singles at streak 1 with the advanced honing upgrades taken out: scenarios/real3_noadv, sets real3na_dev/_test
import json
from pathlib import Path

root = Path('arena_data')
out = root / 'scenarios' / 'real3_noadv'
out.mkdir(exist_ok=True)
names = {'dev': [], 'test': []}
for path in sorted((root / 'scenarios' / 'real3').glob('r3_*_s1.json')):
    scenario = json.loads(path.read_text(encoding='utf-8'))
    payload = scenario['steps'][0]
    kept = [u for u in payload['upgrade_info'] if u['is_normal_honing']]
    if not kept:
        continue
    removed = len(payload['upgrade_info']) - len(kept)
    payload['upgrade_info'] = kept
    name = 'r3na_' + scenario['name'][len('r3_'):]
    scenario['name'] = name
    scenario['meta'].update(upgrades=len(kept), advanced_removed=removed)
    (out / f'{name}.json').write_text(json.dumps(scenario), encoding='utf-8', newline='\n')
    names[scenario['meta']['split']].append(name)
for split, rows in names.items():
    head = f'# iteration 4 {split}: real3 cold singles at streak 1, advanced honing removed (iteration4/simple/scripts/strip_adv.py)'
    (root / 'sets' / f'real3na_{split}.txt').write_text('\n'.join([head] + rows) + '\n', encoding='utf-8', newline='\n')
    print(split, len(rows))
