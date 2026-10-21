"""
Gives every Serca payload under arena_data/scenarios the Serca books added in October 2026, none owned, at the
frontend's fallback price, so they load against the constants that have the books. Payloads that already have
them are left alone; a changed payload is a new case to the arena.

    python scripts/arena/add_serca_books.py
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BOOKS = ["12-15 Armor", "12-15 Weapon", "16-19 Armor", "16-19 Weapon"]
PRICE = 15000.0
TAXED = 14250.0  # 5% tax, floored


def main():
    changed = 0
    for path in sorted((ROOT / "arena_data/scenarios").rglob("*.json")):
        raw = path.read_text(encoding="utf-8")
        scenario = json.loads(raw)
        touched = False
        for step in scenario["steps"]:
            if step["tier"] != 1 or BOOKS[0] in step["material_labels"]:
                continue
            step["material_labels"] += BOOKS
            for book in BOOKS:
                step["materials"][book] = {"bound": 0.0, "roster": 0.0, "tradable": 0.0, "taxed_price": TAXED, "market_price": PRICE}
            touched = True
        if touched:
            # keep the file's own spacing: the arena writes it compact, the seeded sets were written by Python
            separators = (", ", ": ") if raw.startswith('{"name": ') else (",", ":")
            end = "\n" if raw.endswith("\n") else ""
            path.write_text(json.dumps(scenario, separators=separators, ensure_ascii=False) + end, encoding="utf-8", newline="\n")
            changed += 1
    print(f"{changed} scenarios given the Serca books")


if __name__ == "__main__":
    main()
