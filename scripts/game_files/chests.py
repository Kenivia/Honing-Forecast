"""
Writes templates/chests.json and the art in templates/ChestIcons: every chest in the game that opens,
through any number of chests inside it, to something in templates/items.json or to gold or silver.
Run extract.py first, and this again after a game patch or a change to items.json.

    python scripts/game_files/chests.py

A row is one chest as the game has it: id, title, icon (the art's name), rarity, kind, the item
level it asks for, contents as [title, amount, id if it is a chest of this table], and "extra", how
many more rows its tooltip has for things that differ by class. Items that only differ in how they
are bound are one row. "top" rows can sit in a slot: they have an English name, are
not past their expiry date and are not of tiers 1 to 3. The others are only there for being inside
one. Chests with "Cube" or "Engraving" in their title are left out altogether.
Afterwards: python templates/make_msg_pack.py, then pnpm run wasm.
"""

import json
import shutil
from datetime import date

from PIL import Image

from game import ROOT, box, icon_art, icon_info, load_index, table

TEMPLATES = ROOT / "templates"
ART = TEMPLATES / "ChestIcons"
MONEY = {1: "Silver", 2: "Gold"}  # Item.GainMoneyType
UNWANTED = ("Cube", "Engraving")  # in a chest's title: it is left out, and so is what only it leads to
UNWANTED_ICONS = ("all_quest_02_184", "all_quest_02_196", "all_quest_02_198", "all_quest_02_199", "all_quest_02_226")  # likewise, by its art

bare = lambda title: title.replace(" (Bound)", "")


def main():
    materials = {row["title"] for row in json.loads((TEMPLATES / "items.json").read_text(encoding="utf-8"))}
    index = {item[0]: item for item in load_index()}
    items = {int(row["PrimaryKey"]): row for row in table("Item").execute("select * from Item")}

    # kind and contents of everything that opens; a bar of gold opens to its gold
    opens = {}
    for (item_id,) in table("RandomBoxBase").execute("select PrimaryKey from RandomBoxBase"):
        # some boxes are of no item
        if int(item_id) not in items:
            continue
        if any(word in index.get(int(item_id), (0, ""))[1] for word in UNWANTED):
            continue
        if index.get(int(item_id), (0, "", "", ""))[3] in UNWANTED_ICONS:
            continue
        kind, contents, extra = box(int(item_id))
        opens[int(item_id)] = (kind, [(content_id, bare(title), amount) for content_id, title, amount in contents], extra)
    for item_id, row in items.items():
        if item_id not in opens and int(row["GainMoneyType"]) in MONEY and int(row["UseGainCount"]):
            opens[item_id] = ("ObtainAll", [(None, MONEY[int(row["GainMoneyType"])], int(row["UseGainCount"]))], 0)

    # every title a chest opens to, all the way down
    leaves = {}

    def reach(item_id):
        if item_id not in leaves:
            leaves[item_id] = set()
            for content_id, title, _ in opens[item_id][1]:
                leaves[item_id] |= reach(content_id) if content_id in opens else {title}
        return leaves[item_id]

    # a chest with no material is only wanted when it is gold alone or silver alone
    wanted = {
        item_id
        for item_id, (_, _, extra) in opens.items()
        if reach(item_id) & materials or (not extra and reach(item_id) in ({"Gold"}, {"Silver"}))
    }

    today = date.today().isoformat()
    info = icon_info()

    def top(item_id):
        row = items[item_id]
        named = item_id in index and index[item_id][1] and not index[item_id][1].startswith("ZZZ")
        expired = row["ExpireDeadline"] and row["ExpireDeadline"][:10] < today
        return bool(named and not expired and int(row["Tier"]) in (0, 4) and index[item_id][3] in info)

    # Items that read the same are one row, under the smallest id. What a chest holds is compared
    # after the chests inside it were merged, so this goes round until nothing merges.
    merged = {item_id: item_id for item_id in wanted}
    while True:
        rows, now = {}, {}
        for item_id in sorted(wanted):
            kind, contents, extra = opens[item_id]
            _, title, rarity, icon = index.get(item_id, (item_id, "", "common", ""))
            level = int(items[item_id]["ReUseBalanceLevel"])
            inside = [[name, amount, merged[content_id] if content_id in wanted else None] for content_id, name, amount in contents]
            key = json.dumps([bare(title), icon, rarity, kind, level, inside, extra])
            row = rows.setdefault(key, {"id": item_id, "title": bare(title), "icon": icon, "rarity": rarity, "kind": kind})
            if level:
                row["level"] = level
            if extra:
                row["extra"] = extra
            row["top"] = row.get("top", False) or top(item_id)
            row["contents"] = inside
            now[item_id] = row["id"]
        if now == merged:
            break
        merged = now

    # only what a top row can reach is kept
    by_id = {row["id"]: row for row in rows.values()}
    kept, queue = set(), [row["id"] for row in by_id.values() if row["top"]]
    while queue:
        item_id = queue.pop()
        if item_id not in kept:
            kept.add(item_id)
            queue += [content[2] for content in by_id[item_id]["contents"] if content[2]]
    out = [by_id[item_id] for item_id in sorted(kept)]
    for row in out:
        if not row["top"]:
            del row["top"]
            # no template is made for it
            del row["icon"], row["rarity"]

    lines = ["  " + json.dumps(row, ensure_ascii=False) for row in out]
    (TEMPLATES / "chests.json").write_text("[\n" + ",\n".join(lines) + "\n]\n", encoding="utf-8", newline="\n")

    shutil.rmtree(ART, ignore_errors=True)
    ART.mkdir()
    icons = {row["icon"] for row in out if row.get("top")}
    for icon in icons:
        icon_art(icon).resize((64, 64), Image.Resampling.LANCZOS).save(ART / f"{icon}.png")
    tops = [row for row in out if row.get("top")]
    print(f"{len(out)} chests, {len(tops)} of them for slots, {len(icons)} icons, {len({(row['icon'], row['rarity']) for row in tops})} templates")


if __name__ == "__main__":
    main()
