# The items the market scanner can meet and their icon art, from the game's tables (scanner_setup/game_files,
# run extract.py there first): every item that can be listed on the Market in the material categories.
# Writes crates/market/items/items.json ([{name, grade, icon, category, tier}], one per name) and items/<icon>.png.
# Run again after a game update: python crates/market/tools/market_items.py
import json, shutil, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scanner_setup" / "game_files"))
import game  # noqa: E402

OUT = ROOT / "crates" / "market" / "items"
# Item.Category / 100: trade skill loot (701-703), enhancement materials (712-734); and the shard pouches
GROUPS = {701, 702, 703, 712, 718, 728, 732, 734}
POUCHES = 31060


def main():
    items = game.table("Item")
    rows = items.execute(
        "select Name, Grade, lower(Icon || '_' || IconIndex), Category, Tier from Item where EnableMarket = 1"
    ).fetchall()
    out = {}
    for name_key, grade, icon, category, tier in rows:
        if int(category) // 100 not in GROUPS and int(category) != POUCHES:
            continue
        name = game.text(name_key)
        if not name or name in out:
            continue
        out[name] = {"name": name, "grade": int(grade), "icon": icon, "category": int(category), "tier": int(tier)}
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir(parents=True)
    for icon in sorted({x["icon"] for x in out.values()}):
        game.icon_art(icon).save(OUT / f"{icon}.png")
    text = json.dumps(sorted(out.values(), key=lambda x: x["name"]), indent=1)
    (OUT / "items.json").write_text(text, encoding="utf-8", newline="\n")
    print(f"{len(out)} items, {len({x['icon'] for x in out.values()})} icons")


if __name__ == "__main__":
    main()
