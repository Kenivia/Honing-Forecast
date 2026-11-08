# The figures of the iteration 2 report, as SVGs in ../figs/ (light and dark, hover titles on every mark).
# usage (repo root): figs.py [group]   group: the final batch's snapshot (default fin_a). NOREBUILD=1 reads the cache as it is.
import sys, os, pickle, math, statistics, collections
sys.path.insert(0, 'arena_data/lab/tools')
import dump
if os.environ.get('NOREBUILD'):
    dump.load = lambda: pickle.load(open(dump.CACHE, 'rb'))
import an

GROUP = sys.argv[1] if len(sys.argv) > 1 else 'fin_a'
OUT = 'arena_data/lab/iteration2/report/figs'
K, KT = 'kick_stall=32,mem_polish=true', 'kick_types=window+chest+adv+piece'
L, LS = 'sur_stall=32,sur_kinds=rand+chest+empty,lab_on=randreach', 'sur_stall=32,sur_kinds=rand+chest+empty,sur_screen=true,lab_on=randreach'
# short name, variant suffix, role (cand: the recommendation, ref: solve, other)
FINAL = [('solve 48k', '', 'ref'), ('solve 96k', 'max_iters=96000', 'ref'), ('KM', K, 'other'), ('KMR', f'{K},{KT}+rebuild', 'other'),
         ('KMR empty', f'{K},{KT}+rebuildempty', 'cand'), ('KMR price', f'{K},{KT}+rebuildprice', 'other'), ('L', L, 'other'),
         ('Ls', LS, 'other'), ('KMR+Ls', f'{K},{KT}+rebuild,{LS}', 'cand'),
         ('memetic FS', 'algo=memetic,lab_effort=1,lab_on=fromsolve,lab_off=oreplan', 'cand'),
         ('cem FS', 'algo=cem,lab_effort=0.5,lab_on=fullreach+fromsolve+ownstart', 'other')]
SETS = [('real_test', 0, 'real_test'), ('quick_dev', 0, 'quick_dev'), ('speed_test', 0, 'speed_test'), ('large', 0, 'large (10 seeds)'),
        ('real_income_test', 1, 'income (chained)'), ('advanced', 0, 'advanced'), ('free_taps', 0, 'free_taps'), ('juice', 0, 'juice')]

LIGHT = dict(bg='#fcfcfb', t1='#0b0b0b', t2='#52514e', mu='#898781', grid='#e1e0d9', axis='#c3c2b7', oth='#b9b7ae',
             s1='#2a78d6', s2='#eb6834', s3='#1baf7a', s4='#eda100', s5='#e87ba4')
DARK = dict(bg='#1a1a19', t1='#f0efec', t2='#c3c2b7', mu='#898781', grid='#2c2c2a', axis='#383835', oth='#5f5e59',
            s1='#3987e5', s2='#d95926', s3='#199e70', s4='#c98500', s5='#d55181')
SEQ = ['#cde2fb', '#b7d3f6', '#9ec5f4', '#86b6ef', '#6da7ec', '#5598e7', '#3987e5', '#2a78d6', '#256abf', '#1c5cab', '#184f95', '#104281', '#0d366b']

def style():
    def rules(p):
        return (f".bg{{fill:{p['bg']}}} .t1{{fill:{p['t1']}}} .t2{{fill:{p['t2']}}} .mu{{fill:{p['mu']}}} .grid{{stroke:{p['grid']}}} "
                f".axis{{stroke:{p['axis']}}} .ring{{stroke:{p['bg']}}} .oth{{fill:{p['oth']}}} .koth{{stroke:{p['oth']}}} "
                + ' '.join(f".f{i}{{fill:{p[f's{i}']}}} .k{i}{{stroke:{p[f's{i}']}}}" for i in range(1, 6)))
    return (f"<style>svg{{font-family:system-ui,-apple-system,'Segoe UI',sans-serif}} text{{font-size:11px}} .h{{font-size:13px;font-weight:600}} "
            f".tab{{font-variant-numeric:tabular-nums}} {rules(LIGHT)} @media (prefers-color-scheme: dark){{{rules(DARK)}}}</style>")

def esc(s):
    return str(s).replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')

def svg(w, h, body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{style()}<rect class="bg" width="{w}" height="{h}"/>{"".join(body)}</svg>'

def ticks(lo, hi, n=4):
    step = 10 ** math.floor(math.log10((hi - lo) / n))
    for m in (1, 2, 2.5, 5, 10):
        if (hi - lo) / (step * m) <= n: break
    step *= m
    t = math.ceil(lo / step) * step
    out = []
    while t <= hi + 1e-12:
        out.append(round(t, 10)); t += step
    return out

def tick(t, lo, hi):
    ts = ticks(lo, hi)
    d = max(0, -math.floor(math.log10(ts[1] - ts[0]))) if len(ts) > 1 else 3
    return '0' if abs(t) < 1e-12 else f'{t:+.{d}f}'

def fmt(v):
    return f'{v:+.4f}'.replace('+0.0000', '0').replace('-0.0000', '0')

def rows(group, items, sets):
    # cmp() of each variant against the plain snapshot of its group, per set
    out = {}
    for s, ch, title in sets:
        for name, suf, role in items:
            v = f'{group}:{suf}' if suf else group
            r = an.cmp(group, s, ch, group, v)
            if r: out[(s, name)] = r
    return out

def scatter_panel(x0, y0, w, h, title, pts):
    # pts: (name, cpu, diff, role, tip); x from the CPU ratio, y the diff
    body = [f'<text class="h t1" x="{x0}" y="{y0 + 14}">{esc(title)}</text>']
    px0, py0, pw, ph = x0 + 44, y0 + 26, w - 56, h - 54
    xs = [p[1] for p in pts]; ys = [p[2] for p in pts] + [0]
    xlo, xhi = min(0.9, min(xs) * 0.95), max(xs) * 1.06
    ylo, yhi = min(ys), max(ys)
    pad = (yhi - ylo) * 0.12 or 0.001
    ylo, yhi = ylo - pad, yhi + pad
    X = lambda x: px0 + (x - xlo) / (xhi - xlo) * pw
    Y = lambda y: py0 + ph - (y - ylo) / (yhi - ylo) * ph
    for t in ticks(ylo, yhi):
        body.append(f'<line class="grid" x1="{px0}" x2="{px0 + pw}" y1="{Y(t):.1f}" y2="{Y(t):.1f}" stroke-width="1"/>')
        body.append(f'<text class="mu tab" x="{px0 - 6}" y="{Y(t) + 4:.1f}" text-anchor="end">{tick(t, ylo, yhi)}</text>')
    body.append(f'<line class="axis" x1="{px0}" x2="{px0 + pw}" y1="{Y(0):.1f}" y2="{Y(0):.1f}" stroke-width="1"/>')
    for t in ticks(xlo, xhi):
        body.append(f'<text class="mu tab" x="{X(t):.1f}" y="{py0 + ph + 16}" text-anchor="middle">x{t:g}</text>')
    taken = []  # boxes of labels and dots
    order = sorted(pts, key=lambda p: {'cand': 0, 'ref': 1, 'other': 2}[p[3]])
    for name, x, y, role, tip in order:
        taken.append((X(x) - 5, Y(y) - 5, X(x) + 5, Y(y) + 5))
    marks, labels = [], []
    for name, x, y, role, tip in order:
        cx, cy = X(x), Y(y)
        cls = {'cand': 'f1', 'ref': 't1', 'other': 'oth'}[role]
        r = 5.5 if role == 'cand' else 4.5
        marks.append(f'<circle class="{cls} ring" cx="{cx:.1f}" cy="{cy:.1f}" r="{r}" stroke-width="2"><title>{esc(tip)}</title></circle>')
        tw = 6.2 * len(name)
        for dx, dy, anchor in ((8, 4, 'start'), (-8, 4, 'end'), (0, -9, 'middle'), (0, 16, 'middle'), (8, -7, 'start'), (-8, -7, 'end'), (8, 14, 'start'), (-8, 14, 'end')):
            lx = cx + dx - {'start': 0, 'end': tw, 'middle': tw / 2}[anchor]
            box = (lx, cy + dy - 10, lx + tw, cy + dy + 2)
            inside = box[0] >= x0 and box[2] <= x0 + w and box[1] >= py0 - 4 and box[3] <= py0 + ph + 4
            if inside and not any(box[0] < b[2] and box[2] > b[0] and box[1] < b[3] and box[3] > b[1] for b in taken):
                taken.append(box)
                weight = ' font-weight="600"' if role == 'cand' else ''
                labels.append(f'<text class="{"t1" if role != "other" else "t2"}" x="{cx + dx:.1f}" y="{cy + dy:.1f}" text-anchor="{anchor}"{weight}>{esc(name)}</text>')
                break
    return body + marks + labels

def fig_tradeoff(group):
    R = rows(group, FINAL, SETS)
    pw, ph, cols = 330, 250, 4
    body = [f'<text class="h t1" x="16" y="22">Gain against CPU, every candidate in one batch ({group})</text>',
            '<text class="t2" x="16" y="40">y: mean diff against solve 48k in % of flow; x: CPU against solve 48k in the same session. Blue: the recommendation at each CPU budget (section 6); black: solve. Hover a dot for wins, losses and hit.</text>']
    for i, (s, ch, title) in enumerate(SETS):
        pts = []
        for name, suf, role in FINAL:
            r = R.get((s, name))
            if not r: continue
            tip = f"{name} on {title}: diff {fmt(r['diff'])}, {r['win']} wins / {r['loss']} losses, hit {r['hit']:.1f}% (solve {r['bhit']:.1f}), CPU x{r['cpu']:.2f}, evals x{r['ev']:.2f}, {r['runs']} runs"
            pts.append((name, r['cpu'], r['diff'], role, tip))
        if pts:
            body += scatter_panel(16 + (i % cols) * pw, 56 + (i // cols) * ph, pw - 12, ph - 10, title, pts)
    return svg(16 + cols * pw, 56 + math.ceil(len(SETS) / cols) * ph, body)

CURVES = [('solve', 's1', [f'max_iters={n}' for n in (6000, 12000, 24000)] + [''] + ['max_iters=96000']),
          ('price', 's2', [f'algo=price,lab_effort={e}' for e in ('0.125', '0.25', '0.5', '1', '2')]),
          ('memetic', 's3', [f'algo=memetic,lab_effort={e}' for e in ('0.25', '0.5', '1', '2')]),
          ('cem', 's4', [f'algo=cem,lab_effort={e}' for e in ('0.25', '0.5', '1', '2')]),
          ('lns', 's5', [f'algo=lns,lab_effort={e}' for e in ('0.25', '0.5', '1', '2')])]

def fig_curves():
    sets = [s for s in SETS if s[0] != 'large'] + [('large', 0, 'large')]
    pw, ph, cols = 330, 250, 4
    body = [f'<text class="h t1" x="16" y="22">Each prototype from its own start, against solve, at the new free tap cut (pr_a, batch ra)</text>',
            '<text class="t2" x="16" y="40">y: mean diff against solve 48k in % of flow; x: CPU against solve 48k (log scale). Points: solve 6k-96k, prototypes at effort 0.125/0.25-2. Hover a point.</text>']
    lx = 16
    for name, c, _ in CURVES:
        body.append(f'<line class="k{c[1]}" x1="{lx}" x2="{lx + 18}" y1="{58}" y2="{58}" stroke-width="2"/><circle class="f{c[1]} ring" cx="{lx + 9}" cy="58" r="4" stroke-width="2"/><text class="t2" x="{lx + 24}" y="62">{name}</text>')
        lx += 34 + 7 * len(name)
    for i, (s, ch, title) in enumerate(sets):
        x0, y0, w, h = 16 + (i % cols) * pw, 72 + (i // cols) * ph, pw - 12, ph - 10
        series = []
        for name, c, sufs in CURVES:
            pts = []
            for suf in sufs:
                r = an.cmp('ra', s, ch, 'pr_a', f'pr_a:{suf}' if suf else 'pr_a')
                if r: pts.append((r['cpu'], r['diff'], f"{name} {suf.split('=')[-1] or '48000'} on {title}: diff {fmt(r['diff'])}, {r['win']} wins / {r['loss']} losses, hit {r['hit']:.1f}%, CPU x{r['cpu']:.2f}"))
            series.append((c[1], sorted(pts)))
        allx = [p[0] for _, ps in series for p in ps]; ally = [p[1] for _, ps in series for p in ps] + [0]
        if not allx: continue
        body.append(f'<text class="h t1" x="{x0}" y="{y0 + 14}">{esc(title)}</text>')
        px0, py0, pw2, ph2 = x0 + 44, y0 + 26, w - 56, h - 54
        xlo, xhi = math.log2(min(allx)) - 0.15, math.log2(max(allx)) + 0.15
        ylo, yhi = min(ally), max(ally)
        pad = (yhi - ylo) * 0.08 or 0.001
        ylo, yhi = ylo - pad, yhi + pad
        X = lambda x: px0 + (math.log2(x) - xlo) / (xhi - xlo) * pw2
        Y = lambda y: py0 + ph2 - (y - ylo) / (yhi - ylo) * ph2
        for t in ticks(ylo, yhi):
            body.append(f'<line class="grid" x1="{px0}" x2="{px0 + pw2}" y1="{Y(t):.1f}" y2="{Y(t):.1f}" stroke-width="1"/>')
            body.append(f'<text class="mu tab" x="{px0 - 6}" y="{Y(t) + 4:.1f}" text-anchor="end">{tick(t, ylo, yhi)}</text>')
        body.append(f'<line class="axis" x1="{px0}" x2="{px0 + pw2}" y1="{Y(0):.1f}" y2="{Y(0):.1f}" stroke-width="1"/>')
        for t in (0.125, 0.25, 0.5, 1, 2, 4):
            if xlo <= math.log2(t) <= xhi:
                body.append(f'<text class="mu tab" x="{X(t):.1f}" y="{py0 + ph2 + 16}" text-anchor="middle">x{t:g}</text>')
        for c, ps in series:
            if len(ps) > 1:
                body.append(f'<polyline class="k{c}" fill="none" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" points="{" ".join(f"{X(x):.1f},{Y(y):.1f}" for x, y, _ in ps)}"/>')
        for c, ps in series:
            for x, y, tip in ps:
                body.append(f'<circle class="f{c} ring" cx="{X(x):.1f}" cy="{Y(y):.1f}" r="4" stroke-width="2"><title>{esc(tip)}</title></circle>')
    return svg(16 + cols * pw, 72 + math.ceil(len(sets) / cols) * ph, body)

def gaps(group, s, ch, v):
    # mean gap to the best known, per payload step, over seeds
    R = an.runs[(group, s, ch)][v]
    by = collections.defaultdict(list)
    for (st, seed), x in R.items():
        by[st].append((an.best[(st, ch)] - x[0]) / an.flow[(st, ch)] * 100)
    return {st: statistics.mean(g) for st, g in by.items()}

def fig_payloads(group, items=FINAL, sets=SETS, rows_max=34):
    cols = [(n, f'{group}:{suf}' if suf else group) for n, suf, _ in items]
    table = []
    for s, ch, title in sets:
        G = {n: gaps(group, s, ch, v) for n, v in cols}
        for st in G[cols[0][0]]:
            row = [G[n].get(st) for n, _ in cols]
            if None in row: continue
            if max(row) >= 0.02 and max(row) - min(row) >= 0.015:
                sc, step, n = an.steps[st]
                table.append((max(row) - min(row), title.split(' ')[0], f'{sc}:{step}', row))
    table.sort(key=lambda r: -r[0])
    table = sorted(table[:rows_max], key=lambda r: ([t[2] for t in sets].index(next(t[2] for t in sets if t[2].startswith(r[1]))), -r[0]))
    cw, rh, left, top = 66, 20, 300, 112
    body = [f'<text class="h t1" x="16" y="22">The payloads that separate the candidates ({group}): mean gap to the best known, % of flow</text>',
            '<text class="t2" x="16" y="40">Rows: the payload steps where the candidates differ most (gap 0.02 or more somewhere, spread 0.015 or more), by set.</text>',
            '<text class="t2" x="16" y="56">Darker is further from the best; blank is within 0.005. Hover a cell.</text>']
    for j, (n, _) in enumerate(cols):
        x = left + j * cw + cw / 2
        body.append(f'<text class="t2" x="{x:.1f}" y="{top - 8}" text-anchor="start" transform="rotate(-35 {x:.1f} {top - 8})">{esc(n)}</text>')
    vmax = 0.3
    for i, (_, sname, pay, row) in enumerate(table):
        y = top + i * rh
        body.append(f'<text class="t2" x="{left - 8}" y="{y + 14}" text-anchor="end">{esc(sname)}  {esc(pay)}</text>')
        for j, g in enumerate(row):
            k = min(len(SEQ) - 1, int(math.sqrt(min(g, vmax) / vmax) * (len(SEQ) - 1))) if g >= 0.005 else -1
            fill = SEQ[k] if k >= 0 else 'none'
            cls = '' if k >= 0 else ' class="grid"'
            body.append(f'<rect{cls} x="{left + j * cw + 1}" y="{y + 1}" width="{cw - 2}" height="{rh - 2}" rx="3" fill="{fill}" stroke-width="1"{" stroke-opacity=\"0.6\"" if k < 0 else ""}><title>{esc(pay)} ({esc(sname)}), {esc(cols[j][0])}: gap {g:.3f}</title></rect>')
            if g >= 0.005:
                ink = '#ffffff' if k >= 6 else '#0b0b0b'
                body.append(f'<text class="tab" x="{left + j * cw + cw / 2:.1f}" y="{y + 14}" text-anchor="middle" fill="{ink}">{g:.3f}</text>')
    return svg(left + len(cols) * cw + 24, top + len(table) * rh + 16, body), table

if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    if '--curves' in sys.argv or GROUP == 'fin_a':
        open(f'{OUT}/curves.svg', 'w', encoding='utf-8').write(fig_curves())
    open(f'{OUT}/tradeoff.svg', 'w', encoding='utf-8').write(fig_tradeoff(GROUP))
    s, table = fig_payloads(GROUP)
    open(f'{OUT}/payloads.svg', 'w', encoding='utf-8').write(s)
    print('wrote', OUT, len(table), 'payload rows')
