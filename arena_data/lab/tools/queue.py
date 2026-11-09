# Writes a batch's queue for `pnpm lab`: one job per set, at the standard mode and seeds. Every variant must be of one snapshot.
# usage: queue.py iteration2/surrogate/a sur_a:algo=surrogate,lab_effort=1 sur_a:... [--sets a,b] [--filter text] [--seeds n]
import argparse, json, os
SETS = {'real_test': ('cold', 10), 'quick_dev': ('cold', 3), 'speed_test': ('cold', 3), 'advanced': ('cold', 3), 'free_taps': ('cold', 3),
        'juice': ('cold', 3), 'small': ('cold', 3), 'large': ('cold', 3), 'real_income_test': ('chained', 3),
        'speed_test10': ('cold', 3), 'choices_test': ('cold', 3),
        'real3_dev': ('cold', 3), 'real3_test': ('cold', 3), 'real3_income_dev': ('chained', 3), 'real3_income_test': ('chained', 3),
        'real3_seq_dev': ('chained', 3), 'real3_seq_test': ('chained', 3)}
ap = argparse.ArgumentParser()
ap.add_argument('batch')
ap.add_argument('variants', nargs='+')
ap.add_argument('--sets', default=','.join(list(SETS)[:9]))
ap.add_argument('--filter', default='')
ap.add_argument('--seeds', type=int, help='in place of the standard seeds')
a = ap.parse_args()
label = a.variants[0].split(':')[0]
assert all(v.split(':')[0] == label for v in a.variants), 'one snapshot per batch'
name = os.path.basename(a.batch)
path = f'arena_data/lab/{os.path.dirname(a.batch)}/runs/{name}_queue.json'
assert not os.path.exists(path), f'{path} exists'
extra = f' --filter {a.filter}' if a.filter else ''
jobs = [{'name': f'{name}_{s}', 'cmd': f"target\\arena\\bin\\{label}.exe --db arena_data/lab/runs.sqlite --set {s} run {' '.join(a.variants)} --mode {SETS[s][0]} --seeds {a.seeds or SETS[s][1]} --jobs 16 --resume{extra}"}
        for s in a.sets.split(',')]
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, 'w', encoding='utf-8', newline='\n') as f:
    f.write(json.dumps(jobs, indent=1) + '\n')
print(path, len(jobs), 'jobs,', len(a.variants), 'variants')
