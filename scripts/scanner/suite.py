"""
The scanner's native suite: tooltip_test over every recording in scripts/recordings and over the
stills folders, and capture_replay over every debug capture (.hfcap) there, its lines read again
and the user's edits left out. Needs opencv. Runs are kept by name in target/scanner-suite/.

    python scripts/scanner/suite.py run <name> [--seq]   about 5 minutes
    python scripts/scanner/suite.py cmp <a> <b>          what differs between two runs
    python scripts/scanner/suite.py timings <a> [<b>]    stage timers, ms per scanned frame

Each input gives a .txt (what was read) and a .err (brightness changes, stage timings). The
harness repeats exactly, so `cmp` of a run before a change and one after shows what it changed:
.txt byte for byte, .err without timing values and thread ids, so a stage that is called more or
less often still shows. --seq runs the recordings one at a time, which timings need.
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RECORDINGS = ROOT / "scripts/recordings"
STILLS = ROOT / "scripts/brightness"
# 768p and 900p are too small to say much; 900p ends in an out-of-bounds crop, which is expected
STILL_FOLDERS = [
    "720p raw", "768p raw", "900p raw", "1080p raw", "1440p raw", "2160p raw",
    "inputs", "inputs storage", "Hover tooltips",
]
RUNS = ROOT / "target/scanner-suite"
BIN = ROOT / "target/release/tooltip_test.exe"
REPLAY = ROOT / "target/release/capture_replay.exe"
VIDEO = {".mp4", ".mkv", ".webm", ".mov"}
SPENT = rb", [0-9.]+ ms per frame in the tooltip step and OCR"
STAGE = re.compile(r'\("([a-z_/]+)", ([0-9.e-]+), ([0-9]+)\)')


def slug(name):
    return re.sub(r"\s+", "-", name)


def recordings():
    return sorted(x for x in RECORDINGS.iterdir() if x.suffix.lower() in VIDEO)


def run(name, seq):
    import cv2

    out = RUNS / name
    out.mkdir(parents=True, exist_ok=True)
    bins = ["--bin", "tooltip_test", "--bin", "capture_replay"]
    subprocess.run(["cargo", "build", "--release", "-p", "hf-scanner", *bins], cwd=ROOT, check=True)

    def start(command, target, feed=None):
        files = (open(out / f"{target}.txt", "wb"), open(out / f"{target}.err", "wb"))
        return subprocess.Popen(command, cwd=ROOT, stdin=feed, stdout=files[0], stderr=files[1])

    jobs = []
    for path in recordings():
        capture = cv2.VideoCapture(str(path))
        size = [str(int(capture.get(x))) for x in (cv2.CAP_PROP_FRAME_WIDTH, cv2.CAP_PROP_FRAME_HEIGHT)]
        capture.release()
        frames = subprocess.Popen(
            [sys.executable, "scripts/tooltips/dump_frames.py", str(path)], cwd=ROOT, stdout=subprocess.PIPE
        )
        job = start([str(BIN), "--stdin", *size], f"rec-{slug(path.stem)}", frames.stdout)
        frames.stdout.close()
        jobs.append(job)
        if seq:
            job.wait()
    for path in sorted(RECORDINGS.glob("*.hfcap")):
        capture = path.relative_to(ROOT).as_posix()
        jobs.append(start([str(REPLAY), capture, "--reread", "--no-edits"], f"cap-{slug(path.stem)}"))
    for folder in STILL_FOLDERS:
        # relative, as the harness prints each still's path
        images = sorted(x.relative_to(ROOT).as_posix() for x in (STILLS / folder).glob("*.png"))
        jobs.append(start([str(BIN), *images], f"stills-{slug(folder)}"))
    for job in jobs:
        job.wait()
    # the one figure in the output that is a timing
    for path in out.glob("*.txt"):
        path.write_bytes(re.sub(SPENT, b"", path.read_bytes()))
    print(f"{len(list(out.iterdir()))} files in {out.relative_to(ROOT)}")


# a stage is its name and how often it ran; what it took differs from run to run
def stages(line):
    return sorted(f"{name}={calls}" for name, _, calls in STAGE.findall(line))


def comparable(path):
    if path.suffix == ".txt":
        return path.read_bytes()
    text = re.sub(r"thread .main. \([0-9]+\)", "thread main", path.read_text(encoding="utf8", errors="replace"))
    return [stages(x) if x.startswith("stage timings") else x for x in text.splitlines()]


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


# per recording: scanned frames, and each stage's (ms, calls)
def timers(run):
    out = {}
    for err in sorted((RUNS / run).glob("rec-*.err")):
        text = err.with_suffix(".txt").read_text(encoding="utf8", errors="replace")
        frames = int(re.search(r"(\d+) frames, tooltip on", text).group(1))
        skipped = int(re.search(r"(\d+) frames not scanned", text).group(1))
        line = next(x for x in err.read_text(encoding="utf8", errors="replace").splitlines() if x.startswith("stage timings"))
        out[err.stem] = (frames - skipped, {name: (float(ms), int(calls)) for name, ms, calls in STAGE.findall(line)})
    return out


def timings(a, b):
    runs = [timers(x) for x in (a, b) if x]
    for recording in runs[0]:
        if any(recording not in run for run in runs):
            continue
        print(f"\n{recording}: {' / '.join(str(run[recording][0]) for run in runs)} scanned frames")
        print(f"  {'stage':24}" + "".join(f"{x:>12}{'calls':>9}" for x in (a, b) if x) + ("     change" if b else ""))
        for stage in sorted({x for run in runs for x in run[recording][1]}):
            cells = [run[recording][1].get(stage, (0.0, 0)) for run in runs]
            each = [ms / run[recording][0] for (ms, _), run in zip(cells, runs)]
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
