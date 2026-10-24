# Graphs of the pulled analytics, written to ignored/analytics/figures/ (app.py shows them in Streamlit).
# python plot.py [uploads|holdings|diffs|raids|gearscore|all] [--roster <prefix>]
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import pandas as pd

from load import BANDS, FIGURES, load

SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
BAND_COLOR = {"bound": SERIES[0], "roster": SERIES[1], "tradable": SERIES[2]}
plt.rcParams.update({
    "axes.prop_cycle": matplotlib.cycler(color=SERIES), "axes.spines.top": False, "axes.spines.right": False,
    "axes.grid": True, "axes.axisbelow": True, "grid.color": "#e5e4e0", "grid.linewidth": 0.8, "axes.edgecolor": "#8a8984",
    "lines.linewidth": 2, "lines.markersize": 6, "figure.dpi": 110, "savefig.bbox": "tight",
})


def save(fig, name):
    os.makedirs(FIGURES, exist_ok=True)
    path = os.path.join(FIGURES, f"{name}.png")
    fig.savefig(path)
    plt.close(fig)
    print("wrote", path)


def grid(n, w=5, h=3.2):
    cols = min(n, 3)
    rows = (n + cols - 1) // cols
    fig, axes = plt.subplots(rows, cols, figsize=(w * cols, h * rows), squeeze=False)
    for ax in axes.flat[n:]:
        ax.set_visible(False)
    return fig, axes.flat


def uploads(d):
    s = d["snaps"].drop_duplicates("upload")
    day = s.assign(day=pd.to_datetime(s.received, unit="ms").dt.floor("D")).groupby(["day", "region"]).size().unstack(fill_value=0)
    # string labels: a DatetimeIndex with an inferred freq breaks pandas' bar plot
    day.index = day.index.strftime("%m-%d")
    fig, ax = plt.subplots(figsize=(8, 3.5))
    day.plot.bar(ax=ax, stacked=True, width=0.8, edgecolor="white", linewidth=2, rot=0)
    ax.set(title="Uploads per day", xlabel="", ylabel="uploads")
    return fig, None


def band_of(m):
    return m.band.replace({"special": "bound"})


def holdings(d, n=15):
    # the n characters with the longest snapshot span: own bound, + roster's shared roster, + tradable (Silver excluded: no price)
    m = d["mats"][~d["snaps"].set_index("hash").loc[d["mats"].hash].ignored.values]
    m = m.assign(band=band_of(m))
    shared = m[m.char.isna()].groupby(["hash", "band"]).gold.sum().unstack(fill_value=0).reindex(columns=BANDS[1:], fill_value=0)
    c = m[m.char.notna()].groupby(["roster", "char", "hash", "time", "kind"]).gold.sum().rename("bound").reset_index().rename(columns={"roster": "rid"})
    c = c.join(shared, on="hash").fillna(0)
    c[BANDS] = c[BANDS].cumsum(axis=1)
    chars = c.groupby("char").time.agg(lambda t: t.max() - t.min()).nlargest(n).index
    fig, axes = grid(len(chars))
    for ax, ch in zip(axes, chars):
        g = c[c.char == ch].sort_values("time")
        for i, band in enumerate(BANDS):
            ax.plot(range(len(g)), g[band] / 1e3, marker="o", label=("+ " if i else "") + band, color=BAND_COLOR[band])
        ax.set_xticks(range(len(g)), [f"{t:%m-%d %H:%M} {k[0]}" for t, k in zip(g.time, g.kind)], rotation=45, ha="right", fontsize=7)
        ax.set(title=f"{g.rid.iloc[0][:8]} / {ch[:8]}", ylabel="k gold")
    fig.legend(*axes[0].get_legend_handles_labels(), loc="upper right")
    fig.suptitle(f"Holdings value of the {n} longest-tracked characters, bands stacked (B = before export, A = after, M = manual)")
    fig.tight_layout()
    return fig, None


def diffs(d):
    # change between consecutive "After export" snapshots of a roster: what the game gave or took
    s = d["snaps"][(d["snaps"].kind == "After export") & ~d["snaps"].ignored].sort_values("at")
    m = d["mats"][d["mats"].hash.isin(s.hash)]
    m = m.assign(band=band_of(m)).groupby(["roster", "at", "band", "label"])[["amount", "gold"]].sum()
    rows = []
    for r, g in s.groupby("roster"):
        ats = list(g["at"])
        for a, b in zip(ats, ats[1:]):
            x = m.loc[r].loc[b] if b in m.loc[r].index.get_level_values(0) else pd.DataFrame(columns=["amount", "gold"])
            y = m.loc[r].loc[a] if a in m.loc[r].index.get_level_values(0) else pd.DataFrame(columns=["amount", "gold"])
            delta = x.sub(y, fill_value=0)
            delta = delta[delta.amount != 0].reset_index()
            rows.append(delta.assign(roster=r[:8], start=pd.to_datetime(a, unit="ms"), hours=(b - a) / 3.6e6))
    if not rows:
        return None, None
    out = pd.concat(rows)
    per = out.groupby(["roster", "start", "band"]).gold.sum().unstack(fill_value=0)
    fig, ax = plt.subplots(figsize=(max(6, len(per) * 0.6), 4))
    per.plot.bar(ax=ax, width=0.8, edgecolor="white", linewidth=2, color=[BAND_COLOR[b] for b in per.columns])
    ax.set_xticklabels([f"{r} {t:%m-%d %H:%M}" for r, t in per.index], rotation=60, ha="right", fontsize=8)
    ax.axhline(0, color="#8a8984", linewidth=1)
    ax.set(title="Value change between consecutive exports", xlabel="", ylabel="gold")
    return fig, out


def raids(d):
    logs = d["raids"][d["raids"].reason == "logs"]
    if logs.empty:
        return None, None
    weekly = logs.groupby("week").agg(clears=("boss", "size"), chars=("char", "nunique"))
    weekly = weekly[weekly.index >= weekly.index.max() - pd.Timedelta(weeks=12)]
    fig, (a, b) = plt.subplots(1, 2, figsize=(12, 3.5))
    a.plot(weekly.index, weekly.clears, marker="o", label="clears")
    a.plot(weekly.index, weekly.chars, marker="o", label="characters")
    a.set(title="Logged clears per week (last 12)", ylabel="count")
    a.tick_params(axis="x", labelrotation=30, labelsize=8)
    a.legend()
    latest = logs.sort_values("week").groupby("char").gearScore.last()
    b.hist(latest, bins=range(int(latest.min()) // 5 * 5, int(latest.max()) + 10, 5), color=SERIES[0], edgecolor="white", linewidth=2)
    b.set(title="Latest logged gear score per character", xlabel="gear score", ylabel="characters")
    return fig, logs.groupby(["boss", "difficulty"]).char.nunique().sort_values(ascending=False).head(20).rename("chars").reset_index()


def gearscore(d):
    # each character's bound holdings in its roster's latest snapshot against its latest gear score
    s = d["snaps"].sort_values("at").groupby("roster").tail(1)
    m = d["mats"][d["mats"].hash.isin(s.hash) & d["mats"].char.notna()].groupby("char").gold.sum()
    logs = d["raids"][d["raids"].reason == "logs"]
    gs = logs.sort_values("week").groupby("char").gearScore.last()
    both = pd.concat([gs, m], axis=1, join="inner")
    if both.empty:
        return None, None
    fig, ax = plt.subplots(figsize=(6, 4))
    ax.scatter(both.gearScore, both.gold / 1e3, s=40, color=SERIES[0], edgecolor="white", linewidth=1.5)
    ax.set(title="Char-bound holdings vs gear score", xlabel="gear score", ylabel="k gold", yscale="log")
    return fig, None


PLOTS = dict(uploads=uploads, holdings=holdings, diffs=diffs, raids=raids, gearscore=gearscore)

if __name__ == "__main__":
    args = sys.argv[1:]
    d = load("--dev" in args)
    if "--roster" in args:
        prefix = args[args.index("--roster") + 1]
        for k in ("snaps", "mats", "chests", "raids"):
            d[k] = d[k][d[k].roster.str.startswith(prefix)]
    names = [a for a in args if a in PLOTS] or (["all"] if "all" in args else [])
    for name in PLOTS if names in ([], ["all"]) else names:
        fig, table = PLOTS[name](d)
        if fig is None:
            print("no data for", name)
            continue
        save(fig, name)
        if table is not None:
            print(table.to_string(index=False))
