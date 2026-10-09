"""
Scores the output of the number_bench bin: wrong reads per clean-up variant and alphabet.

    python scripts/tooltips/number_bench.py <tsv>...

Stills of one scene at several brightness settings (a folder of them) are scored against the read
most of them agree on. Recordings are scored against the tooltip amounts, per read and per slot
as the scanner would end up: the last read, and the vote over the last five.
"""

import collections
import os
import re
import sys

VOTES = 5


def digits(text):
    # a single item shows no number
    return re.sub(r"\D", "", text) or "1"


def vote(reads):
    reads = reads[-VOTES:]
    count = collections.Counter(reads)
    return next(x for x in reversed(reads) if count[x] == max(count.values()))


names, reads, truth = [], [], {}
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8", errors="replace"):
        part = line.rstrip("\n").split("\t")
        if part[0] == "VARIANTS":
            names = [f"{name}, {alphabet}" for name in part[1:] for alphabet in ("full", "digits")]
        elif part[0] == "READ":
            source = path if part[1] == "stdin" else part[1]
            reads.append((source, part[3], part[4], (part[5:] + [""] * len(names))[: len(names)]))
        elif part[0] == "TRUTH":
            truth[(path, part[2], part[3])] = digits(part[4])

rows = collections.defaultdict(dict)

# stills: one scene per folder
scenes = collections.defaultdict(lambda: collections.defaultdict(list))
for source, slot, icon, texts in reads:
    if source.endswith(".png"):
        scenes[os.path.dirname(source)][slot].append(texts)
for folder, slots in scenes.items():
    if max(map(len, slots.values())) < 5:
        continue
    total = sum(map(len, slots.values()))
    for index, name in enumerate(names):
        wrong = 0
        for texts in slots.values():
            agreed = collections.Counter(digits(x) for row in texts for x in row).most_common(1)[0][0]
            wrong += sum(digits(row[index]) != agreed for row in texts)
        rows[name][f"{os.path.basename(folder)} /{total}"] = wrong

# recordings
by_slot = collections.defaultdict(list)
for source, slot, icon, texts in reads:
    by_slot[(source, slot, icon)].append(texts)
known = {key: (amount if len(amount) <= 4 else "9999") for key, amount in truth.items() if key in by_slot}
total = sum(len(by_slot[key]) for key in known)
failed = collections.defaultdict(list)
for index, name in enumerate(names):
    wrong = last = voted = 0
    for key, amount in known.items():
        sequence = [digits(row[index]) for row in by_slot[key]]
        wrong += sum(x != amount for x in sequence)
        last += sequence[-1] != amount
        if vote(sequence) != amount:
            voted += 1
            failed[name].append((key[1], key[2], amount, sequence))
    rows[name][f"recording reads /{total}"] = wrong
    rows[name][f"slots, last read /{len(known)}"] = last
    rows[name][f"slots, vote of {VOTES} /{len(known)}"] = voted

columns = list(rows[names[0]])
print("wrong reads".ljust(34) + "".join(column.rjust(28) for column in columns))
for name in names:
    print(name.ljust(34) + "".join(str(rows[name].get(column, "")).rjust(28) for column in columns))
for name in names[:2]:
    print(f"\nslots the vote still gets wrong, {name}:")
    for slot, icon, amount, sequence in failed[name]:
        print(f"  {slot}  {icon}  really {amount}  read {sequence}")
