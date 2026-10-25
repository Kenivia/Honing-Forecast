# Market viewer: daily average prices (history.jsonl) or lostark.bible item level spread (uwuowo.jsonl), with game release markers. Run: pnpm market
import json
import re
from pathlib import Path

import pandas as pd
import plotly.express as px
import streamlit as st

HERE = Path(__file__).parent
ROOT = HERE.parents[1]
SRC = ROOT / "ignored" / "data_fetcher"
SOURCES = {"prices": [SRC / "history.jsonl"], "population": [SRC / "uwuowo.jsonl"], "estimate": [SRC / "uwuowo.jsonl", SRC / "uwuowo2.jsonl", SRC / "wayback_cpd.jsonl"]}
CACHE = HERE / ".cache"
DEFAULT_MATS = ["destiny-guardian-stone", "destiny-destruction-stone", "destiny-leapstone", "abidos-fusion-material"]
DEFAULT_THRESHOLDS = [1640, 1680, 1700, 1720, 1740, 1760, 1780]
PATCH = pd.Timestamp("2026-10-21 07:00")  # Belgardin; lostark.bible rebuilt its combat power samples after it
DASHES = ["solid", "dot", "dash", "longdash", "dashdot", "longdashdot"]
KIND_STYLE = {"raid": ("gray", "solid"), "event": ("gray", "dash"), "mode": ("gray", "dot"), "honing": ("teal", "solid")}


@st.cache_data
def site_colors():
    # Site graph colour per slug: MarketDataFetcher slug -> MATERIAL_TABLE colour -> theme.css
    front = ROOT / "frontend"
    fetcher = (front / "Utils" / "MarketDataFetcher.ts").read_text(encoding="utf-8")
    block = fetcher[fetcher.index("ITEM_SLUG_TO_LABEL: Record") :]
    slug_label = re.findall(r'"([a-z0-9-]+)":\s*\[?"([^"]+)"', block[: block.index("};")])
    consts = (front / "Utils" / "Constants.ts").read_text(encoding="utf-8")
    table = consts[consts.index("const MATERIAL_TABLE") :]
    label_color = {k.strip(): c for k, c in re.findall(r'"?([^"\n:]+?)"?:\s*\[\s*\[[\d, ]*\],\s*"(\w+)"', table[: table.index("\n};")])}
    hexes = dict(re.findall(r"--series-(\w+):\s*(#\w+)", (front / "theme.css").read_text(encoding="utf-8")))
    return {slug: hexes[label_color[label]] for slug, label in slug_label}


def parse_prices():
    rows = []
    with open(SOURCES["prices"][0]) as f:
        for line in f:
            e = json.loads(line)
            for d in e["data"] or []:
                rows.append((e["region"], e["material"], pd.Timestamp(d["day"]), d["avg_price"]))
    df = pd.DataFrame(rows, columns=["region", "item", "time", "price"])
    # Fetches overlap by a day; the later fetch of a day is the complete one
    return df.drop_duplicates(["region", "item", "time"], keep="last").sort_values("time").reset_index(drop=True)


def obj(s, key):
    # A JS object literal from SvelteKit SSR data, as a dict
    c = s.find(key + ":")
    i = s.index("{", c)
    d = 0
    for k in range(i, len(s)):
        d += s[k] == "{"
        d -= s[k] == "}"
        if d == 0:
            break
    return json.loads(re.sub(r'([{,])([A-Za-z_]\w*):', r'\1"\2":', s[i : k + 1]))


def stats_pages():
    # lostark.bible /stats/generic pages, NA + CE together: (time, {unit: {ilvl: count at or above}}, total per unit, classCounts)
    pages = {}
    with open(SOURCES["population"][0]) as f:
        for line in f:
            s = json.loads(line)["data"]
            time = pd.to_datetime(int(re.search(r"generic:\{lastUpdate:new Date\((\d+)\)", s).group(1)), unit="ms")
            at_least, totals = {}, {}
            for unit, key in [("characters", "ilvlCounts"), ("rosters", "rosterIlvlCounts")]:
                # 10-ilvl brackets [start, count], start 0 = below the lowest
                counts = json.loads(re.search(key + r":(\[\[.*?\]\])", s).group(1))
                totals[unit] = sum(c for _, c in counts)
                at_least[unit] = {b: sum(c for x, c in counts if x >= b) for b, _ in counts if b > 0}
            # classCounts is cumulative per class and reaches higher than the top bracket
            classes = obj(s, "classCounts")
            for b, per in classes.items():
                if int(b) > 0:
                    at_least["characters"].setdefault(int(b), sum(per.values()))
            pages[time] = (at_least, totals, classes)
    return [(t, *pages[t]) for t in sorted(pages)]


def parse_population():
    rows = [(t, unit, b, 100 * n / totals[unit]) for t, at_least, totals, _ in stats_pages() for unit in at_least for b, n in at_least[unit].items()]
    return pd.DataFrame(rows, columns=["time", "unit", "ilvl", "share"])


def cpd_at_least(dist):
    # Both combat power types of one class over 10-ilvl brackets, as count at or above each bracket
    per = {}
    for k, v in dist.items():
        _, b = k.split("-")
        if b != "all":
            per[int(b)] = per.get(int(b), 0) + v["count"]
    return {b: sum(c for x, c in per.items() if x >= b) for b in per}


def parse_estimate():
    # Characters' share at or above X through the stats gap, from per-class combat power samples:
    #  estimate: own daily bard sample, scaled to the stats page's bards (at the last page before), bards -> all classes and 1700+ share interpolated between the stats pages around the gap
    #  check: archived character pages of nearly every class on one day, summed, same bard scaling
    # Both run ~1-2 points low unscaled at 1720+; verified against stats pages and each other (within ~0.5 points)
    pages = stats_pages()
    own = []
    with open(SOURCES["estimate"][1], encoding="utf-8") as f:
        for line in f:
            e = json.loads(line)
            t = pd.Timestamp(e["timestamp"])
            if t < PATCH:
                own.append((t, cpd_at_least(obj(e["data"], "combatPowerDistribution")["bard"])))
    before = max((p for p in pages if p[0] <= own[0][0]), key=lambda p: p[0])
    after = min((p for p in pages if p[0] >= PATCH), key=lambda p: p[0])
    xs = [x for x in range(1720, 1820, 20) if str(x) in before[3] and str(x) in after[3]]

    def cond(at_least, x):
        return at_least.get(x, 0) / at_least[1700]

    def stat(page, x, cls=None):
        per = page[3]
        return per[str(x)].get(cls, 0) / per["1700"][cls] if cls else sum(per[str(x)].values()) / sum(per["1700"].values())

    def interp(t, f):
        w = (t - before[0]) / (after[0] - before[0])
        return f(before) * (1 - w) + f(after) * w

    scale = {x: stat(before, x, "bard") / cond(own[0][1], x) for x in xs}

    def share(t, c, x, bard=True):
        to_all = interp(t, lambda p: stat(p, x) / stat(p, x, "bard")) if bard else 1
        base = interp(t, lambda p: p[1]["characters"][1700] / p[2]["characters"])
        return 100 * c * scale[x] * to_all * base

    rows = [(t, x, share(t, cond(a, x), x), "estimate") for t, a in own for x in xs]
    days = {}
    with open(SOURCES["estimate"][2], encoding="utf-8") as f:
        for line in f:
            e = json.loads(line)
            for cls, dist in (e["cpd"] or {}).items():
                days.setdefault(pd.Timestamp(e["timestamp"][:8]), {})[cls] = cpd_at_least(dist)
    for day, classes in days.items():
        if len(classes) < 20 or not before[0] <= day < PATCH:
            continue
        total = {b: sum(a.get(b, 0) for a in classes.values()) for b in [1700] + xs}
        rows += [(day, x, share(day, cond(total, x), x, bard=False), "check") for x in xs]
    return pd.DataFrame(rows, columns=["time", "ilvl", "share", "kind"])


@st.cache_data
def load(name, mtime):
    path = CACHE / f"{name}.parquet"
    if path.exists() and path.stat().st_mtime > mtime:
        return pd.read_parquet(path)
    df = {"prices": parse_prices, "population": parse_population, "estimate": parse_estimate}[name]()
    CACHE.mkdir(exist_ok=True)
    df.to_parquet(path)
    return df


def broken(wide):
    # Lines break over gaps > 7 days
    gaps = wide.index.to_series().diff() > pd.Timedelta(days=7)
    breaks = pd.DataFrame(index=wide.index[gaps] - pd.Timedelta(days=1), columns=wide.columns)
    return pd.concat([wide, breaks]).sort_index().astype(float)


def add_events(fig, events, kinds, start, end):
    shown = events[events["kind"].isin(kinds) & (events["date"].dt.date >= start) & (events["date"].dt.date <= end)]
    for date, group in shown.groupby("date"):
        for r in group.itertuples():
            color, dash = KIND_STYLE[r.kind]
            fig.add_shape(type="line", x0=date, x1=date, y0=0, y1=1, yref="paper", line=dict(color=color, width=1, dash=dash))
        text = " · ".join(f"<span style='color:{KIND_STYLE[r.kind][0]}'>{r.name}</span>" for r in group.itertuples())
        fig.add_annotation(x=date, y=1, yref="paper", text=text, textangle=-90, xanchor="right", yanchor="top", showarrow=False, font=dict(size=11))


def mtime(name):
    return max(p.stat().st_mtime for p in SOURCES[name])


st.set_page_config(page_title="Market prices", layout="wide")
view = st.radio("Show", ["Prices", "Population"], horizontal=True, label_visibility="collapsed")
events = pd.read_csv(HERE / "events.csv", parse_dates=["date"])

with st.sidebar:
    if view == "Prices":
        df = load("prices", mtime("prices"))
        region = st.radio("Region", sorted(df["region"].unique()), horizontal=True)
        df = df[df["region"] == region]
        items = sorted(df["item"].unique())
        mats = st.multiselect("Materials", items, default=[m for m in DEFAULT_MATS if m in items])
    else:
        df = load("population", mtime("population"))
        unit = st.radio("Count", ["characters", "rosters"], horizontal=True)
        df = df[df["unit"] == unit]
        brackets = sorted(x for x in df["ilvl"].unique() if x > 0)
        thresholds = st.multiselect("Item level at or above", brackets, default=[x for x in DEFAULT_THRESHOLDS if x in brackets])
    lo, hi = df["time"].min().date(), df["time"].max().date()
    start, end = st.slider("Timeframe", lo, hi, (lo, hi), format="YYYY-MM-DD", key=f"range_{view}")
    log = st.checkbox("Log scale")
    kinds = st.multiselect("Markers", list(KIND_STYLE), default=["raid", "event", "honing"])

df = df[(df["time"].dt.date >= start) & (df["time"].dt.date <= end)]
if view == "Prices":
    sel = df[df["item"].isin(mats)]
    if sel.empty:
        st.info("Pick at least one material.")
        st.stop()
    fig = px.line(sel, x="time", y="price", color="item", category_orders={"item": mats}, log_y=log, render_mode="webgl")
    # Selected materials sharing a site colour get distinct dashes, in selection order
    colors, used = site_colors(), {}
    for trace in fig.data:
        if trace.name in colors:
            color = colors[trace.name]
            used[color] = used.get(color, -1) + 1
            trace.line.update(color=color, dash=DASHES[used[color] % len(DASHES)])
    caption = "Daily average prices."
else:
    if df.empty or not thresholds:
        st.info("Pick at least one item level.")
        st.stop()
    thresholds = sorted(thresholds)
    palette = dict(zip(thresholds, px.colors.sample_colorscale("Viridis", [i / max(len(thresholds) - 1, 1) for i in range(len(thresholds))])))
    wide = broken(df[df["ilvl"].isin(thresholds)].pivot(index="time", columns="ilvl", values="share"))
    fig = px.line(wide.rename(columns=lambda x: f"{x}+"), markers=True, log_y=log, color_discrete_sequence=list(palette.values()), labels={"index": "time", "value": f"% of {unit}", "variable": ""})
    caption = f"Share of {unit} lostark.bible has seen (NA + CE) at or above each item level; it only knows characters someone looked up."
    if unit == "characters":
        est = load("estimate", mtime("estimate"))
        est = est[est["ilvl"].isin(thresholds) & (est["time"].dt.date >= start) & (est["time"].dt.date <= end)]
        for (x, kind), g in est.groupby(["ilvl", "kind"]):
            style = dict(mode="lines", line=dict(color=palette[x], dash="dash")) if kind == "estimate" else dict(mode="markers", marker=dict(color=palette[x], symbol="circle-open", size=10))
            fig.add_scatter(x=g["time"], y=g["share"], name=f"{x}+ {kind}", legendgroup=kind, **style)
        caption += " Dashed: estimate through the gap from lostark.bible's daily bard combat power sample; hollow: check from archived pages of nearly every class (scripts/market/app.py, parse_estimate)."
add_events(fig, events, kinds, start, end)
fig.update_layout(height=650, hovermode="x unified", legend_title=None)
st.plotly_chart(fig, width="stretch")
st.caption(caption + " Markers (scripts/market/events.csv): grey solid raid, dashed event, dotted mode; teal honing change.")
