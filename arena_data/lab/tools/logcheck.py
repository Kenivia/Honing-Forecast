# Is a batch clean? Every queued job done, with a variant summary naming every variant it ran,
# and no run died or failed its Monte Carlo check. Fails if it parses nothing.
# usage (repo root): logcheck.py <batch>, e.g. iteration2/cem/b
import sys, json, os, re
batch = sys.argv[1]
d, name = os.path.split(batch)
stem = f'arena_data/lab/{d}/runs/{name}'
queue = json.load(open(f'{stem}_queue.json'))
done = json.load(open(f'{stem}_done.json')) if os.path.exists(f'{stem}_done.json') else {}
log = open(f'{stem}_run.log', encoding='utf-8').read().splitlines()

# the log's sections, by the "== <job>" line the driver writes
sections, cur = {}, None
for line in log:
    m = re.match(r'^== (\S+)$', line)
    if m:
        cur = m.group(1)
        sections.setdefault(cur, [])
    elif cur:
        sections[cur].append(line)

def variant_rows(lines):
    rows, inside = [], False
    for line in lines:
        if line.startswith('│ variant') and 'mc fails' in line:
            inside = 'header'
        elif inside == 'header' and line.startswith('╞'):
            inside = True
        elif inside is True and line.startswith('│'):
            rows.append(line.strip('│ ').split())
        elif inside is True:
            inside = False
    return rows

bad, parsed = 0, 0
for job in queue:
    n = job['name']
    problems = []
    if n not in done:
        problems.append('not done')
    lines = sections.get(n, [])
    cmd = job['cmd'].split()
    want = set(cmd[cmd.index('run') + 1:next(i for i, t in enumerate(cmd) if t.startswith('--') and i > cmd.index('run'))]) if 'run' in cmd else set()
    rows = variant_rows(lines)
    parsed += len(rows)
    seen = {r[0] for r in rows}
    if want and not rows:
        problems.append('no variant summary')
    elif want - seen:
        problems.append(f'missing from summary: {sorted(want - seen)}')
    for r in rows:
        died, mc = int(r[-2]), int(r[-1])
        if died or mc:
            problems.append(f'{r[0]}: {died} died, {mc} mc fails')
    for line in lines:
        if 'panicked' in line or line.startswith('error'):
            problems.append(line.strip()[:160])
    bad += bool(problems)
    print(f"{'ok ' if not problems else 'BAD'} {n}: {len(rows)} summary rows" + ''.join(f'\n    {p}' for p in problems))
print(f'{len(queue)} jobs, {bad} bad, {parsed} summary rows parsed')
sys.exit(1 if bad or not parsed else 0)
