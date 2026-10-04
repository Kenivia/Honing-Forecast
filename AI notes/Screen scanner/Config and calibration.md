# Scanner config and calibration

The scanner recognises things by comparing screen crops with stored templates. This note covers where those templates and the brightness constants come from. All of it is developer tooling; end users do not calibrate anything.

## ScannerConfig.msgpack

`public/ScannerConfig.msgpack` is a list of templates. Each has raw RGBA pixels, a name, a tag, an offset rectangle, and an optional required confidence. The required confidence overrides the 0.9 pass limit of whichever comparison the template goes through (fixed-position or template matching); it is 0.8 on every page tab and null on everything else, and is edited per row on the Setup page (new captures start as null). The frontend fetches it at startup and hands it to Rust when the scanner starts. The OCR model is loaded the same way.

Two tags exist:

- **Icon**: one per material, a fixed-size square. Matched against inventory slots.
- **Anchor**: fixed pieces of UI used to locate things, and page tabs. Offsets are at the reference resolution. A page tab's offset is relative to its inventory window's origin. An anchor's offset is relative to the root it produces: the window origin for anything that locates a window, the UI block's top-left (so plain screen coordinates on a 16:9 capture) for the Storage button. The character inventory page tabs are 74x48; the storage ones are narrower and were cropped at 60x44. Template sizes divisible by 4 scale cleanly to 1080p; a text template that is not loses several points of match score.

Template **names are a contract** with the Rust constants: anchor and page-tab names are looked up by string, and a missing name panics. The tag strings are matched literally too.

## How it is produced

The file is built from two sources, in this order:

1. **Icons, from Python.** `templates/make_msg_pack.py` composites each icon in `templates/Icons` over its rarity background from `templates/Backgrounds` and writes `templates/ScannerConfig.msgpack`. The script appends to whatever is already in that file.
2. **Anchors, appended in the app.** The Setup sub-page of a character adds to that config. With a live screen share running, the developer marks a rectangle, names and tags it, and Rust crops and normalises it from the current frame. The page lists the entries for editing and reordering, and downloads the combined result as msgpack.
3. **Storage page tabs and window anchors, one-off.** The storage page tabs, the two "move all duplicates" templates and the storage inventory's bottom anchor were cropped from the 1440p storage screenshots by a throwaway script that applied the same brightness normalisation, rather than captured in the app. Capturing them in the app works too, but the app stores screen positions, so the offsets must then be edited to be window-relative.
4. **Manual placement.** The downloaded file is copied into `public/` by hand. Nothing automates this, so `templates/` and `public/` hold different files.

## Brightness calibration

The game's brightness slider changes every pixel, so a template captured at one setting will not match at another. The approach:

1. Estimate the player's brightness setting from the mean brightness of a matched anchor, using a fitted curve per anchor.
2. Remap each observed crop to a fixed reference brightness before comparing.

The fitted curves are per-variant constants in the scanner crate, produced by `scripts/brightness/calibrate_anchor.py`, which prints a line to paste into the anchor's entry. It has two modes:

- **From the model (default, no screenshots).** A stored template is already normalised, so the script runs the brightness model backwards on it to get the anchor's mean at every setting, then fits the curve. Run it with anchor names, or with none for every anchor in `public/ScannerConfig.msgpack`. This is how every storage-layout template was calibrated, and is the way to calibrate a new anchor.
- **From a sweep (`--sweep`).** Fits real crops instead. `scripts/brightness/inputs` holds screenshots of the same scene at each setting, and `crates/scanner/src/main.rs` (a native harness with hardcoded local paths, not part of the app) runs the scanner over them and dumps the crops. This is how the character inventory anchors and the model itself were calibrated.

For the character inventory anchors the two modes agree to within about 5 settings at the dark end and 1 at the bright end.

The script's greyscale weights and model constants must match the Rust side for the fit to be valid.

## Keeping things in sync

- Adding a material means adding its icon and rarity to the script, regenerating, and making sure the name matches what the rest of the app expects.
- Adding or renaming an anchor means updating its entry in the Rust anchor list and rerunning brightness calibration for it. A bound must be at least the size of its template; slack beyond that is free, because the match position sets the root. The same template may be used by several anchors, but a piece of UI that is greyed out or recoloured in another context needs its own template.
- The TypeScript interfaces describing scanner structs are hand-written copies of the Rust ones and are not checked against them.
