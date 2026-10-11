"""
Writes templates/decoys.json and the art in templates/DecoyIcons: item icons of the game that we
do not count and that look like a template of ours. A slot that is closer to a decoy than to any
template holds nothing we count (Pipeline.md, "Decoys").
Run chests.py first, and this again after it or after a change to items.json.

    python scripts/game_files/decoys.py

A decoy is an icon on a rarity background some item of the game has it on. It is one when it
comes within NEAR of a template as the scanner scores a slot at 1080p, and is not the same
picture. The art and rarity of a material's own items are never decoys.
Afterwards: python templates/make_msg_pack.py, then pnpm run wasm.
"""

import json
import shutil

import numpy as np
from PIL import Image

from find_icon import load_icons
from game import ROOT, load_index

TEMPLATES = ROOT / "templates"
ART = TEMPLATES / "DecoyIcons"
NEAR = 0.93  # the slot limit is 0.95, and a slot scores up to 0.02 off what its template would
SAME = 0.99
TOP = 22 * 64 / 61  # rows of the 64 px art the number is over
SIZE = (46, 29)  # what a slot's icon is compared at, at 1080p


def background(rarity):
    image = Image.open(TEMPLATES / "Backgrounds" / f"{rarity}.webp").convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS)
    return np.asarray(Image.alpha_composite(Image.new("RGBA", image.size, (0, 0, 0, 255)), image).convert("RGB"), dtype=np.float32)


# art (n, 64, 64, 4) over a background, cut and scaled as a template is, without its outermost pixels
def render(art, back):
    alpha = art[..., 3:].astype(np.float32) / 255
    full = (art[..., :3] * alpha + back * (1 - alpha)).round().astype(np.uint8)
    scale = lambda image: np.asarray(Image.fromarray(image).resize(SIZE, Image.Resampling.BILINEAR, box=(0, TOP, 64, 64)), dtype=np.float32)
    return np.stack([scale(x) for x in full])[:, 1:-1, 1:-1]


def main():
    items = json.loads((TEMPLATES / "items.json").read_text(encoding="utf-8"))
    chests = json.loads((TEMPLATES / "chests.json").read_text(encoding="utf-8"))
    ours = {row["icon"]: (TEMPLATES / "Icons", row["icon"], row["rarity"]) for row in items if "icon" in row}
    ours |= {f"{row['icon']}@{row['rarity']}": (TEMPLATES / "ChestIcons", row["icon"], row["rarity"]) for row in chests if row.get("top")}
    names = sorted(ours)
    backs = {path.stem: background(path.stem) for path in (TEMPLATES / "Backgrounds").glob("*.webp")}
    own = []
    for name in names:
        folder, icon, rarity = ours[name]
        path = next(folder / f"{icon}{x}" for x in (".png", ".webp") if (folder / f"{icon}{x}").exists())
        art = np.asarray(Image.open(path).convert("RGBA").resize((64, 64), Image.Resampling.LANCZOS))
        own.append(render(art[None], backs[rarity])[0])
    own = np.stack(own)

    art_names, arts = load_icons()
    at = {name: index for index, name in enumerate(art_names)}
    index = [x for x in load_index() if x[1] and not x[1].startswith("ZZZ") and x[3] in at]
    # a material's own art is the game's, a little off where ours is a lossy copy of it
    materials = {row["title"] for row in items if "title" in row}
    counted = {(icon, rarity) for _, title, rarity, icon in index if title.replace(" (Bound)", "") in materials}
    pairs = sorted({(icon, rarity) for _, _, rarity, icon in index} - counted)

    out = []
    for rarity in sorted({x[1] for x in pairs}):
        icons = [icon for icon, r in pairs if r == rarity]
        for start in range(0, len(icons), 500):
            chunk = icons[start : start + 500]
            images = render(arts[[at[x] for x in chunk]], backs[rarity])
            alike = 1 - np.abs(images[:, None] - own[None]).mean((2, 3, 4)) / 255
            for found in np.flatnonzero((alike.max(1) >= NEAR) & (alike.max(1) < SAME)):
                out.append({"icon": chunk[found], "rarity": rarity, "like": names[int(alike[found].argmax())], "alike": round(float(alike[found].max()), 3)})
    out.sort(key=lambda row: (row["icon"], row["rarity"]))
    lines = ["  " + json.dumps(row, ensure_ascii=False) for row in out]
    (TEMPLATES / "decoys.json").write_text("[\n" + ",\n".join(lines) + "\n]\n", encoding="utf-8", newline="\n")
    shutil.rmtree(ART, ignore_errors=True)
    ART.mkdir()
    for icon in sorted({row["icon"] for row in out}):
        Image.fromarray(arts[at[icon]]).save(ART / f"{icon}.png")
    print(f"{len(out)} decoys of {len(pairs)} icons on a background, {len(list(ART.iterdir()))} pieces of art")


if __name__ == "__main__":
    main()
