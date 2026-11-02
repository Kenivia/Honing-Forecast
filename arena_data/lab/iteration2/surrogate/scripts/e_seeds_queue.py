# Writes the queue of batch e_seeds: the payloads where the loop's variants differ, at 10 seeds. usage: e_seeds_queue.py
import json, os
R = 'sur_stall=32,sur_kinds=rand+chest+empty'
T = 'lab_on=randreach+truedescent'
O = 'lab_off=dorder+dadv+dchest'
K = 'sur_stall=32,sur_kinds=lprice+rand+chest+empty'
V = ['', f'{R},lab_on=randreach', f'{R},lab_on=randreach,lab_off=descent', f'{R},lab_on=randreach,lab_off=skew+calib', f'{R},{T},{O}', f'{R},{T}+d1,{O}',
     'sur_stall=32,sur_kinds=price+rand+chest+empty,lab_on=randreach', 'sur_stall=16,sur_kinds=price+rand+chest+empty,lab_on=randreach',
     'sur_stall=32,lab_on=randreach', 'sur_stall=32,sur_kinds=price+rand+empty,lab_on=randreach', 'sur_stall=32,lab_on=randboth',
     f'{K},{T},{O}', f'{K},{T}+d1+skipchest,{O}', f'{K.replace("32", "16")},{T}+d1+skipchest,{O}']
variants = ' '.join('sur_e' + (':' + v if v else '') for v in V)
PAYS = {'quick_dev': ['chain_0017', 'b10_single_3300', 'b10_week_inc_0004', 'single_3116', 'single_0085', 'single_3328'],
        'speed_test': ['chain_1051', 'single_3397', 'single_3375', 'single_3337', 'week_hone_0009', 'single_1010'],
        'large': ['single_0044', 'single_0067', 'single_0161']}
jobs = [{'name': f'e_seeds_{s}_{f}', 'cmd': f'target\\arena\\bin\\sur_e.exe --db arena_data/lab/runs.sqlite --set {s} run {variants} --mode cold --seeds 10 --jobs 16 --resume --filter {f}'}
        for s, fs in PAYS.items() for f in fs]
path = 'arena_data/lab/iteration2/surrogate/runs/e_seeds_queue.json'
assert not os.path.exists(path)
with open(path, 'w', encoding='utf-8', newline='\n') as f:
    f.write(json.dumps(jobs, indent=1) + '\n')
print(path, len(jobs), 'jobs,', len(V), 'variants')
