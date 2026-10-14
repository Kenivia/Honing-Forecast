"""Overnight test of the coarse lattice and the one saddlepoint leftover against the saddlepoint pair.

    python scripts/arena/saddle_overnight.py              run every task not done yet, in order
    python scripts/arena/saddle_overnight.py --list       what is done
    python scripts/arena/saddle_overnight.py --only bench_time   only tasks whose name contains this
    python scripts/arena/saddle_overnight.py --report     write summary.md from what is done
    python scripts/arena/saddle_overnight.py --rebuild    build and copy the driver again first

Ctrl-C stops it, and running it again carries on: a task half done starts over, a `run` half
done resumes. The driver is built once and copied, so the tree can be edited meanwhile.
Everything goes into .tmp/saddle_overnight (keep it from `pnpm run remove-tmp`), the optimizer
runs into test_cases/saddle_overnight.sqlite. summary.md is written after every task.

The reference is a coarse lattice of 512 points a standard deviation (agrees with
max_lattice_work=2e9 to 1e-8 of flow on pilot, 17 times faster); the exact one is run too where
it is affordable, to check that.
"""
import json, os, re, shutil, signal, sqlite3, subprocess, sys, time
from collections import defaultdict
from statistics import mean, median

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OUT = os.path.join(ROOT, ".tmp", "saddle_overnight")
DRIVER = os.path.join(OUT, "driver.exe")
DB = "test_cases/saddle_overnight.sqlite"
THREADS = max(1, (os.cpu_count() or 4) - 2)

VARIANTS = {
    "base": "",
    "direct": "direct_put=true",
    "c8": "coarse_cells=8",
    "c16": "coarse_cells=16",
    "c32": "coarse_cells=32",
    "c64": "coarse_cells=64",
}
FINE = "coarse_cells=512,max_coarse_work=4000000000"
EXACT = "max_lattice_work=2000000000"

SETS = {
    "pilot": ["--set", "pilot"],
    "juice": ["--set", "juice"],
    "nh_quick": ["--set", "nh_quick"],
    "large": ["--set", "large"],
    "advanced": ["--set", "advanced"],
    "free_taps": ["--set", "free_taps"],
    "regressions": ["--scenarios", "test_cases/scenarios/regressions"],
    "weekly_dev": ["--set", "weekly_dev"],
    "nh_dev": ["--set", "nh_dev"],
    "stress": ["--set", "stress"],
}
STRESS_SETS = ["pilot", "juice", "nh_quick", "large", "advanced", "free_taps", "regressions", "weekly_dev", "nh_dev", "stress"]
BENCH_SETS = STRESS_SETS
EXACT_SETS = ["pilot", "juice", "nh_quick", "regressions", "weekly_dev", "nh_dev"]
VERIFY = ["base", "direct", "c16", "c64"]
STRESS_SOLVES = ["base", "direct", "c16", "c32"]


def spec(name):
    return "current" + (":" + VARIANTS[name] if VARIANTS[name] else "")


def tuning(*parts):
    return ",".join(part for part in parts if part)


# ---------------------------------------------------------------- tasks

class Task:
    def __init__(self, name, args, after=None, note=""):
        self.name, self.args, self.after, self.note = name, args, after, note

    @property
    def log(self):
        return os.path.join(OUT, self.name + ".txt")

    @property
    def done(self):
        return os.path.exists(os.path.join(OUT, self.name + ".done"))


def tasks():
    out = []
    # 1. probabilities and the expected leftover against an exact convolution
    for set_name in STRESS_SETS:
        for variant, tune in VARIANTS.items():
            out.append(Task(f"stress_{set_name}_{variant}",
                            SETS[set_name] + ["stress", "--scrambles", "3", "--tuning", tune],
                            note="stress"))
    # 2. one evaluation of each state, one thread, no memo: time and metric
    for set_name in BENCH_SETS:
        for variant, tune in VARIANTS.items():
            name = f"bench_time_{set_name}_{variant}"
            out.append(Task(name, SETS[set_name] + ["bench", "--jobs", "1", "--seconds", "0.05",
                                                    "--tuning", tuning("max_memo_size=0", tune),
                                                    "--out", os.path.join(OUT, name + ".tsv")]))
    # 3. the references for the same states
    for set_name in BENCH_SETS:
        name = f"bench_fine_{set_name}"
        out.append(Task(name, SETS[set_name] + ["bench", "--jobs", str(THREADS), "--seconds", "0",
                                                "--tuning", FINE, "--out", os.path.join(OUT, name + ".tsv")]))
    for set_name in EXACT_SETS:
        name = f"bench_exact_{set_name}"
        out.append(Task(name, SETS[set_name] + ["bench", "--jobs", str(THREADS), "--seconds", "0",
                                                "--tuning", EXACT, "--out", os.path.join(OUT, name + ".tsv")]))
    # 4. whole solves, one at a time, for the time
    for set_name in ["pilot", "nh_quick"]:
        out.append(Task(f"solve_time_{set_name}",
                        ["--db", DB] + SETS[set_name] + ["run", *[spec(v) for v in VARIANTS],
                                                          "--seeds", "2", "--mode", "cold", "--jobs", "1", "--resume"]))
    # 5. whole solves for the plans they find, then those plans under the reference
    quality = [("juice", list(VARIANTS), 3),
               ("nh_dev", ["base", "direct", "c8", "c16", "c32"], 3),
               ("weekly_dev", ["base", "direct", "c16"], 2)]
    for set_name, variants, seeds in quality:
        run_name = f"solve_quality_{set_name}"
        out.append(Task(run_name, ["--db", DB] + SETS[set_name] + ["run", *[spec(v) for v in variants],
                                                                   "--seeds", str(seeds), "--mode", "cold", "--resume"]))
        for variant in variants:
            name = f"plans_fine_{set_name}_{variant}"
            states = os.path.join(OUT, name + ".states.json")
            out.append(Task(name, SETS[set_name] + ["bench", "--jobs", str(THREADS), "--seconds", "0", "--tuning", FINE,
                                                    "--states", states, "--out", os.path.join(OUT, name + ".tsv")],
                            after=(run_name, variant, states)))
    # 6. monte carlo
    for set_name in ["juice", "regressions"]:
        out.append(Task(f"verify_{set_name}",
                        ["--db", DB] + SETS[set_name] + ["verify", *[spec(v) for v in VERIFY], "--scrambles", "2"]))
    # 7. every upgrade a character has: solves, then their plans under the reference
    out.append(Task("solve_quality_stress", ["--db", DB] + SETS["stress"] + ["run", *[spec(v) for v in STRESS_SOLVES],
                                                                            "--seeds", "1", "--mode", "cold", "--jobs", "4", "--resume"]))
    for variant in STRESS_SOLVES:
        name = f"plans_fine_stress_{variant}"
        states = os.path.join(OUT, name + ".states.json")
        out.append(Task(name, SETS["stress"] + ["bench", "--jobs", str(THREADS), "--seconds", "0", "--tuning", FINE,
                                                "--states", states, "--out", os.path.join(OUT, name + ".tsv")],
                        after=("solve_quality_stress", variant, states)))
    return out


def session_of(task_name):
    path = os.path.join(OUT, task_name + ".txt")
    if not os.path.exists(path):
        return None
    found = re.findall(r"^session (\d+):", open(path, encoding="utf-8", errors="replace").read(), re.M)
    return int(found[-1]) if found else None


def export_states(run_task, variant, path):
    """the states a variant's run ended on, as bench --states wants them: "scenario:step" -> [state by seed]"""
    session = session_of(run_task)
    conn = sqlite3.connect(os.path.join(ROOT, DB))
    rows = conn.execute(
        "SELECT s.scenario, s.step, r.seed, r.state FROM run r JOIN scenario_step s ON s.id = r.step_id "
        "WHERE r.session_id = ? AND r.variant = ? AND r.state IS NOT NULL ORDER BY r.seed", (session, spec(variant))).fetchall()
    states, seeds = defaultdict(list), defaultdict(list)
    for scenario, step, seed, state in rows:
        states[f"{scenario}:{step}"].append(json.loads(state))
        seeds[f"{scenario}:{step}"].append(seed)
    json.dump(states, open(path, "w"))
    json.dump(seeds, open(path + ".seeds", "w"))


def build():
    print("building the arena", flush=True)
    subprocess.run(["cargo", "build", "--release", "-p", "hf-arena"], cwd=ROOT, check=True)
    os.makedirs(OUT, exist_ok=True)
    shutil.copy(os.path.join(ROOT, "target", "release", "hf-arena.exe"), DRIVER)


def run_task(task):
    if task.after:
        export_states(*task.after)
    started = time.time()
    with open(task.log, "w", encoding="utf-8") as log:
        log.write("$ driver " + " ".join(task.args) + "\n")
        log.flush()
        child = subprocess.Popen([DRIVER] + task.args, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        try:
            code = child.wait()
        except KeyboardInterrupt:
            # the driver of a run gets the ctrl-c too and stops its workers cleanly
            try:
                child.wait(timeout=30)
            except (subprocess.TimeoutExpired, KeyboardInterrupt):
                child.kill()
            raise
    seconds = time.time() - started
    if code != 0:
        print(f"  {task.name} failed with {code}, see {task.log}", flush=True)
        return False
    if "stopped; with --resume" in open(task.log, encoding="utf-8", errors="replace").read():
        raise KeyboardInterrupt
    with open(os.path.join(OUT, task.name + ".done"), "w") as done:
        done.write(f"{seconds:.0f}\n")
    print(f"  done in {seconds / 60:.1f} min", flush=True)
    return True


# ---------------------------------------------------------------- report

def read_tsv(path):
    out = {}
    if not os.path.exists(path):
        return out
    for line in open(path):
        parts = line.rstrip("\n").split("\t")
        out[parts[0]] = [float(x) for x in parts[1:]]
    return out


def stress_rows(path):
    """(method, region, kind) -> calls, histogram, max, mean, signed, us"""
    rows, notes = {}, {}
    if not os.path.exists(path):
        return rows, notes
    for line in open(path, encoding="utf-8", errors="replace"):
        if line.startswith("│"):
            cells = line.strip().strip("│").split()
            if len(cells) == 15 and cells[3].isdigit():
                rows[tuple(cells[:3])] = dict(calls=int(cells[3]), hist=[int(c) for c in cells[4:9]],
                                              max=float(cells[9]), mean=float(cells[10]),
                                              signed=float(cells[11]), us=float(cells[12]))
        elif ": " in line and not line.startswith(" "):
            key, _, value = line.rpartition(": ")
            if value.strip().isdigit():
                notes[key.strip()] = int(value)
    return rows, notes


def fmt(x, digits=1):
    return "-" if x is None else f"{x:.{digits}e}"


def accuracy(values, reference):
    """errors as a share of flow; positive is the evaluator finding it cheaper than it is"""
    errs = [(values[k][0] - reference[k][0]) / max(abs(reference[k][2]), 1.0)
            for k in reference if k in values and len(reference[k]) > 2]
    if not errs:
        return None
    a = [abs(e) for e in errs]
    over = [e for e in errs if abs(e) > 1e-4]
    return dict(n=len(errs), over4=sum(x > 1e-4 for x in a) / len(a), over3=sum(x > 1e-3 for x in a) / len(a),
                worst=max(a), mean_abs=mean(a), signed=mean(errs),
                optimistic=(sum(e > 0 for e in over) / len(over)) if over else None)


def report():
    lines = ["# Saddlepoint overnight", "",
             f"Written {time.strftime('%Y-%m-%d %H:%M')}. Variants: " +
             ", ".join(f"`{k}` = `{v or 'default'}`" for k, v in VARIANTS.items()) + f". Reference `{FINE}`.", ""]

    # stress: the expected leftover
    lines += ["## Stress: expected leftover E[(t - S)+] against an exact convolution",
              "Errors in standard deviations of the sum, over every threshold the stress test asks for. "
              "`signed` < 0 is optimistic. `sp` is the share of calls answered by an approximation "
              "(not lattice, brute or trivial).", "",
              "| set | variant | calls | sp | mean abs | mean signed | max | >1e-4 | >1e-2 | us/call | panics |",
              "|---|---|---|---|---|---|---|---|---|---|---|"]
    by_method = defaultdict(lambda: defaultdict(lambda: [0, 0.0, 0.0, 0.0]))
    for set_name in STRESS_SETS:
        for variant in VARIANTS:
            path = os.path.join(OUT, f"stress_{set_name}_{variant}.txt")
            rows, notes = stress_rows(path)
            rows = {k: v for k, v in rows.items() if k[2] == "leftover"}
            if not rows:
                continue
            calls = sum(r["calls"] for r in rows.values())
            approx = sum(r["calls"] for k, r in rows.items() if k[0] not in ("lattice", "brute", "trivial"))
            hist = [sum(r["hist"][i] for r in rows.values()) for i in range(5)]
            panics = re.findall(r"^panics: (\d+)", open(path, encoding="utf-8", errors="replace").read(), re.M)
            lines.append(f"| {set_name} | {variant} | {calls} | {approx / calls:.0%} | "
                         f"{fmt(sum(r['mean'] * r['calls'] for r in rows.values()) / calls)} | "
                         f"{sum(r['signed'] * r['calls'] for r in rows.values()) / calls:+.1e} | "
                         f"{fmt(max(r['max'] for r in rows.values()))} | {sum(hist[2:])} | {sum(hist[3:])} | "
                         f"{sum(r['us'] * r['calls'] for r in rows.values()) / calls:.1f} | {panics[-1] if panics else '?'} |")
            for (method, _, _), r in rows.items():
                cell = by_method[variant][method]
                cell[0] += r["calls"]; cell[1] += r["mean"] * r["calls"]; cell[2] += r["signed"] * r["calls"]
                cell[3] = max(cell[3], r["max"])
    lines += ["", "By method, every set together:", "", "| variant | method | calls | mean abs | mean signed | max |", "|---|---|---|---|---|---|"]
    for variant, methods in by_method.items():
        for method, (calls, total, signed, worst) in sorted(methods.items()):
            lines.append(f"| {variant} | {method} | {calls} | {fmt(total / calls)} | {signed / calls:+.1e} | {fmt(worst)} |")

    # evaluator time and metric
    lines += ["", "## One evaluation: time (one thread, no memo) and metric against the reference",
              "Time is ms to evaluate every state once, and against `base`. Errors are a share of the state's flow; "
              "`opt` is the share of those over 1e-4 where the evaluator is cheaper than the reference.", "",
              "| set | variant | states | ms | x base | >1e-4 | >1e-3 | worst | mean abs | mean signed | opt |",
              "|---|---|---|---|---|---|---|---|---|---|---|"]
    for set_name in BENCH_SETS:
        reference = read_tsv(os.path.join(OUT, f"bench_fine_{set_name}.tsv"))
        base_ms = None
        for variant in VARIANTS:
            values = read_tsv(os.path.join(OUT, f"bench_time_{set_name}_{variant}.tsv"))
            if not values:
                continue
            ms = sum(v[1] for v in values.values()) * 1e3
            base_ms = ms if variant == "base" else base_ms
            acc = accuracy(values, reference) if reference else None
            row = f"| {set_name} | {variant} | {len(values)} | {ms:.1f} | {ms / base_ms:.2f} |" if base_ms else f"| {set_name} | {variant} | {len(values)} | {ms:.1f} | - |"
            if acc:
                row += (f" {acc['over4']:.1%} | {acc['over3']:.1%} | {fmt(acc['worst'])} | {fmt(acc['mean_abs'])} | "
                        f"{acc['signed']:+.1e} | {'-' if acc['optimistic'] is None else format(acc['optimistic'], '.0%')} |")
            else:
                row += " - | - | - | - | - | - |"
            lines.append(row)
    lines += ["", "The reference against the exact lattice (max_lattice_work=2e9):", "",
              "| set | states | >1e-6 | worst |", "|---|---|---|---|"]
    for set_name in EXACT_SETS:
        exact = read_tsv(os.path.join(OUT, f"bench_exact_{set_name}.tsv"))
        fine = read_tsv(os.path.join(OUT, f"bench_fine_{set_name}.tsv"))
        acc = accuracy(fine, exact) if exact and fine else None
        if acc:
            lines.append(f"| {set_name} | {acc['n']} | {sum(1 for k in exact if k in fine and abs(fine[k][0] - exact[k][0]) / max(abs(exact[k][2]), 1) > 1e-6)} | {fmt(acc['worst'])} |")

    # solves
    db_path = os.path.join(ROOT, DB)
    if os.path.exists(db_path):
        conn = sqlite3.connect(db_path)
        lines += ["", "## Whole solves", "",
                  "| task | variant | runs | died | mean cpu s | x base | eval s | own metric - default evaluator's, of flow |",
                  "|---|---|---|---|---|---|---|---|"]
        for task in [t for t in tasks() if t.name.startswith("solve_") and t.done]:
            session = session_of(task.name)
            base_cpu = None
            for variant in VARIANTS:
                rows = conn.execute("SELECT cpu_s, eval_s, metric, metric_ref, flow, error FROM run WHERE session_id = ? AND variant = ?",
                                    (session, spec(variant))).fetchall()
                if not rows:
                    continue
                ok = [r for r in rows if r[0] is not None]
                cpu = mean(r[0] for r in ok) if ok else float("nan")
                base_cpu = cpu if variant == "base" else base_cpu
                own = [(r[2] - r[3]) / max(abs(r[4] or 1), 1) for r in ok if r[2] is not None and r[3] is not None]
                lines.append(f"| {task.name} | {variant} | {len(ok)} | {sum(1 for r in rows if r[5])} | {cpu:.2f} | "
                             f"{cpu / base_cpu:.2f} | {mean(r[1] or 0 for r in ok):.2f} | {mean(own) if own else 0:+.1e} |"
                             if base_cpu else f"| {task.name} | {variant} | {len(ok)} | - | {cpu:.2f} | - | - | - |")

        # plans under the reference
        lines += ["", "## The plans each variant found, under the reference",
                  "Per payload the mean over seeds of the reference metric, against `base`'s, as a share of flow. "
                  "`better` / `worse` count payloads moved by more than 1e-4. `own error` is the variant's own metric "
                  "of its plan less the reference's: how much its plans flatter it.", "",
                  "| set | variant | payloads | mean vs base | median | better | worse | own error mean | own error worst |",
                  "|---|---|---|---|---|---|---|---|---|"]
        for set_name in ["nh_dev", "weekly_dev", "juice", "stress"]:
            run_task = "solve_quality_stress" if set_name == "stress" else f"solve_quality_{set_name}"
            session = session_of(run_task)
            per_variant = {}
            for variant in VARIANTS:
                name = f"plans_fine_{set_name}_{variant}"
                values = read_tsv(os.path.join(OUT, name + ".tsv"))
                seeds_path = os.path.join(OUT, name + ".states.json.seeds")
                if not values or not os.path.exists(seeds_path):
                    continue
                seeds = json.load(open(seeds_path))
                found = defaultdict(list)
                own = []
                for label, v in values.items():
                    m = re.match(r"(.+) best(\d+)$", label)
                    if not m:
                        continue
                    key, index = m.group(1), int(m.group(2))
                    found[key].append((v[0], v[2]))
                    seed = seeds[key][index]
                    scenario, step = key.rsplit(":", 1)
                    row = conn.execute("SELECT r.metric FROM run r JOIN scenario_step s ON s.id = r.step_id WHERE r.session_id = ? "
                                       "AND r.variant = ? AND r.seed = ? AND s.scenario = ? AND s.step = ?",
                                       (session, spec(variant), seed, scenario, int(step))).fetchone()
                    if row and row[0] is not None:
                        own.append((row[0] - v[0]) / max(abs(v[2]), 1))
                per_variant[variant] = ({k: (mean(x[0] for x in v), v[0][1]) for k, v in found.items()}, own)
            if "base" not in per_variant:
                continue
            base = per_variant["base"][0]
            for variant, (values, own) in per_variant.items():
                diffs = [(values[k][0] - base[k][0]) / max(abs(base[k][1]), 1) for k in values if k in base]
                if not diffs:
                    continue
                lines.append(f"| {set_name} | {variant} | {len(diffs)} | {mean(diffs):+.1e} | {median(diffs):+.1e} | "
                             f"{sum(d > 1e-4 for d in diffs)} | {sum(d < -1e-4 for d in diffs)} | "
                             f"{mean(own) if own else 0:+.1e} | {fmt(max(map(abs, own)) if own else None)} |")

        # monte carlo
        lines += ["", "## Monte Carlo (verify)", "",
                  "| set | variant | rows | failing | max abs z | mean z |", "|---|---|---|---|---|---|"]
        for set_name in ["juice", "regressions"]:
            session = session_of(f"verify_{set_name}")
            if session is None or not os.path.exists(os.path.join(OUT, f"verify_{set_name}.done")):
                continue
            for variant in VERIFY:
                rows = conn.execute("SELECT m.pass, m.z FROM mc m JOIN run r ON r.id = m.run_id WHERE r.session_id = ? AND r.variant = ?",
                                    (session, spec(variant))).fetchall()
                zs = [r[1] for r in rows if r[1] is not None]
                if zs:
                    lines.append(f"| {set_name} | {variant} | {len(rows)} | {sum(1 for r in rows if not r[0])} | "
                                 f"{max(map(abs, zs)):.1f} | {mean(zs):+.2f} |")

    # counters of the timing benches
    lines += ["", "## Counters per state (timing benches)", "",
              "| set | variant | Lattice | Coarse | CoarseTooBig | Saddlepoint | Direct | DirectFallback | Ks |", "|---|---|---|---|---|---|---|---|---|"]
    for set_name in BENCH_SETS:
        for variant in VARIANTS:
            path = os.path.join(OUT, f"bench_time_{set_name}_{variant}.txt")
            if not os.path.exists(path):
                continue
            text = open(path, encoding="utf-8", errors="replace").read()
            get = lambda key: (re.findall(rf"^{key}\s+\d+\s+([\d.]+) per state", text, re.M) or ["-"])[-1]
            lines.append(f"| {set_name} | {variant} | " + " | ".join(get(k) for k in
                         ["Lattice", "Coarse", "CoarseTooBig", "Saddlepoint", "Direct", "DirectFallback", "Ks"]) + " |")

    lines += ["", "## Tasks", "", "| task | minutes |", "|---|---|"]
    for task in tasks():
        if task.done:
            lines.append(f"| {task.name} | {float(open(os.path.join(OUT, task.name + '.done')).read()) / 60:.1f} |")
    with open(os.path.join(OUT, "summary.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


# ---------------------------------------------------------------- main

def main():
    argv = sys.argv[1:]
    os.makedirs(OUT, exist_ok=True)
    all_tasks = tasks()
    only = argv[argv.index("--only") + 1] if "--only" in argv else ""
    chosen = [t for t in all_tasks if only in t.name]
    if "--list" in argv:
        for t in chosen:
            print(("done " if t.done else "     ") + t.name)
        print(f"{sum(t.done for t in chosen)} of {len(chosen)} done")
        return
    if "--report" in argv:
        report()
        print(os.path.join(OUT, "summary.md"))
        return
    if "--rebuild" in argv or not os.path.exists(DRIVER):
        build()
    todo = [t for t in chosen if not t.done]
    print(f"{len(todo)} tasks to do, {THREADS} threads where parallel; results in {OUT}", flush=True)
    failed = []
    try:
        for index, task in enumerate(todo):
            print(f"[{index + 1}/{len(todo)}] {task.name}  ({time.strftime('%H:%M')})", flush=True)
            if not run_task(task):
                failed.append(task.name)
            report()
    except KeyboardInterrupt:
        print("stopped; run the same command again to carry on", flush=True)
        report()
        sys.exit(1)
    print("all done" + (f"; failed: {', '.join(failed)}" if failed else ""), flush=True)
    print(os.path.join(OUT, "summary.md"))


if __name__ == "__main__":
    main()
