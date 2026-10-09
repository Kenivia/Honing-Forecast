"""
Puts the stage images the number_probe bin wrote for each read into one picture per read. Needs opencv.

    python scripts/tooltips/number_stages.py <probe out dir>

Writes <dir>/f<frame>.png.
"""

import glob
import os
import sys

import cv2
import numpy as np

STAGES = [
    ("0_icon", "slot icon below the number (normalised)", 10),
    ("1_captured", "1. number crop as captured", 10),
    ("2_background", "2. background expected from the template", 10),
    ("3_alpha_fit", "3. white needed, by fit (white = all digit)", 10),
    ("4_alpha_colour", "4. white needed, by colour", 10),
    ("5_shadow", "5. pixels classed as the number's shadow", 10),
    ("6_mask", "6. mask", 10),
    ("7_sent", "7. strip sent to OCR", 4),
]
WIDTH = 640

folder = sys.argv[1]
for first in sorted(glob.glob(os.path.join(folder, "f*_0_icon.png"))):
    read = os.path.basename(first)[: -len("_0_icon.png")]
    parts = []
    for stage, label, scale in STAGES:
        image = cv2.imread(os.path.join(folder, f"{read}_{stage}.png"))
        image = cv2.resize(image, None, fx=scale, fy=scale, interpolation=cv2.INTER_NEAREST)[:, : WIDTH - 16]
        row = np.full((image.shape[0] + 34, WIDTH, 3), 40, np.uint8)
        cv2.putText(row, label, (8, 22), cv2.FONT_HERSHEY_SIMPLEX, 0.55, (255, 255, 255), 1, cv2.LINE_AA)
        row[30 : 30 + image.shape[0], 8 : 8 + image.shape[1]] = image
        parts.append(row)
    cv2.imwrite(os.path.join(folder, f"{read}.png"), np.vstack(parts))
    print(os.path.join(folder, f"{read}.png"))
