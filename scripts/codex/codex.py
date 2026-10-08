"""What the scripts here share: where things are kept and how lostarkcodex.com is asked."""

import json
import re
import time
from html import unescape
from pathlib import Path

import requests

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
ICONS = HERE / "icons"  # <icon name>.webp, as the site names them
INDEX = HERE / "items.json"  # written by fetch_items.py
SITE = "https://lostarkcodex.com"
RARITIES = ["common", "uncommon", "rare", "epic", "legendary", "relic", "ancient", "sidereal"]
DELAY = 0.2

session = requests.Session()
session.headers["User-Agent"] = "Mozilla/5.0"


def get(path):
    time.sleep(DELAY)
    response = session.get(SITE + path, timeout=60)
    response.raise_for_status()
    return response


def plain(html):
    return unescape(re.sub(r"<[^>]+>", "", html)).strip()


# every item as [id, title, rarity, icon name]
def load_index():
    return json.loads(INDEX.read_text(encoding="utf-8"))


# icon name -> the items drawn with it
def by_icon():
    out = {}
    for item in load_index():
        out.setdefault(item[3], []).append(item)
    return out
