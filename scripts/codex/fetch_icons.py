"""
Pulls every icon items.json names that is not in scripts/codex/icons yet. Run fetch_items.py first.

    python scripts/codex/fetch_icons.py
"""

from codex import ICONS, get, load_index


def main():
    ICONS.mkdir(exist_ok=True)
    names = sorted({item[3] for item in load_index() if item[3]})
    missing = [name for name in names if not (ICONS / f"{name}.webp").exists() or not (ICONS / f"{name}.webp").stat().st_size]
    print(f"{len(missing)} of {len(names)} icons to fetch")
    # what the site sends for an icon it does not have
    placeholder = get("/icons/__none__.webp").content
    saved = 0
    for count, name in enumerate(missing, 1):
        content = get(f"/icons/{name}.webp").content
        if content != placeholder:
            (ICONS / f"{name}.webp").write_bytes(content)
            saved += 1
        if count % 100 == 0:
            print(f"{count} / {len(missing)}", flush=True)
    print(f"saved {saved}, the site has no art for the other {len(missing) - saved}")


if __name__ == "__main__":
    main()
