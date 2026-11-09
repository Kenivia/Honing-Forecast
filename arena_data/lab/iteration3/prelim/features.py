# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# payload features vs how good solve's start (iter 1) and its first 500 / 4000 evaluations are; real vs generated
import pickle, json, sqlite3, collections, statistics
S = 'ignored/lab_cache'
out = pickle.load(open(S + '/anytime.pkl', 'rb'))
c = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
ids = {o[1] for o in out}
pay = {i: json.loads(p) for i, p in c.execute(f"select id, payload from scenario_step where id in ({','.join(map(str, ids))})")}
JUICE = ("Breath", "Armor", "Weapon", "Scroll", "Book", "Solar", "Grace", "Blessing", "Protection")

def feats(p, b, fl):
    m = p['materials']
    owned = lambda d: d.get('bound', 0) + d.get('roster', 0) + d.get('tradable', 0)
    juice_gold = sum(owned(d) * d.get('market_price', 0) for k, d in m.items() if any(j in k for j in JUICE))
    adv = sum(not u['is_normal_honing'] for u in p['upgrade_info'])
    ch = p.get('chests') or {}
    stacks = ch.values() if isinstance(ch, dict) else ch
    choice = sum(len(s.get('options', [])) > 1 for s in stacks)
    return dict(n=len(p['upgrade_info']), adv=adv > 0, tier=p['tier'], res=p.get('min_resolution', 1), sp=p.get('special_budget', 0) > 0,
                choice=choice, juice=juice_gold / fl, cost=-b / fl)

rows = []
for s, st, seed, ch, res, n, b, fl, mref, ev, tr in out:
    if ch or not tr: continue
    tr = sorted(tr)
    g = lambda N: max(0, b - max(x for i, x in tr if i <= N)) / fl * 100
    f = feats(pay[st], b, fl)
    rows.append(dict(f, set=s, st=st, g1=g(1), g500=g(500), g4k=g(4000), gend=g(10**9)))

def show(title, key, bins):
    print(f'\n{title}')
    for s in ('real_test', 'quick_dev', 'speed_test'):
        for lo, hi, lab in bins:
            R = [r for r in rows if r['set'] == s and lo <= key(r) < hi]
            if len(R) < 6: continue
            print(f'  {s:10s} {lab:22s} n{len(R):4d}  start {statistics.mean(r["g1"] for r in R):6.3f} hit {100*sum(r["g1"]<=.01 for r in R)/len(R):3.0f}%  @500 {statistics.mean(r["g500"] for r in R):6.3f} hit {100*sum(r["g500"]<=.01 for r in R)/len(R):3.0f}%  @4k {statistics.mean(r["g4k"] for r in R):6.3f} hit {100*sum(r["g4k"]<=.01 for r in R)/len(R):3.0f}%')

show('net cost / flow (0 = all owned, 1 = all bought)', lambda r: r['cost'], [(-9, .05, '<0.05'), (.05, .3, '0.05-0.3'), (.3, .7, '0.3-0.7'), (.7, .95, '0.7-0.95'), (.95, 9, '>=0.95')])
show('owned juice+books gold / flow', lambda r: r['juice'], [(-1, .001, 'none'), (.001, .05, '<5%'), (.05, .2, '5-20%'), (.2, 99, '>20%')])
show('free taps owned', lambda r: r['sp'], [(0, .5, 'no'), (.5, 2, 'yes')])
show('upgrades', lambda r: r['n'], [(0, 6, '1-5'), (6, 13, '6-12'), (13, 25, '13-24'), (25, 999, '25+')])
show('stacks with a choice', lambda r: r['choice'], [(0, 1, '0'), (1, 4, '1-3'), (4, 999, '4+')])
print('\nreal_test feature spread:')
R = [r for r in rows if r['set'] == 'real_test' and r['g1'] >= 0]
seen = set()
for r in sorted(R, key=lambda r: -r['g1']):
    if r['st'] in seen: continue
    seen.add(r['st'])
    print(f"  n{r['n']:3d} t{r['tier']} res{r['res']:<3d} adv{int(r['adv'])} sp{int(r['sp'])} choice{r['choice']:2d} juice {r['juice']:.3f} cost {r['cost']:.2f} | start {r['g1']:.3f} @500 {r['g500']:.3f} @4k {r['g4k']:.3f} end {r['gend']:.3f}")
