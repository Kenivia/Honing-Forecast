"""
Cuts each client language's UI font down to what a slot's number uses ("0123456789+"), for the scanner's
shape reader to draw numbers with. Reads the fonts extract.py wrote; writes <out>/<Language>.ttf, a few KB each.

    python scanner_setup/game_files/fonts.py [out folder, default public/fonts]

The font is the one each language's FontMap.xml gives the UI's main style ($YG760); Korean is the
default map's. French, German and Spanish use the English file.
"""

import re
import sys
from pathlib import Path

from fontTools import subset

from game import FONTS, ROOT

LANGUAGES = {"English": "English", "Korean": "", "Russian": "Russian", "Japanese": "Japanese", "Chinese": "Chinese", "Taiwanese": "Taiwanese"}


def font_file(folder: str) -> Path:
    fontmap = (FONTS / folder / "FontMap.xml").read_text(encoding="utf-8")
    name = re.search(r'Key="\$YG760" File="([^"]+)"', fontmap).group(1)
    # the map's case does not always match the file's
    found = [path for path in (FONTS / folder).iterdir() if path.name.lower() == name.lower()]
    return found[0]


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "public" / "fonts"
    out.mkdir(parents=True, exist_ok=True)
    options = subset.Options()
    options.hinting = False
    options.layout_features = []
    options.name_IDs = ["*"]
    for language, folder in LANGUAGES.items():
        source = font_file(folder)
        font = subset.load_font(str(source), options)
        subsetter = subset.Subsetter(options)
        subsetter.populate(text="0123456789+")
        subsetter.subset(font)
        target = out / f"{language}.ttf"
        subset.save_font(font, str(target), options)
        print(f"{language}: {source.name} -> {target.name} ({target.stat().st_size:,} bytes)")


if __name__ == "__main__":
    main()
