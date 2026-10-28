"""
Writes frontend/Data/income.json: what the client says recurring content gives, for the Income page's "Auto" income.
Raid clears (gold by band, free materials, the bonus chest), the current Paradise Crucible season's stages with their
unlock dates, the daily playtime reward, the weekly mission reward choice and the shops the page buys from. Run after
scanner_setup/game_files/extract.py.

    python scripts/game_files/income.py

An item is {name, options: [{name, bag}]}: a select-one chest is one option per pick, anything else one option.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scanner_setup/game_files"))
from bonus_chests import names
from game import ROOT, box, listed, table, text

OUT = ROOT / "frontend/Data/income.json"
ITEMS = ROOT / "scanner_setup/templates/items.json"
MONEY = {1: "Silver", 2: "Gold", 18: "Shards"}  # Money.PrimaryKey
GOLD_BANDS = {2: 2, 4: 1, 5: 0}  # DropEntity.NormalType of gold -> band (char, roster, tradable)
MIN_LEVEL = 1600
GUILD = 889019  # Sylmael Bloodstone
MEDAL = 52325000  # Clear Medal
GUILD_RESEARCH = 6  # the guild shop rows of the highest research level
GUILD_ITEMS = {71100020, 66102006, 66102106, 66102007, 66102107}  # breath chest, destruction and guardian stones
EVENT_SHOPS = [10025]  # DirectShopMeta of the live event shops worth buying from

LABELS = {row["title"]: row["label"] for row in json.loads(ITEMS.read_text(encoding="utf-8"))}


def plain(title):
    return title.removesuffix(" (Bound)")


def add(a, b, times=1):
    out = dict(a)
    for label, amount in b.items():
        out[label] = out.get(label, 0) + amount * times
    return out


def gold_of(item_id):
    row = table("Item").execute("select GainMoneyType, UseGainCount from Item where PrimaryKey = ?", (item_id,)).fetchone()
    return int(row[1]) if row and int(row[0]) == 2 else 0


# what an item gives, as items; a random chest is its average
def expand(item_id, count):
    if gold_of(item_id):
        return [fixed(listed(item_id), {"Gold": gold_of(item_id) * count})]
    kind, inside, _ = box(item_id)
    title = plain(listed(item_id))
    if kind is None:
        label = LABELS.get(title)
        return [fixed(title, {label: count})] if label else []
    if kind == "SelectOne":
        options = [{"name": f"{plain(t)} x{n * count}", "bag": flat(i, t, n * count)} for i, t, n in inside]
        return [{"name": f"{title} x{count}", "options": options}] if any(o["bag"] for o in options) else []
    bags = [flat(i, t, n * count) for i, t, n in inside]
    if kind == "Random":
        bag = {}
        for one in bags:
            bag = add(bag, one, 1 / len(bags))
        return [fixed(f"{title} x{count}", bag)] if bag else []
    out = []
    for i, t, n in inside:
        out += expand(i, n * count) if i else [fixed(t, money(t, n * count))]
    return [item for item in out if any(o["bag"] for o in item["options"])]


def fixed(name, bag):
    return {"name": name, "options": [{"name": name, "bag": bag}]}


def money(title, amount):
    label = {"Silver": "Silver", "Gold": "Gold", "Destiny Shard": "Shards"}.get(title)
    return {label: amount} if label else {}


# one bag for an option, the first pick of anything nested
def flat(item_id, title, count):
    if not item_id:
        return money(title, count)
    bag = {}
    for item in expand(item_id, count):
        bag = add(bag, item["options"][0]["bag"])
    return bag


# a drop's (materials, gold by band); Clear Medals, which the medal shop takes, under "Clear Medal"
def drop(index):
    bag, gold = {}, [0, 0, 0]
    for base in table("DropBase").execute("select * from DropBase where PrimaryKey = ?", (index,)):
        times = int(base["Repetition"]) or 1
        rows = table("DropEntity").execute("select * from DropEntity where PrimaryKey = ?", (base["EntityIndex"],)).fetchall()
        # several rows are a draw of one, never a material here
        if int(base["RewardType"]) == 1 or len(rows) != 1:
            continue
        row = rows[0]
        amount = int(row["NormalMinCount"]) * times
        if int(row["NormalId"]):
            label = "Clear Medal" if int(row["NormalId"]) == MEDAL else LABELS.get(plain(listed(row["NormalId"])))
            if label:
                bag[label] = bag.get(label, 0) + amount
        elif int(row["NormalMoneyType"]) == 2:
            gold[GOLD_BANDS[int(row["NormalType"])]] += amount
        elif int(row["NormalMoneyType"]) in MONEY:
            label = MONEY[int(row["NormalMoneyType"])]
            bag[label] = bag.get(label, 0) + amount
    return bag, gold


def raids():
    raid_names = names()
    out = {}
    query = "select * from StagePerReward where RewardDropIndex != 0 order by PrimaryKey, GateIndex"
    for stage in table("StagePerReward").execute(query):
        if int(stage["PrimaryKey"]) not in raid_names:
            continue
        raid, difficulty, _, level = raid_names[int(stage["PrimaryKey"])]
        if level < MIN_LEVEL:
            continue
        mats, gold = drop(stage["RewardDropIndex"])
        gate = {"gate": int(stage["GateIndex"]), "gold": gold, "medals": mats.pop("Clear Medal", 0), "mats": mats}
        if int(stage["PaidRewardDropIndex"]):
            cost = table("PaidActionCost").execute(
                "select GameMoneyAmount from PaidActionCost where PrimaryKey = ? and SecondaryKey = ?",
                (stage["PaidRewardType"], stage["PaidRewardCostSecondaryKey"]),
            ).fetchone()[0]
            gate["bonus"] = {"cost": int(cost), "mats": drop(stage["PaidRewardDropIndex"])[0]}
        key = (raid, difficulty)
        out.setdefault(key, {"group": raid, "difficulty": difficulty, "item_level": level, "gates": []})["gates"].append(gate)
    return sorted(out.values(), key=lambda raid: -raid["item_level"])


def date(year, month, day):
    return f"{int(year):04}-{int(month):02}-{int(day):02}" if int(year) else None


def crucible():
    season = table("ContentsSeason").execute(
        "select * from ContentsSeason where Divine != 0 order by Divine desc"
    ).fetchone()
    stages = []
    query = "select * from TrinityDivine where PrimaryKey = ? order by SecondaryKey"
    for row in table("TrinityDivine").execute(query, (season["Divine"],)):
        phase = table("TrinityDivineEnergyPhase").execute(
            "select * from TrinityDivineEnergyPhase where PrimaryKey = ?", (row["EnergyPhaseId"],)
        ).fetchone()
        items = []
        for k in range(1, 6):
            if int(phase[f"RewardItemId{k}"]):
                items += expand(int(phase[f"RewardItemId{k}"]), int(phase[f"RewardItemCount{k}"]))
        stages.append({
            "stage": int(row["SecondaryKey"]),
            "opens": date(row["ContentsOpenYear"], row["ContentsOpenMonth"], row["ContentsOpenDay"]),
            "item_level": int(row["ConditionItemLevel"]),
            "items": items,
        })
    return {
        "start": date(season["StartYear"], season["StartMonth"], season["StartDay"]),
        "end": date(season["EndYear"], season["EndMonth"], season["EndDay"]),
        "stages": stages,
    }


def playtime():
    out = []
    for row in table("PCRoomDailyReward").execute("select * from PCRoomDailyReward order by PlayTime"):
        for k in ["", "2"]:
            if int(row[f"RewardItemId{k}"]):
                out += expand(int(row[f"RewardItemId{k}"]), int(row[f"RewardItemCount{k}"]))
            if int(row[f"RewardMoneyType{k}"]) in MONEY:
                out.append(fixed(MONEY[int(row[f"RewardMoneyType{k}"])], {MONEY[int(row[f"RewardMoneyType{k}"])]: int(row[f"RewardMoneyValue{k}"])}))
    return out


# the weekly mission reward: one choice, the same at every item level
def weekly_mission():
    band = table("GuideWeeklyMissionReward").execute("select min(PrimaryKey) from GuideWeeklyMissionReward").fetchone()[0]
    options = []
    for row in table("GuideWeeklyMissionReward").execute("select * from GuideWeeklyMissionReward where PrimaryKey = ? order by SecondaryKey", (band,)):
        bag, _ = drop(row["RewardItemDropIndex1"])
        items = [entity for base in table("DropBase").execute("select * from DropBase where PrimaryKey = ?", (row["RewardItemDropIndex1"],))
                 for entity in table("DropEntity").execute("select * from DropEntity where PrimaryKey = ?", (base["EntityIndex"],))]
        name = ", ".join(f"{plain(listed(e['NormalId']))} x{e['NormalMinCount']}" for e in items if int(e["NormalId"]))
        for e in items:
            if int(e["NormalId"]) and not LABELS.get(plain(listed(e["NormalId"]))):
                bag = add(bag, flat(int(e["NormalId"]), "", int(e["NormalMinCount"])))
        options.append({"name": name, "bag": bag})
    return [{"name": "Weekly mission reward", "options": options}]


def shop_row(row, shop, start=None, end=None):
    limit = table("AcquireLimit").execute("select * from AcquireLimit where PrimaryKey = ?", (row["ShopLimitIndex"],)).fetchone()
    items = expand(int(row["ItemIndex"]), int(row["BundleAmount"]))
    if not items or not limit:
        return None
    level = int(row["ConditionValue1_1"]) if int(row["ConditionType1"]) == 1 else 0
    return {
        "shop": shop,
        "name": f"{plain(listed(row['ItemIndex']))} x{row['BundleAmount']}",
        "item_level": level,
        "limit": int(limit["LimitCount"]),
        "weekly": int(limit["RefreshType"]) == 2,
        "per_roster": int(limit["ExpeditionLimit"]) == 1,
        "cost": int(row["BarterQuantity"]),
        "currency": plain(listed(row["BarterItemIndex"])),
        **({"start": start, "end": end} if start else {}),
        "items": items,
    }


def shops():
    out = []
    for row in table("DirectShop").execute("select * from DirectShop where BarterItemIndex = ? and ConditionType2 = 2 and ConditionValue2_2 = ?", (GUILD, GUILD_RESEARCH)):
        if int(row["ItemIndex"]) in GUILD_ITEMS:
            out.append(shop_row(row, "Guild shop"))
    seen = set()
    for row in table("DirectShop").execute("select * from DirectShop where BarterItemIndex = ? order by SortOrder", (MEDAL,)):
        key = (int(row["ItemIndex"]), int(row["BundleAmount"]), int(row["ConditionValue1_1"]))
        if key not in seen:
            seen.add(key)
            out.append(shop_row(row, "Clear Medal shop"))
    for meta in EVENT_SHOPS:
        info = table("DirectShopMeta").execute("select * from DirectShopMeta where PrimaryKey = ?", (meta,)).fetchone()
        for row in table("DirectShop").execute("select * from DirectShop where ShopMetaId = ? order by ShopTabIndex, SortOrder", (meta,)):
            out.append(shop_row(row, text(info["ShopName"]) or f"Event shop {meta}", info["StartDate"], info["EndDate"]))
    return [row for row in out if row]


def main():
    out = {
        "raids": raids(),
        "crucible": crucible(),
        "playtime": playtime(),
        "weekly_mission": weekly_mission(),
        "shops": shops(),
    }
    OUT.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"{len(out['raids'])} raid difficulties, {len(out['crucible']['stages'])} stages, {len(out['shops'])} shop rows to {OUT}")


if __name__ == "__main__":
    main()
