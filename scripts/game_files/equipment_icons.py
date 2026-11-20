"""
Writes public/Icons/Equipments/<T4|Serca>/<class>/<Piece>.webp: each class's honed gear art, the ancient "Destined
Hellfire" set for T4 and "Destined Tremor" for Serca; the vambrace (Serca only) is the class's ancient Tremor one.
Serca pieces are drawn with the blue honed-gear border (icon_border_inherit.png, from lostark.bible); T4 has none.
Run after scanner_setup/game_files/extract.py.

    python scripts/game_files/equipment_icons.py
"""

import shutil
import sys
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scanner_setup/game_files"))
from game import ROOT, icon_art, table

OUT = ROOT / "public/Icons/Equipments"
# Item.For<column> -> the class name the site shows
CLASSES = {
    "Berserker": "Berserker", "BerserkerFemale": "Slayer", "Destroyer": "Destroyer", "Warlord": "Gunlancer",
    "HolyKnight": "Paladin", "HolyKnightFemale": "Valkyrie", "Arcana": "Arcanist", "Summoner": "Summoner",
    "Bard": "Bard", "ElementalMaster": "Sorceress", "BattleMaster": "Wardancer", "Infighter": "Scrapper",
    "ForceMaster": "Soulfist", "LanceMaster": "Glaivier", "BattleMasterMale": "Striker", "InfighterMale": "Breaker",
    "Blade": "Deathblade", "Demonic": "Shadowhunter", "Reaper": "Reaper", "SoulEater": "Souleater",
    "HawkEye": "Sharpshooter", "DevilHunter": "Deadeye", "Blaster": "Artillerist", "Scouter": "Machinist",
    "DevilHunterFemale": "Gunslinger", "Yinyangshi": "Artist", "WeatherArtist": "Aeromancer", "Alchemist": "Wildsoul",
    "DimensionMaster": "Dimensionalist", "DragonKnight": "Guardianknight",
}
PIECES = {10201: "Helmet", 10202: "Chest", 10203: "Pants", 10204: "Glove", 10205: "Shoulder", 10206: "Vambrace"}
SETS = {"T4": (134611000, 134619999), "Serca": (134621000, 134629999)}  # Destined Hellfire, Destined Tremor
ANCIENT = 6
# the blue border the game draws on honed gear, taken from lostark.bible (not found in the client's UI packages)
BORDER = Image.open(Path(__file__).with_name("icon_border_inherit.png")).convert("RGBA")


def piece(category):
    return "Weapon" if 10100 <= category < 10200 else PIECES.get(category)


def save(art, path, border):
    path.parent.mkdir(parents=True, exist_ok=True)
    art = art.convert("RGBA")
    (Image.alpha_composite(art, BORDER) if border else art).save(path, "WEBP", quality=90)


shutil.rmtree(OUT / "T4", ignore_errors=True)
shutil.rmtree(OUT / "Serca", ignore_errors=True)
items = table("Item")
for tier, (low, high) in SETS.items():
    rows = items.execute(
        "select * from Item where Grade = ? and ((PrimaryKey between ? and ?) or (Category = 10206 and ? = 'Serca'))",
        (ANCIENT, low, high, tier),
    )
    written = set()
    for row in rows:
        name = piece(int(row["Category"]))
        if not name:
            continue
        for column, label in CLASSES.items():
            key = (label, name)
            if row[f"For{column}"] and key not in written:
                written.add(key)
                save(icon_art(f"{row['Icon']}_{row['IconIndex']}".lower()), OUT / tier / label.lower() / f"{name}.webp", tier == "Serca")
    expected = len(CLASSES) * (7 if tier == "Serca" else 6)
    assert len(written) == expected, (tier, len(written), expected)
    print(tier, len(written))
