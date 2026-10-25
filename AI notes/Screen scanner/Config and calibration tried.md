# Scanner config and calibration: don't try these

Warnings behind `Config and calibration.md`: what failed or was ruled out, and the tools that are gone. Only read this when trying something new there.

## Templates and anchors

- Don't capture anchors from a live screen share in the app (the old Setup sub-page, a `setup` step in Rust): it stored screen positions, so offsets had to be edited by hand afterwards, and none of the current anchors came from it. Cut them from stills with `anchors.py`.

## The ancient background

- Don't use a stored background without checking it against recordings: the ancient one was 20 to 38 levels too dark, most at the bottom, so everything ancient scored 0.93 to 0.95 and was not read, on lossless stills too (the Level 4 scrolls and every ancient chest).
- How it was fixed, for the next time: `slot_scores` with `DUMP` wrote 20 ancient slots brightness-normalised; the pixels the art leaves bare (alpha under 3 and two pixels clear of it, digits excluded) are the background, covering 1,509 of a slot's 3,721 pixels, all round the edge; a cubic in x and y per channel was fitted to their median, dropping outliers over 8 levels (a sparkle in one icon). The residual is 3 levels. Ancient slots then score 0.966 to 0.98.
- Don't refit the other rarities: the same fit for relic, legendary, epic and rare came out within 3 to 8 levels of the stored files, which is the fit's own error.

## calibrate_anchor.py

- Don't calibrate from the model alone (`calibrate_anchor.py`, default mode, no screenshots: it runs the brightness model backwards on the stored template): it was off by up to 4 settings for the NPC button and by 9 for the icon groups at 1080p. Only the pet Storage button still has such a curve, for lack of captures. The two modes agree to within about 5 settings at the dark end and 1 at the bright end for the character inventory anchors.
- Don't expect `--sweep` to run: it read crops dumped by the `crop_icons` bin, which was removed with the scanner's debug info, and the native harness that produced them (`crates/scanner/src/main.rs`, hardcoded local paths) went on 2026-10-06. It only read the old `scanner_setup/brightness/inputs`.
