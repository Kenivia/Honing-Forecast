# Where a surrogate run's time goes and what finds new bests, per set. Gains in % of flow, summed over runs then per run. usage: attr.py <variant> [scratch db] [set]
import sys, collections, statistics, re, json
sys.path.insert(0, 'arena_data/lab/tools')
from db import connect
var = sys.argv[1]
c, dbs = connect(sys.argv[2] if len(sys.argv) > 2 else None)
only = sys.argv[3] if len(sys.argv) > 3 else None
d = dbs[-1]
rows = c.execute(f"""select r.id, s.scenario, s.step, r.cpu_s, r.flow, r.metric_ref, ss.args, s.payload
 from {d}.run r join {d}.scenario_step s on s.id=r.step_id join {d}.session ss on ss.id=r.session_id
 where r.kind='optimize' and r.error is null and r.variant=?""", (var,)).fetchall()
tr = collections.defaultdict(dict)
for rid, key, val in c.execute(f"select run_id, key, value from {d}.trace where key like 'sur_%' and key != 'sur_best' and run_id in (select id from {d}.run where variant=?)", (var,)):
    tr[rid][key] = val
by = collections.defaultdict(list)
for rid, sc, st, cpu, flow, m, args, p in rows:
    s = re.search(r'--set (\S+)', args).group(1)
    if only and s != only: continue
    t = tr[rid]; t['cpu'] = cpu or 0; t['flow'] = max(flow or 1, 1); t['n'] = len(json.loads(p)['upgrade_info']); t['key'] = f'{sc}:{st}'; t['m'] = m
    by[s].append(t)
kicks = ['first', 'price', 'empty', 'rand', 'piece', 'last']
moves = ['start', 'descent', 'sur', 'blend', 'true', 'flip', 'adv', 'order', 'chest']
for s, ts in sorted(by.items()):
    tot = sum(t.get('sur_all_s', 0) for t in ts) or 1
    print(f'== {s}: {len(ts)} runs, mean cpu {statistics.mean(t["cpu"] for t in ts):.2f}s, rounds med {statistics.median(t.get("sur_rounds",0) for t in ts):.0f}, best round med {statistics.median(t.get("sur_best_round",0) for t in ts):.0f} p90 {sorted(t.get("sur_best_round",0) for t in ts)[int(.9*len(ts))]:.0f}')
    print('   time share: ' + ' '.join(f'{k} {sum(t.get("sur_"+k+"_s",0) for t in ts)/tot:.2f}' for k in ['main', 'descent', 'repair', 'polish', 'true']) + f' | best found at {sum(t.get("sur_best_at",0) for t in ts)/tot:.2f} of the time')
    print('   first descent gap to end %: med ' + f'{statistics.median((t["m"]-t.get("sur_first",t["m"]))/t["flow"]*100 for t in ts):.4f} mean {statistics.mean((t["m"]-t.get("sur_first",t["m"]))/t["flow"]*100 for t in ts):.4f} | after first polish med {statistics.median((t["m"]-t.get("sur_first_polished",t["m"]))/t["flow"]*100 for t in ts):.4f} mean {statistics.mean((t["m"]-t.get("sur_first_polished",t["m"]))/t["flow"]*100 for t in ts):.4f}')
    print('   gain by restart: ' + ' '.join(f'{k} {sum(t.get("sur_k_"+k,0)/t["flow"]*100 for t in ts)/len(ts):.4f}' for k in kicks[1:]))
    print('   gain by move:    ' + ' '.join(f'{k} {sum(t.get("sur_m_"+k,0)/t["flow"]*100 for t in ts)/len(ts):.4f}' for k in moves))
