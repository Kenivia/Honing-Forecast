"""What the scripts here share: the tables and icon sheets extract.py pulls out of the installed game."""

import re
import sqlite3
import struct
from functools import cache, lru_cache
from html import unescape
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
TABLES = HERE / "tables"  # EFTable_<name>.db and IconInfo.loa
ATLAS = HERE / "atlas"  # the icon sheets, as umodel exports them
RARITIES = ["common", "uncommon", "rare", "epic", "legendary", "relic", "ancient", "sidereal"]
KINDS = {2: "Random", 3: "SelectOne"}  # RandomBoxBase.Type; the others obtain all


@cache
def table(name):
    connection = sqlite3.connect(f"file:{(TABLES / f'EFTable_{name}.db').as_posix()}?mode=ro", uri=True)
    connection.row_factory = sqlite3.Row
    return connection


# the English text behind a key, without its markup
def text(key):
    row = table("GameMsg").execute("select MSG from GameMsg_English where KEY = ?", (key,)).fetchone()
    return unescape(re.sub(r"<[^>]+>", "", row[0])).strip() if row else ""


# every item as (id, title, rarity, icon name)
@cache
def load_index():
    items = table("Item")
    items.execute(f"attach 'file:{(TABLES / 'EFTable_GameMsg.db').as_posix()}?mode=ro' as msg")
    rows = items.execute(
        "select i.PrimaryKey, m.MSG, i.Grade, lower(i.Icon || '_' || i.IconIndex) "
        "from Item i join msg.GameMsg_English m on m.KEY = i.Name"
    )
    return [(item_id, title.strip(), RARITIES[min(int(grade), 7)], icon) for item_id, title, grade, icon in rows]


# icon name -> the items drawn with it
@cache
def by_icon():
    out = {}
    for item in load_index():
        out.setdefault(item[3], []).append(item)
    return out


def field(item_id, column):
    row = table("Item").execute(f"select {column} from Item where PrimaryKey = ?", (item_id,)).fetchone()
    # some contents name an item the client does not have
    return text(row[0]) if row else ""


# How a chest opens, what it holds as (id, title, amount), and how many more of its rows differ by
# class and are left out. A currency has no id.
def box(item_id):
    base = table("RandomBoxBase").execute("select * from RandomBoxBase where PrimaryKey = ?", (item_id,)).fetchone()
    if not base:
        return None, [], 0
    rows = []
    if int(base["RandomBoxEntityId"]):
        query = "select * from RandomBoxEntity where PrimaryKey = ?"
        rows += [(row, 1) for row in table("RandomBoxEntity").execute(query, (base["RandomBoxEntityId"],))]
    if int(base["DropIndex"]):
        for drop in table("DropBase").execute("select * from DropBase where PrimaryKey = ?", (base["DropIndex"],)):
            query = "select * from DropEntity where PrimaryKey = ?"
            # a drop is given Repetition times over
            rows += [(row, int(drop["Repetition"]) or 1) for row in table("DropEntity").execute(query, (drop["EntityIndex"],))]
    contents, by_class = [], 0
    for row, times in rows:
        amount = int(row["NormalMinCount"]) * times
        if int(row["NormalId"]):
            contents.append((int(row["NormalId"]), field(row["NormalId"], "Name"), amount))
        elif int(row["NormalMoneyType"]):
            name = table("Money").execute("select Name from Money where PrimaryKey = ?", (row["NormalMoneyType"],)).fetchone()
            contents.append((None, text(name[0]), amount))
        else:
            by_class += 1
    return KINDS.get(int(base["Type"]), "ObtainAll"), contents, by_class


# icon name -> (sheet, x, y, width, height), from the game's own list
@cache
def icon_info():
    data = (TABLES / "IconInfo.loa").read_bytes()
    offset = 16

    def string():
        nonlocal offset
        length = struct.unpack_from("<i", data, offset)[0]
        offset += 4 + length
        return data[offset - length : offset - 1].decode("latin1").lower()

    out = {}
    for _ in range(struct.unpack_from("<i", data, 12)[0]):
        name, sheet = string(), string()
        rect = struct.unpack_from("<4i", data, offset)
        offset += 16
        string()  # the language, which the name ends with too
        offset += 8
        out[name.removesuffix(".png")] = (sheet, *rect)
    return out


@cache
def sheets():
    return {path.stem.lower(): path for path in ATLAS.rglob("*.png")}


@lru_cache(maxsize=8)
def sheet(name):
    return Image.open(sheets()[name]).convert("RGBA")


# the art of an icon, as the English client draws it
def icon_art(name):
    info = icon_info()
    where, x, y, width, height = info.get(f"{name}_usa_english") or info[name]
    return sheet(where).crop((x, y, x + width, y + height))
