# Seed runs a variant won over a base by more than 0.01% of flow: what differs between the two end states. usage: joint.py <base> <variant> [set,set]
import sys, json, re, statistics, collections
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
base, var = sys.argv[1:3]
sets = sys.argv[3].split(',') if len(sys.argv) > 3 else None
c, _ = connect()
q = """select s.scenario, s.step, r.seed, r.metric_ref, r.flow, r.state, ss.args, s.id from run r join scenario_step s on s.id=r.step_id
  join session ss on ss.id=r.session_id where r.kind='optimize' and r.error is null and r.chained=0 and r.variant=?"""
A = {(sc, st, seed): (m, fl, state, re.search(r'--set (\S+)', args).group(1)) for sc, st, seed, m, fl, state, args, _ in c.execute(q, (base,))}
B = {(sc, st, seed): (m, fl, state, re.search(r'--set (\S+)', args).group(1)) for sc, st, seed, m, fl, state, args, _ in c.execute(q, (var,))}
pay = {}
out = []
for k, a in A.items():
    if k not in B or (sets and B[k][3] not in sets): continue
    b = B[k]
    gain = (b[0] - a[0]) / max(a[1], 1) * 100
    if gain <= 0.01: continue
    sa, sb = json.loads(a[2]), json.loads(b[2])
    if k[:2] not in pay:
        pay[k[:2]] = json.loads(c.execute('select payload from scenario_step where scenario=? and step=?', k[:2]).fetchone()[0])
    ui = pay[k[:2]]['upgrade_info']
    plans = [i for i, (x, y) in enumerate(zip(sa['state_arr'], sb['state_arr'])) if x != y]
    normal = [i for i in plans if ui[i]['is_normal_honing']]
    adv = [i for i in plans if not ui[i]['is_normal_honing']]
    types = len({ui[i]['piece_index'] > 0 for i in normal})
    order = sa['special_state'] != sb['special_state']
    first = next((p for p, (x, y) in enumerate(zip(sa['special_state'], sb['special_state'])) if x != y), -1)
    chest = sa.get('chest_states') != sb.get('chest_states')
    out.append((b[3], k, gain, len(ui), len(normal), len(adv), order, first, chest))
out.sort(key=lambda o: (o[0], -o[2]))
print(f'{len(out)} seed wins over {len({o[1][:2] for o in out})} payloads')
print('set scenario:step seed gain n | normal plans moved, adv moved, order moved (first place), chest moved')
for o in out:
    print(f'{o[0]:11s} {o[1][0]}:{o[1][1]} s{o[1][2]} {o[2]:.3f} n={o[3]} | {o[4]} {o[5]} {"O@"+str(o[7]) if o[6] else "-"} {"C" if o[8] else "-"}')
n = len(out)
if n:
    print(f'one normal plan only {sum(o[4]==1 and o[5]==0 and not o[6] and not o[8] for o in out)}, normal plans alone {sum(o[5]==0 and not o[6] and not o[8] for o in out)}, with order {sum(o[6] for o in out)}, with chest {sum(o[8] for o in out)}, with adv {sum(o[5]>0 for o in out)}; median normal moved {statistics.median(o[4] for o in out)}')
