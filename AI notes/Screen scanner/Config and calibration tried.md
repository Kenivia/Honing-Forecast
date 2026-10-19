# Scanner config and calibration: what was tried

How the templates and brightness curves of `Config and calibration.md` got to where they are, and the tools that are gone. Only read this when trying something new there.

## Templates and anchors

- **`anchors.py` was checked against what was there**: `locate` finds all seven anchors tried at 0.00 levels apart at setting 70, and cutting them again with `add` leaves the file byte for byte the same.
- **The app used to have a Setup sub-page** that captured anchors from a live screen share (a `setup` step in Rust). It is gone: it stored screen positions, so offsets had to be edited by hand afterwards, and none of the current anchors came from it.
- The sort button and the four character page tabs used to be captures made in the app from some other frame; the stills differ from those by 0.3 to 1.8 levels on average.

## The ancient background

**The ancient one was wrong**: 20 to 38 levels too dark, most at the bottom, so everything ancient scored 0.93 to 0.95 and was not read, on lossless stills too. That was the Level 4 scrolls and every ancient chest. It was rebuilt from the two 1440p recordings of 2026-10-08:

- `slot_scores` with `DUMP` wrote 20 ancient slots (scrolls, bars of gold, chests), brightness-normalised.
- For each, the pixels the art leaves bare (alpha under 3 and two pixels clear of it, digits excluded) are the background. The median over the slots covers 1,509 of the slot's 3,721 pixels, all round the edge.
- A cubic in x and y was fitted to those per channel, dropping outliers over 8 levels (a sparkle in one icon); the residual is 3 levels. The centre is extrapolated and under the art anyway.

Ancient slots now score 0.966 to 0.98, at 1080p and 1440p. The same fit for relic, legendary, epic and rare came out within 3 to 8 levels of the stored files, which is the fit's own error, so those were left. The script was a throwaway.

## calibrate_anchor.py

`scripts/brightness/calibrate_anchor.py` is defunct and kept for reference: the crops its sweep mode reads were dumped by the `crop_icons` bin, which was removed with the scanner's debug info. It printed a line to paste into an anchor's entry, in two modes:

- **From the model (default, no screenshots).** A stored template is already normalised, so the script runs the brightness model backwards on it to get the anchor's mean at every setting, then fits the curve. Run it with anchor names, or with none for every anchor in `public/ScannerConfig.msgpack`. Only the pet Storage button still uses this. It was off by up to 4 settings for the NPC button and by 9 for the icon groups at 1080p.
- **From a sweep (`--sweep`).** Fits real crops instead. `scripts/brightness/inputs` holds screenshots of the same scene at each setting, and `crates/scanner/src/main.rs` (a native harness with hardcoded local paths, removed on 2026-10-06) ran the scanner over them and dumped the crops. This is how the model itself was calibrated. The dump reads `scripts/brightness/inputs` only.

For the character inventory anchors the two modes agree to within about 5 settings at the dark end and 1 at the bright end.
