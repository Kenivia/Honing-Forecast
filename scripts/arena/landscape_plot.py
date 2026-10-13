"""Plots what `pnpm arena landscape --out file.tsv` wrote: for each upgrade the metric over the
two streak lengths, as a surface and as a map with the pity tap drawn on it, and the metric
along the corners, the points with the least juice for each pity tap.

python scripts/arena/landscape_plot.py file.tsv out_dir [--column second]
"""
import csv
import sys
from collections import defaultdict
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

source, out_dir = sys.argv[1], Path(sys.argv[2])
column = sys.argv[sys.argv.index("--column") + 1] if "--column" in sys.argv else "second"
out_dir.mkdir(parents=True, exist_ok=True)

groups = defaultdict(list)
for row in csv.DictReader(open(source), delimiter="\t"):
    groups[(row["scenario"], row["upgrade"], row["name"])].append(row)

for (scenario, upgrade, name), rows in groups.items():
    size = max(max(int(r["x"]), int(r["y"])) for r in rows) + 1
    metric = np.full((size, size), np.nan)
    pity = np.full((size, size), np.nan)
    for r in rows:
        metric[int(r["y"]), int(r["x"])] = float(r[column])
        pity[int(r["y"]), int(r["x"])] = int(r["pity"])
    # past the pity nothing changes, so only the part that does is drawn
    live = int(np.nanmax(pity))
    metric, pity = metric[:live, :live], pity[:live, :live]
    best = np.nanmax(metric)
    loss = best - metric
    xs, ys = np.meshgrid(np.arange(live), np.arange(live))

    figure = plt.figure(figsize=(17, 5.2))
    figure.suptitle(f"{scenario}  upgrade {upgrade} ({name}): gold below the best of the grid", fontsize=11)

    surface = figure.add_subplot(1, 3, 1, projection="3d")
    surface.plot_surface(xs, ys, -loss, cmap="Blues_r", linewidth=0, antialiased=False)
    surface.set_xlabel("x streak (taps)")
    surface.set_ylabel("y streak (taps)")
    surface.set_zlabel("gold")
    surface.view_init(elev=32, azim=-125)

    flat = figure.add_subplot(1, 3, 2)
    image = flat.imshow(loss, origin="lower", cmap="Blues", aspect="auto")
    figure.colorbar(image, ax=flat, label="gold below the best")
    levels = np.arange(np.nanmin(pity) + 0.5, np.nanmax(pity))
    if len(levels):
        flat.contour(xs, ys, pity, levels=levels, colors="#444444", linewidths=0.6)
    at = np.unravel_index(np.nanargmin(loss), loss.shape)
    flat.plot(at[1], at[0], marker="o", color="#b3261e", markersize=7, linestyle="none", label="best")
    flat.set_xlabel("x streak (taps)")
    flat.set_ylabel("y streak (taps)")
    flat.set_title("lines: where the pity tap moves", fontsize=9)
    flat.legend(loc="upper right", fontsize=8, frameon=False)

    # along one axis, the other at the best: the steps, and the corners that lead each one
    ridge = figure.add_subplot(1, 3, 3)
    line = loss[at[0], :]
    ridge.plot(np.arange(live), -line, color="#1f5fa8", linewidth=2, label="x streak, y at its best")
    row_pity = pity[at[0], :]
    corners = [x for x in range(live) if x == 0 or row_pity[x] < row_pity[x - 1]]
    ridge.plot(corners, -line[corners], marker="o", color="#b3261e", markersize=6, linestyle="none", label="first tap of a pity")
    ridge.set_xlabel("x streak (taps)")
    ridge.set_ylabel("gold")
    ridge.grid(color="#dddddd", linewidth=0.5)
    ridge.legend(fontsize=8, frameon=False)

    figure.tight_layout()
    figure.savefig(out_dir / f"{scenario}_{upgrade}_{name}.png", dpi=110)
    plt.close(figure)
    print(scenario, upgrade, name, "best", best, "at", (int(at[1]), int(at[0])))
