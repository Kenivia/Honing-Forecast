"""What the exploratory arena run shows: hit rate, time and when the best was found, by
size, axis, sweep level and chain step kind. See `AI notes/Calculator core/Arena.md`.

    python scripts/arena/explore_report.py [session=20] [folder=test_cases/scenarios/explore]
"""
import sqlite3,collections,json,math,glob,os,sys
SESSION=int(sys.argv[1]) if len(sys.argv)>1 else 20
FOLDER=sys.argv[2] if len(sys.argv)>2 else 'test_cases/scenarios/explore'
c=sqlite3.connect('file:test_cases/arena.sqlite?mode=ro',uri=True)
meta={}
for f in glob.glob(FOLDER+'/*.json'):
    s=json.load(open(f)); meta[os.path.basename(f)[:-5]]=s['meta']
rows=c.execute("""select r.step_id,r.seed,r.metric_ref,s.scenario,s.step,r.wall_s,
 (select max(iter) from trace t where t.run_id=r.id and t.key='best'), r.state, r.flow
 from run r join scenario_step s on s.id=r.step_id where r.session_id=? and r.error is null""",(SESSION,)).fetchall()
P={}
for sid,seed,m,sc,step,w,f,state,flow in rows:
    d=P.setdefault(sid,{'sc':sc,'step':step,'m':{}, 'w':[], 'f':[], 'state':{}})
    d['m'][seed]=m; d.setdefault('flow',{})[seed]=flow or abs(m); d['w'].append(w); d['f'].append(f or 0); d['state'][seed]=state
for sid,d in P.items():
    p=json.loads(c.execute('select payload from scenario_step where id=?',(sid,)).fetchone()[0])
    d['n']=len(p['upgrade_info']); d['tier']=p['tier']; d['up']=[(u['piece_index'],u['upgrade_index'],u['is_normal_honing']) for u in p['upgrade_info']]
    d['nadv']=sum(1 for u in d['up'] if not u[2])
    b=max(d['m'].values()); d['best']=b
    scale=max(d['flow'][max(d['m'],key=d['m'].get)],1)
    g=[(b-v)/scale for v in d['m'].values()]; d['gaps']=g
    d['hit']=sum(x<=1e-4 for x in g)/len(g); d['wall']=sum(d['w'])/len(d['w']); d['kind']=d['sc'].split('_')[0]
    d['found']=sorted(d['f'])[len(d['f'])//2]
    d['bestseed']=max(d['m'],key=d['m'].get)
print('payloads',len(P),'seeds per payload',collections.Counter(len(d['m']) for d in P.values()))
def summ(items,label):
    n=len(items)
    if not n: return
    h=[d['hit'] for d in items]
    print(f"{label:28s} n={n:4d} hit mean {100*sum(h)/n:3.0f}%  all-hit {100*sum(x==1 for x in h)/n:3.0f}%  hit<=25% {100*sum(x<=0.25 for x in h)/n:3.0f}%  wall {sum(d['wall'] for d in items)/n:5.1f}s  found@ {sorted(d['found'] for d in items)[n//2]}")
for k in ('single','sweep','chain'): summ([d for d in P.values() if d['kind']==k],k)
print('--- by size (all)')
for lo,hi in ((1,1),(2,3),(4,7),(8,15),(16,31),(32,99)): summ([d for d in P.values() if lo<=d['n']<=hi],f'n {lo}-{hi}')
print('--- singles by axis tercile')
S=[d for d in P.values() if d['kind']=='single' and d['n']>=4]
for ax in ['tier','level','pieces','jitter','adv','progress','wealth','imbalance','split','juice_owned','juice_price','free_taps','plan','price_noise']:
    out=[]
    for lo,hi in ((0,1/3),(1/3,2/3),(2/3,1.01)):
        it=[d for d in S if lo<=meta[d['sc']]['axes'][ax]<hi]
        out.append('%3.0f%%/%4.1fs'%(100*sum(d['hit'] for d in it)/len(it), sum(d['wall'] for d in it)/len(it)))
    print(f'{ax:12s}',' | '.join(out))
print('--- sweeps: hit% per level, mean of 3 anchors')
sw=collections.defaultdict(list)
for d in P.values():
    if d['kind']=='sweep':
        _,a,*rest=d['sc'].split('_'); 
        parts=d['sc'].split('_'); lvl=parts[-3]; ax='_'.join(parts[2:-3])
        sw[(ax,int(lvl))].append(d)
for ax in sorted(set(k[0] for k in sw)):
    print(f'{ax:12s}',' '.join('%3.0f%%/%4.1fs'%(100*sum(d['hit'] for d in sw[(ax,l)])/len(sw[(ax,l)]), sum(d['wall'] for d in sw[(ax,l)])/len(sw[(ax,l)])) for l in sorted(k[1] for k in sw if k[0]==ax)))
print('--- chains by what happened before the step')
ch=collections.defaultdict(list)
for d in P.values():
    if d['kind']=='chain': ch[meta[d['sc']]['kinds'][d['step']]].append(d)
for k,v in ch.items(): summ(v,k)
# carry-over: how much of the previous step's best plan survives
byc=collections.defaultdict(dict)
for d in P.values():
    if d['kind']=='chain': byc[d['sc']][d['step']]=d
same=collections.defaultdict(list); order=collections.defaultdict(list)
for sc,steps in byc.items():
    for k in sorted(steps):
        if k==0 or k-1 not in steps: continue
        a,b=steps[k-1],steps[k]
        ea=json.loads(a['state'][a['bestseed']]); eb=json.loads(b['state'][b['bestseed']])
        ma=dict(zip(a['up'],[json.dumps(x) for x in ea['state_arr']])); mb=dict(zip(b['up'],[json.dumps(x) for x in eb['state_arr']]))
        shared=[u for u in mb if u in ma]
        if not shared: continue
        kind=meta[sc]['kinds'][k]
        same[kind].append(sum(ma[u]==mb[u] for u in shared)/len(shared))
        # free tap order: relative order of shared upgrades
        oa=[a['up'][i] for i in ea['special_state'] if a['up'][i] in mb]; ob=[b['up'][i] for i in eb['special_state'] if b['up'][i] in ma]
        order[kind].append(oa==ob)
for k in same: print(f'{k:8s} steps {len(same[k]):4d}  upgrades whose best state is unchanged: mean {100*sum(same[k])/len(same[k]):3.0f}%  all unchanged {100*sum(x==1 for x in same[k])/len(same[k]):3.0f}%  free tap order unchanged {100*sum(order[k])/len(order[k]):3.0f}%')
