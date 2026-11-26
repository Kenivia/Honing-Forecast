"""
Rebuilds the normal honing of the constants JSONs from the game's tables, says which fields differ from the JSON
each starts from, and writes the result. Serca gets everything normal honing has; T4 only its rates (its costs
are discounted, AI notes/Calculator core/Further learnings/Honing data in the game files.md). The rest is kept from the base, with the chances
turned into the game's integers. Run after scanner_setup/game_files/extract.py.

    python scripts/game_files/honing.py
"""

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scanner_setup/game_files"))
from game import ROOT, table, text

CONSTANTS = ROOT / "crates/core/src/constants"
SERCA = (CONSTANTS / "Serca Aug 2026 (Vambrace).json", CONSTANTS / "Serca Oct 2026.json")
T4 = (CONSTANTS / "T4 July 2026 (Vambrace padded).json", CONSTANTS / "T4 Nov 2026.json")
# piece type (armour, weapon, vambrace) -> ItemEnhanceCommon group; T4 has no vambrace
SERCA_GROUPS = {0: 411101, 1: 411100, 2: 411102}
T4_GROUPS = {0: 410501, 1: 410500}
PIECES = {0: "ARMOR", 1: "WEAPON", 2: "VAMBRACE"}
ROWS = ["Red", "Blue", "Leaps", "Shards", "Fusion", "Gold", "Silver"]
MONEY_ROW = {18: 3, 2: 5, 1: 6}  # LevelUpCostType: shards, gold, silver
JUICES = {66111132: 0, 66111131: 1}  # Glacier's, Lava's
# juice ids from 2 on, in the order of the frontend's MATERIAL_TABLE
BOOKS = {
    "Tailoring: Tremor [12-15]": 2,
    "Metallurgy: Tremor [12-15]": 3,
    "Tailoring: Tremor [16-19]": 4,
    "Metallurgy: Tremor [16-19]": 5,
}
RATES = ["NORMAL_CHANCES", "NORMAL_FAIL_BONUS", "NORMAL_FAIL_BONUS_MAX", "ARTISAN_THRESHOLD"]


def name(item_id):
    row = table("Item").execute("select Name from Item where PrimaryKey = ?", (item_id,)).fetchone()
    return text(row[0]) if row else ""


def cost_row(item_id):
    title = name(item_id)
    for word, row in [("Destruction", 0), ("Guardian", 1), ("Leapstone", 2), ("Fusion", 4)]:
        if word in title:
            return row
    raise ValueError(title)


def whole(value, scale):
    out = round(value * scale)
    assert abs(out - value * scale) < 1e-6, value
    return out


def levels(group):
    return {r["SecondaryKey"]: r for r in table("ItemEnhanceCommon").execute("select * from ItemEnhanceCommon where PrimaryKey = ?", (group,))}


def rates(group):
    # out of 10,000, as in the game
    rows = [levels(group)[100 + n] for n in range(1, 26)]
    return [[r[col] for r in rows] for col in ["LevelUpSuccessRate", "LevelUpFailureBonusRate", "LevelUpFailureBonusMaxRate", "LevelUpGuaranteeThreshold"]]


def piece(piece_type, group):
    rows = levels(group)
    # every item of the group lists the same stones; take the first
    first = table("ItemQualityOption").execute("select PrimaryKey from ItemQualityOption where EnhanceCommonId = ?", (group,)).fetchone()[0]
    quality = {r["SecondaryKey"]: r for r in table("ItemQualityOption").execute("select * from ItemQualityOption where PrimaryKey = ?", (first,))}
    special, juice = [], []
    cost, unlock = [[0] * 25 for _ in ROWS], [[0] * 25 for _ in ROWS]
    for n in range(1, 26):
        row, prev, q = rows[100 + n], rows[99 + n], quality[100 + n]
        special.append(row["SpecialEnhanceMaterialAmount"])
        for k in range(3):
            cost[MONEY_ROW[row[f"LevelUpCostType0{k}"]]][n - 1] = row[f"LevelUpCostAmount0{k}"]
        for k in range(5):
            if q[f"EnhanceLevelUpMaterialId0{k}"]:
                cost[cost_row(q[f"EnhanceLevelUpMaterialId0{k}"])][n - 1] = q[f"EnhanceLevelUpMaterialCount0{k}"]
        # unlocking is shards fed as experience, with silver per shard
        exp = row["AccExp"] - prev["AccExp"]
        unlock[3][n - 1], unlock[6][n - 1] = exp, exp * row["FeedSubCost"]
        # juice and book chances out of 100,000 (juice is stored so; books are out of 10,000)
        for k in range(3):
            if row[f"LevelUpAdditiveId0{k}"]:
                count = row[f"LevelUpAdditiveMax0{k}"]
                chance = min(row[f"LevelUpAdditiveRate0{k}"] * count, row["LevelUpAdditiveMaxRate"])
                juice.append([JUICES[row[f"LevelUpAdditiveId0{k}"]], 0, piece_type, n, chance, count, 0, 0])
        books = table("ItemEnhanceMaterial").execute("select * from ItemEnhanceMaterial where PrimaryKey = ?", (row["SpecialMaterialGroupId"],))
        for book in {(re.sub(r" \(Bound\)$", "", name(b["SecondaryKey"])), b["EffectType00"], b["EffectArg00"]) for b in books}:
            assert book[1] == 1 and book[0] in BOOKS, book
            juice.append([BOOKS[book[0]], 0, piece_type, n, book[2] * 10, 1, 0, 0])
    return special, cost, unlock, juice


def integers(base):
    """The base's float chances as the integers the core reads."""
    out = dict(base)
    out["NORMAL_CHANCES"] = [[whole(x, 10000) for x in row] for row in base["NORMAL_CHANCES"]]
    out["NORMAL_FAIL_BONUS"] = [[x // 10 for x in row] for row in out["NORMAL_CHANCES"]]
    out["NORMAL_FAIL_BONUS_MAX"] = [list(row) for row in out["NORMAL_CHANCES"]]
    out["ARTISAN_THRESHOLD"] = [[21500] * 25 for _ in out["NORMAL_CHANCES"]]
    out["EVENT_NORMAL_EXTRA_CHANCE"] = [[whole(x, 10000) for x in row] for row in base["EVENT_NORMAL_EXTRA_CHANCE"]]
    out["EVENT_ARTISAN_MULTIPLIER"] = [[whole(x, 100) for x in row] for row in base["EVENT_ARTISAN_MULTIPLIER"]]
    out["JUICE_BOOKS_AVAIL"] = [row[:4] + [whole(row[4], 100000)] + row[5:] for row in base["JUICE_BOOKS_AVAIL"]]
    return out


def report(out, base):
    for key in out:
        if key not in base:
            print(f"{key}: NEW")
            continue
        same = out[key] == base[key]
        if key == "JUICE_BOOKS_AVAIL":
            same = sorted(map(tuple, out[key])) == sorted(map(tuple, base[key]))
        print(f"{key}: {'same' if same else 'CHANGED'}")


def serca(base):
    out = integers(base)
    out["SPECIAL_LEAPS_COST"], normal = [], []
    for piece_type, group in SERCA_GROUPS.items():
        for key, value in zip(RATES, rates(group)):
            out[key][piece_type] = value
        special, cost, unlock, juice = piece(piece_type, group)
        out["SPECIAL_LEAPS_COST"].append(special)
        out[f"NORMAL_{PIECES[piece_type]}_COST"] = cost
        out[f"NORMAL_{PIECES[piece_type]}_UNLOCK"] = unlock
        normal += juice
    # the core wants ids in first-seen order
    adv = [row for row in out["JUICE_BOOKS_AVAIL"] if row[1] == 1]
    old = out["JUICE_BOOKS_AVAIL"]
    out["JUICE_BOOKS_AVAIL"] = sorted(normal, key=lambda row: (row[0], row[2], row[3])) + adv
    new = {tuple(row) for row in out["JUICE_BOOKS_AVAIL"]} - {tuple(row) for row in old}
    gone = {tuple(row) for row in old} - {tuple(row) for row in out["JUICE_BOOKS_AVAIL"]}
    print(f"juice rows: {len(new)} new (ids {sorted({row[0] for row in new})}), {len(gone)} gone")
    return out


def t4(base):
    out = integers(base)
    for piece_type, group in T4_GROUPS.items():
        for key, value in zip(RATES, rates(group)):
            out[key][piece_type] = value
    return out


def main():
    for (base_path, out_path), build in [(SERCA, serca), (T4, t4)]:
        print(f"== {out_path.name}")
        base = json.loads(base_path.read_text(encoding="utf-8"))
        out = build(base)
        report(out, integers(base))
        out_path.write_text(json.dumps(out, separators=(",", ":")), encoding="utf-8", newline="\n")
        print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
