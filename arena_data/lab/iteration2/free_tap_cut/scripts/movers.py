# Each variant against its twin: payloads over 0.03% of flow, seed by seed, gap to the best known; plus a sign test.
import sqlite3, collections, math
con = sqlite3.connect('file:arena_data/lab/runs.sqlite?mode=ro', uri=True)
pairs = [('cut_a', 'lab_r1'), ('cut_a:max_iters=96000', 'lab_r1:max_iters=96000')]
pairs += [(f'cut_a:algo={a},lab_effort=1', f'lab_r1:algo={a},lab_effort=1') for a in ('price', 'surrogate', 'memetic', 'cem', 'lns')]
pairs += [('cut_a:algo=surrogate,lab_effort=1,lab_on=points128', 'sur_b:algo=surrogate,lab_effort=1,lab_on=points128')]
best, flow = collections.defaultdict(lambda: -1e300), {}
for sc, st, m, fl in con.execute("select s.scenario, s.step, r.metric_ref, r.flow from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null"):
    best[(sc, st)] = max(best[(sc, st)], m)
    flow[(sc, st)] = max(flow.get((sc, st), 1), fl or 1)
names = {v for p in pairs for v in p}
runs = collections.defaultdict(dict)
q = ','.join('?' * len(names))
for v, sc, st, seed, ch, m in con.execute(f"select r.variant, s.scenario, s.step, r.seed, r.chained, r.metric_ref from run r join scenario_step s on s.id=r.step_id where r.kind='optimize' and r.error is null and r.variant in ({q})", list(names)):
    runs[v][(sc, st, ch, seed)] = m
for new, old in pairs:
    a, b = runs[new], runs[old]
    steps = collections.defaultdict(list)
    for k in a:
        if k in b:
            steps[k[:3]].append(k)
    gap = lambda r, k: (best[k[:2]] - r[k]) / flow[k[:2]] * 100
    rows = []
    for s, ks in steps.items():
        d = sum(gap(b, k) - gap(a, k) for k in ks) / len(ks)
        rows.append((d, s, ks))
    wins = sum(d > 0.01 for d, *_ in rows)
    losses = sum(d < -0.01 for d, *_ in rows)
    n = wins + losses
    p = sum(math.comb(n, i) for i in range(0, min(wins, losses) + 1)) * 2 / 2 ** n if n else 1
    print(f'\n{new}: {len(rows)} steps, {wins} wins {losses} losses, sign test p={min(p, 1):.2f}, mean {sum(r[0] for r in rows) / len(rows):+.4f}')
    for d, s, ks in sorted(rows, key=lambda r: -abs(r[0]))[:7]:
        if abs(d) < 0.03:
            continue
        ks = sorted(ks, key=lambda k: k[3])
        print(f'  {d:+.3f} {s[0]}:{s[1]}{" ch" if s[2] else ""}  old ' + ' '.join(f'{gap(b, k):.3f}' for k in ks) + '  new ' + ' '.join(f'{gap(a, k):.3f}' for k in ks))
