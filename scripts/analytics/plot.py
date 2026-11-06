# Graphs of the pulled analytics, written to ignored/analytics/figures/ (app.py shows them in Streamlit).
# python plot.py [uploads|materials|raids|all] [--roster <prefix>] [--bands bound,roster,tradable]
import os
import sys

import matplotlib

matplotlib.use("Agg")
import matplotlib.dates as mdates
import matplotlib.pyplot as plt
import pandas as pd

from load import BANDS, FIGURES, load

SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
plt.rcParams.update({
    "axes.prop_cycle": matplotlib.cycler(color=SERIES), "axes.spines.top": False, "axes.spines.right": False,
    "axes.grid": True, "axes.axisbelow": True, "grid.color": "#e5e4e0", "grid.linewidth": 0.8, "axes.edgecolor": "#8a8984",
    "lines.linewidth": 2, "lines.markersize": 6, "figure.dpi": 110, "savefig.bbox": "tight",
})

# main materials in the site's graph colours (frontend/theme.css); special leaps dashed
MAIN = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver", "Special Leap"]
MAIN_COLOR = {"Red": "#ff0000", "Blue": "#6666ff", "Leaps": "#b300ff", "Shards": "#f13b9f", "Fusion": "#ff7824",
              "Gold": "#facc15", "Silver": "#9ca3af", "Special Leap": "#b300ff"}
SERCA = ("Red", "Blue", "Leaps", "Fusion", "Special Leap")
SERCA_BOOKS = ("12-15", "16-19")
JUICE = {"Weapon": "Lava's Breath", "Armor": "Glacier's Breath"}
SLOT = {"bracer": "V", "weapon": "W", "head": "H", "shoulder": "S", "upper_body": "C", "lower_body": "P", "hand": "G"}
MIN_DAYS = 1  # span of a character's own exports to get a row
RESET = pd.Timedelta(hours=10)  # daily content resets at 10:00 UTC


def reset_day(t):
    return (t - RESET).floor("D") + RESET


def save(fig, name):
    os.makedirs(FIGURES, exist_ok=True)
    path = os.path.join(FIGURES, f"{name}.png")
    fig.savefig(path)
    plt.close(fig)
    print("wrote", path)


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


def short(x):
    for unit, n in (("B", 1e9), ("M", 1e6), ("k", 1e3)):
        if abs(x) >= n:
            return f"{x / n:.3g}{unit}"
    return f"{x:.0f}"


def optional(label, tier, kind):
    return label == JUICE[kind] or (label.endswith(kind) and label.startswith(SERCA_BOOKS) == (tier == 1))


def honing_events(gear):
    # per character, (time, text) whenever a slot's honing passes its highest so far in lostark.bible's most recent loadout;
    # a different item at a lower level (T4 to Serca) restarts the slot, a higher one (vambrace +10 to +15) is honing
    g = gear[(gear.reason == "gear") & gear.honing.notna()]
    best = g.sort_values("updated").groupby(["char", "upload"]).loadout.last()
    g = g.join(best.rename("best"), on=["char", "upload"])
    g = g[g.loadout == g.best].sort_values("received")
    out = {}
    for ch, x in g.groupby("char"):
        top, ev = {}, {}
        for slot, item, h, t in zip(x.slot, x.id, x.honing, x.updated):
            if slot in top and h > top[slot][1]:
                ev.setdefault(t, []).append(f"{SLOT[slot]} {top[slot][1]:.0f}->{h:.0f}")
            if slot not in top or h > top[slot][1] or item != top[slot][0]:
                top[slot] = (item, h)
        out[ch] = [(t, ", ".join(v)) for t, v in ev.items()]
    return out


def materials(d, bands=BANDS):
    # per character: main materials, optional weapon, optional armour, each line as % of its first non-zero value
    s = d["snaps"][(d["snaps"].kind == "After export") & ~d["snaps"].ignored].set_index("hash")
    m = d["mats"][d["mats"].hash.isin(s.index)]
    m = m.assign(band=band_of(m))
    m = m[m.band.isin(bands)]
    owned = d["mats"][d["mats"].char.notna() & d["mats"].hash.isin(s.index)]
    own = s[s.by.notna()].groupby("by").time.agg(lambda t: (t.max() - t.min()).total_seconds() / 86400)
    events = honing_events(d["gear"])
    opt = sorted(l for l in d["mats"].label.unique() if l.endswith(("Weapon", "Armor")) or l in JUICE.values())
    opt_color = {l: plt.cm.tab20(i % 20) for i, l in enumerate(opt)}

    rows = []
    for c, days in own[own >= MIN_DAYS].items():
        hs = s.loc[owned[owned.char == c].hash.unique()].sort_values("time").index
        x = m[m.hash.isin(hs) & (m.char.isna() | (m.char == c))]
        t = x.pivot_table(index="hash", columns=["band", "label"], values="amount", aggfunc="sum").reindex(hs)
        # a band missing from a snapshot takes its nearest known amounts (previous, else next) instead of zero
        for b in t.columns.unique("band"):
            blk = t[b].fillna(0)
            blk[t[b].isna().all(axis=1)] = float("nan")
            t.loc[:, b] = blk.ffill().bfill().fillna(0).values
        t = t.T.groupby(level="label").sum().T
        t["mine"] = (s.loc[hs].by == c).values
        t.index = s.loc[hs].time
        ev = [(at, txt) for at, txt in events.get(c, []) if t.index[0] <= at <= t.index[-1]]
        # the latest snapshot of each reset day; big dot on days it exported
        day = reset_day(t.index)
        mine = t.mine.groupby(day).any().values
        t = t.drop(columns="mine").groupby(day).last()
        tier = owned[owned.char == c].sort_values("at").tier.dropna().iloc[-1]
        rows.append(dict(roster=s.loc[hs[0]].roster, char=c, tier=tier, days=days, clean=not ev, ev=ev, t=t, mine=mine))
    if not rows:
        return None, None
    rows.sort(key=lambda r: (not r["clean"], -r["days"]))

    fig, axes = plt.subplots(len(rows), 3, figsize=(17, 3.6 * len(rows)), squeeze=False, sharex="row")
    for r, ax3 in zip(rows, axes):
        tier, t = r["tier"], r["t"]
        main = [(f"Serca {l}" if tier == 1 and l in SERCA else l, MAIN_COLOR[l], "--" if l == "Special Leap" else "-") for l in MAIN]
        panels = [main] + [[(l, opt_color[l], "-") for l in opt if optional(l, tier, k)] for k in ("Weapon", "Armor")]
        for ax, items in zip(ax3, panels):
            for label, color, style in items:
                if label not in t or not t[label].any():
                    continue
                y = t[label]
                base = y[y != 0].iloc[0]
                y = y / base * 100
                ax.plot(t.index, y, color=color, linestyle=style, linewidth=1.5, marker="o", markersize=3, label=f"{label} ({short(base)})")
                ax.plot(t.index[r["mine"]], y[r["mine"]], "o", color=color, markersize=6)
            for at, txt in r["ev"]:
                ax.axvline(reset_day(at), color="#8a8984", linestyle=":", linewidth=1.2)
                if ax is ax3[0]:
                    ax.text(reset_day(at), 0.98, txt, transform=ax.get_xaxis_transform(), rotation=90, fontsize=7, va="top", ha="right")
            if ax.lines:
                ax.legend(fontsize=7, loc="best")
            ax.xaxis.set_major_locator(mdates.DayLocator(interval=max(1, (t.index[-1] - t.index[0]).days // 5)))
            ax.xaxis.set_major_formatter(mdates.DateFormatter("%m-%d"))
        name = f"{r['roster'][:8]} / {r['char'][:8]} {'Serca' if tier == 1 else 'T4'}, {r['days']:.1f} days, {'clean' if r['clean'] else 'honed'}"
        ax3[0].set(title=name, ylabel="% of first")
        ax3[1].set(title="optional weapon")
        ax3[2].set(title="optional armour")
    fig.suptitle(f"Holdings per character ({' + '.join(bands)}), latest snapshot of each day (10:00 UTC reset), big dots = its own exports, dotted lines = honing on lostark.bible "
                 "(V vambrace, W weapon, H/S/C/P/G armour)", y=1.0)
    fig.tight_layout()
    table = pd.DataFrame([dict(roster=r["roster"][:8], char=r["char"][:8], tier="Serca" if r["tier"] == 1 else "T4", days=round(r["days"], 1),
                               days_plotted=len(r["t"]), honing="; ".join(f"{at:%m-%d %H:%M} {txt}" for at, txt in r["ev"])) for r in rows])
    return fig, table


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


PLOTS = dict(uploads=uploads, materials=materials, raids=raids)

if __name__ == "__main__":
    args = sys.argv[1:]
    d = load("--dev" in args)
    if "--roster" in args:
        prefix = args[args.index("--roster") + 1]
        for k in ("snaps", "mats", "chests", "raids"):
            d[k] = d[k][d[k].roster.str.startswith(prefix)]
    bands = args[args.index("--bands") + 1].split(",") if "--bands" in args else BANDS
    names = [a for a in args if a in PLOTS] or (["all"] if "all" in args else [])
    for name in PLOTS if names in ([], ["all"]) else names:
        fig, table = PLOTS[name](d, bands) if name == "materials" else PLOTS[name](d)
        if fig is None:
            print("no data for", name)
            continue
        save(fig, name)
        if table is not None:
            print(table.to_string(index=False))
