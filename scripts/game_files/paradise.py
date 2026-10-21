"""
Writes frontend/Data/paradise.json: the rewards of Paradise (TrinityInferno, "Hell" in the client) for the current
season, as choices for the Choices page. One choice per mode, item level and reward level; each option is one main
reward plus the additional rewards, and again with the Wealth ones instead. Run after
scanner_setup/game_files/extract.py.

    python scripts/game_files/paradise.py
"""

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scanner_setup/game_files"))
from game import ROOT, box, listed, table, text

OUT = ROOT / "frontend/Data/paradise.json"
ITEMS = ROOT / "scanner_setup/templates/items.json"
MODES = {2: "Destiny", 3: "Netherworld"}  # TrinityInferno.InfernoType; 1 is Destiny too, 4 Netherworld's frost keys
PERMANENT = 19  # ClassifyType of the keys sold all season; the others are events
LEVELS = [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]  # TrinityInfernoReward.SecondaryKey: Base, Level 1 to 10


def plain(title):
    return re.sub(r" \(Bound\)$", "", title)


def gold_of(item_id):
    row = table("Item").execute("select GainMoneyType, UseGainCount from Item where PrimaryKey = ?", (item_id,)).fetchone()
    return int(row[1]) if row and int(row[0]) == 2 else 0


# a drop as [(item id or None, title, amount)], a drop of several rows being one of them per repetition
def drop(index):
    out = []
    for base in table("DropBase").execute("select * from DropBase where PrimaryKey = ? order by SecondaryKey", (index,)):
        assert int(base["RewardType"]) != 1, index
        rows = table("DropEntity").execute("select * from DropEntity where PrimaryKey = ?", (base["EntityIndex"],)).fetchall()
        assert len(rows) == 1, index
        row, times = rows[0], int(base["Repetition"]) or 1
        amount = int(row["NormalMinCount"]) * times
        if int(row["NormalId"]):
            out.append((int(row["NormalId"]), plain(listed(row["NormalId"])), amount))
        else:
            name = table("Money").execute("select Name from Money where PrimaryKey = ?", (row["NormalMoneyType"],)).fetchone()
            out.append((None, text(name[0]), amount))
    return out


def bag_of(contents, labels):
    bag = {}
    for item_id, title, amount in contents:
        gold = gold_of(item_id) if item_id else 0
        label, amount = ("Gold", gold * amount) if gold else (labels.get(title), amount)
        if label:
            bag[label] = bag.get(label, 0) + amount
    return bag


def add(a, b):
    return {label: a.get(label, 0) + b.get(label, 0) for label in {**a, **b}}


# A main reward as [(name, what it gives)]: a select-one chest of materials is one option per material.
def options_of(contents, labels):
    if len(contents) == 1 and contents[0][0]:
        item_id, title, count = contents[0]
        kind, inside, _ = box(item_id)
        picks = [(plain(t), n * count) for _, t, n in inside if plain(t) in labels]
        if kind == "SelectOne" and picks:
            return [(f"{title} x{count}: {t} x{n}", {labels[t]: n}) for t, n in picks]
        if kind == "ObtainAll" and picks:
            return [(f"{title} x{count}", bag_of([(None, plain(t), n * count) for _, t, n in inside], labels))]
    return [(", ".join(f"{t} x{n}" for _, t, n in contents), bag_of(contents, labels))]


# (mode, item level) -> reward id; an item level has had several sets, the newest is the highest
def reward_sets():
    season = table("TrinityInferno").execute("select max(PrimaryKey) from TrinityInferno where ClassifyType = ?", (PERMANENT,)).fetchone()[0]
    out = {}
    query = "select * from TrinityInferno where ClassifyType = ? and PrimaryKey in (?, ?) and InfernoType in (2, 3)"
    for row in table("TrinityInferno").execute(query, (PERMANENT, season - 1, season)):
        level = table("Item").execute("select ReUseBalanceLevel from Item where PrimaryKey = ?", (row["SecondaryKey"],)).fetchone()[0]
        key = (MODES[row["InfernoType"]], int(level))
        out[key] = max(out.get(key, 0), int(row["InfernoRewardId"]))
    return out


def main():
    labels = {row["title"]: row["label"] for row in json.loads(ITEMS.read_text(encoding="utf-8"))}
    out = []
    for (mode, item_level), reward in sorted(reward_sets().items()):
        for index, level in enumerate(LEVELS):
            row = table("TrinityInfernoReward").execute(
                "select * from TrinityInfernoReward where PrimaryKey = ? and SecondaryKey = ?", (reward, level)
            ).fetchone()
            extra = bag_of(drop(row["SubDropId"]), labels) if int(row["SubDropId"]) else {}
            wealth = bag_of(drop(row["PlentyDropId"]), labels) if int(row["PlentyDropId"]) else {}
            names, options = [], []
            for i in range(1, 21):
                if int(row[f"MainDropId{i}"]) and int(row[f"Ratio{i}"]):
                    for name, bag in options_of(drop(row[f"MainDropId{i}"]), labels):
                        names.append(name)
                        options.append(bag)
            out.append(
                {
                    "id": f"paradise {reward} {level}",
                    "group": f"Paradise: {mode}",
                    "title": "Base" if index == 0 else f"Level {index}",
                    "difficulty": str(item_level),
                    "item_level": item_level,
                    "band": 0,
                    "names": names,
                    "options": [add(bag, extra) for bag in options],
                    "wealth": [add(bag, wealth) for bag in options],
                }
            )
    OUT.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(out)} choices to {OUT}")


if __name__ == "__main__":
    main()
