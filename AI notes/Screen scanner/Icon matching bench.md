# Icon matching bench

An offline test of how a slot is told to be a template's icon ("close enough", `Pipeline.md`, "Slots"), on the debug captures. It is what was measured on 2026-10-11 to decide what to change; the decoys that came out of it are in the scanner.

## The tools

1. `cargo run --release --bin slot_dump -- <file.hfcap> target/slot-bench/<name>` replays a capture and writes every distinct look of every slot: the normalised icon crop the scanner compares (`_i`), the same with 3 px around it (`_m`), the whole slot as captured (`_r`), and `index.jsonl` with the scans each look was seen on and how many of those had a tooltip over or beside the slot. It also writes the end record and the templates at that UI height. Captures were dumped as `a-<time>` (the four annotated ones) and `b-<time>` (the older ones).
2. `scripts/scanner/icon_data.py` labels the slots from the end record and picks each slot's clean look.
3. `scripts/scanner/icon_bench.py [--brief] [--lossy] [method ...]` scores comparison methods, each without and with the decoys (the `~` templates `slot_dump` wrote). Its docstring says what it prints. Scores are cached per method in `target/slot-bench/scores`.

- **A slot's hash has to be of the icon area alone.** The frame around a slot lets the game world through, so with a few pixels of margin every frame is a new look.
- **The clean look** is the one the other tooltip-free looks are closest to, weighted by scans. The look seen longest is often one under a tooltip that was not detected for a few scans.
- **The two 1440p captures (`b-19-22`, `b-22-31`) come from a compressed recording** and are left out unless `--lossy` is given: a screen share is lossless.
- **Labels were checked against the game's art.** Every chest slot and every unlabelled slot scoring 0.9 or more was looked up among all item icons (`find_icon.py`) and its rarity told by its bare background. That found slots the end records have wrong; they are in `LABELS` in `icon_data.py`:
  - never recognised, at 0.946 to 0.948: "[Event] 1,000 Gold Bars" (`use_8_248@epic`), a honing material selection chest (`shop_icon_10804@epic`), the Emergency Honing Materials Chest (`use_14_4@legendary`);
  - a T3 "Oreha Fusion Material" (`use_9_71`, rare) read as the T4 "Fusion" at 0.966, in three captures;
  - chests of a template's art with an item level across them that is not read through (irrelevant ones), left out.
- `find_icon.py` cannot be trusted on a slot with an item level across it (scores of 40 and more).

## What was found

On the seven lossless 1080p captures: 532 distinct slots with a template, about 1,400 without. They are two rosters and four characters, so the hard cases are few.

- **The current score cannot separate them.** The lowest true match is 0.946 and the highest slot with no template 0.966. At the shipped limit of 0.95 that is 5 slots missed (3 kinds) and 5 let through (4 kinds).
- **What gets through is of three kinds**: another art that looks alike (a chest in another trim: `use_4_227` as `use_4_228`, `use_4_219` as `use_4_223`, `use_4_221` as `shop_icon_31`, `use_10_97` as `use_12_175`), the same art on another rarity's background (`use_2_7` rare against the epic template, 0.937 to 0.946), and a small art on a shared background (the fusion materials, whose art is 17% of the slot).
- **No other score fixes it alone.** 24 were tried (root mean square, art-weighted, art and background apart, worst cells, trimmed, luma and chroma apart, correlation, edges, softened, half size, and mixes). The best get 1 to 3 slots wrong at their best limit against 5, and 4 to 5 against 8 with the limit set on other captures. Weighting by the art alone is much worse: it throws the rarity away.
- **A true match's error is fine detail.** Softening both sides with a 1-2-1 kernel halves it (8.3 levels to 4.6), softening one side makes it worse, and no other way of scaling the template (box, bicubic, Lanczos, point-sampled, blurred, shifted by a quarter pixel) beats the bilinear one. Softening helps look-alikes about as much, so it buys little.
- **Decoys fix it with any score.** A decoy is an item icon of the game, on a rarity some item has it on, that is no template of ours and renders close to one. A template counts for nothing on a slot that some decoy is closer to. With the current score:

  | Decoys (how alike to a template) | Count | Lowest true match | Highest other slot | Gap |
  | --- | --- | --- | --- | --- |
  | none | 0 | 0.949 | 0.966 | -0.017 |
  | 0.94 or more | 129 | 0.949 | 0.929 | +0.020 |
  | 0.93 or more | 757 | 0.949 | 0.926 | +0.023 |
  | 0.90 or more | 6,530 | 0.949 | 0.896 | +0.053 |

  No true match is vetoed. Art and background scored apart, the worse of the two counting (`art_bare_mad`), widens the gap to +0.026, +0.047 and +0.084, but is worse on the lossy 1440p captures, where the plain score with decoys still has +0.089.
- These figures are with templates made in Python, which score true matches 0.003 higher than the scanner's own. **With the scanner's own templates and its 757 decoys** the lowest true match is 0.946 and the highest other slot 0.924, a gap of +0.022, with a limit of 0.935 getting every slot right; `art_bare_mad` has 0.937 against 0.891.

## Not done

- The bench scores the 60 nearest templates of all, not the scanner's two shortlists of 16, and only clean looks: nothing under the hover highlight.
- One inventory was captured lossless at 1080p, 1440p and 2160p (`scripts/recordings/new captures`, 2026-10-11; dumped as `c-1080`, `c-1440`, `c-2160`). Their labels are the scanner's own, not checked by hand, so they say whether something changes, not whether it was right. The overlay measurements in `Pipeline.md` use them.
- An item level across a decoy chest: the decoy is now compared with the same parts left out as the icon (`Pipeline.md`, "Overlays"). No capture has such a slot next to an icon of ours that it looks like, so this is untested.
