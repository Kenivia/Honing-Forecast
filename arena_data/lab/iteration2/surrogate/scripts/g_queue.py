# Writes the queue of batch g: true order moves in the re-plan, and cheaper polishes after the kick, at 10 seeds on the payloads of batch f. usage: g_queue.py
import json, os
R = 'sur_stall=32,sur_kinds=rand+chest+empty'
V = ['', f'{R},lab_on=randreach', f'{R},lab_on=randreach+truedescent', f'{R},lab_on=randreach+truedescent+noisy',
     f'{R},lab_on=randreach+truedescent+trueorder', f'{R},lab_on=randreach+truedescent+trueorder+noisy',
     f'{R},lab_on=randreach+truedescent+trueorder,lab_off=dadv+dchest',
     f'{R},sur_skip=0.1,lab_on=randreach', f'{R},sur_skip=0.05,lab_on=randreach', f'{R},sur_screen=true,lab_on=randreach',
     f'{R},sur_screen=true,lab_on=randreach+truedescent+trueorder+noisy']
variants = ' '.join('sur_g' + (':' + v if v else '') for v in V)
PAYS = {'large': [''], 'quick_dev': ['chain_0017', 'b10_single_3300', 'b10_week_inc_0004', 'single_3116', 'single_0085', 'single_3328'],
        'speed_test': ['chain_1051', 'single_3397', 'single_3375', 'single_3337', 'week_hone_0009', 'single_1010']}
exe = 'target\\arena\\bin\\sur_g.exe'
jobs = [{'name': f'g_{s}_{f or "all"}', 'cmd': f'{exe} --db arena_data/lab/runs.sqlite --set {s} run {variants} --mode cold --seeds 10 --jobs 16 --resume' + (f' --filter {f}' if f else '')}
        for s, fs in PAYS.items() for f in fs]
path = 'arena_data/lab/iteration2/surrogate/runs/g_queue.json'
assert not os.path.exists(path)
with open(path, 'w', encoding='utf-8', newline='\n') as f:
    f.write(json.dumps(jobs, indent=1) + '\n')
print(path, len(jobs), 'jobs,', len(V), 'variants')
