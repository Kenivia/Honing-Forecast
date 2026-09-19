"""
Derive the ANCHORS_LOOKUP calibration constants used by est_ingame_brightness().

For each anchor image, across a set of known in-game brightness settings:

  1. Compute best_mean_f exactly as the Rust pipeline does: the mean of
     `image::imageops::grayscale()`, i.e. Rec.709 luma applied directly to the
     raw (gamma-encoded) u8 channel values:

         luma = 0.2126*R + 0.7152*G + 0.0722*B

  2. Fit the FORWARD relationship best_mean_f(setting) as a sanity check
     (should be ~linear, printed for diagnostics only).

  3. Fit the INVERSE relationship setting(best_mean_f) as a degree-2
     polynomial (quadratic beats linear here - see printed max error).
     This inverse polynomial is what actually gets stored, because at
     runtime we observe best_mean_f and need to recover the setting.

  4. Print the coefficients formatted as a Rust snippet ready to paste into
     ANCHORS_LOOKUP, plus a per-anchor accuracy report.

Expected layout
---------------
ROOT_DIR/
    0/     <anchor files...>.png
    5/     <anchor files...>.png
    10/    <anchor files...>.png
    ...
    100/   <anchor files...>.png

i.e. one subdirectory per known brightness setting (named exactly by the
integer setting value), each containing the same set of anchor PNGs. This is
the same layout produced by unzipping a `debug-icons-<N>.zip` per setting
into a folder named <N>.

"""

from pathlib import Path

import numpy as np
from PIL import Image

ROOT_DIR = Path("./scripts/brightness/all icons")
ANCHOR_FILES = [
    "Char Inventory Anchor 1.png",
    "Char Inventory Anchor 2.png",
]
INVENTORY_TYPE = "DUMMY_TYPE"

# Rec.709 luma weights, matching image::imageops::grayscale() in the `image` crate.
LUMA_WEIGHTS = np.array([0.2126, 0.7152, 0.0722])


def discover_settings(root: Path) -> list[int]:
    settings = []
    for p in root.iterdir():
        if p.is_dir():
            try:
                settings.append(int(p.name))
            except ValueError:
                continue
    return sorted(settings)


def best_mean_f(path: Path) -> float:
    img = np.array(Image.open(path).convert("RGBA")).astype(np.float64)[..., :3]
    return float((img * LUMA_WEIGHTS).sum(axis=-1).mean())


def fit_anchor(root: Path, anchor_file: str, settings: list[int]):
    settings_arr = np.array(settings, dtype=np.float64)
    mean_f = np.array(
        [best_mean_f(root / str(s) / anchor_file) for s in settings], dtype=np.float64
    )

    # Forward fit (diagnostics only): best_mean_f as a function of setting.
    fwd_slope, fwd_intercept = np.polyfit(settings_arr, mean_f, 1)
    fwd_pred = fwd_slope * settings_arr + fwd_intercept
    fwd_r2 = 1.0 - np.sum((mean_f - fwd_pred) ** 2) / np.sum(
        (mean_f - mean_f.mean()) ** 2
    )

    # Inverse fit (what gets stored): setting as a function of best_mean_f.
    inv_linear = np.polyfit(mean_f, settings_arr, 1)
    inv_quad = np.polyfit(mean_f, settings_arr, 2)

    lin_err = np.abs(np.polyval(inv_linear, mean_f) - settings_arr)
    quad_err = np.abs(np.polyval(inv_quad, mean_f) - settings_arr)

    return {
        "anchor_file": anchor_file,
        "settings": settings,
        "mean_f": mean_f,
        "fwd_slope": fwd_slope,
        "fwd_intercept": fwd_intercept,
        "fwd_r2": fwd_r2,
        "inv_quad": inv_quad,
        "lin_max_err": lin_err.max(),
        "quad_max_err": quad_err.max(),
    }


def format_rust_snippet(results: list[dict], variant_names: list[str]) -> str:
    lines = [f"(\n    InventoryType::{INVENTORY_TYPE},\n    vec!["]
    for name, r in zip(variant_names, results):
        c = r["inv_quad"]
        lines.append(
            f'        ("{name}".to_string(), None, '
            f"[{c[0]:.10e}, {c[1]:.10f}, {c[2]:.7f}]),"
        )
    lines.append("    ],\n),")
    return "\n".join(lines)


def main():
    if not ROOT_DIR.is_dir():
        raise SystemExit(f"ROOT_DIR {ROOT_DIR} is not a directory")

    settings = discover_settings(ROOT_DIR)
    if len(settings) < 3:
        raise SystemExit(f"Need at least 3 brightness settings for a quadratic fit; found {settings}")

    anchor_files = ANCHOR_FILES
    if not anchor_files:
        raise SystemExit("ANCHOR_FILES must contain at least one filename")

    variant_names = [Path(f).stem for f in anchor_files]

    print(f"Settings found: {settings}")
    print(f"Anchor files:   {anchor_files}\n")

    results = []
    for f in anchor_files:
        r = fit_anchor(ROOT_DIR, f, settings)
        results.append(r)
        print("=" * 78)
        print(f"Anchor: {f}")
        print("=" * 78)
        print(f"  best_mean_f at each setting:")
        for s, mf in zip(r["settings"], r["mean_f"]):
            print(f"    setting {s:5.1f}  ->  best_mean_f = {mf:8.3f}")
        print(f"\n  Forward fit (diagnostic): best_mean_f = {r['fwd_slope']:.4f} * setting "
              f"+ {r['fwd_intercept']:.3f}   (R^2 = {r['fwd_r2']:.6f})")
        print(f"  Inverse fit max error:  linear = {r['lin_max_err']:.2f} settings   "
              f"quadratic = {r['quad_max_err']:.2f} settings")
        c = r["inv_quad"]
        print(f"  Stored quadratic coefficients [a, b, c] "
              f"(setting = a*mean_f^2 + b*mean_f + c):")
        print(f"    [{c[0]:.10e}, {c[1]:.10f}, {c[2]:.7f}]")
        print()

    print("=" * 78)
    print("Rust snippet for ANCHORS_LOOKUP")
    print("=" * 78)
    print(format_rust_snippet(results, variant_names))


if __name__ == "__main__":
    main()