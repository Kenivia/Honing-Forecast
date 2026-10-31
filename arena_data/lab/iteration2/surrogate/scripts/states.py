# End states of two runs side by side: metric, free tap order, chests, plans (run-length coded). usage: states.py <scenario> <step> <variantA> <seedA> <variantB> <seedB> [scratch db]
import sys, json, itertools
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
sc, st, va, sa, vb, sb = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4]), sys.argv[5], int(sys.argv[6])
c, dbs = connect(sys.argv[7] if len(sys.argv) > 7 else None)
def get(v, seed):
    for d in dbs:
        r = c.execute(f"select r.state, r.metric_ref, r.flow, s.payload, r.cpu_s from {d}.run r join {d}.scenario_step s on s.id=r.step_id where s.scenario=? and s.step=? and r.variant=? and r.seed=? and r.kind='optimize' and r.error is null and r.chained=0", (sc, st, v, seed)).fetchone()
        if r: return r
    raise SystemExit(f'no run {v} {seed}')
A = get(va, sa); B = get(vb, sb)
sta, stb = json.loads(A[0]), json.loads(B[0])
flow = A[2]
print(f'metric A {A[1]:.1f} B {B[1]:.1f} A-B {(A[1]-B[1])/flow*100:+.4f}% of flow {flow:.0f}, cpu {A[4]:.1f} / {B[4]:.1f}')
p = json.loads(A[3])
ui = p['upgrade_info']
print('keys', list(p.keys())[:30])
print('first upgrade', json.dumps(ui[0])[:300])
def rle(plan):
    out = []
    for k, g in itertools.groupby(plan, key=lambda s: tuple(s)):
        out.append(f"{'+'.join(map(str, k)) or '-'}x{len(list(g))}")
    return ' '.join(out)
print('order A', sta['special_state'])
print('order B', stb['special_state'])
print('chests A', sta.get('chest_states'))
print('chests B', stb.get('chest_states'))
for i, (a, b) in enumerate(zip(sta['state_arr'], stb['state_arr'])):
    same = a == b
    print(f'{i:3d} {"same" if same else "DIFF"} {json.dumps(ui[i])[:60]}')
    print(f'     A {rle(a)}')
    if not same: print(f'     B {rle(b)}')
