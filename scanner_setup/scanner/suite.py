"""
The scanner's native suite: tooltip_test over every folder of stills under scanner_setup/test_stills
and ignored/scanner_tests/stills, and capture_replay over every debug capture under
ignored/scanner_tests/recordings. Runs are kept by name in target/scanner-suite/.

    python scanner_setup/scanner/suite.py run <name> [--seq]   about 5 minutes
    python scanner_setup/scanner/suite.py cmp <a> <b>          what differs between two runs
    python scanner_setup/scanner/suite.py timings <a> [<b>]    stage timers, ms per scanned frame

Each input gives a .txt (what was read) and a .err (stage timings of captures). The harness repeats
exactly, so `cmp` of a run before a change and one after shows what it changed: .txt byte for byte,
.err by stage names and call counts. A run starts from an empty folder. --seq runs one job at a
time, which timings need; otherwise a few run at once (captures are held whole in memory).
"""

import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RECORDINGS = ROOT / "ignored/scanner_tests/recordings"
STILLS = {"stills": ROOT / "scanner_setup/test_stills", "local": ROOT / "ignored/scanner_tests/stills"}
RUNS = ROOT / "target/scanner-suite"
BIN = ROOT / "target/release/tooltip_test.exe"
REPLAY = ROOT / "target/release/capture_replay.exe"
PARALLEL = 4
# capture_replay --timings: name, total, calls
STAGE = re.compile(r"^(\S+)\t([0-9.]+) ms\t([0-9]+) calls$", re.M)


def slug(name):
    return re.sub(r"[\s/]+", "-", name)


def jobs():
    out = []
    for path in sorted(RECORDINGS.rglob("*.hfcap")):
        out.append(([str(REPLAY), path.relative_to(ROOT).as_posix(), "--timings"], f"cap-{slug(path.stem)}"))
    for prefix, root in STILLS.items():
        if not root.exists():
            continue
        for folder in sorted({x.parent for x in root.rglob("*.png")}):
            # relative, as the harness prints each still's path
            images = sorted(x.relative_to(ROOT).as_posix() for x in folder.glob("*.png"))
            out.append(([str(BIN), *images], f"{prefix}-{slug(folder.relative_to(root).as_posix())}"))
    return out


def run(name, seq):
    out = RUNS / name
    shutil.rmtree(out, ignore_errors=True)
    out.mkdir(parents=True)
    bins = ["--bin", "tooltip_test", "--bin", "capture_replay"]
    subprocess.run(["cargo", "build", "--release", "-p", "hf-scanner", *bins], cwd=ROOT, check=True)
    waiting, running = jobs(), []
    while waiting or running:
        running = [x for x in running if x.poll() is None]
        while waiting and len(running) < (1 if seq else PARALLEL):
            command, target = waiting.pop(0)
            files = (open(out / f"{target}.txt", "wb"), open(out / f"{target}.err", "wb"))
            running.append(subprocess.Popen(command, cwd=ROOT, stdout=files[0], stderr=files[1]))
        time.sleep(0.2)
    print(f"{len(list(out.iterdir()))} files in {out.relative_to(ROOT)}")


def comparable(path):
    if path.suffix == ".txt":
        return path.read_bytes()
    text = path.read_text(encoding="utf8", errors="replace")
    return [(name, calls) for name, _, calls in STAGE.findall(text)]


def cmp(a, b):
    names = sorted({x.name for run in (a, b) for x in (RUNS / run).iterdir()})
    differ = 0
    for name in names:
        one, other = RUNS / a / name, RUNS / b / name
        if not (one.exists() and other.exists()):
            print(f"ONLY IN {a if one.exists() else b}  {name}")
        elif comparable(one) != comparable(other):
            print(f"DIFF  {name}")
        else:
            continue
        differ += 1
    print(f"compared {len(names)} files, {differ} differ")


# per capture: scanned frames, and each stage's (ms, calls)
def timers(run):
    out = {}
    for err in sorted((RUNS / run).glob("cap-*.err")):
        text = err.with_suffix(".txt").read_text(encoding="utf8", errors="replace")
        found = re.search(r"(\d+) scans, (\d+) of them skipped", text)
        if not found:
            continue
        scanned = int(found.group(1)) - int(found.group(2))
        stages = STAGE.findall(err.read_text(encoding="utf8", errors="replace"))
        out[err.stem] = (scanned, {name: (float(ms), int(calls)) for name, ms, calls in stages})
    return out


def timings(a, b):
    runs = [timers(x) for x in (a, b) if x]
    for capture in runs[0]:
        if any(capture not in run for run in runs):
            continue
        print(f"\n{capture}: {' / '.join(str(run[capture][0]) for run in runs)} scanned frames")
        print(f"  {'stage':24}" + "".join(f"{x:>12}{'calls':>9}" for x in (a, b) if x) + ("     change" if b else ""))
        for stage in sorted({x for run in runs for x in run[capture][1]}):
            cells = [run[capture][1].get(stage, (0.0, 0)) for run in runs]
            each = [ms / max(run[capture][0], 1) for (ms, _), run in zip(cells, runs)]
            row = "".join(f"{ms:12.3f}{calls:9}" for ms, (_, calls) in zip(each, cells))
            print(f"  {stage:24}{row}" + (f"{each[1] - each[0]:+11.3f}" if b else ""))


if __name__ == "__main__":
    args = [x for x in sys.argv[1:] if not x.startswith("--")]
    if args[:1] == ["run"] and len(args) == 2:
        run(args[1], "--seq" in sys.argv)
    elif args[:1] == ["cmp"] and len(args) == 3:
        cmp(args[1], args[2])
    elif args[:1] == ["timings"] and len(args) in (2, 3):
        timings(args[1], args[2] if len(args) == 3 else None)
    else:
        sys.exit(__doc__)
