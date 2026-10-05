"""
Writes every frame of a recording to stdout as raw RGBA, for the Rust tooltip harness. Needs opencv.

    python scripts/tooltips/dump_frames.py <mp4> | cargo run --release --bin tooltip_test -- --stdin <width> <height>
"""

import sys

import cv2

capture = cv2.VideoCapture(sys.argv[1])
while True:
    ok, frame = capture.read()
    if not ok:
        break
    sys.stdout.buffer.write(cv2.cvtColor(frame, cv2.COLOR_BGR2RGBA).tobytes())
