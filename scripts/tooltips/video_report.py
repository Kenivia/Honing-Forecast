"""
Run tooltip_poc over every frame of a recording, group frames into hovers and vote within each.
Exploration only. Needs opencv and rapidocr-onnxruntime.

    python scripts/tooltips/video_report.py <mp4> [roster amounts csv]

The csv is the roster storage page as a grid of "Amount Stacked" values; with it, each hover over
roster storage is checked against the slot the tooltip's position points at.
"""

import csv
import json
import sys
from collections import Counter
from pathlib import Path

import cv2
import numpy as np
from PIL import Image
from rapidocr_onnxruntime import RapidOCR

import tooltip_poc as poc

OUT_DIR = Path(__file__).resolve().parent / "out"
TOLERANCE = 8
# storage npc layout at 1080p: roster slot (0, 0) and the grid pitch
ROSTER_ORIGIN = (258.0, 193.06)
PITCH = 52.3125
ROSTER_COLS = 6


# rows of clean title colour above the title text, counted from the bar's real top
def title_strip(im, layout):
    x0, y0, x1, bottom = layout["title"]
    colour = np.median(im[y0 + 1 : y0 + 4, x1 - 150 : x1 - 4].reshape(-1, 3).astype(int), axis=0)
    top = y0
    while top > 0 and np.abs(im[top - 1, x1 - 150 : x1 - 4].astype(int) - colour).max(axis=1).mean() <= TOLERANCE:
        top -= 1
    text_rows = np.flatnonzero((im[top:bottom, x0 + 10 : x1 - 10].max(axis=2) > 120).sum(axis=1) >= 3)
    return (int(text_rows[0]) if len(text_rows) else -1), bottom - top


def scan(path, icons, ocr):
    capture = cv2.VideoCapture(str(path))
    records, index = [], 0
    while True:
        ok, frame = capture.read()
        if not ok:
            break
        im = np.ascontiguousarray(frame[..., ::-1])
        layout = poc.analyse(im, icons, tolerance=TOLERANCE, find_slot=False)
        if layout is None:
            records.append(None)
        else:
            (best, name), _ = layout["icon"]
            strip, title_height = title_strip(im, layout)
            title, centred = poc.read_title(ocr, im, layout)
            records.append(
                {
                    "frame": index,
                    "title": title,
                    "centred": centred,
                    "x": layout["title"][0],
                    "y": layout["title"][1],
                    "strip": strip,
                    "title_height": title_height,
                    "item": name if best < poc.ICON_PASS else None,
                    "trade": poc.describe_trade(layout),
                    "stacked": poc.read_amount(ocr, im, layout["amounts"][0], layout["scale"]) if layout["amounts"] else None,
                    "chest": layout["chest_kind"],
                    "chest_items": [m["match"][0][1] if m["match"][0][0] < poc.PANEL_ICON_PASS else "?" for m in layout["chest"]],
                }
            )
            if index % 15 == 0:
                x0, y0, x1, _ = layout["title"]
                view = im[max(y0 - 10, 0) : y0 + 330, max(x0 - 60, 0) : x1 + 10]
                Image.fromarray(view).save(OUT_DIR / "frames" / f"{index:04d}.jpg", quality=80)
        index += 1
        if index % 200 == 0:
            print(f"  {index} frames", flush=True)
    return records


# a hover is a run of frames whose tooltip stays put; a few missed frames inside it do not split it
def group(records, gap=4):
    hovers, current, missing = [], [], 0
    for record in records:
        same = record and current and abs(record["x"] - current[-1]["x"]) <= 4 and abs(record["y"] - current[-1]["y"]) <= 4
        if record and (not current or same):
            current.append(record)
            missing = 0
            continue
        missing += 1
        if current and (record or missing > gap):
            hovers.append(current)
            current, missing = ([record] if record else []), 0
    if current:
        hovers.append(current)
    return hovers


def vote(values):
    counts = Counter(v for v in values if v not in (None, ""))
    if not counts:
        return None, 0.0
    value, n = counts.most_common(1)[0]
    return value, n / len(values)


def roster_slot(x, y):
    col = (x - poc.SLOT_RIGHT_OF_TITLE - poc.SLOT_SIZE + 1 - ROSTER_ORIGIN[0]) / PITCH
    row = (y - ROSTER_ORIGIN[1]) / PITCH
    if abs(col - round(col)) > 0.08 or not 0 <= round(col) < ROSTER_COLS:
        return None
    return round(col), row


def main():
    video = Path(sys.argv[1])
    cache = OUT_DIR / f"{video.stem}.json"
    (OUT_DIR / "frames").mkdir(parents=True, exist_ok=True)
    if cache.exists():
        records = json.loads(cache.read_text())
    else:
        records = scan(video, poc.load_icons(), RapidOCR())
        cache.write_text(json.dumps(records))

    grid = None
    if len(sys.argv) > 2:
        grid = [row for row in csv.reader(open(sys.argv[2]))]

    seen = [r for r in records if r]
    print(f"{len(records)} frames, tooltip found on {len(seen)}")
    print("title height x clean strip rows (frames):", sorted(Counter((r["title_height"], r["strip"]) for r in seen).items()))

    hovers = group(records)
    print(f"{len(hovers)} hovers\n")
    print("frames      at          title/strip  item (icon)       item (title)      trade                 stacked (read/located)   chest")
    checked = correct = 0
    names = []
    for hover in hovers:
        first, last = hover[0]["frame"], hover[-1]["frame"]
        x, y = int(np.median([r["x"] for r in hover])), int(np.median([r["y"] for r in hover]))
        item, _ = vote([r["item"] for r in hover])
        trade, trade_share = vote([r["trade"] for r in hover])
        stacked, share = vote([r["stacked"] for r in hover])
        located = np.mean([r["stacked"] is not None for r in hover])
        chest, _ = vote([r["chest"] for r in hover])
        contents, _ = vote(["+".join(r["chest_items"]) for r in hover])
        height, _ = vote([r["title_height"] for r in hover])
        strip, _ = vote([r["strip"] for r in hover])
        # frames where the title is centred outvote the rest
        title, title_share = vote([r["title"] for r in hover if r["centred"]] or [r["title"] for r in hover])
        by_title = poc.match_title(title)
        names.append((item, by_title, title))
        line = f"{first:4d}-{last:4d}  ({x:4d},{y:4d})  {height}/{strip:<9}  {str(item):16s}  {str(by_title):16s}  {trade:20s}{str(stacked):>7s} ({share:.2f}/{located:.2f})    {chest or ''} {contents or ''}"

        slot = roster_slot(x, y) if grid else None
        if slot:
            col, row = slot
            aligned = abs(row - round(row)) < 0.06 and 0 <= round(row) < len(grid)
            if aligned:
                expected = [grid[round(row)][col]]
                where = f"roster ({col},{round(row)})"
            else:  # pushed up by the screen bottom: any slot lower in the column
                expected = [grid[r][col] for r in range(max(int(np.ceil(row)), 0), len(grid))]
                where = f"roster col {col}, pushed up"
            expected = [e for e in expected if e.strip()]
            if expected:
                numbers = [e for e in expected if e != "N/A"]
                ok = (stacked in numbers) if stacked else ("N/A" in expected)
                checked += 1
                correct += ok
                line += f"   | {where}: csv {'/'.join(expected)} -> {'ok' if ok else 'WRONG'}"
        print(line + f"   | '{title}' ({title_share:.2f})")

    both = [(i, t) for i, t, _ in names if i and t]
    print(f"\nhovers identified by icon: {sum(bool(i) for i, _, _ in names)}, by title: {sum(bool(t) for _, t, _ in names)}, by either: {sum(bool(i or t) for i, t, _ in names)}")
    print(f"both available on {len(both)}, agreeing on {sum(i == t for i, t in both)}; disagreements: {[(i, t) for i, t in both if i != t]}")
    print("known-title hovers the icon missed:", dict(Counter(t for i, t, _ in names if t and not i)))
    print("icon hits whose title is not in the table:", dict(Counter(f"{i}: {n}" for i, t, n in names if i and not t)))
    if grid:
        print(f"\nroster hovers checked against the csv: {correct}/{checked} correct")


if __name__ == "__main__":
    main()
