# c_quick_dev crosses a stop: CPU ratios against sur_c on (scenario, seed) pairs no job of which started before it, with evals. usage: qd_split.py
import sys, collections, re
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
log = open('arena_data/lab/iteration2/surrogate/runs/c_run.log', encoding='utf-8', errors='replace').read().splitlines()[6301:7306]
early = {(m.group(1), int(m.group(2))) for m in (re.search(r' (\S+) seed (\d+) cold', x) for x in log) if m}
c, _ = connect()
rows = c.execute("""select r.id, r.variant, s.scenario, s.step, r.seed, r.cpu_s, json_extract(r.counters,'$.StatesEvaluated') from run r join scenario_step s on s.id=r.step_id
 where r.session_id=105 and r.kind='optimize' and r.error is null order by r.id""").fetchall()
print(len(rows), 'runs,', len(early), 'of', len({(r[2], r[4]) for r in rows}), 'scenario+seed pairs touch the time before the stop; seeds there:', sorted({k[1] for k in early}))
def total(pairs):
    t = collections.defaultdict(lambda: [0.0, 0.0])
    for r in rows:
        if pairs((r[2], r[4])):
            t[r[1]][0] += r[5] or 0; t[r[1]][1] += r[6] or 0
    return t
late, every = total(lambda k: k not in early), total(lambda k: True)
b, be = late['sur_c'], every['sur_c']
for v in sorted(late):
    print(f'{v[6:]:58s} after the stop cpu x{late[v][0]/b[0]:.2f} evals x{late[v][1]/b[1]:.2f} | all cpu x{every[v][0]/be[0]:.2f}')
