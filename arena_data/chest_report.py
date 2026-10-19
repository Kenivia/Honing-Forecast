"""Variants of a session side by side, with the gap as a share of the gold flowing through the plan.

python arena_data/chest_report.py [session] [--eps 1e-4] [--by stacks|choice|kind]
"""
import json, sqlite3, statistics, sys, collections

args = sys.argv[1:]
eps = float(args[args.index('--eps') + 1]) if '--eps' in args else 1e-4
by = args[args.index('--by') + 1] if '--by' in args else None
conn = sqlite3.connect('file:arena_data/arena.sqlite?mode=ro', uri=True)
numbers = [a for a in args if a.isdigit()]
session = int(numbers[0]) if numbers else conn.execute('select max(id) from session').fetchone()[0]
print('session', session, conn.execute('select args from session where id=?', (session,)).fetchone()[0])

rows = conn.execute(
    'select variant, step_id, seed, metric_ref, flow, cpu_s, wall_s, eval_s, error from run '
    "where session_id=? and kind='optimize'", (session,)).fetchall()
steps = {r[1] for r in rows}
best = {}
for step in steps:
    best[step] = conn.execute('select max(metric_ref) from run where step_id=?', (step,)).fetchone()[0]


def group_of(step):
    name, payload = conn.execute('select scenario, payload from scenario_step where id=?', (step,)).fetchone()
    chests = json.loads(payload)['chests']
    if by == 'kind':
        return name.split('_')[0]
    if by == 'stacks':
        return 'stacks %d' % len(chests)
    if by == 'choice':
        choice = [c for c in chests.values() if len(c['options']) > 1]
        special = any('Special' in label for c in choice for o in c['options'] for label in o)
        return 'no choice' if not choice else 'free tap choice' if special else 'material choice'
    return ''


groups = {step: group_of(step) for step in steps} if by else {step: '' for step in steps}
table = collections.defaultdict(lambda: collections.defaultdict(list))
died = collections.Counter()
for variant, step, seed, metric, flow, cpu, wall, ev, error in rows:
    if error is not None or metric is None:
        died[variant] += 1
        continue
    gap = (best[step] - metric) / max(abs(flow or 0.0), 1.0)
    cell = table[(groups[step], variant)]
    cell['gap'].append(gap)
    cell['cpu'].append(cpu)
    cell['decide'].append((wall - ev) if ev is not None else 0.0)
    cell['wall'].append(wall)

print(f"{'group':16} {'variant':52} {'runs':>5} {'mean gap%':>10} {'median%':>9} {'p90%':>9} {'worst%':>9} {'hit%':>6} {'cpu s':>8} {'decide%':>8} {'died':>5}")
for (group, variant), cell in sorted(table.items()):
    gaps = sorted(cell['gap'])
    p90 = gaps[int(0.9 * (len(gaps) - 1))]
    hit = sum(g <= eps for g in gaps) / len(gaps)
    print(f"{group:16} {variant:52} {len(gaps):5} {100 * statistics.mean(gaps):10.4f} {100 * statistics.median(gaps):9.4f} "
          f"{100 * p90:9.4f} {100 * gaps[-1]:9.3f} {100 * hit:6.1f} {sum(cell['cpu']):8.1f} "
          f"{100 * sum(cell['decide']) / max(sum(cell['wall']), 1e-9):8.1f} {died[variant]:5}")
