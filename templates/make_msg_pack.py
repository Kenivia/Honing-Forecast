
from pathlib import Path
from PIL import Image
import msgpack

DATA = {"Armor Book" : "relic",
"Blue" : "common",
"Fusion" : "rare",
"Glacier's Breath" : "epic",
"Lava's Breath" : "epic",
"Leapstone" : "rare",
"Red" : "common",
"Scroll 1 Armor" : "epic",
"Scroll 1 Weapon" : "epic",
"Scroll 2 Armor" : "legendary",
"Scroll 2 Weapon" : "legendary",
"Scroll 3 Armor" : "relic",
"Scroll 3 Weapon" : "relic",
"Scroll 4 Armor" : "ancient",
"Scroll 4 Weapon" : "ancient",
"Special Leapstone" : "relic",
"Weapon Book" : "relic",
"Serca Red" : "common",
"Serca Blue" : "common", 
"Serca Leapstone" : "rare", 
"Serca Special Leapstone" : "Relic", 
"Serca Fusion": "Epic",
}



WIDTH = 28
HEIGHT = 18


# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

SCRIPT_DIR = Path(__file__).resolve().parent
BACKGROUNDS_DIR = SCRIPT_DIR / "Backgrounds"
ICONS_DIR = SCRIPT_DIR / "Icons"
OUTPUT_PATH = SCRIPT_DIR / "ScannerConfig.msgpack"


def find_icon_path(name: str) -> Path:
    """Locate an icon file for `name`, trying .webp then .png. Errors if missing."""
    for ext in (".webp", ".png"):
        candidate = ICONS_DIR / f"{name}{ext}"
        if candidate.exists():
            return candidate
    raise FileNotFoundError(
        f"No icon found for '{name}' in {ICONS_DIR} (tried .webp and .png)"
    )


def find_background_path(rarity: str) -> Path:
    candidate = BACKGROUNDS_DIR / f"{rarity}.webp"
    if not candidate.exists():
        raise FileNotFoundError(f"No background found for rarity '{rarity}' at {candidate}")
    return candidate


def resize_and_crop_bottom(img: Image.Image, width: int, height: int) -> Image.Image:
    """Resize to width x width, then crop to the bottom `height` pixels."""
    img = img.convert("RGBA")
    img = img.resize((width, width), Image.Resampling.LANCZOS)
    top = width - height
    return img.crop((0, top, width, width))


def flatten_opaque(img: Image.Image, backdrop=(0, 0, 0, 255)) -> Image.Image:
    """Composite `img` over an opaque backdrop so the result has alpha=255 everywhere."""
    if img.mode != "RGBA":
        img = img.convert("RGBA")
    canvas = Image.new("RGBA", img.size, backdrop)
    return Image.alpha_composite(canvas, img)


def build_icon_config(name: str, rarity: str) -> dict:
    icon_path = find_icon_path(name)
    background_path = find_background_path(rarity)

    icon = Image.open(icon_path)
    background = Image.open(background_path)

    icon = resize_and_crop_bottom(icon, WIDTH, HEIGHT)
    background = resize_and_crop_bottom(background, WIDTH, HEIGHT)

    # Ensure the background itself is fully opaque before compositing the icon on top.
    background = flatten_opaque(background)

    # Composite icon (with its own transparency) over the now-opaque background.
    # Because the background is fully opaque, the result is guaranteed opaque too.
    final = Image.alpha_composite(background, icon)

    if final.mode != "RGBA":
        final = final.convert("RGBA")

    pixel_bytes = final.tobytes()  # RGBA, row-major

    return {
        "name": name,
        "offset": {
            "top_left": [0.0, 0.0],
            "width": WIDTH,
            "height": HEIGHT,
        },
        "data": list(pixel_bytes),
        "tag": 'Icon',
    }


def load_existing_configs() -> list:
    if not OUTPUT_PATH.exists():
        return []
    with open(OUTPUT_PATH, "rb") as f:
        raw = f.read()
    if not raw:
        return []
    existing = msgpack.unpackb(raw, raw=False)
    if not isinstance(existing, list):
        raise ValueError(f"{OUTPUT_PATH} does not contain a msgpack list; refusing to append")
    return existing


def main():
    configs = load_existing_configs()

    for name, rarity in DATA.items():
        print(f"Processing '{name}' -> background '{rarity}'...")
        configs.append(build_icon_config(name, rarity))

    with open(OUTPUT_PATH, "wb") as f:
        f.write(msgpack.packb(configs, use_bin_type=True))

    print(f"Wrote {len(configs)} total icon configs to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()