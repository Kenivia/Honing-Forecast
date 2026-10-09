"""
Adds and edits the Anchor templates in public/ScannerConfig.msgpack: fixed UI the scanner locates
windows by, and page tabs. Icons are made by templates/make_msg_pack.py and left alone here.

Every anchor is also written to anchors/<name>.png next to this script, for review.

    python "scripts/anchor setup/anchors.py" list
    python "scripts/anchor setup/anchors.py" export
    python "scripts/anchor setup/anchors.py" locate <still> <name> [--brightness 70]
    python "scripts/anchor setup/anchors.py" add <still> <name> <x> <y> <width> <height>
        --offset <x> <y> [--brightness 70] [--confidence 0.8]
    python "scripts/anchor setup/anchors.py" confidence <name> <value | none>
    python "scripts/anchor setup/anchors.py" remove <name>

The still has to be a 1440p UI (templates are kept at the reference resolution), the rectangle
whole pixels in it, and --brightness the in-game setting it was taken at. --offset is where the
template sits relative to the root it gives: the window origin, or the UI block for the Storage
button. A name already in the config is replaced in place. The names are looked up by the anchor
list in crates/scanner/src/constants.rs.
"""

import argparse
from pathlib import Path

import cv2
import msgpack
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
CONFIG_PATH = HERE.parent.parent / "public" / "ScannerConfig.msgpack"
OUT_DIR = HERE / "anchors"

# must match crates/scanner/src/image_utils/brightness.rs
GAMMA_RATIO = 62.6
TARGET_BRIGHTNESS = 50.0


def normalize(rgb: np.ndarray, setting: float) -> np.ndarray:
    exponent = (GAMMA_RATIO + min(max(setting, 0.0), 100.0)) / (GAMMA_RATIO + TARGET_BRIGHTNESS)
    lut = np.round(255.0 * (np.arange(256) / 255.0) ** exponent).astype(np.uint8)
    return lut[rgb]


def load() -> list:
    return msgpack.unpackb(CONFIG_PATH.read_bytes(), raw=False)


def save(config: list):
    CONFIG_PATH.write_bytes(msgpack.packb(config, use_bin_type=True))


def pixels(entry: dict) -> np.ndarray:
    size = (entry["offset"]["height"], entry["offset"]["width"], 4)
    return np.array(entry["data"], dtype=np.uint8).reshape(size)


def find(config: list, name: str) -> int:
    return next(i for i, entry in enumerate(config) if entry["name"] == name and entry["tag"] == "Anchor")


def write_png(entry: dict):
    OUT_DIR.mkdir(exist_ok=True)
    Image.fromarray(pixels(entry), "RGBA").save(OUT_DIR / f"{entry['name']}.png")


def still_rgb(path: str) -> np.ndarray:
    return np.array(Image.open(path).convert("RGB"))


def cut(still: np.ndarray, x: int, y: int, width: int, height: int, setting: float) -> np.ndarray:
    crop = normalize(still[y : y + height, x : x + width], setting)
    assert crop.shape[:2] == (height, width), "the rectangle runs off the still"
    return np.dstack([crop, np.full((height, width), 255, np.uint8)])


def describe(entry: dict) -> str:
    offset = entry["offset"]
    return (
        f"{entry['name']:32} {offset['width']:4}x{offset['height']:<4} "
        f"offset {offset['top_left']}  confidence {entry.get('required_confidence')}"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("list")
    commands.add_parser("export")
    locate = commands.add_parser("locate")
    locate.add_argument("still")
    locate.add_argument("name")
    locate.add_argument("--brightness", type=float, default=70.0)
    add = commands.add_parser("add")
    add.add_argument("still")
    add.add_argument("name")
    for field in ["x", "y", "width", "height"]:
        add.add_argument(field, type=int)
    add.add_argument("--offset", type=float, nargs=2, required=True)
    add.add_argument("--brightness", type=float, default=70.0)
    add.add_argument("--confidence", type=float)
    confidence = commands.add_parser("confidence")
    confidence.add_argument("name")
    confidence.add_argument("value")
    commands.add_parser("remove").add_argument("name")
    args = parser.parse_args()

    config = load()
    anchors = [entry for entry in config if entry["tag"] == "Anchor"]
    if args.command == "list":
        print("\n".join(describe(entry) for entry in anchors))
    elif args.command == "export":
        for entry in anchors:
            write_png(entry)
        print(f"wrote {len(anchors)} anchors to {OUT_DIR}")
    elif args.command == "locate":
        # where a stored anchor is in a still, to cut it again or check a still's setting
        template = pixels(config[find(config, args.name)])[..., :3]
        still = normalize(still_rgb(args.still), args.brightness)
        scores = cv2.matchTemplate(still, template, cv2.TM_SQDIFF)
        _, _, (x, y), _ = cv2.minMaxLoc(scores)
        height, width = template.shape[:2]
        apart = np.abs(still[y : y + height, x : x + width].astype(int) - template).mean()
        print(f"{args.name}: x {x} y {y} width {width} height {height}, {apart:.2f} levels apart on average")
    elif args.command == "add":
        # in the key order the file has
        entry = {
            "data": cut(still_rgb(args.still), args.x, args.y, args.width, args.height, args.brightness).flatten().tolist(),
            "name": args.name,
            # whole numbers stay whole, as the rest of the file has them
            "offset": {"top_left": [int(v) if v == int(v) else v for v in args.offset], "width": args.width, "height": args.height},
            "tag": "Anchor",
            "normalized": True,
            "required_confidence": args.confidence,
        }
        if any(old["name"] == args.name for old in anchors):
            config[find(config, args.name)] = entry
        else:
            config.insert(0, entry)
        save(config)
        write_png(entry)
        print(describe(entry))
    elif args.command == "confidence":
        entry = config[find(config, args.name)]
        entry["required_confidence"] = None if args.value == "none" else float(args.value)
        save(config)
        print(describe(entry))
    elif args.command == "remove":
        del config[find(config, args.name)]
        save(config)
        (OUT_DIR / f"{args.name}.png").unlink(missing_ok=True)
        print(f"removed {args.name}")


if __name__ == "__main__":
    main()
