"""
Scores ways of telling whether a slot shows a template's icon, on the slot looks of the debug
captures (icon_data.py; `slot_dump` writes them). The scanner's own way is "mad".

    python scanner_setup/scanner/icon_bench.py [--brief] [--lossy] [method ...]      all of METHODS by default

A method gives every template a score for a slot, higher for more alike. Per method this prints,
over the slots' clean looks (identical crops counted once):
  top1        slots with a template whose own template scores best
  gap         lowest score of a slot's own template, less the highest any template gets on a slot
              that has none; over 0 means one limit tells them all apart
  best limit  the limit that gets the fewest slots wrong: those with a template it misses, and
              those without one it lets through
  held out    the same two with the limit set on the other captures (the annotated ones against
              the rest, and the other way round), summed
and, without --brief, the slots nearest the limit on either side. Scores are kept in
target/slot-bench/scores.
--lossy counts the captures made from a compressed recording too.
Each method is scored twice: with the decoys ignored, and ("+decoys") as the scanner does it, a
template counting for nothing on a slot that some decoy ("~" templates) is closer to. The 60
nearest of all icons and decoys are scored, not the scanner's shortlist.
"""

import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
from icon_data import BENCH, LOSSY, art_alpha, captures, crop, empty, slots, template_info, templates

LEVEL_ROWS = (18.0, 33.0)  # of the compared part at 1440p, as in slots.rs
SHORTLIST = 60  # templates scored per slot; the rest count as nothing alike
BARE = 0.15  # what a pixel of bare background weighs against one of art, where art is weighted

SHIFTS = [(y, x) for y in range(3) for x in range(3)]


# the nine one-pixel shifts of the crop against the template's inside, as the scanner does it
def shifted(image):
    h, w = image.shape[:2]
    return [image[y : y + h - 2, x : x + w - 2] for y, x in SHIFTS]


def grey(image):
    return image @ np.array([0.299, 0.587, 0.114], dtype=np.float32)


def sobel(image):
    g = grey(image)
    gx = g[..., 1:-1, 2:] - g[..., 1:-1, :-2]
    gy = g[..., 2:, 1:-1] - g[..., :-2, 1:-1]
    return np.stack([gx, gy], axis=-1)


def cells(values, rows, columns):
    h, w = values.shape[-2:]
    out = []
    for r in range(rows):
        for c in range(columns):
            part = values[..., r * h // rows : (r + 1) * h // rows, c * w // columns : (c + 1) * w // columns]
            out.append(part.mean(axis=(-2, -1)))
    return np.stack(out, axis=-1)


# Each takes the templates' insides (n, h, w, 3), one shifted crop (h, w, 3) and weights (n, h, w)
# that are 0 on rows left out, and gives a distance per template: 0 for the same, 255 for opposite.
def mad(a, b, w, alpha):
    return (np.abs(a - b).mean(-1) * w).sum((1, 2)) / w.sum((1, 2))


def rms(a, b, w, alpha):
    return np.sqrt((((a - b) ** 2).mean(-1) * w).sum((1, 2)) / w.sum((1, 2)))


def mad_art(a, b, w, alpha):
    w = w * (BARE + alpha)
    return (np.abs(a - b).mean(-1) * w).sum((1, 2)) / w.sum((1, 2))


def rms_art(a, b, w, alpha):
    w = w * (BARE + alpha)
    return np.sqrt((((a - b) ** 2).mean(-1) * w).sum((1, 2)) / w.sum((1, 2)))


# the mean over the closest three quarters of the pixels, as the tooltip's large icon is matched
def trimmed(a, b, w, alpha):
    d = np.where(w > 0, np.abs(a - b).mean(-1), np.nan).reshape(len(a), -1)
    d.sort(axis=1)
    keep = (np.isfinite(d).sum(1) * 3 // 4)[:, None]
    return np.where(np.arange(d.shape[1])[None] < keep, d, 0).sum(1) / keep[:, 0]


# the worst parts decide: the mean of the three most different of 12 cells
def worst_cells(a, b, w, alpha):
    d = np.abs(a - b).mean(-1) * w
    per = cells(d, 3, 4) / np.maximum(cells(w, 3, 4), 1e-6)
    per.sort(axis=1)
    return per[:, -3:].mean(1)


# luma and chroma apart, chroma at half weight: a shared screen is 4:2:0
def luma_chroma(a, b, w, alpha):
    d = a - b
    y = np.abs(grey(d))
    cb, cr = np.abs(d[..., 2] - grey(d)), np.abs(d[..., 0] - grey(d))
    return ((y + 0.5 * (cb + cr)) / 2 * w).sum((1, 2)) / w.sum((1, 2))


# one minus the correlation of the two, each less its mean, over all three channels
def zncc(a, b, w, alpha):
    w3 = w[..., None]
    total = w.sum((1, 2))[:, None, None, None] * 3
    am = a - (a * w3).sum((1, 2, 3), keepdims=True) / total
    bm = b - (b * w3).sum((1, 2, 3), keepdims=True) / total
    top = (am * bm * w3).sum((1, 2, 3))
    bottom = np.sqrt((am**2 * w3).sum((1, 2, 3)) * (bm**2 * w3).sum((1, 2, 3))) + 1e-6
    return 255 * (1 - top / bottom) / 2


# edges only: what stays of the art when the background and the brightness are off
def edges(a, b, w, alpha):
    d = np.abs(sobel(a) - sobel(b)).mean(-1)
    w = w[:, 1:-1, 1:-1]
    return (d * w).sum((1, 2)) / w.sum((1, 2)) / 2


def mad_edges(a, b, w, alpha):
    return (mad(a, b, w, alpha) + edges(a, b, w, alpha)) / 2


def mad_art_edges(a, b, w, alpha):
    return (mad_art(a, b, w, alpha) + edges(a, b, w, alpha)) / 2


def mad_art_worst(a, b, w, alpha):
    return (mad_art(a, b, w, alpha) + worst_cells(a, b, w * (BARE + alpha), alpha)) / 2


def mad_art_zncc(a, b, w, alpha):
    return np.maximum(mad_art(a, b, w, alpha), zncc(a, b, w * (BARE + alpha), alpha))


# The art and the bare background each have to agree, and the worse of the two is the score: the
# same art on another rarity's background fails on the background, another chest on the same
# background on the art, however little of the slot either takes up.
MIN_PART = 40.0  # weights' worth of pixels under which a part says nothing


def parts(distance):
    def both(a, b, w, alpha):
        art, bare = w * alpha, w * (1 - alpha)
        d = distance(a, b)
        of = lambda part: (d * part).sum((1, 2)) / np.maximum(part.sum((1, 2)), 1e-6)
        art_d = np.where(art.sum((1, 2)) >= MIN_PART, of(art), 0)
        bare_d = np.where(bare.sum((1, 2)) >= MIN_PART, of(bare), 0)
        return np.maximum(np.maximum(art_d, bare_d), 0)

    return both


def art_bare_mad(a, b, w, alpha):
    return parts(lambda a, b: np.abs(a - b).mean(-1))(a, b, w, alpha)


def art_bare_rms(a, b, w, alpha):
    return np.sqrt(parts(lambda a, b: ((a - b) ** 2).mean(-1))(a, b, w, alpha))


# the same with the whole slot as a third part, so that neither part can hide in the other
def art_bare_whole(a, b, w, alpha):
    return np.maximum(art_bare_mad(a, b, w, alpha), mad(a, b, w, alpha))


def soften(image):
    out = image.copy()
    out[..., 1:-1, :, :] = (image[..., :-2, :, :] + 2 * image[..., 1:-1, :, :] + image[..., 2:, :, :]) / 4
    image = out.copy()
    out[..., :, 1:-1, :] = (image[..., :, :-2, :] + 2 * image[..., :, 1:-1, :] + image[..., :, 2:, :]) / 4
    return out


# both a little blurred first: the game does not scale its art the way the template was
def mad_soft(a, b, w, alpha):
    return mad(soften(a), soften(b), w, alpha)


def art_bare_soft(a, b, w, alpha):
    return art_bare_mad(soften(a), soften(b), w, alpha)


def art_bare_worst(a, b, w, alpha):
    return np.maximum(art_bare_mad(a, b, w, alpha), worst_cells(a, b, w, alpha) * 0.6)


# both at half the size, each pixel the mean of four
def halve(values):
    h, w = values.shape[1] // 2 * 2, values.shape[2] // 2 * 2
    v = values[:, :h, :w]
    return (v[:, 0::2, 0::2] + v[:, 1::2, 0::2] + v[:, 0::2, 1::2] + v[:, 1::2, 1::2]) / 4


def mad_half(a, b, w, alpha):
    return mad(halve(a), halve(b[None])[0], halve(w), halve(alpha))


def art_bare_half(a, b, w, alpha):
    return art_bare_mad(halve(a), halve(b[None])[0], halve(w), halve(alpha))


def rms_soft(a, b, w, alpha):
    return rms(soften(a), soften(b), w, alpha)


def worst_soft(a, b, w, alpha):
    return worst_cells(soften(a), soften(b), w, alpha)


# softened, and the worse of the whole slot and its most different cells at half weight
def soft_whole_worst(a, b, w, alpha):
    a, b = soften(a), soften(b)
    return np.maximum(mad(a, b, w, alpha), worst_cells(a, b, w, alpha) * 0.5)


METHODS = {x.__name__: x for x in [mad, rms, mad_art, rms_art, trimmed, worst_cells, luma_chroma, zncc, edges, mad_edges, mad_art_edges, mad_art_worst, mad_art_zncc, art_bare_mad, art_bare_rms, art_bare_whole, mad_soft, art_bare_soft, art_bare_worst, mad_half, art_bare_half, rms_soft, worst_soft, soft_whole_worst]}


class Bench:
    def __init__(self):
        self.limits, levelled, _ = template_info()
        self.lossy = "--lossy" in sys.argv
        self.samples = []
        seen = set()
        for capture in captures():
            for slot in slots(capture):
                image = crop(capture.name, slot["looks"][0])
                key = (slot["label"], image.tobytes())
                if empty(image) or key in seen:
                    continue
                seen.add(key)
                self.samples.append({**slot, "image": image, "height": slot["looks"][0]["ui_height"]})
        self.heights = {}
        for height in sorted({x["height"] for x in self.samples}):
            names, images = templates(height)
            alphas = np.stack([art_alpha(name, images.shape[2:0:-1]) for name in names])
            images = images.astype(np.float32)
            h, w = images.shape[1:3]
            s = height / 1440
            # round half away from zero, as Rust does
            rows = [int(x * s + 0.5) for x in LEVEL_ROWS]
            plain = np.ones((h - 2, w - 2), dtype=np.float32)
            masked = plain.copy()
            masked[max(rows[0] - 1, 0) : rows[1] - 1] = 0
            self.heights[height] = {
                "names": names,
                "inside": images[:, 1 : h - 1, 1 : w - 1],
                "alpha": alphas[:, 1 : h - 1, 1 : w - 1],
                "plain": plain,
                "masked": masked,
                "levelled": np.array([name in levelled for name in names]),
                "decoy": np.array([name.startswith("~") for name in names]),
            }

    # every template's score for every sample, 1 for the same
    def scores(self, method):
        cache = BENCH / "scores" / f"{method}.npz"
        tag = np.array([len(self.samples), sum(int(x["image"].sum()) for x in self.samples)])
        if cache.exists() and (np.load(cache)["tag"] == tag).all():
            return list(np.load(cache, allow_pickle=True)["scores"])
        function, out = METHODS[method], []
        for sample in self.samples:
            t = self.heights[sample["height"]]
            image = sample["image"].astype(np.float32)
            crops = shifted(image)
            # only the templates that are anywhere near, by the plain difference unshifted
            near = np.argsort(np.abs(t["inside"] - crops[4]).mean((1, 2, 3)))[:SHORTLIST]
            inside, alpha = t["inside"][near], t["alpha"][near]
            weights = lambda rows, count: np.broadcast_to(rows, (count, *rows.shape))
            best = np.min([function(inside, b, weights(t["plain"], len(near)), alpha) for b in crops], axis=0)
            # a chest with an item level across it is compared without those rows too
            lv = t["levelled"][near]
            if lv.any():
                without = np.min([function(inside[lv], b, weights(t["masked"], lv.sum()), alpha[lv]) for b in crops], axis=0)
                best[lv] = np.minimum(best[lv], without)
            score = np.zeros(len(t["names"]), dtype=np.float32)
            score[near] = 1 - best / 255
            out.append(score)
        cache.parent.mkdir(exist_ok=True)
        np.savez(cache, tag=tag, scores=np.array(out, dtype=object))
        return out

    # veto: a template scores nothing on a slot some decoy is closer to
    def report(self, method, verbose=True, veto=False):
        own, other, top1 = [], [], 0
        for sample, score in zip(self.samples, self.scores(method)):
            if sample["capture"] in LOSSY and not self.lossy:
                continue
            t = self.heights[sample["height"]]
            names, decoy = t["names"], t["decoy"]
            score = np.where(decoy, 0, score) if not veto else np.where(decoy | (score < score[decoy].max(initial=0)), 0, score)
            best = int(np.argmax(score))
            if sample["label"]:
                at = names.index(sample["label"])
                top1 += best == at
                own.append((score[at], sample, names[best], score[best]))
            else:
                other.append((score[best], sample, names[best], score[best]))
        own_scores, other_scores = np.array([x[0] for x in own]), np.array([x[0] for x in other])
        gap = own_scores.min() - other_scores.max()
        limit = best_limit(own_scores, other_scores)
        missed, through = int((own_scores <= limit).sum()), int((other_scores > limit).sum())

        # the limit that gets the fewest wrong on some captures, tried on the others
        def held_out(test):
            train_own = np.array([x[0] for x in own if not test(x[1])])
            train_other = np.array([x[0] for x in other if not test(x[1])])
            limit = best_limit(train_own, train_other)
            test_own = np.array([x[0] for x in own if test(x[1])])
            test_other = np.array([x[0] for x in other if test(x[1])])
            return int((test_own <= limit).sum()), int((test_other > limit).sum())

        groups = sorted({x[1]["capture"][0] for x in own})
        held = [held_out(lambda s: s["capture"][0] == group) for group in groups] if len(groups) > 1 else []
        name = method + ("+decoys" if veto else "")
        print(
            f"{name:22} top1 {top1}/{len(own)}  gap {gap:+.3f}  best limit {limit:.3f}: {missed} missed, {through} let through"
            f"  held out: {sum(x[0] for x in held)} missed, {sum(x[1] for x in held)} let through"
        )
        if verbose:
            describe = lambda x: f"{x[0]:.3f} {x[1]['capture'][:10]} {x[1]['slot']:22} {x[1]['label'] or '-'}  best {x[2]} {x[3]:.3f}"
            for x in sorted(own, key=lambda x: x[0])[:6]:
                print("      own lowest   ", describe(x))
            for x in sorted(other, key=lambda x: -x[0])[:8]:
                print("      none highest ", describe(x))
        return own, other


# the limit that gets the fewest wrong, in the middle of where it does
def best_limit(own, other):
    candidates = np.unique(np.concatenate([own, other]))
    middles = (candidates[1:] + candidates[:-1]) / 2
    wrong = np.array([(own <= x).sum() + (other > x).sum() for x in middles])
    best = np.flatnonzero(wrong == wrong.min())
    return float(middles[best[len(best) // 2]])


if __name__ == "__main__":
    bench = Bench()
    labelled = sum(1 for x in bench.samples if x["label"])
    print(f"{len(bench.samples)} distinct looks, {labelled} with a template, from {len(captures())} captures")
    methods = [x for x in sys.argv[1:] if not x.startswith("--")] or list(METHODS)
    # one process a method for those not scored yet
    from concurrent.futures import ProcessPoolExecutor

    with ProcessPoolExecutor(8) as pool:
        list(pool.map(bench.scores, methods))
    for method in methods:
        bench.report(method, "--brief" not in sys.argv)
        bench.report(method, "--brief" not in sys.argv, veto=True)
