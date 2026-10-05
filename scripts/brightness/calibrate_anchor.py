"""
Derive the `brightness` coefficients of an AnchorVariant (crates/scanner/src/constants.rs),
used by est_ingame_brightness() to turn an anchor's mean intensity into the in-game setting.

    python scripts/brightness/calibrate_anchor.py                    every anchor in the config
    python scripts/brightness/calibrate_anchor.py "Storage anchor pet" ...
    python scripts/brightness/calibrate_anchor.py --sweep "Char Inventory Anchor 1" ...

Default: no screenshots needed. The template in the config is already normalized to the
reference brightness, so the brightness model is run backwards on it to get what the anchor
would look like at every setting.

--sweep: measure real crops instead, from ROOT_DIR/<setting>/<anchor name>.png
(the layout crates/scanner/src/main.rs dumps).

Either way setting(mean) is fitted as a quadratic. The model constants and grey weights
below must match crates/scanner/src/image_utils/brightness.rs.
"""

import sys
from pathlib import Path

import msgpack
import numpy as np
from PIL import Image

CONFIG_PATH = Path("./public/ScannerConfig.msgpack")
ROOT_DIR = Path("./scripts/brightness/all icons")

GAMMA_RATIO = 62.6
TARGET_BRIGHTNESS = 50.0

# Rec.709 luma weights, matching image::imageops::grayscale() in the `image` crate.
LUMA_WEIGHTS = np.array([0.2126, 0.7152, 0.0722])


def mean_f(rgb: np.ndarray) -> float:
    return float((rgb.astype(np.float64) * LUMA_WEIGHTS).sum(axis=-1).mean())


# inverse of normalize_brightness(): what a normalized template looked like at this setting
def denormalize(rgb: np.ndarray, setting: float) -> np.ndarray:
    exponent = (GAMMA_RATIO + setting) / (GAMMA_RATIO + TARGET_BRIGHTNESS)
    return (255 * (rgb / 255) ** (1 / exponent)).round()


def model_means(name: str, templates: dict) -> tuple[list[int], list[float]]:
    settings = list(range(0, 101, 5))
    return settings, [mean_f(denormalize(templates[name], s)) for s in settings]


def sweep_means(name: str) -> tuple[list[int], list[float]]:
    settings = sorted(int(p.name) for p in ROOT_DIR.iterdir() if p.name.isdigit())
    return settings, [
        mean_f(np.array(Image.open(ROOT_DIR / str(s) / f"{name}.png").convert("RGB")))
        for s in settings
    ]


def load_anchor_templates() -> dict:
    config = msgpack.unpackb(CONFIG_PATH.read_bytes(), raw=False)
    return {
        e["name"]: np.array(e["data"], dtype=np.float64).reshape(
            e["offset"]["height"], e["offset"]["width"], 4
        )[..., :3]
        for e in config
        if e["tag"] == "Anchor"
    }


def main():
    args = sys.argv[1:]
    sweep = "--sweep" in args
    names = [a for a in args if a != "--sweep"]
    templates = load_anchor_templates()
    for name in names or templates.keys():
        settings, means = sweep_means(name) if sweep else model_means(name, templates)
        c = np.polyfit(means, settings, 2)
        max_err = np.abs(np.polyval(c, means) - settings).max()
        print(f"// {name}: mean {means[0]:.1f}..{means[-1]:.1f}, max error {max_err:.2f} settings")
        print(f"brightness: [{c[0]:.10e}, {c[1]:.10f}, {c[2]:.7f}],")


if __name__ == "__main__":
    main()
