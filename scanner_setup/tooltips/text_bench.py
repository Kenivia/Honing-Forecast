"""
Scores the output of the text_bench bin: per text strip variant, how many tooltip texts read right.

    python scanner_setup/tooltips/text_bench.py <tsv>... [--show N]

A title or a chest row's name is right when it is a title the game has, to the letter (a misread is
almost never another real one). A count, an amount, a description line or a currency line has no
such truth: it is scored against what most variants read, so only read it as a hint. --show lists
N units each where the first variant and another disagree.
"""

import collections
import html
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "game_files"))
from game import CARDS, load_index, table  # noqa: E402


def clean(text):
    return " ".join(html.unescape(re.sub(r"<[^>]+>", "", text)).split())


def vocabulary():
    titles = {clean(x[1]) for x in load_index()}
    cards = table("Item").execute(
        "select m.MSG from Item i join msg.GameMsg_English m on m.KEY = i.Name where i.Category = ?", (CARDS,)
    )
    titles |= {clean(x[0]).removesuffix(" Card") for x in cards}
    for column in ("Name", "Desc"):
        try:
            rows = table("Money").execute(f"select m.MSG from Money x join msg.GameMsg_English m on m.KEY = x.{column}")
            titles |= {clean(x[0]) for x in rows}
        except Exception:
            pass
    return titles - {""}


def title_of(text):
    # the white "[X n]" after the name is not part of it
    return re.sub(r"\s*\[\s*[xX]\s*[\d,.]*\s*\]?\s*$", "", text).strip()


def number(text):
    return re.sub(r"\D", "", text)


def main():
    args = sys.argv[1:]
    show = 0
    if "--show" in args:
        at = args.index("--show")
        show = int(args[at + 1])
        del args[at : at + 2]
    load_index()  # attaches GameMsg to the Item connection
    known = vocabulary()

    variants = None
    units = collections.OrderedDict()  # (source, role, unit) -> per variant, the lines in order
    for path in args:
        for line in Path(path).read_text(encoding="utf-8").splitlines():
            parts = line.split("\t")
            if parts[0] == "VARIANTS":
                variants = parts[1:]
            elif parts[0] == "LINE":
                source, _id, role, unit = parts[1:5]
                reads = parts[5:] + [""] * (len(variants) - len(parts[5:]))
                units.setdefault((source, role, unit), []).append(reads)

    texts = {}  # unit -> per variant, its text
    for key, lines in units.items():
        texts[key] = [" ".join(clean(x[v]) for x in lines) for v in range(len(variants))]

    def right(key, text, all_texts):
        role = key[1]
        if role == "title":
            return title_of(text) in known
        if role == "name":
            return text.removesuffix(" (Bound)") in known or text in known
        # no truth: the read most variants agree on
        majority = collections.Counter(number(x) if role in ("count", "amount") else x for x in all_texts)
        best = majority.most_common(1)[0][0]
        return (number(text) if role in ("count", "amount") else text) == best

    roles = sorted({key[1] for key in texts})
    scored = {key: [right(key, t, ts) for t in ts] for key, ts in texts.items()}
    width = max(len(x) for x in variants)
    print(f"{'':{width}}  " + "  ".join(f"{r:>13}" for r in roles))
    for v, name in enumerate(variants):
        cells = []
        for role in roles:
            keys = [k for k in texts if k[1] == role]
            wrong = sum(not scored[k][v] for k in keys)
            cells.append(f"{wrong:>5} of {len(keys):<5}")
        print(f"{name:{width}}  " + "  ".join(cells) + "   wrong")
    print("(title and name: not a title the game has; the rest: not what most variants read)")

    for v, name in list(enumerate(variants))[1:]:
        fixed = [k for k in texts if k[1] in ("title", "name") and scored[k][v] and not scored[k][0]]
        broke = [k for k in texts if k[1] in ("title", "name") and scored[k][0] and not scored[k][v]]
        print(f"\n{name}: {len(fixed)} titles and names fixed, {len(broke)} broken against {variants[0]}")
        for label, keys in (("fixed", fixed), ("broke", broke)):
            for k in keys[:show]:
                print(f"  {label} {k[0]} {k[2]}: {texts[k][0]!r} -> {texts[k][v]!r}")


if __name__ == "__main__":
    main()
