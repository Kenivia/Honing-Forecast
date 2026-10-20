"""
Writes frontend/Data/bonus_chests.json: the chest a raid gate offers for gold after its clear, as a choice of
nothing or its contents for that gold. Run after extract.py.

    python scanner_setup/game_files/bonus_chests.py
"""

import json

from game import ROOT, listed, table, text

OUT = ROOT / "frontend/Data/bonus_chests.json"
ITEMS = ROOT / "scanner_setup/templates/items.json"
DIFFICULTY = {0: "Normal", 1: "Hard", 2: "Nightmare", 4: "Solo", 5: "Extreme"}


# the item level a difficulty asks for, by its ClassifyIndex
def item_level(classify):
    row = table("ContentsRequirement").execute(
        "select OptimalItemLevelMin from ContentsRequirement where PrimaryKey = ?", (classify,)
    ).fetchone()
    return int(row[0]) if row else 0


# stage reward key -> (raid, difficulty, its order, item level); a zone has its name on one of its rows only
def names():
    gates = table("ZoneEpicGate").execute("select * from ZoneEpicGate where IsUse = 1").fetchall()
    raids = {}
    for gate in gates:
        raids.setdefault(gate["PrimaryKey"], text(gate["EnterUIName"]).strip() or text(gate["GroupIdTitle"]))
        raids[gate["PrimaryKey"]] = raids[gate["PrimaryKey"]] or text(gate["EnterUIName"]).strip()
    # the old legion raids are named in their description only
    for gate in gates:
        raids[gate["PrimaryKey"]] = raids[gate["PrimaryKey"]] or text(gate["EnterUIDesc"]).split("\n")[0].strip()
    out = {}
    for gate in gates:
        level = int(gate["SecondaryKey"])
        name = text(gate["ZoneLevelDisplayName"]) or DIFFICULTY.get(level, "")
        out[int(gate["StagePerRewardId"])] = (raids[gate["PrimaryKey"]], name, level, item_level(gate["ClassifyIndex"]))
    return out


# (label -> amount, what is not a material)
def contents(drop_index, labels):
    bag, others = {}, {}
    for base in table("DropBase").execute("select * from DropBase where PrimaryKey = ? order by SecondaryKey", (drop_index,)):
        times = int(base["Repetition"]) or 1
        # a smart drop, which is a core for the player's class
        if int(base["RewardType"]) == 1:
            others["Ark Grid Core"] = others.get("Ark Grid Core", 0) + 1
            continue
        rows = table("DropEntity").execute("select * from DropEntity where PrimaryKey = ?", (base["EntityIndex"],)).fetchall()
        titles = []
        for row in rows:
            if int(row["NormalId"]):
                titles.append(listed(row["NormalId"]).removesuffix(" (Bound)"))
            elif int(row["NormalMoneyType"]):
                name = table("Money").execute("select Name from Money where PrimaryKey = ?", (row["NormalMoneyType"],)).fetchone()
                titles.append(text(name[0]))
            else:
                titles.append("Class item")
        amount = int(rows[0]["NormalMinCount"]) * times if rows else 0
        # one of several is a draw, never a material here
        if len(rows) == 1 and titles[0] in labels:
            bag[labels[titles[0]]] = bag.get(labels[titles[0]], 0) + amount
        elif rows:
            title = " / ".join(titles)
            others[title] = others.get(title, 0) + (times if len(rows) > 1 else amount)
    return bag, [f"{title} x{count}" for title, count in others.items()]


def main():
    labels = {row["title"]: row["label"] for row in json.loads(ITEMS.read_text(encoding="utf-8"))}
    raid_names = names()
    rows = {}
    query = "select * from StagePerReward where PaidRewardDropIndex != 0 order by PrimaryKey desc, GateIndex"
    for stage in table("StagePerReward").execute(query):
        if int(stage["PrimaryKey"]) not in raid_names:
            continue
        raid, *level = raid_names[int(stage["PrimaryKey"])]
        cost = table("PaidActionCost").execute(
            "select GameMoneyAmount from PaidActionCost where PrimaryKey = ? and SecondaryKey = ?",
            (stage["PaidRewardType"], stage["PaidRewardCostSecondaryKey"]),
        ).fetchone()[0]
        bag, others = contents(stage["PaidRewardDropIndex"], labels)
        if not set(bag) - {"Gold", "Silver"}:
            continue
        option = {"Gold": -int(cost), **bag}
        # difficulties that sell the same chest are one row
        key = (raid, int(stage["GateIndex"]), json.dumps(option, sort_keys=True))
        if key in rows:
            rows[key]["levels"].append(level)
            continue
        rows[key] = {
            "id": f"bonus {stage['PrimaryKey']} {stage['GateIndex']}",
            "group": raid,
            "levels": [level],
            "gate": int(stage["GateIndex"]),
            "band": 0,
            "options": [{}, option],
            "others": others,
        }
    out = []
    for row in rows.values():
        levels = sorted(row.pop("levels"), key=lambda level: level[1])
        difficulty = " / ".join(name for name, *_ in levels)
        # difficulties merged sell the same chest and ask for the same level
        title = f"Gate {row.pop('gate')}"
        out.append({**row, "title": title, "difficulty": difficulty, "item_level": min(level[2] for level in levels)})
    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(out)} choices to {OUT}")


if __name__ == "__main__":
    main()
