"""
What icon_bench.py scores on: the slot looks slot_dump wrote for each debug capture
(target/slot-bench/<capture>), labelled from the capture's end record, and the templates.

A slot's label is the icon it ended as on the page, the user's corrections included: a template
name, or None for a slot that holds nothing we have a template for. LABELS below are slots whose
end record is wrong. They were found by looking every chest slot, and every other slot that
scores 0.9 or more against some template, up among all of the game's item icons (find_icon.py)
and telling its rarity by its bare background.
"""

import json
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
BENCH = ROOT / "target/slot-bench"
TEMPLATES = ROOT / "templates"

# 1440p, made from a compressed recording, which a screen share is not: left out unless asked for
LOSSY = {"b-19-22", "b-22-31"}
SKIP = "skip"  # left out: a chest of a template's art with an item level across it that is not read through
# (capture, slot) -> template or None, where the end record is wrong
LABELS = {
    ("a-14-58-25-919Z", "CharInventory 0 0,2"): "use_14_4@legendary",  # Emergency Honing Materials Chest (L), scored 0.946
    ("a-15-53-29-195Z", "CharStorage 1 9,8"): None,  # a T3 fusion material read as the T4 one
    ("b-19-22", "CharStorage 0 7,9"): None,  # the same
    # set irrelevant by hand for what they open to, but drawn with the template's art
    ("a-14-58-25-919Z", "CharStorage 1 0,2"): "use_4_224@legendary",
    ("a-15-53-29-195Z", "Roster 0 5,4"): "use_2_219@ancient",
    # never recognised, at 0.946 to 0.948: a honing material selection chest and a bar of gold
    ("a-15-25-19-940Z", "CharInventory 0 0,5"): "shop_icon_10804@epic",
    ("b-00-08", "CharInventory 0 0,5"): "shop_icon_10804@epic",
    ("b-18-47", "CharInventory 0 0,5"): "shop_icon_10804@epic",
    ("b-19-56", "CharInventory 0 0,5"): "shop_icon_10804@epic",
    ("a-15-53-29-195Z", "Roster 0 2,5"): "use_8_248@epic",
    ("a-14-58-25-919Z", "CharStorage 0 3,3"): SKIP,
    ("a-15-53-29-195Z", "CharStorage 1 1,4"): SKIP,
    ("a-15-53-29-195Z", "CharStorage 1 1,7"): SKIP,
    ("a-15-53-29-195Z", "Roster 0 3,5"): SKIP,
    ("b-19-22", "Roster 0 0,5"): SKIP,
    ("b-22-31", "Roster 0 0,5"): SKIP,
}


def load(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.int16)


def templates(ui_height):
    folder = BENCH / f"templates-{ui_height}"
    names = sorted(x.stem for x in folder.glob("*.png"))
    return names, np.stack([load(folder / f"{name}.png") for name in names])


# per template: its pass limit, and whether a chest drawn with it has its item level across the slot
def template_info():
    items = json.loads((TEMPLATES / "items.json").read_text(encoding="utf8"))
    chests = json.loads((TEMPLATES / "chests.json").read_text(encoding="utf8"))
    limits = {row["icon"]: row.get("confidence") or 0.95 for row in items if "icon" in row}
    levelled = {f"{row['icon']}@{row['rarity']}" for row in chests if row.get("top") and row.get("level") and not row.get("irrelevant")}
    by_label = {row["label"]: row["icon"] for row in items if "icon" in row and "label" in row}
    return limits, levelled, by_label


# The art's opacity at the template's size, 0 to 1: where the icon is drawn and where only the
# rarity background shows. Cut and scaled as icon_lookup does the template.
def art_alpha(name, size):
    icon = name.split("@")[0].lstrip("~")
    folders = ["DecoyIcons"] if name.startswith("~") else ["ChestIcons", "Icons"]
    paths = [TEMPLATES / folder / f"{icon}{x}" for folder in folders for x in (".png", ".webp")]
    art = Image.open(next(x for x in paths if x.exists())).convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    alpha = art.getchannel("A").resize(size, Image.Resampling.BILINEAR, box=(0, 22 * 64 / 61, 64, 64))
    return np.asarray(alpha, dtype=np.float32) / 255


def captures():
    return sorted(x for x in BENCH.iterdir() if (x / "index.jsonl").exists())


# Every slot of a capture: its label and its looks, the clean one first. Of the looks seen with
# no tooltip over or beside the slot, that is the one the others are closest to, each counting
# for as many scans as it was seen clean on. On a lossless capture that is the look seen most; on
# one made from a compressed recording every frame is a look of its own, and a frame of something
# fading over the slot is the odd one out.
def slots(capture):
    _, _, by_label = template_info()
    reference = json.loads((capture / "reference.json").read_text(encoding="utf8"))
    key = lambda a: f"{a['inventory_type']} {a['page_num']} {a['pos_in_inv'][0]},{a['pos_in_inv'][1]}"
    labels = {}
    for slot in reference["slots"]:
        icon = slot.get("icon_name_score")
        labels[key(slot["address"])] = icon[0] if icon else None
    for name, edit in reference["edits"].items():
        item = edit.get("item")
        # set by hand to a material, or to the chest its icon is
        labels[name] = by_label.get(item, item if item and "@" in item else None) if item else None
    for (name, slot), label in LABELS.items():
        if name == capture.name:
            labels[slot] = label
    looks = {}
    for line in (capture / "index.jsonl").read_text().splitlines():
        look = json.loads(line)
        looks.setdefault(look["slot"], []).append(look)
    out = []
    for slot, seen in sorted(looks.items()):
        if slot not in labels or labels[slot] == SKIP:
            continue
        seen.sort(key=lambda x: (x["hovered"] - x["scans"], -x["first"]))
        clean = [x for x in seen if x["hovered"] < x["scans"]]
        if len(clean) > 1:
            crops = np.stack([crop(capture.name, x) for x in clean]).astype(np.float32)
            weights = np.array([x["scans"] - x["hovered"] for x in clean])
            apart = []
            for one in crops:
                distances = np.abs(crops - one).mean((1, 2, 3))
                order = np.argsort(distances)
                apart.append(distances[order][np.searchsorted(np.cumsum(weights[order]), weights.sum() / 2)])
            middle = clean[int(np.argmin(apart))]
            seen.remove(middle)
            seen.insert(0, middle)
        out.append({"capture": capture.name, "slot": slot, "label": labels[slot], "looks": seen})
    return out


def crop(capture, look, kind="i"):
    return load(BENCH / capture / f"{look['file']}_{kind}.png")


# nothing in the slot: dark and flat
def empty(image):
    return image.mean() < 40 and image.std() < 12
