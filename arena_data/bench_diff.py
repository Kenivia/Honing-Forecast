"""Two `pnpm arena bench --out` files, state by state: how far the metric moved and the time.

    python arena_data/bench_diff.py before.tsv after.tsv [rows=5]
"""
import math, sys

before = [line.rstrip("\n").split("\t") for line in open(sys.argv[1])]
after = [line.rstrip("\n").split("\t") for line in open(sys.argv[2])]
assert [row[0] for row in before] == [row[0] for row in after], "not the same states"

moved = []
for old, new in zip(before, after):
    a, b = float(old[1]), float(new[1])
    moved.append((abs(a - b) / max(abs(a), abs(b), 1.0), old[0], a, b))
moved.sort(reverse=True)
ratios = [float(new[2]) / float(old[2]) for old, new in zip(before, after)]
total_before = sum(float(row[2]) for row in before)
total_after = sum(float(row[2]) for row in after)

print(f"{len(before)} states, {total_before * 1e3:.2f} ms -> {total_after * 1e3:.2f} ms for one evaluation of each "
      f"({total_after / total_before:.3f}x), geometric mean {math.exp(sum(map(math.log, ratios)) / len(ratios)):.3f}x")
print("metric moved by more than:", "  ".join(
    f"{edge:g}: {sum(row[0] > edge for row in moved)}" for edge in (0, 1e-9, 1e-7, 1e-6, 1e-5, 1e-4)))
for row in moved[: int(sys.argv[3]) if len(sys.argv) > 3 else 5]:
    print("  %.2e  %s  %r -> %r" % row)
