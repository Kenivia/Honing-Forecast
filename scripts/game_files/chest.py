"""
Adds a chest to the scanner's tables from the game's own files. Run extract.py first.

    python scripts/game_files/chest.py "<title>"
    python scripts/game_files/chest.py <capture.png or .mp4> --rect x,y,size [--frame n] [--brightness b] [--title part]

A title, or an icon cut from a capture, can be several chests with different contents. They are
all listed, and nothing is written until --id picks one. Plain chest art is shared by a hundred
chests, so with a capture give --title, a part of the name, to look up only those.

Writes templates/chest.json (title, icon, rarity; the art goes to templates/Icons) and, for every
chest inside it, templates/inner_chests.json (what it holds, which no tooltip says). --inner
writes the chest itself there and leaves chest.json alone. Rows already there are kept: delete one
to have it written again.
Afterwards: python templates/make_msg_pack.py, then pnpm run wasm.
"""

import argparse
import json
import re
from pathlib import Path

import numpy as np
from PIL import Image

from find_icon import add_crop_arguments, load_crop, search
from game import ROOT, box, field, icon_art, load_index

TEMPLATES = ROOT / "templates"
CHESTS = TEMPLATES / "chest.json"
INNER = TEMPLATES / "inner_chests.json"
ART = TEMPLATES / "Icons"

bare = lambda title: title.replace(" (Bound)", "")


def read_rows(path):
    return json.loads(path.read_text(encoding="utf-8"))


# one row a line, as the tables are kept
def write_rows(path, rows):
    lines = ["  { " + json.dumps(row, ensure_ascii=False)[1:-1] + " }" for row in rows]
    path.write_text("[\n" + ",\n".join(lines) + "\n]\n", encoding="utf-8", newline="\n")


def describe(item):
    item_id, title, rarity, icon = item
    kind, contents = box(item_id)
    contents = [(content_id, bare(content), amount) for content_id, content, amount in contents]
    description = field(item_id, "Desc").replace("\n", " ")
    return {"id": item_id, "title": bare(title), "rarity": rarity, "icon": icon, "contents": contents, "kind": kind, "description": description}


def show(chest):
    contents = ", ".join(f"{title} x{amount}" for _, title, amount in chest["contents"])
    print(f"  --id {chest['id']}  {chest['title']}  ({chest['rarity']}, {chest['icon']})  {chest['kind']}: {contents}")
    print(f"      {chest['description']}")


# the template in templates/Icons that is this art, else a new one named after the chest
def icon_name(chest, wanted):
    art = icon_art(chest["icon"]).resize((64, 64), Image.Resampling.LANCZOS)

    def pixels(image):
        rgba = np.asarray(image.convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)).astype(float)
        return rgba[..., :3] * rgba[..., 3:] / 255

    if not wanted:
        # the templates already there came lossy from elsewhere, so close is the same (about 5 against 30)
        same = [path.stem for path in ART.iterdir() if np.abs(pixels(Image.open(path)) - pixels(art)).mean() < 15]
        wanted = same[0] if same else re.sub(r'[\\/:*?"<>|]', "", chest["title"])
    if not any((ART / f"{wanted}{ext}").exists() for ext in (".webp", ".png")):
        art.save(ART / f"{wanted}.png")
        print(f"new art: templates/Icons/{wanted}.png")
    return wanted


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("source", help="a title, or a capture to cut the icon from")
    parser.add_argument("--id", type=int, help="the item to write")
    parser.add_argument("--icon", help="name of its template in templates/Icons")
    parser.add_argument("--inner", action="store_true", help="only a chest inside other chests")
    parser.add_argument("--title", default="", help="with a capture: part of the chest's name")
    add_crop_arguments(parser)
    args = parser.parse_args()

    index = load_index()
    by_id = {item[0]: item for item in index}
    if args.id:
        candidates = [by_id[args.id]]
    elif Path(args.source).exists():
        name, score, _ = search(load_crop(args.source, args), args.brightness, 1)[0]
        print(f"icon {name} ({score:.1f})")
        candidates = [item for item in index if item[3] == name and args.title.lower() in item[1].lower()]
    else:
        candidates = [item for item in index if bare(item[1]).lower() == args.source.lower()]
    chests = [chest for chest in map(describe, candidates) if chest["contents"]]
    print(f"{len(candidates)} items, {len(chests)} with contents")
    for chest in chests:
        show(chest)
    same = lambda chest: (chest["title"], chest["kind"], [content[1:] for content in chest["contents"]])
    if not chests or any(same(chest) != same(chests[0]) for chest in chests):
        print("nothing written" + (": pick one with --id" if chests else ""))
        return
    chest = chests[0]

    known = {item["title"] for item in read_rows(TEMPLATES / "items.json") if "title" in item}
    inner = read_rows(INNER)
    wrote = []

    # A row for this chest and for each chest inside it. False for one that holds nothing the
    # scanner knows, which gets no row.
    def add_inner(chest):
        # a row with only a title is a chest nobody has picked the contents of yet
        if any(row["title"] == chest["title"] and "items" in row for row in inner):
            print(f"kept inner chest: {chest['title']}")
            return True
        items = {}
        for content_id, title, amount in chest["contents"]:
            if title in known or (content_id and add_inner(describe(by_id[content_id]))):
                items[title] = amount
            else:
                print(f"left out of {chest['title']}: {title} x{amount}")
        if not items:
            return False
        show(chest)
        inner[:] = [row for row in inner if row["title"] != chest["title"]]
        inner.append({"title": chest["title"], "kind": chest["kind"], "items": items})
        wrote.append(chest["title"])
        return True

    if args.inner:
        add_inner(chest)
    else:
        for content_id, title, _ in chest["contents"]:
            if title not in known and content_id:
                add_inner(describe(by_id[content_id]))
        rows = read_rows(CHESTS)
        if any(row["title"] == chest["title"] for row in rows):
            print(f"kept chest: {chest['title']}")
        else:
            rows.append({"title": chest["title"], "icon": icon_name(chest, args.icon), "rarity": chest["rarity"]})
            write_rows(CHESTS, rows)
            print(f"wrote chest: {chest['title']}")
    write_rows(INNER, inner)
    print(f"wrote inner chests: {', '.join(wrote) or 'none'}")


if __name__ == "__main__":
    main()
