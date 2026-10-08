"""
Finds the icon art a slot on screen is drawn with, among the game's item icons. Run extract.py first.

    python scripts/game_files/find_icon.py <crop.png>
    python scripts/game_files/find_icon.py <capture.png or .mp4> --rect x,y,size [--frame n]

The crop is one whole slot, number and all, cut roughly: a few pixels off is searched for.
--brightness is the in-game setting the capture was taken at (default 50).
Prints the best matches with the items drawn with them, and writes .tmp/find_icon.png: the crop,
then the matches in order. The icons are read once and kept in scripts/game_files/icons.npz.
"""

import argparse

import numpy as np
from PIL import Image, ImageDraw

from game import HERE, ROOT, by_icon, icon_art, icon_info

CACHE = HERE / "icons.npz"
SHEET = ROOT / ".tmp" / "find_icon.png"
SIZE = 64
KEPT = 0.75  # share of an icon's opaque pixels scored: the rest absorbs the number and the cursor
MIN_OPAQUE = 1000  # an icon with less to go by matches anything
REFINED = 300  # best matches that are tried again a few pixels off
SHIFTS = 3
GAMMA_RATIO = 62.6  # as in crates/scanner/src/image_utils/brightness.rs


def load_icons():
    # sheet by sheet, so each is opened once
    info = icon_info()
    names = sorted((name for name in by_icon() if name in info), key=lambda name: (info[name][0], name))
    if CACHE.exists():
        cache = np.load(CACHE)
        # icons added by a patch since are not in it
        if len(cache["names"]) == len(names):
            return list(cache["names"]), cache["icons"]
    print(f"reading {len(names)} icons, once")
    icons = np.zeros((len(names), SIZE, SIZE, 4), np.uint8)
    for index, name in enumerate(names):
        icons[index] = np.asarray(icon_art(name).resize((SIZE, SIZE), Image.Resampling.LANCZOS))
    np.savez_compressed(CACHE, names=np.array(names), icons=icons)
    return names, icons


def add_crop_arguments(parser):
    parser.add_argument("--rect", help="x,y,size of the slot in the capture")
    parser.add_argument("--frame", type=int, default=0)
    parser.add_argument("--brightness", type=float, default=50)


def load_crop(source, args):
    if source.lower().endswith(".mp4"):
        import cv2

        capture = cv2.VideoCapture(source)
        capture.set(cv2.CAP_PROP_POS_FRAMES, args.frame)
        image = Image.fromarray(cv2.cvtColor(capture.read()[1], cv2.COLOR_BGR2RGB))
    else:
        image = Image.open(source).convert("RGB")
    if args.rect:
        x, y, size = map(int, args.rect.split(","))
        image = image.crop((x, y, x + size, y + size))
    return image


# the mean difference over the closest KEPT of each icon's opaque pixels
def scores(seen, icons):
    out = np.empty(len(icons))
    for start in range(0, len(icons), 1000):
        chunk = icons[start : start + 1000]
        difference = np.abs(chunk[..., :3].astype(np.int16) - seen).sum(-1).reshape(len(chunk), -1)
        opaque = chunk[..., 3].reshape(len(chunk), -1) >= 250
        difference[~opaque] = 10_000
        difference.sort(axis=1)
        totals = np.cumsum(difference, axis=1, dtype=np.int64)
        kept = np.maximum((opaque.sum(1) * KEPT).astype(int), 1)
        out[start : start + 1000] = totals[np.arange(len(chunk)), kept - 1] / kept
        out[start : start + 1000][opaque.sum(1) < MIN_OPAQUE] = np.inf
    return out


# the icons closest to the crop, best first, as (name, score, art)
def search(crop, brightness, top):
    # a margin for the shifts, then to the brightness the art is drawn at
    wide = crop.resize((SIZE, SIZE), Image.Resampling.LANCZOS)
    wide = np.pad(np.asarray(wide), ((SHIFTS, SHIFTS), (SHIFTS, SHIFTS), (0, 0)), mode="edge")
    wide = 255 * (wide / 255) ** ((GAMMA_RATIO + brightness) / (GAMMA_RATIO + 50))
    wide = wide.round().astype(np.int16)
    window = lambda dx, dy: wide[SHIFTS + dy : SHIFTS + dy + SIZE, SHIFTS + dx : SHIFTS + dx + SIZE]

    names, icons = load_icons()
    best = scores(window(0, 0), icons)
    refined = np.argsort(best)[:REFINED]
    for dy in range(-SHIFTS, SHIFTS + 1):
        for dx in range(-SHIFTS, SHIFTS + 1):
            best[refined] = np.minimum(best[refined], scores(window(dx, dy), icons[refined]))
    return [(str(names[index]), best[index], icons[index]) for index in np.argsort(best)[:top]]


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("source")
    add_crop_arguments(parser)
    parser.add_argument("--top", type=int, default=12)
    args = parser.parse_args()

    crop = load_crop(args.source, args)
    found = search(crop, args.brightness, args.top)
    items = by_icon()

    scale = 3
    cell = SIZE * scale
    sheet = Image.new("RGB", ((len(found) + 1) * (cell + 8) + 8, cell + 30), (40, 40, 40))
    draw = ImageDraw.Draw(sheet)
    sheet.paste(crop.resize((cell, cell), Image.Resampling.NEAREST), (8, 8))
    draw.text((8, cell + 12), "crop", fill="white")
    for rank, (name, score, art) in enumerate(found):
        titles = sorted({item[1] for item in items.get(name, [])})
        more = f" and {len(titles) - 4} more" if len(titles) > 4 else ""
        print(f"{rank + 1:3}  {score:6.1f}  {name}  {'; '.join(titles[:4])}{more}")
        icon = Image.fromarray(art).resize((cell, cell), Image.Resampling.NEAREST)
        left = 8 + (rank + 1) * (cell + 8)
        sheet.paste(icon, (left, 8), icon)
        draw.text((left, cell + 12), f"{rank + 1}: {score:.1f}", fill="white")
    SHEET.parent.mkdir(exist_ok=True)
    sheet.save(SHEET)
    print(f"wrote {SHEET}")


if __name__ == "__main__":
    main()
