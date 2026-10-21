"""
Rebuilds the normal honing of the Serca constants JSON from the game's tables, books included, says which
fields differ from the JSON it starts from, and writes the result. Advanced honing and the EVENT_* rows are
kept from the base. Run after scanner_setup/game_files/extract.py.

    python scripts/game_files/honing.py [base json] [out json]

T4 is not done: its base tables are undiscounted (AI notes/Calculator core/Domain model.md).
"""

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scanner_setup/game_files"))
from game import ROOT, table, text

CONSTANTS = ROOT / "crates/core/src/constants"
BASE = CONSTANTS / "Serca Aug 2026 (Vambrace).json"
OUT = CONSTANTS / "Serca Oct 2026.json"
GROUPS = {0: 411101, 1: 411100, 2: 411102}  # piece type (armour, weapon, vambrace) -> ItemEnhanceCommon group
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


def name(item_id):
    row = table("Item").execute("select Name from Item where PrimaryKey = ?", (item_id,)).fetchone()
    return text(row[0]) if row else ""


def cost_row(item_id):
    title = name(item_id)
    for word, row in [("Destruction", 0), ("Guardian", 1), ("Leapstone", 2), ("Fusion", 4)]:
        if word in title:
            return row
    raise ValueError(title)


def piece(piece_type, group):
    levels = {r["SecondaryKey"]: r for r in table("ItemEnhanceCommon").execute("select * from ItemEnhanceCommon where PrimaryKey = ?", (group,))}
    # every item of the group lists the same stones; take the first
    first = table("ItemQualityOption").execute("select PrimaryKey from ItemQualityOption where EnhanceCommonId = ?", (group,)).fetchone()[0]
    quality = {r["SecondaryKey"]: r for r in table("ItemQualityOption").execute("select * from ItemQualityOption where PrimaryKey = ?", (first,))}
    chances, special, juice = [], [], []
    cost, unlock = [[0] * 25 for _ in ROWS], [[0] * 25 for _ in ROWS]
    for n in range(1, 26):
        row, prev, q = levels[100 + n], levels[99 + n], quality[100 + n]
        # artisan is a tenth of the base chance up to 21,500, as the core assumes
        assert row["LevelUpFailureBonusRate"] * 10 == row["LevelUpSuccessRate"] and row["LevelUpGuaranteeThreshold"] == 21500
        chances.append(row["LevelUpSuccessRate"] / 10000)
        special.append(row["SpecialEnhanceMaterialAmount"])
        for k in range(3):
            cost[MONEY_ROW[row[f"LevelUpCostType0{k}"]]][n - 1] = row[f"LevelUpCostAmount0{k}"]
        for k in range(5):
            if q[f"EnhanceLevelUpMaterialId0{k}"]:
                cost[cost_row(q[f"EnhanceLevelUpMaterialId0{k}"])][n - 1] = q[f"EnhanceLevelUpMaterialCount0{k}"]
        # unlocking is shards fed as experience, with silver per shard
        exp = row["AccExp"] - prev["AccExp"]
        unlock[3][n - 1], unlock[6][n - 1] = exp, exp * row["FeedSubCost"]
        # juice rates are out of 100,000 since the October 2026 update
        for k in range(3):
            if row[f"LevelUpAdditiveId0{k}"]:
                count = row[f"LevelUpAdditiveMax0{k}"]
                chance = min(row[f"LevelUpAdditiveRate0{k}"] * count, row["LevelUpAdditiveMaxRate"]) / 100000
                juice.append([JUICES[row[f"LevelUpAdditiveId0{k}"]], 0, piece_type, n, round(chance, 6), count, 0, 0])
        books = table("ItemEnhanceMaterial").execute("select * from ItemEnhanceMaterial where PrimaryKey = ?", (row["SpecialMaterialGroupId"],))
        for book in {(re.sub(r" \(Bound\)$", "", name(b["SecondaryKey"])), b["EffectType00"], b["EffectArg00"]) for b in books}:
            assert book[1] == 1 and book[0] in BOOKS, book
            juice.append([BOOKS[book[0]], 0, piece_type, n, book[2] / 10000, 1, 0, 0])
    return chances, special, cost, unlock, juice


def main():
    base_path = Path(sys.argv[1]) if len(sys.argv) > 1 else BASE
    out_path = Path(sys.argv[2]) if len(sys.argv) > 2 else OUT
    base = json.loads(base_path.read_text(encoding="utf-8"))
    out = dict(base)
    out["NORMAL_CHANCES"], out["SPECIAL_LEAPS_COST"], normal = [], [], []
    for piece_type, group in GROUPS.items():
        chances, special, cost, unlock, juice = piece(piece_type, group)
        out["NORMAL_CHANCES"].append(chances)
        out["SPECIAL_LEAPS_COST"].append(special)
        out[f"NORMAL_{PIECES[piece_type]}_COST"] = cost
        out[f"NORMAL_{PIECES[piece_type]}_UNLOCK"] = unlock
        normal += juice
    # the core wants ids in first-seen order
    adv = [row for row in base["JUICE_BOOKS_AVAIL"] if row[1] == 1]
    out["JUICE_BOOKS_AVAIL"] = sorted(normal, key=lambda row: (row[0], row[2], row[3])) + adv
    for key in out:
        same = out[key] == base[key]
        if key == "JUICE_BOOKS_AVAIL":
            same = sorted(map(tuple, out[key])) == sorted(map(tuple, base[key]))
        print(f"{key}: {'same' if same else 'CHANGED'}")
    new = {tuple(row) for row in out["JUICE_BOOKS_AVAIL"]} - {tuple(row) for row in base["JUICE_BOOKS_AVAIL"]}
    gone = {tuple(row) for row in base["JUICE_BOOKS_AVAIL"]} - {tuple(row) for row in out["JUICE_BOOKS_AVAIL"]}
    print(f"juice rows: {len(new)} new (ids {sorted({row[0] for row in new})}), {len(gone)} gone")
    out_path.write_text(json.dumps(out, separators=(",", ":")), encoding="utf-8", newline="\n")
    print(f"wrote {out_path}")


if __name__ == "__main__":
    main()
