# Scanner config and calibration

The scanner recognises things by comparing screen crops with stored templates. This note covers where those templates and the brightness constants come from. All of it is developer tooling; end users do not calibrate anything.

## ScannerConfig.msgpack

`public/ScannerConfig.msgpack` is a list of templates. Each has raw RGBA pixels, a name, a tag, and an offset rectangle. The frontend fetches it at startup and hands it to Rust when the scanner starts. The OCR model is loaded the same way.

Two tags exist:

- **Icon**: one per material, a fixed-size square. Matched against inventory slots.
- **Anchor**: fixed pieces of the inventory window's chrome and its page tabs. Their offsets are relative to the window origin at the reference resolution, which is how finding an anchor locates the window.

Template **names are a contract** with the Rust constants: anchor and page-tab names are looked up by string, and a missing name panics. The tag strings are matched literally too.

## How it is produced

The file is built from two sources, in this order:

1. **Icons, from Python.** `templates/make_msg_pack.py` composites each icon in `templates/Icons` over its rarity background from `templates/Backgrounds` and writes `templates/ScannerConfig.msgpack`. The script appends to whatever is already in that file.
2. **Anchors, appended in the app.** The Setup sub-page of a character adds to that config. With a live screen share running, the developer marks a rectangle, names and tags it, and Rust crops and normalises it from the current frame. The page lists the entries for editing and reordering, and downloads the combined result as msgpack.
3. **Manual placement.** The downloaded file is copied into `public/` by hand. Nothing automates this, so `templates/` and `public/` hold different files.

## Brightness calibration

The game's brightness slider changes every pixel, so a template captured at one setting will not match at another. The approach:

1. Estimate the player's brightness setting from the mean brightness of a matched anchor, using a fitted curve per anchor.
2. Remap each observed crop to a fixed reference brightness before comparing.

The fitted curves are constants in the scanner crate, produced offline:

- `scripts/brightness/inputs` holds screenshots of the same scene at each brightness setting.
- `crates/scanner/src/main.rs` is a native harness, not part of the app. It runs the scanner over those screenshots and dumps the crops it found. It uses hardcoded local paths.
- `scripts/brightness/calibrate_anchor.py` fits the curves from those crops and prints a Rust snippet to paste into the scanner constants.

The script's greyscale weights and anchor list must match the Rust side for the fit to be valid.

## Keeping things in sync

- Adding a material means adding its icon and rarity to the script, regenerating, and making sure the name matches what the rest of the app expects.
- Adding or renaming an anchor means updating the Rust constants that reference it and rerunning brightness calibration for it.
- The TypeScript interfaces describing scanner structs are hand-written copies of the Rust ones and are not checked against them.
