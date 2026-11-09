# iteration 3 preliminary analysis; run from the repo root (traces.py first for the anytime scripts)
# Inventory of candidate real cases across all analytics pulls, with a near-duplicate rule.
import json, glob, os, collections, re
DATA = "ignored/analytics"
DONE, WANT = (0, 3), 1

rows = []
for f in sorted(glob.glob(os.path.join(DATA, "uploads-*.json"))):
    for r in json.load(open(f, encoding="utf-8")):
        if r["version"].endswith("dev") or str(r["version"]).startswith("followup"):
            continue
        r["file"] = os.path.basename(f)[8:18]
        rows.append(r)

def placeholder(d):
    return any(l != "Silver" and (re.fullmatch(r"9+", str(int(a))) or a >= 999_000_000) for l, a in d.items() if isinstance(a, (int, float)))

def cells(grid, want):
    return frozenset((i, j) for i, g in enumerate(grid or []) for j, v in enumerate(g) if (v == WANT if want else v in DONE))

def price(s, label):
    b = s.get("bundles", {}).get(label, 1) or 1
    return (s.get("prices", {}).get(f"{label} x{b}", 0) or 0) / b

seen = set(); cases = []
keys_seen = collections.Counter()
for r in rows:
    for s in r["snapshots"]:
        for cid, c in s["chars"].items():
            keys_seen.update(c.keys())
            if not c.get("normal_grid"): continue
            wn, wa = cells(c.get("normal_grid"), 1), cells(c.get("adv_grid"), 1)
            if not (wn or wa): continue
            if placeholder(c.get("bound", {})) or placeholder(s.get("roster_owned", {})): continue
            key = (s["roster_id"], cid)
            mats = collections.Counter()
            for band in (c.get("bound", {}), s.get("roster_owned", {}), s.get("tradable", {}), c.get("special", {})):
                for l, a in band.items():
                    if isinstance(a, (int, float)): mats[l] += a
            content = json.dumps([c.get("bound"), s.get("roster_owned"), s.get("tradable"), c.get("special"), c.get("normal_grid"), c.get("adv_grid"), c.get("tier")], sort_keys=True)
            if (key, content) in seen: continue
            seen.add((key, content))
            gold = {l: a * price(s, l) for l, a in mats.items()}
            cases.append(dict(key=key, roster=s["roster_id"], at=s["at"], file=r["file"], tier=c.get("tier"), wn=wn, wa=wa,
                              done=cells(c.get("normal_grid"), 0) | {("a",) + x for x in cells(c.get("adv_grid"), 0)}, mats=mats, gold=gold, chests=len(c.get("chests", [])) + len(s.get("roster_chests", []))))
print("char keys", keys_seen.most_common())
print("distinct (char, content) cases", len(cases), "chars", len({c['key'] for c in cases}), "rosters", len({c['roster'] for c in cases}))

def changed(a, b, tol):
    labels = set(a["gold"]) | set(b["gold"])
    tot = sum(a["gold"].values()) + 1e-9
    d = sum(abs(a["gold"].get(l, 0) - b["gold"].get(l, 0)) for l in labels)
    # count-based fallback where unpriced
    rel = max((abs(a["mats"].get(l, 0) - b["mats"].get(l, 0)) / max(a["mats"].get(l, 0), b["mats"].get(l, 0), 1) for l in set(a["mats"]) | set(b["mats"])), default=0)
    return d / tot > tol, rel

for tol in (0.05, 0.10, 0.20):
    fams = []
    by = collections.defaultdict(list)
    for c in sorted(cases, key=lambda c: c["at"]): by[c["key"]].append(c)
    kept = []
    for key, lst in by.items():
        reps = []
        for c in lst:
            new = True
            for r in reps:
                if r["wn"] == c["wn"] and r["wa"] == c["wa"] and r["done"] == c["done"] and not changed(r, c, tol)[0]:
                    new = False; break
            if new: reps.append(c)
        kept += reps
    print(f"tol {tol}: families {len(kept)} from {len(cases)}; chars {len(by)}; per-char max {max(len([k for k in kept if k['key']==key]) for key in by)}")
    if tol == 0.10:
        K = kept
new_since = [c for c in K if c["file"] > "2026-10-27"]
print("families first seen in pulls after 10-27:", len(new_since), "rosters new:", len({c['roster'] for c in K if c['file'] > '2026-10-27'} - {c['roster'] for c in K if c['file'] <= '2026-10-27'}))
print("tier", collections.Counter(c["tier"] for c in K))
nw = sorted(len(c["wn"]) + len(c["wa"]) for c in K)
print("wanted cells quartiles", [nw[int(q * (len(nw) - 1))] for q in (0, .1, .25, .5, .75, .9, 1)])
print("with adv wanted", sum(bool(c["wa"]) for c in K), "with chests", sum(c["chests"] > 0 for c in K))
print("rosters", len({c['roster'] for c in K}), "cases per roster", sorted(collections.Counter(c['roster'] for c in K).values(), reverse=True)[:15])

old = {m.group(1) for f in ('real_test', 'real_income_test') for l in open(f'arena_data/sets/{f}.txt', encoding='utf-8') for m in [re.match(r'real_(?:income_)?([0-9a-f]{8})', l)] if m}
fam_old = [c for c in K if c['roster'][:8] in old]
print('old held-out rosters', len(old), 'families in them', len(fam_old), '; other rosters', len({c['roster'] for c in K}) - len({c['roster'] for c in fam_old}), 'families', len(K) - len(fam_old))
per_char = collections.Counter(c['key'] for c in K)
print('characters with >=3 families', sum(v >= 3 for v in per_char.values()), 'with >=5', sum(v >= 5 for v in per_char.values()), 'families in those (>=3)', sum(v for v in per_char.values() if v >= 3))

for lab, grp in (('old', [c for c in K if c['roster'][:8] in old]), ('new', [c for c in K if c['roster'][:8] not in old])):
    print(lab, 'rosters', len({c['roster'] for c in grp}), 'families per roster', sorted(collections.Counter(c['roster'] for c in grp).values(), reverse=True))
