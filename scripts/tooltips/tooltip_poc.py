"""
Hover tooltip parsing, proof of concept. Exploration only, not wired into the scanner.

    python scripts/tooltips/tooltip_poc.py                      every still in the example folders
    python scripts/tooltips/tooltip_poc.py <image> ...          those stills
    python scripts/tooltips/tooltip_poc.py --video <mp4> [fps]  per-frame summary (needs opencv)

Annotated copies go to scripts/tooltips/out. Needs numpy + pillow; rapidocr-onnxruntime is used to
read the located text if it is installed.

All pixel constants are 1080p and scaled by the UI height. Colours are for the reference
brightness; the production scanner would normalise first.

The cursor is never matched and may cover the tooltip's left edge, so everything is anchored on
the tooltip's right side and icon matching ignores its worst pixels.
"""

import difflib
import json
import re
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = Path(__file__).resolve().parent / "out"
ICONS_DIR = ROOT / "templates" / "Icons"
EXAMPLES = [
    *sorted((ROOT / "scripts/brightness/Hover tooltips").glob("*.png")),
    ROOT / "scripts/brightness/1440p raw/hover tooltip.png",
    ROOT / "scripts/brightness/1080p raw/hover tooltip.png",
    # no tooltip on screen, must come back empty
    ROOT / "scripts/brightness/1080p raw/inventory.png",
    ROOT / "scripts/brightness/1080p raw/Storage npc char-stoage-page-1.png",
    ROOT / "scripts/brightness/1440p raw/storage page 1.png",
    ROOT / "scripts/brightness/1440p raw/storage page 3.png",
    ROOT / "scripts/brightness/1440p raw/pet icon.png",
]

TITLE_WIDTH = 307  # flat part of the title bar
TITLE_MIN_ROWS = 4  # the clean strip above the title text is 11 rows; compression can break some of them
TITLE_COLOUR = np.array([30, 34, 39])  # at the reference brightness
TITLE_HEIGHTS = (25, 80)  # 34 with a one-line title, 52 with two
BODY_TO_TITLE = np.array([0.47, 0.56, 0.51])  # body colour (14, 19, 20) over title colour (30, 34, 39)
ICON_SIZE = 64
ICON_OFFSET = (7, 17)  # from title left / title bottom
ICON_PASS = 20.0
KEEP = 0.75  # share of an icon's pixels that are scored, so a cursor over the rest does not matter
SLOT_SIZE = 46
SLOT_RIGHT_OF_TITLE = 8  # tooltip on the slot's right: gap from the slot's right edge to the title
SLOT_LEFT_OF_TITLE = 12  # tooltip on the slot's left: gap from the title's right edge to the slot
SLOT_PASS = 30.0
PANEL_ICON_LEFT = 18  # chest content icons: 30px squares at a fixed x inside the black panel
PANEL_ICON_SIZES = (29, 30, 31)
PANEL_ICON_PASS = 24.0  # small icons are resampled by the game, so they sit further from the source art
CURSOR_REACH = 100  # how far the largest cursor gets into the tooltip from its left edge
CHARACTER_BIND_WIDTH = 120  # "Bound to Character" ends 130px in, "Bound to Roster" 107px. English only

# in-game title -> icon template (or the title itself when it has none), from the shared item table
ITEM_TITLES = {
    item["title"]: item.get("icon", item["title"])
    for name in ("items.json", "chest.json")
    for item in json.loads((ROOT / "templates" / name).read_text(encoding="utf-8"))
    if "title" in item
}
TITLE_PASS = 0.92  # similarity to a known title; "... Stone Pouch" is 0.86 similar to "... Stone"
TITLE_MARGIN = 0.05  # over the next best


def ui_scale(height):
    return round(height / 360) / 3  # 1078 and 1079 tall captures are still 1080p


# ---------------------------------------------------------------- 1. is there a tooltip


# The title bar is fully opaque and one colour, so the strip above its text is a flat run that ends at
# the tooltip's right edge. Only half of it has to be visible.
# Lossless captures need no colour at all: flat means identical neighbours. Lossy ones are matched
# against the known title colour instead, since compression leaves nothing identical.
def find_title(im, s, tolerance=0):
    width = round(TITLE_WIDTH * s)
    signed = im.astype(np.int16)
    if tolerance:
        near = np.abs(signed - TITLE_COLOUR).max(axis=2) <= tolerance
        flat = near[:, 1:] & near[:, :-1]
    else:
        flat = np.all(im[:, 1:] == im[:, :-1], axis=2) & (im[:, 1:].max(axis=2) < 100)
    window = width // 2
    csum = np.concatenate([np.zeros((im.shape[0], 1), int), np.cumsum(flat, axis=1)], axis=1)
    long_enough = (csum[:, window:] - csum[:, :-window]) == window

    slack = 3 if tolerance else 1  # compression smears the run's ends
    ends = {}
    for y in np.flatnonzero(long_enough.any(axis=1)):
        edges = np.flatnonzero(np.diff(np.concatenate([[0], flat[y].astype(np.int8), [0]])))
        for start, stop in zip(edges[::2], edges[1::2]):
            if window <= stop - start <= width + slack:
                ends.setdefault(int(stop), []).append(int(y))

    for right, rows in sorted(ends.items(), key=lambda kv: -len(kv[1])):
        rows = sorted({r for near in range(right - slack, right + slack + 1) for r in ends.get(near, [])})
        need = round(TITLE_MIN_ROWS * s)
        top = next((r for i, r in enumerate(rows) if rows[i : i + need] == list(range(r, r + need))), None)
        x = right - width + 1
        if top is None or x < 0:
            continue

        # below the title the right margin drops to the darker body colour and stays there
        column = signed[top:, right - 2]
        title_colour = np.median(signed[top : top + need, right - width // 2 : right - 4].reshape(-1, 3), axis=0)
        if title_colour[2] - title_colour[0] < 4:  # blue-grey; rules out the neutral greys of the other windows
            continue
        off_title = np.flatnonzero(np.abs(column - title_colour).max(axis=1) > tolerance + 3)
        title_height = int(off_title[0]) if len(off_title) else 0
        if not TITLE_HEIGHTS[0] * s <= title_height <= TITLE_HEIGHTS[1] * s:
            continue
        body = column[title_height + 3 : title_height + 3 + round(100 * s)]
        body_colour = np.median(body, axis=0)
        steady = (np.abs(body - body_colour).max(axis=1) <= 14).mean() > 0.7
        # ~97% opaque over anything from black to white, so at most ~8 levels above its own colour
        lift = body_colour - title_colour * BODY_TO_TITLE
        if len(body) == round(100 * s) and steady and lift.min() >= -2 - tolerance and lift.max() <= 9 + tolerance:
            return x, top, width, title_height
    return None


# ---------------------------------------------------------------- 3. where things are inside it


def text_lines(mask, min_height):
    rows = mask.any(axis=1)
    edges = np.flatnonzero(np.diff(np.concatenate([[0], rows.astype(np.int8), [0]])))
    return [(a, b) for a, b in zip(edges[::2], edges[1::2]) if b - a >= min_height]


def colour_masks(rgb, lossy):
    r, g, b = (rgb[..., i].astype(int) for i in range(3))
    if lossy:
        yellow = (r > 190) & (b < 110) & (g > 0.75 * r) & (g < r)
    else:
        yellow = (r > 200) & (b < 70) & (g > 0.75 * r) & (g < 0.92 * r)
    red = (r > 130) & (g < 0.6 * r) & (b < 0.6 * r)
    white = (rgb.min(axis=2) > 150) & (rgb.max(axis=2).astype(int) - rgb.min(axis=2) < 30)
    return yellow, red, white


def parse_layout(im, s, x, y, width, title_height, lossy=False):
    title_bottom = y + title_height
    # the body is ~97% opaque, so its margins stay within a few levels of one colour
    body_colour = np.median(im[title_bottom + 3 : title_bottom + 9, x + width - 3].astype(int), axis=0)
    body_bottom, misses = title_bottom, 0
    for yy in range(title_bottom, im.shape[0]):
        left = np.abs(im[yy, x + 1 : x + 4].astype(int) - body_colour).max()
        right = np.abs(im[yy, x + width - 4 : x + width - 1].astype(int) - body_colour).max()
        misses = 0 if min(left, right) <= 14 else misses + 1
        if misses == 3:
            break
        body_bottom = yy if misses == 0 else body_bottom

    pad = round(4 * s)
    body = im[title_bottom:body_bottom, x + pad : x + width - pad]
    to_abs = lambda bx, by: (x + pad + bx, title_bottom + by)
    yellow, red, white = colour_masks(body, lossy)
    text = body.max(axis=2) > 110
    black = np.flatnonzero((body.max(axis=2) <= (10 if lossy else 2)).mean(axis=1) > 0.5)
    panel = (black[0], black[-1]) if len(black) >= 30 * s else None

    # lines as seen from column `start` rightwards
    def measure(start):
        out = []
        for top, bottom in text_lines(text[:, start:], 6 * s):
            cols = start + np.flatnonzero(text[top:bottom, start:].any(axis=0))
            yellow_cols = start + np.flatnonzero(yellow[top:bottom, start:].any(axis=0))
            line = {"top": top, "bottom": bottom, "left": int(cols.min()), "right": int(cols.max())}
            enough = len(yellow_cols) and yellow[top:bottom, start:].sum() >= 4
            line["yellow"] = (int(yellow_cols.min()), int(yellow_cols.max())) if enough else None
            line["red"] = red[top:bottom, start:].sum() > 0.5 * (body[top:bottom, start:].max(axis=2) > 150).sum()
            line["in_panel"] = panel is not None and panel[0] <= top <= panel[1]
            out.append(line)
        return out

    lines = measure(0)
    # the largest cursor reaches about 90px into the tooltip and would merge with any line it touches,
    # so the header and the amounts are read from the columns beyond it
    cursor_reach = round(CURSOR_REACH * s) - pad
    safe = [l for l in measure(cursor_reach) if l["bottom"] - l["top"] <= 20 * s and not l["in_panel"]]

    # "Amount Stacked: N" / "Total Amount Owned: N": white label, then yellow up to the end of the line.
    # "Consumed before tradable items." has no white before the yellow; the chest header has white after it.
    amounts = []
    for line in safe:
        if not line["yellow"]:
            continue
        start, end = line["yellow"]
        label_end = white[line["top"] : line["bottom"], max(start - round(16 * s), 0) : start]
        if end >= line["right"] - 2 and label_end.sum() >= 6 * s:
            amounts.append(line)
    # stacked and total are always one line apart, which rules out a description line that happens to qualify
    pitch = [b["top"] - a["top"] for a, b in zip(amounts, amounts[1:])]
    first = next((i for i, gap in enumerate(pitch) if 17 * s <= gap <= 23 * s), 0)
    amounts = amounts[first : first + 2]

    header = [l for l in safe if not amounts or l["top"] < amounts[0]["top"]]
    untradable = next((l for l in header if l["red"] and l["left"] > 0.45 * width), None)
    # "Bound to ..." is the line above "Untradable" that runs in from the left and stops short.
    # Only where it ends is used: "Character" ends further right than "Roster".
    bind = None
    if untradable:
        below_icon = (ICON_OFFSET[1] + ICON_SIZE) * s
        above = [
            l
            for l in header
            if below_icon <= l["top"] < untradable["top"] and l["left"] <= cursor_reach + 3 * s and l["right"] < 0.6 * width
        ]
        bind = above[-1] if above else None

    def box(line, left=None, right=None, grow=3):
        x0, y0 = to_abs(line["left"] if left is None else left, line["top"])
        x1, y1 = to_abs(line["right"] if right is None else right, line["bottom"])
        return tuple(int(v) for v in (x0 - grow, y0 - grow, x1 + grow + 1, y1 + grow))

    # chest contents sit on a pure black panel inset in the body: icon, then name over "xN"
    chest = []
    icon_left = round(PANEL_ICON_LEFT * s) - pad - 2
    icon_size = round(30 * s) + 4
    for line in lines:
        if not line["in_panel"] or line["bottom"] - line["top"] < 22 * s:
            continue
        cols = text[line["top"] : line["bottom"]].any(axis=0)
        name_left = icon_left + icon_size + int(np.argmax(cols[icon_left + icon_size :]))
        icon_top = (line["top"] + line["bottom"] - icon_size) // 2
        count_top, count_bottom = text_lines(text[line["top"] : line["bottom"], name_left:], 5 * s)[-1]
        count_cols = np.flatnonzero(text[line["top"] + count_top : line["top"] + count_bottom, name_left:].any(axis=0))
        chest.append(
            {
                "icon": tuple(int(v) for v in (*to_abs(icon_left, icon_top), *to_abs(icon_left + icon_size, icon_top + icon_size))),
                "count": tuple(
                    int(v)
                    for v in (
                        *to_abs(name_left - 3, line["top"] + count_top - 3),
                        *to_abs(name_left + count_cols.max() + 4, line["top"] + count_bottom + 3),
                    )
                ),
            }
        )
    # the header's yellow phrase tells the three kinds apart without reading it:
    # "Obtain [all] of the..." is one short word, "There is a [chance that you can obtain one]" runs to the
    # end of the line, "You can [select and obtain 1] of the" has white after it
    chest_header = next((l for l in lines if l["in_panel"] and l["yellow"]), None)
    chest_kind = None
    if chest_header:
        start, end = chest_header["yellow"]
        if end - start < 40 * s:
            chest_kind = "all"
        elif end >= chest_header["right"] - 2:
            chest_kind = "chance"
        else:
            chest_kind = "select"

    bind_kind = None
    if bind:
        bind_kind = "character" if bind["right"] + pad > CHARACTER_BIND_WIDTH * s else "roster"
    return {
        "scale": s,
        "title": (x, y, x + width, title_bottom),
        "body_bottom": body_bottom,
        "amounts": [box(l, left=l["yellow"][0], right=l["yellow"][1]) for l in amounts],
        "untradable": untradable is not None,
        "bind": box(bind) if bind else None,
        "bind_kind": bind_kind,
        "chest": chest,
        "chest_kind": chest_kind,
    }


# ---------------------------------------------------------------- 2. what is hovered


def load_icons():
    return {p.stem: Image.open(p).convert("RGBA") for p in sorted(ICONS_DIR.iterdir())}


# The raw icon's own alpha is the mask: the tooltip draws it over a gradient the slot templates do not have.
# Only the closest KEEP of the pixels count, so a cursor over the rest does not move the score.
def masked_distance(crop, icon, size):
    template = np.array(icon.resize((size, size), Image.LANCZOS)).astype(float)
    mask = template[..., 3] > 230
    keep = int(mask.sum() * KEEP)
    best = 999.0
    for dy in range(crop.shape[0] - size + 1):
        for dx in range(crop.shape[1] - size + 1):
            diff = np.abs(crop[dy : dy + size, dx : dx + size] - template[..., :3]).mean(axis=2)[mask]
            best = min(best, float(np.partition(diff, keep)[:keep].mean()))
    return best


def identify(crop, icons, sizes):
    scores = sorted((min(masked_distance(crop, icon, k) for k in sizes), name) for name, icon in icons.items())
    return scores[0], scores[1]


def tooltip_icon(im, layout, icons):
    s = layout["scale"]
    x, _, _, title_bottom = layout["title"]
    size = round(ICON_SIZE * s)
    left, top = x + round(ICON_OFFSET[0] * s) - 1, title_bottom + round(ICON_OFFSET[1] * s) - 1
    crop = im[top : top + size + 2, left : left + size + 2].astype(float)
    return (left, top, left + size + 2, top + size + 2), identify(crop, icons, [size])


# Fallback for when the cursor covers the icon. Every title line is centred, so a line whose two side
# gaps differ has something over it (the cursor) and the read is not trusted on its own:
# "Great Destiny Leapstone" with its start covered reads as a different, valid item.
def read_title(ocr, im, layout):
    s = layout["scale"]
    x0, y0, x1, bottom = layout["title"]
    bar = im[y0:bottom, x0:x1]
    text = bar.max(axis=2) > 120
    centred = True
    for top, low in text_lines(text, 6 * s):
        cols = np.flatnonzero(text[top:low].any(axis=0))
        centred &= abs(int(cols.min()) - (x1 - x0 - 1 - int(cols.max()))) <= 6 * s
    raw = read_text(ocr, im, (x0, y0, x1, bottom))
    name = raw.encode("ascii", "ignore").decode()
    name = re.sub(r"[\[(]\s*X[\s\d,.\]]*$", "", name.split("[")[0])  # drop the "[X n]" count
    name = re.sub(r"\(?Bound\)?", "", name)
    return " ".join(name.split()), bool(centred)


def match_title(name):
    if not name:
        return None
    # one wrong digit barely changes the similarity ("Level 3" / "Level 4"), so digits have to match exactly
    digits = re.findall(r"\d", name)
    titles = [title for title in ITEM_TITLES if re.findall(r"\d", title) == digits]
    scores = sorted(((difflib.SequenceMatcher(None, name.lower(), title.lower()).ratio(), title) for title in titles), reverse=True)
    scores += [(0.0, "")] * 2
    (best, title), (second, _) = scores[0], scores[1]
    return ITEM_TITLES[title] if best >= TITLE_PASS and best - second >= TITLE_MARGIN else None


# The tooltip hugs the hovered slot's column on either side. It shares the slot's top, unless the
# screen bottom pushed it up, in which case the slot is somewhere lower in the same column.
def slot_columns(layout):
    s = layout["scale"]
    x0, _, x1, _ = layout["title"]
    return {"right": x0 - round(SLOT_RIGHT_OF_TITLE * s) - round(SLOT_SIZE * s), "left": x1 + round(SLOT_LEFT_OF_TITLE * s)}


# The scanner would look these columns up in its slot grid and use what it remembers of each slot.
# The POC has no grid, so it slides the tooltip's icon down both columns. Fails when the cursor hides the slot.
def hovered_slot(im, layout, icon):
    size = round(SLOT_SIZE * layout["scale"])
    scores = []
    for side, left in slot_columns(layout).items():
        if left < 1 or left + size + 1 > im.shape[1]:
            continue
        for top in range(layout["title"][1] - 1, im.shape[0] - size, 1):
            crop = im[top : top + size, left - 1 : left + size + 1].astype(float)
            scores.append((masked_distance(crop, icon, size), side, (left, top, left + size, top + size)))
    best = min(scores)
    elsewhere = min(sc for sc in scores if abs(sc[2][1] - best[2][1]) > size // 2 or sc[1] != best[1])
    return best, elsewhere


# ---------------------------------------------------------------- driver


def analyse(im, icons, tolerance=0, find_slot=True):
    s = ui_scale(im.shape[0])
    title = find_title(im, s, tolerance)
    if title is None:
        return None
    layout = parse_layout(im, s, *title, lossy=tolerance > 0)
    layout["icon_box"], layout["icon"] = tooltip_icon(im, layout, icons)
    (best, name), _ = layout["icon"]
    layout["hovered"] = hovered_slot(im, layout, icons[name]) if find_slot and best < ICON_PASS else None
    sizes = sorted({round(k * s) for k in PANEL_ICON_SIZES})
    for item in layout["chest"]:
        x0, y0, x1, y1 = item["icon"]
        item["match"] = identify(im[y0:y1, x0:x1].astype(float), icons, sizes)
    return layout


def describe_icon(match, limit=ICON_PASS):
    (best, name), (second, runner_up) = match
    verdict = name if best < limit else "unknown"
    return f"{verdict} ({best:.1f}, next {runner_up} {second:.1f})"


def describe_trade(layout):
    return f"untradable/{layout['bind_kind']}" if layout["untradable"] else "tradable"


def read_text(ocr, im, box):
    if ocr is None or box is None:
        return ""
    x0, y0, x1, y1 = box
    crop = Image.fromarray(im[y0:y1, x0:x1]).resize(((x1 - x0) * 3, (y1 - y0) * 3), Image.LANCZOS)
    padded = Image.new("RGB", (crop.width + 60, crop.height + 60), tuple(int(v) for v in im[y0, x0]))
    padded.paste(crop, (30, 30))
    result, _ = ocr(np.array(padded))
    return " ".join(r[1] for r in result) if result else ""


# this OCR returns nothing for a lone digit, so a failed read is retried with the end of the label ("ed: ")
def read_amount(ocr, im, box, s):
    if ocr is None:
        return ""
    digits = "".join(ch for ch in read_text(ocr, im, box) if ch.isdigit())
    if digits:
        return digits
    x0, y0, x1, y1 = box
    text = read_text(ocr, im, (x0 - round(28 * s), y0, x1, y1))
    return "".join(ch for ch in text.split(":")[-1] if ch.isdigit()) if ":" in text else ""


def run_still(path, icons, ocr):
    im = np.array(Image.open(path).convert("RGB"))
    layout = analyse(im, icons)
    print(f"\n{path.parent.name}/{path.name}  {im.shape[1]}x{im.shape[0]}")
    if layout is None:
        print("  no tooltip")
        return

    x0, y0, x1, title_bottom = layout["title"]
    print(f"  tooltip at ({x0}, {y0}), title {title_bottom - y0}px, body to y={layout['body_bottom']}")
    print(f"  item: {describe_icon(layout['icon'])}")
    if ocr:
        name, centred = read_title(ocr, im, layout)
        print(f"  title: '{name}' -> {match_title(name)}" + ("" if centred else "  (off centre, something is over it)"))
    if layout["hovered"]:
        (best, side, box), (second, *_) = layout["hovered"]
        found = f"{box[:2]}, tooltip on its {side}, {box[1] - y0}px below the tooltip top" if best < SLOT_PASS else "not found"
        print(f"  hovered slot: {found} ({best:.1f}, next best elsewhere {second:.1f})")
    else:
        print(f"  hovered slot: icon not in templates, column x={list(slot_columns(layout).values())}")
    for label, box in zip(("stacked", "total"), layout["amounts"]):
        print(f"  {label}: box {box}  ocr '{read_amount(ocr, im, box, layout['scale'])}'")
    print(f"  {describe_trade(layout)}" + (f", bind line ocr '{read_text(ocr, im, layout['bind'])}'" if layout["bind"] else ""))
    if layout["chest"]:
        print(f"  chest: obtain {layout['chest_kind']}")
    for item in layout["chest"]:
        print(f"    {describe_icon(item['match'], PANEL_ICON_PASS)}  count ocr '{read_text(ocr, im, item['count'])}'")

    canvas = Image.fromarray(im)
    draw = ImageDraw.Draw(canvas)
    draw.rectangle((x0, y0, x1, layout["body_bottom"]), outline=(0, 255, 0))
    draw.rectangle(layout["title"], outline=(0, 255, 0))
    draw.rectangle(layout["icon_box"], outline=(255, 0, 255))
    for box in layout["amounts"]:
        draw.rectangle(box, outline=(255, 0, 0))
    if layout["bind"]:
        draw.rectangle(layout["bind"], outline=(0, 160, 255))
    for item in layout["chest"]:
        draw.rectangle(item["icon"], outline=(255, 0, 255))
        draw.rectangle(item["count"], outline=(255, 0, 0))
    size = round(SLOT_SIZE * layout["scale"])
    if layout["hovered"] and layout["hovered"][0][0] < SLOT_PASS:
        draw.rectangle(layout["hovered"][0][2], outline=(0, 255, 255), width=2)
    else:
        for left in slot_columns(layout).values():
            draw.rectangle((left, y0, left + size, y0 + size), outline=(0, 255, 255))
    pad = 70
    view = (max(x0 - pad, 0), max(y0 - 20, 0), min(x1 + pad, im.shape[1]), min(layout["body_bottom"] + 20, im.shape[0]))
    OUT_DIR.mkdir(exist_ok=True)
    canvas.crop(view).save(OUT_DIR / f"{path.parent.name} - {path.stem[:60]}.png")


# compressed video: nothing is an exact colour any more, so flatness gets a tolerance
def run_video(path, icons, fps):
    import cv2

    capture = cv2.VideoCapture(str(path))
    native = capture.get(cv2.CAP_PROP_FPS)
    step = max(1, round(native / fps)) if fps else 1
    print(f"{path.name}: {native:.0f} fps, reading every {step} frame(s)")
    index, previous = 0, None
    while True:
        ok, frame = capture.read()
        if not ok:
            break
        if index % step == 0:
            layout = analyse(frame[..., ::-1], icons, tolerance=8, find_slot=False)
            if layout is None:
                state, key = "none", None
            else:
                amounts = f"{len(layout['amounts'])} amount line(s)"
                state = f"tooltip at {layout['title'][:2]}  {describe_icon(layout['icon'])}  {describe_trade(layout)}  {amounts}  {len(layout['chest'])} chest row(s)"
                key = (layout["title"][:2], state.split(" (")[1:2], describe_trade(layout), amounts)
            if key != previous or index == 0:
                print(f"  {index / native:5.2f}s  frame {index:3d}  {state}")
                previous = key
        index += 1


def main():
    args = sys.argv[1:]
    icons = load_icons()
    if args and args[0] == "--video":
        run_video(Path(args[1]), icons, float(args[2]) if len(args) > 2 else 0)
        return
    try:
        from rapidocr_onnxruntime import RapidOCR

        ocr = RapidOCR()
    except ImportError:
        ocr = None
    for path in [Path(a) for a in args] or EXAMPLES:
        run_still(path, icons, ocr)


if __name__ == "__main__":
    main()
