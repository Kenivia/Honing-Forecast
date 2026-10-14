"""Variants against a fixed baseline variant, payload by payload, signed.
See `AI notes/Calculator core/Arena.md`, "Against a fixed baseline".

    python scripts/arena/diff_report.py 50,51 [--base v0] [--by n|kind|week] [--worst 10]
        [--only text] [--like text] [--strip text]

The sessions may be several; the baseline's runs are its cold ones in any of them, and a variant
is compared with it on the seeds they share. `--only` keeps the scenarios whose name has the
text, `--like` the variants, and `--strip` writes `~` for a text that many variants share.
Everything is a share of the flow of the baseline's best seed, in percent.
"""
import sqlite3, sys, json, collections, argparse, statistics

ap = argparse.ArgumentParser()
ap.add_argument('sessions')
ap.add_argument('--base', default='v0')
ap.add_argument('--by', default='')
ap.add_argument('--worst', type=int, default=0)
ap.add_argument('--only', default='')
ap.add_argument('--like', default='')
ap.add_argument('--strip', default='')
ap.add_argument('--eps', type=float, default=1e-4)
args = ap.parse_args()
EPS = args.eps
sessions = [int(x) for x in args.sessions.split(',')]
c = sqlite3.connect('file:test_cases/arena.sqlite?mode=ro', uri=True)
marks = ','.join('?' * len(sessions))
rows = c.execute(f"""select r.variant, r.chained, r.step_id, r.seed, r.metric_ref, r.flow, r.cpu_s, s.scenario, s.step,
    json_extract(r.counters, '$.StatesEvaluated')
    from run r join scenario_step s on s.id = r.step_id
    where r.session_id in ({marks}) and r.kind = 'optimize' and r.error is null and r.metric_ref is not null""", sessions).fetchall()
# variant, step, seed
runs = collections.defaultdict(lambda: collections.defaultdict(dict))
info = {}
for variant, chained, step_id, seed, metric, flow, wall, scenario, step, evals in rows:
    name = variant + (' chained' if chained else '')
    if (args.only and args.only not in scenario) or (args.like and args.like not in name and name != args.base):
        continue
    runs[name][step_id][seed] = (metric, flow, (wall, evals or 0))
    info[step_id] = (scenario, step)
if args.base not in runs:
    sys.exit(f'no runs of {args.base} in these sessions; there are {sorted(runs)}')
base = runs[args.base]
best_known = {step_id: c.execute('select max(metric_ref) from run where step_id = ? and error is null', (step_id,)).fetchone()[0] for step_id in info}
scale = {step_id: max(max(v.values(), key=lambda x: x[0])[1] or 1, 1) for step_id, v in base.items()}
upgrades = {}

def size(step_id):
    if step_id not in upgrades:
        payload = json.loads(c.execute('select payload from scenario_step where id = ?', (step_id,)).fetchone()[0])
        upgrades[step_id] = len(payload['upgrade_info'])
    return upgrades[step_id]

def kind(scenario):
    return '_'.join(scenario.split('_')[:2]) if scenario.startswith('week') else scenario.split('_')[0]

def group(step_id):
    scenario, step = info[step_id]
    if args.by == 'n':
        n = size(step_id)
        return 'n ' + next(label for low, label in ((32, '32+'), (16, '16-31'), (8, '08-15'), (4, '04-07'), (0, '01-03')) if n >= low)
    if args.by == 'kind':
        return kind(scenario)
    if args.by == 'week':
        return kind(scenario) + ' ' + next(label for low, label in ((20, '20+'), (10, '10-19'), (1, '01-09'), (0, '00')) if step >= low)
    return 'all'

def pct(values, q):
    values = sorted(values)
    return values[min(len(values) - 1, int(q * len(values)))]

def line(label, steps, variant):
    mine = runs[variant]
    steps = [s for s in steps if s in mine]
    if not steps:
        return
    diff, gaps, spread, wall, base_wall, evals, base_evals = [], [], [], 0.0, 0.0, 0.0, 0.0
    for s in steps:
        seeds = [seed for seed in mine[s] if seed in base[s]] or list(mine[s])
        ms = [mine[s][seed][0] for seed in seeds]
        bs = [base[s][seed][0] for seed in seeds if seed in base[s]] or [m for m, _, _ in base[s].values()]
        diff.append((statistics.mean(ms) - statistics.mean(bs)) / scale[s] * 100)
        spread.append((max(ms) - min(ms)) / scale[s] * 100)
        gaps += [(best_known[s] - m) / scale[s] * 100 for m in ms]
        wall += statistics.mean(w[0] for _, _, w in mine[s].values())
        base_wall += statistics.mean(w[0] for _, _, w in base[s].values())
        evals += statistics.mean(w[1] for _, _, w in mine[s].values())
        base_evals += statistics.mean(w[1] for _, _, w in base[s].values())
    better = sum(d > EPS * 100 for d in diff)
    worse = sum(d < -EPS * 100 for d in diff)
    hit = lambda eps: 100 * sum(gap <= eps * 100 for gap in gaps) / len(gaps)
    name = variant.replace(args.strip, '~') if args.strip else variant
    print(f"{label:14s} {name[:46]:46s} {len(steps):4d} | {statistics.mean(diff):+8.4f} {pct(diff, 0.1):+8.4f} {min(diff):+8.3f} | {better:3d} {worse:3d} |"
          f" {hit(EPS):5.1f} {hit(10 * EPS):5.1f} {statistics.mean(gaps):7.4f} {pct(gaps, 0.9):7.4f} {max(gaps):6.2f} |"
          f" {statistics.mean(spread):7.4f} {pct(spread, 0.9):7.4f} {100 * sum(x <= EPS * 100 for x in spread) / len(spread):5.1f} | {wall / len(steps):5.2f} {wall / max(base_wall, 1e-9):5.2f} {evals / max(base_evals, 1):5.2f}")

print(f"{'':14s} {'variant':46s} {'n':>4s} | {'diff':>8s} {'p10':>8s} {'worst':>8s} | {'+':>3s} {'-':>3s} |"
      f" {'hit%':>5s} {'x10':>5s} {'gap':>7s} {'p90':>7s} {'worst':>6s} | {'spread':>7s} {'p90':>7s} {'same%':>5s} | {'cpu s':>5s} {'x':>5s} {'evals':>5s}")
groups = collections.defaultdict(list)
for step_id in base:
    groups[group(step_id)].append(step_id)
for label in sorted(groups):
    for variant in sorted(runs):
        line(label, groups[label], variant)
print("diff: mean over seeds less the baseline's, positive is better; +/-: payloads better or worse by more than eps;")
print("hit, x10, gap: against the best any run has reached, within eps and ten times it; spread: best seed less worst;")
print("same: payloads whose seeds all agree; x, evals: cpu time and evaluations over the baseline's")

if args.worst:
    for variant in sorted(runs):
        if variant == args.base:
            continue
        mine = runs[variant]
        mean = lambda of, s: statistics.mean(m for m, _, _ in of[s].values())
        each = sorted(((mean(mine, s) - mean(base, s)) / scale[s] * 100, s) for s in base if s in mine)
        print(variant, 'worst:', ', '.join(f"{info[s][0]}:{info[s][1]} {d:+.3f}" for d, s in each[:args.worst]))
        print(variant, 'best: ', ', '.join(f"{info[s][0]}:{info[s][1]} {d:+.3f}" for d, s in each[-args.worst:]))
