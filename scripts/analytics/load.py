# Loads every pull in ignored/analytics/ (pnpm pull-analytics) into tidy DataFrames.
import glob
import hashlib
import json
import os

import pandas as pd

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
DATA = os.path.join(ROOT, "ignored", "analytics")
FIGURES = os.path.join(DATA, "figures")
BANDS = ["bound", "roster", "tradable"]


def raw_uploads(include_dev=False):
    rows = []
    for f in sorted(glob.glob(os.path.join(DATA, "uploads-*.json"))):
        for r in json.load(open(f, encoding="utf8")):
            r["file"] = os.path.basename(f)
            rows.append(r)
    # follow-ups are "followup <version>"
    return rows if include_dev else [r for r in rows if not r["version"].endswith("dev")]


def content_hash(snap):
    # roster ids differ when one save was migrated (v9 -> v10) in two places; char ids and `at` do not
    s = {k: v for k, v in snap.items() if k not in ("roster_id", "sent")}
    return hashlib.md5(json.dumps(s, sort_keys=True).encode()).hexdigest()


def price_of(snap, label):
    b = snap["bundles"].get(label, 1)
    return snap["prices"].get(f"{label} x{b}", 0) / b


def load(include_dev=False):
    uploads = raw_uploads(include_dev)
    snaps, mats, chests, raids, gear, seen = [], [], [], [], [], set()
    for u in uploads:
        for s in u["snapshots"]:
            h = content_hash(s)
            if h in seen:
                continue
            seen.add(h)
            base = dict(roster=u["roster"], region=u["region"], version=u["version"], at=s["at"], hash=h)
            snaps.append(dict(base, upload=u["id"], received=u["received"], kind=s["kind"],
                              chars=len(s["chars"]), ignored=bool(s.get("ignored")),
                              has_grids=any("normal_grid" in c for c in s["chars"].values())))
            cols = [("roster", None, s["roster_owned"]), ("tradable", None, s["tradable"])]
            cols += [(band, cid, c[band]) for cid, c in s["chars"].items() for band in ("bound", "special")]
            for band, cid, col in cols:
                off = s["chars"][cid].get("disabled", []) if band == "bound" else []
                for label, n in col.items():
                    mats.append(dict(base, kind=s["kind"], char=cid, band=band, label=label, amount=n,
                                     gold=n * price_of(s, label), disabled=label in off))
            for cid, c in [(None, {"chests": s["roster_chests"]})] + list(s["chars"].items()):
                for ch in c["chests"]:
                    chests.append(dict(base, char=cid, title=ch["title"], band=ch["band"], count=ch["count"],
                                       obtain_all=ch["obtain_all"], options=len(ch["options"])))
        # one row per item per loadout, or one with the reason there is no gear
        for cid, g in (u.get("gear") or {}).items():
            row = dict(roster=u["roster"], region=u["region"], char=cid, upload=u["id"], received=u["received"],
                       followup=u["version"].startswith("followup"))
            if isinstance(g, str):
                gear.append(dict(row, reason=g))
                continue
            for lo in g["loadouts"]:
                for it in lo["items"]:
                    gear.append(dict(row, reason="gear", ilvl=g["ilvl"], refreshed=g["refreshed"], loadout=lo["kind"],
                                     updated=lo["updated"], loadout_ilvl=lo["ilvl"], **it))
        for cid, v in u["raids"].items():
            if isinstance(v, str):
                raids.append(dict(roster=u["roster"], region=u["region"], char=cid, upload=u["id"], reason=v))
                continue
            for log in v:
                raids.append(dict(roster=u["roster"], region=u["region"], char=cid, upload=u["id"], reason="logs", **log))
    raids = pd.DataFrame(raids)
    if "week" in raids:
        # overlapping uploads repeat the same logs
        logs = raids[raids.reason == "logs"].drop(columns="upload").drop_duplicates()
        raids = pd.concat([logs, raids[raids.reason != "logs"].drop_duplicates(["char", "reason"])])
        raids["week"] = pd.to_datetime(raids["week"], unit="ms")
    out = dict(uploads=uploads, snaps=pd.DataFrame(snaps), mats=pd.DataFrame(mats),
               chests=pd.DataFrame(chests), raids=raids, gear=pd.DataFrame(gear))
    for k in ("received", "refreshed", "updated"):
        if k in out["gear"]:
            out["gear"][k] = pd.to_datetime(out["gear"][k], unit="ms")
    for k in ("snaps", "mats", "chests"):
        if len(out[k]):
            out[k]["time"] = pd.to_datetime(out[k]["at"], unit="ms")
    return out


if __name__ == "__main__":
    d = load()
    for k in ("snaps", "mats", "chests", "raids", "gear"):
        print(k, d[k].shape)
