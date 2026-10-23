# Sanity report over every pull: run after each `pnpm pull-analytics`.
import collections
import sys

from load import content_hash, load, raw_uploads

include_dev = "--dev" in sys.argv
uploads = raw_uploads(include_dev)
d = load(include_dev)
snaps, mats, raids = d["snaps"], d["mats"], d["raids"]

print(f"{len(uploads)} uploads, {snaps.roster.nunique()} rosters, {len(snaps)} unique snapshots")
print("by version:", dict(collections.Counter(u["version"] for u in uploads)))
print("by region:", snaps.drop_duplicates("roster").region.value_counts().to_dict())

print("\n# duplicates")
hashes = collections.defaultdict(set)
char_rosters = collections.defaultdict(set)
for u in uploads:
    for s in u["snapshots"]:
        hashes[content_hash(s)].add(u["roster"][:8])
        for cid in s["chars"]:
            char_rosters[cid].add(u["roster"][:8])
total = sum(len(u["snapshots"]) for u in uploads)
print(f"resent snapshots dropped: {total - len(snaps)}")
print("same snapshot under several rosters:", collections.Counter(tuple(sorted(v)) for v in hashes.values() if len(v) > 1))
print("char ids in several rosters:", collections.Counter(tuple(sorted(v)) for v in char_rosters.values() if len(v) > 1))

print("\n# privacy")
leaks = [(u["id"], cid) for u in uploads for s in u["snapshots"] for cid, c in s["chars"].items() if c.get("name")]
print("names left in snapshots:", leaks or "none")
bad = [u["id"] for u in uploads for v in u["raids"].values() if not isinstance(v, str)
       for log in v if set(log) != {"boss", "difficulty", "isBus", "gearScore", "class", "week"}]
print("raid logs with unexpected keys:", bad or "none")

print("\n# coverage")
print("snapshots with tier/grids:", f"{snaps.has_grids.sum()}/{len(snaps)}")
print("snapshot kinds:", snaps.kind.value_counts().to_dict())
per_roster = snaps.groupby("roster").agg(snaps=("at", "size"), uploads=("upload", "nunique"), chars=("chars", "max"),
                                          first=("time", "min"), last=("time", "max"))
per_roster.index = per_roster.index.str[:8]
print(per_roster.to_string())

print("\n# raids")
reasons = raids.drop_duplicates(["char", "reason"]).groupby("reason").char.nunique()
print("chars by reason:", reasons.to_dict())
for roster, g in raids.groupby("roster"):
    if (g.reason != "logs").all():
        print(f"  roster {roster[:8]} ({g.region.iloc[0]}): no logs at all, {g.reason.value_counts().to_dict()}")
if "week" in raids:
    logs = raids[raids.reason == "logs"]
    off = logs[(logs.week.dt.dayofweek != 3) | (logs.week.dt.hour != 10)]
    print(f"{len(logs)} unique logs, {len(off)} with week not Thu 10:00 UTC")

print("\n# values")
print("negative amounts:", mats[mats.amount < 0][["roster", "band", "label", "amount"]].to_string() if (mats.amount < 0).any() else "none")
capped = mats[mats.amount >= 999_999_999]
print("amounts at 999,999,999 (cap / typed placeholder):", capped.groupby(["band", "label"]).roster.nunique().to_dict() or "none")
big = mats[(mats.label != "Silver") & (mats.gold > 5e6)].drop_duplicates(["roster", "char", "band", "label", "amount"])
big = big.assign(roster=big.roster.str[:8], char=big.char.str[:6])[["roster", "char", "band", "label", "amount", "gold"]]
print("non-silver cells worth > 5M gold:\n" + (big.to_string(index=False) if len(big) else "none"))
skew = (snaps["at"] - snaps.received) / 1000
print(f"snapshot at - received (s): max {skew.max():.1f} (positive = client clock ahead)")
