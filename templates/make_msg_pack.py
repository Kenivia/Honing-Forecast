"""
Rebuilds the Icon templates in public/ScannerConfig.msgpack from items.json. Anchors are left alone.

To add an item: put its icon art in Icons/, add a row to items.json, run this.
A row is {"title": in-game tooltip title, "icon": file name in Icons/, "rarity": background}.
Rows sharing an icon share one template; a row without an icon is only a title the tooltip reader accepts.
"""

import json
from pathlib import Path

import msgpack
from PIL import Image

WIDTH = 64
HEIGHT = 64

SCRIPT_DIR = Path(__file__).resolve().parent
BACKGROUNDS_DIR = SCRIPT_DIR / "Backgrounds"
ICONS_DIR = SCRIPT_DIR / "Icons"
ITEMS_PATH = SCRIPT_DIR / "items.json"
OUTPUT_PATH = SCRIPT_DIR.parent / "public" / "ScannerConfig.msgpack"


def find_icon_path(name: str) -> Path:
    for ext in (".webp", ".png"):
        candidate = ICONS_DIR / f"{name}{ext}"
        if candidate.exists():
            return candidate
    raise FileNotFoundError(f"No icon found for '{name}' in {ICONS_DIR} (tried .webp and .png)")


def resize_and_crop_bottom(img: Image.Image, width: int, height: int) -> Image.Image:
    img = img.convert("RGBA")
    img = img.resize((width, width), Image.Resampling.LANCZOS)
    return img.crop((0, width - height, width, width))


def build_icon_config(name: str, rarity: str) -> dict:
    icon = resize_and_crop_bottom(Image.open(find_icon_path(name)), WIDTH, HEIGHT)
    background = resize_and_crop_bottom(Image.open(BACKGROUNDS_DIR / f"{rarity}.webp"), WIDTH, HEIGHT)
    # the background goes over black first so the result is opaque
    background = Image.alpha_composite(Image.new("RGBA", background.size, (0, 0, 0, 255)), background)
    final = Image.alpha_composite(background, icon)

    return {
        "name": name,
        "offset": {"top_left": [0, 0], "width": WIDTH, "height": HEIGHT},
        "data": list(final.tobytes()),
        "tag": "Icon",
        "normalized": True,
        "required_confidence": None,
    }


def main():
    rarities = {}
    for item in json.loads(ITEMS_PATH.read_text(encoding="utf-8")):
        if "icon" in item:
            assert rarities.setdefault(item["icon"], item["rarity"]) == item["rarity"], f"'{item['icon']}' has two rarities"

    configs = msgpack.unpackb(OUTPUT_PATH.read_bytes(), raw=False)
    configs = [config for config in configs if config["tag"] != "Icon"]
    configs += [build_icon_config(name, rarity) for name, rarity in rarities.items()]
    OUTPUT_PATH.write_bytes(msgpack.packb(configs, use_bin_type=True))
    print(f"Wrote {len(rarities)} icons ({len(configs)} templates in total) to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
