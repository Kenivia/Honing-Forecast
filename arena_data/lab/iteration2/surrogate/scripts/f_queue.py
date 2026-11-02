# Writes the queue of batch f: model against true responses at each level of the descent, and price noise, at 10 seeds on large and the payloads that move. usage: f_queue.py
import json, os
R = 'sur_stall=32,sur_kinds=rand+chest+empty'
T = 'lab_on=randreach+truedescent'
O = 'lab_off=dorder+dadv+dchest'
V = ['', f'{R},lab_on=randreach', f'{R},{T},{O}', f'{R},lab_on=randreach,{O}', f'{R},{T}', f'{R},lab_on=randreach,lab_off=dplan',
     f'{R},{T}+noisy,{O}', f'{R},{T}+noisy3,{O}', f'{R},{T}+noisy']
variants = ' '.join('sur_f' + (':' + v if v else '') for v in V)
PAYS = {'large': [''], 'quick_dev': ['chain_0017', 'b10_single_3300', 'b10_week_inc_0004', 'single_3116', 'single_0085', 'single_3328'],
        'speed_test': ['chain_1051', 'single_3397', 'single_3375', 'single_3337', 'week_hone_0009', 'single_1010']}
exe = 'target\\arena\\bin\\sur_f.exe'
jobs = [{'name': f'f_{s}_{f or "all"}', 'cmd': f'{exe} --db arena_data/lab/runs.sqlite --set {s} run {variants} --mode cold --seeds 10 --jobs 16 --resume' + (f' --filter {f}' if f else '')}
        for s, fs in PAYS.items() for f in fs]
path = 'arena_data/lab/iteration2/surrogate/runs/f_queue.json'
assert not os.path.exists(path)
with open(path, 'w', encoding='utf-8', newline='\n') as f:
    f.write(json.dumps(jobs, indent=1) + '\n')
print(path, len(jobs), 'jobs,', len(V), 'variants')
