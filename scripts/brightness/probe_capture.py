# Prints what a recording really is, and flags anything the browser or the scanner will not like.
# Usage: python scripts/brightness/probe_capture.py "<file.mp4>" ...

import os
import re
import sys

CHROMA = {0: "monochrome", 1: "4:2:0", 2: "4:2:2", 3: "4:4:4"}
PROFILE = {
    66: "Baseline",
    77: "Main",
    100: "High",
    110: "High 10",
    122: "High 4:2:2",
    244: "High 4:4:4 Predictive",
}
# boxes worth descending into when looking for a leaf
NESTED = (b"moov", b"trak", b"mdia", b"minf", b"stbl")


def find(buf, name):
    i = 0
    while i + 8 <= len(buf):
        ln = int.from_bytes(buf[i : i + 4], "big")
        if ln < 8:
            return None
        typ, body = buf[i + 4 : i + 8], buf[i + 8 : i + ln]
        if typ == name:
            return body
        # stsd and the sample entries carry fixed-size headers before their children
        skip = {b"stsd": 8, b"avc1": 78, b"hev1": 78, b"hvc1": 78}.get(typ)
        if typ in NESTED or skip:
            hit = find(body[skip or 0 :], name)
            if hit is not None:
                return hit
        i += ln
    return None


class Bits:
    def __init__(self, data):
        self.d, self.p = data, 0

    def u(self, n):
        v = 0
        for _ in range(n):
            v = (v << 1) | ((self.d[self.p >> 3] >> (7 - (self.p & 7))) & 1)
            self.p += 1
        return v

    def ue(self):
        z = 0
        while self.u(1) == 0:
            z += 1
        return (1 << z) - 1 + self.u(z) if z else 0

    def se(self):
        k = self.ue()
        return (k + 1) // 2 if k % 2 else -(k // 2)


def unescape(b):
    out, i = bytearray(), 0
    while i < len(b):
        if b[i : i + 3] == b"\x00\x00\x03":
            out += b"\x00\x00"
            i += 3
        else:
            out.append(b[i])
            i += 1
    return bytes(out)


# profile, chroma format, lossless flag and the colour tags out of the SPS
def read_sps(sps):
    r = Bits(unescape(sps[1:]))
    profile = r.u(8)
    r.u(8)
    level = r.u(8)
    r.ue()
    chroma, lossless = 1, 0
    if profile in (100, 110, 122, 244, 44, 83, 86, 118, 128):
        chroma = r.ue()
        if chroma == 3:
            r.u(1)
        r.ue()
        r.ue()
        lossless = r.u(1)
        if r.u(1):  # scaling lists
            for k in range(8 if chroma != 3 else 12):
                if r.u(1):
                    last = nxt = 8
                    for _ in range(16 if k < 6 else 64):
                        if nxt:
                            nxt = (last + r.se() + 256) % 256
                        last = nxt or last
    r.ue()
    poc = r.ue()
    if poc == 0:
        r.ue()
    elif poc == 1:
        r.u(1)
        r.se()
        r.se()
        for _ in range(r.ue()):
            r.se()
    r.ue()
    r.u(1)
    r.ue()
    r.ue()
    if r.u(1) == 0:
        r.u(1)
    r.u(1)
    if r.u(1):  # cropping
        for _ in range(4):
            r.ue()
    rng, matrix = None, None
    if r.u(1):  # VUI
        if r.u(1) and r.u(8) == 255:
            r.u(16)
            r.u(16)
        if r.u(1):
            r.u(1)
        if r.u(1):
            r.u(3)
            rng = r.u(1)
            if r.u(1):
                r.u(8)
                r.u(8)
                matrix = r.u(8)
    return profile, level, chroma, lossless, rng, matrix


# x264 writes its whole option list into an SEI; hardware encoders write nothing
def read_x264_options(data):
    i = data.find(b"x264 - core")
    if i < 0:
        return None
    text = data[i : i + 2000].split(b"\x00")[0].decode("latin1")
    core = re.match(r"x264 - core (\d+)", text)
    opts = dict(
        kv.split("=", 1)
        for kv in text.split("options: ")[-1].split(" ")
        if "=" in kv
    )
    return core.group(1) if core else "?", opts


def probe(path):
    size = os.path.getsize(path)
    data = open(path, "rb").read()
    print(os.path.basename(path))

    avcc = find(data, b"avcC")
    if avcc is None:
        print("   no H.264 track")
        return
    sps_len = int.from_bytes(avcc[6:8], "big")
    profile, level, chroma, lossless, rng, matrix = read_sps(avcc[8 : 8 + sps_len])

    avc1 = find(data, b"avc1")
    width = int.from_bytes(avc1[24:26], "big")
    height = int.from_bytes(avc1[26:28], "big")
    mvhd = find(data, b"mvhd")
    secs = int.from_bytes(mvhd[16:20], "big") / int.from_bytes(mvhd[12:16], "big")
    frames = int.from_bytes(find(data, b"stsz")[8:12], "big")
    colr = find(data, b"colr")

    print(
        f"   {width}x{height}  {frames} frames  {secs:.1f}s"
        f"  {frames / secs:.1f} fps  {size * 8 / secs / 1e6:.1f} Mbps"
    )
    print(
        f"   profile {profile} ({PROFILE.get(profile, '?')})  level {level / 10:.1f}"
        f"  chroma {CHROMA.get(chroma, chroma)}  lossless={lossless}"
    )
    range_name = {0: "limited", 1: "full"}.get(rng, "absent")
    tags = (colr[:4].decode() + " " + colr[4:].hex()) if colr else "absent"
    print(f"   range={range_name}  matrix={matrix}  colr box: {tags}")

    x264 = read_x264_options(data[: 3 << 20])
    if x264:
        core, opts = x264
        rc = opts.get("rc", "?")
        amount = opts.get("crf") or opts.get("qp") or opts.get("bitrate", "?")
        print(
            f"   x264 core {core}  {rc}={amount}  ref={opts.get('ref')}"
            f" subme={opts.get('subme')} bframes={opts.get('bframes')}"
        )
    else:
        print("   no x264 options (hardware encoder)")

    if profile == 244:
        print("   !! High 4:4:4 Predictive: Firefox refuses it, VLC mangles the colours")
    if chroma != 1:
        print("   !! not 4:2:0, so it does not match what a browser screen share gives")
    if rng == 1:
        print("   !! full range: the browser assumes limited and will crush blacks")
    if abs(frames / secs - 30) > 1:
        print(f"   !! {frames / secs:.1f} fps, the scanner captures at 30")


for arg in sys.argv[1:]:
    probe(arg)
