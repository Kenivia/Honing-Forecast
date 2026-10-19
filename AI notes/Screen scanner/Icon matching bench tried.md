# Icon matching bench: what was found

What the bench (`Icon matching bench.md`) measured on 2026-10-11 to decide what to change. The decoys that came out of it are in the scanner. Only read this when trying a new way of matching a slot.

## Labels the end records had wrong

- never recognised, at 0.946 to 0.948: "[Event] 1,000 Gold Bars" (`use_8_248@epic`), a honing material selection chest (`shop_icon_10804@epic`), the Emergency Honing Materials Chest (`use_14_4@legendary`);
- a T3 "Oreha Fusion Material" (`use_9_71`, rare) read as the T4 "Fusion" at 0.966, in three captures;
- chests of a template's art with an item level across them that is not read through (irrelevant ones), left out.

## What was found

On the seven lossless 1080p captures: 532 distinct slots with a template, about 1,400 without. They are two rosters and four characters, so the hard cases are few.

- **The score alone cannot separate them.** The lowest true match is 0.946 and the highest slot with no template 0.966. At the limit of 0.95 then that is 5 slots missed (3 kinds) and 5 let through (4 kinds).
- **What gets through is of three kinds**: another art that looks alike (a chest in another trim: `use_4_227` as `use_4_228`, `use_4_219` as `use_4_223`, `use_4_221` as `shop_icon_31`, `use_10_97` as `use_12_175`), the same art on another rarity's background (`use_2_7` rare against the epic template, 0.937 to 0.946), and a small art on a shared background (the fusion materials, whose art is 17% of the slot).
- **No other score fixes it alone.** 24 were tried (root mean square, art-weighted, art and background apart, worst cells, trimmed, luma and chroma apart, correlation, edges, softened, half size, and mixes). The best get 1 to 3 slots wrong at their best limit against 5, and 4 to 5 against 8 with the limit set on other captures. Weighting by the art alone is much worse: it throws the rarity away.
- **A true match's error is fine detail.** Softening both sides with a 1-2-1 kernel halves it (8.3 levels to 4.6), softening one side makes it worse, and no other way of scaling the template (box, bicubic, Lanczos, point-sampled, blurred, shifted by a quarter pixel) beats the bilinear one. Softening helps look-alikes about as much, so it buys little.
- **Decoys fix it with any score.** A decoy is an item icon of the game, on a rarity some item has it on, that is no template of ours and renders close to one. A template counts for nothing on a slot that some decoy is closer to. With the plain score:

  | Decoys (how alike to a template) | Count | Lowest true match | Highest other slot | Gap |
  | --- | --- | --- | --- | --- |
  | none | 0 | 0.949 | 0.966 | -0.017 |
  | 0.94 or more | 129 | 0.949 | 0.929 | +0.020 |
  | 0.93 or more | 757 | 0.949 | 0.926 | +0.023 |
  | 0.90 or more | 6,530 | 0.949 | 0.896 | +0.053 |

  No true match is vetoed. Art and background scored apart, the worse of the two counting (`art_bare_mad`), widens the gap to +0.026, +0.047 and +0.084, but is worse on the lossy 1440p captures, where the plain score with decoys still has +0.089.
- **With the scanner's own templates and its 757 decoys** the lowest true match is 0.946 and the highest other slot 0.924, a gap of +0.022, with a limit of 0.935 getting every slot right; `art_bare_mad` has 0.937 against 0.891.
- The overlay measurements in `Pipeline tried.md` use the `c-*` captures.
