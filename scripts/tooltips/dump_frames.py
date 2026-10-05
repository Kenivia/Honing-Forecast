"""
Writes every frame of a recording to stdout as raw RGBA, for the Rust tooltip harness. Needs opencv.
With a step, only every step-th frame is written: the browser scans about one frame a second.

    python scripts/tooltips/dump_frames.py <mp4> [step] | cargo run --release --bin tooltip_test -- --stdin <width> <height>
"""

import sys

import cv2

capture = cv2.VideoCapture(sys.argv[1])
step = int(sys.argv[2]) if len(sys.argv) > 2 else 1
index = 0
while True:
    ok, frame = capture.read()
    if not ok:
        break
    if index % step == 0:
        sys.stdout.buffer.write(cv2.cvtColor(frame, cv2.COLOR_BGR2RGBA).tobytes())
    index += 1
