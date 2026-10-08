"""
Pulls lostarkcodex.com's item list (one 60 MB request) into scripts/codex/items.json, which is not tracked.

    python scripts/codex/fetch_items.py
"""

import json
import re

from codex import INDEX, RARITIES, get, plain


def main():
    rows = json.loads(get("/query.php?a=items&l=us").content.decode("utf-8-sig"))["aaData"]
    items = []
    for row in rows:
        icon = re.search(r"/icons/([^\"]+)\.webp", row[1])
        rarity = min(row[5], len(RARITIES) - 1)
        items.append([row[0], plain(row[2]), RARITIES[rarity], icon.group(1) if icon else None])
    INDEX.write_text(json.dumps(items, ensure_ascii=False), encoding="utf-8")
    print(f"wrote {len(items)} items, {len({item[3] for item in items})} icons, to {INDEX}")


if __name__ == "__main__":
    main()
