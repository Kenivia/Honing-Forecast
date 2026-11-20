# Checks the crops a pass stored (`market_scan --strips`) against a second reader, PP-OCRv5 mobile English
# (Hugging Face monkt/paddleocr-onnx, languages/english: rec.onnx + dict.txt in ignored/market/models/en_PP-OCRv5_mobile_rec).
# Numbers agree when their digits do; text when it is alike after dropping case, spaces and punctuation.
# Writes <strips>/disagree_<n>.png, each disagreement drawn with both reads, for checking by eye.
# Needs onnxruntime, numpy, pillow. Usage: python verify_strips.py [strips folder, default the newest]
import difflib, json, re, sys
from pathlib import Path
import numpy as np, onnxruntime as ort
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[3] / "ignored" / "market"
MODEL = ROOT / "models" / "en_PP-OCRv5_mobile_rec"


def load():
    sess = ort.InferenceSession(str(MODEL / "rec.onnx"), providers=["CPUExecutionProvider"])
    chars = (MODEL / "dict.txt").read_text(encoding="utf-8").split("\n")
    return sess, chars


# light text on dark: inverted, cut to the ink's columns, padded, 48 px high (102 of 102 on the dataset)
def prep(im):
    g = np.asarray(im.convert("RGB")).astype(np.float32).mean(2)
    d = np.abs(g - np.median(g))
    xs = np.where(d.max(0) > max(40, 0.35 * d.max()))[0]
    if len(xs):
        # the largest run of ink columns, split where the gap is over twice the height (a table's divider)
        runs = np.split(xs, np.where(np.diff(xs) > 2 * g.shape[0])[0] + 1)
        run = max(runs, key=len)
        g = g[:, max(0, run[0] - 2) : run[-1] + 3]
    g = np.pad(g, 4, constant_values=float(np.median(g)))
    lo, hi = np.percentile(g, 1), np.percentile(g, 99.5)
    g = 255 - np.clip((g - lo) / max(hi - lo, 1), 0, 1) * 255
    im2 = Image.fromarray(g.astype(np.uint8))
    im2 = im2.resize((max(8, round(im2.width * 48 / im2.height)), 48), Image.BICUBIC)
    x = (np.asarray(im2).astype(np.float32) / 255 - 0.5) / 0.5
    return np.stack([x] * 3)[None]


def recognise(sess, chars, im):
    out = sess.run(None, {sess.get_inputs()[0].name: prep(im)})[0][0]
    s, prev = [], 0
    for i in out.argmax(-1):
        if i != prev and i != 0:
            s.append(chars[i - 1] if i - 1 < len(chars) else " ")
        prev = i
    return "".join(s)


def agree(kind, ours, theirs):
    if kind == "glyph":
        return "?" not in ours and re.sub(r"\D", "", ours) == re.sub(r"\D", "", theirs)
    a, b = (re.sub(r"[^a-z0-9]", "", t.lower()) for t in (ours, theirs))
    return a == b or difflib.SequenceMatcher(None, a, b).ratio() >= 0.9


def sheet(rows, path):
    w = max(max(im.width for im, _ in rows) * 2, 320) + 20
    h = sum(im.height * 2 + 26 for im, _ in rows) + 10
    out = Image.new("RGB", (w, h), (40, 40, 40))
    d, y = ImageDraw.Draw(out), 5
    for im, label in rows:
        d.text((10, y), label, fill=(255, 255, 0))
        out.paste(im.convert("RGB").resize((im.width * 2, im.height * 2), Image.NEAREST), (10, y + 14))
        y += im.height * 2 + 26
    out.save(path)


def main():
    dirs = sorted(p for p in (ROOT / "strips").iterdir() if p.is_dir())
    folder = Path(sys.argv[1]) if len(sys.argv) > 1 else dirs[-1]
    sess, chars = load()
    lines = [json.loads(l) for l in (folder / "index.jsonl").read_text(encoding="utf-8").splitlines()]
    bad, n = [], {"glyph": 0, "ocr": 0}
    for e in lines:
        im = Image.open(folder / e["file"])
        theirs = recognise(sess, chars, im)
        n[e["kind"]] += 1
        if not agree(e["kind"], e["read"], theirs):
            bad.append((im, f'{e["file"]} [{e["context"]}] ours {e["read"]!r} | pp {theirs!r}'))
    for f in folder.glob("disagree_*.png"):
        f.unlink()
    for i in range(0, len(bad), 25):
        sheet(bad[i : i + 25], folder / f"disagree_{i // 25}.png")
    print(f"{folder.name}: {n['glyph']} numbers, {n['ocr']} texts, {len(bad)} disagree")
    for _, label in bad:
        print("  " + label)


if __name__ == "__main__":
    main()
