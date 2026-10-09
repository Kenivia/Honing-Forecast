# Scanner config and calibration

The scanner recognises things by comparing screen crops with stored templates. This note covers where those templates and the brightness constants come from. All of it is developer tooling; end users do not calibrate anything.

## ScannerConfig.msgpack

`public/ScannerConfig.msgpack` is a list of templates. Each has raw RGBA pixels, a name, a tag, an offset rectangle, and an optional required confidence. The required confidence overrides the 0.9 pass limit of whichever comparison the template goes through (fixed-position or template matching); it is 0.8 on every page tab and null on everything else, and is edited per row on the Setup page (new captures start as null). The frontend fetches it at startup and hands it to Rust when the scanner starts. The OCR model is loaded the same way.

Two tags exist:

- **Icon**: one per material, a fixed-size square. Matched against inventory slots.
- **Anchor**: fixed pieces of UI used to locate things, and page tabs. Offsets are at the reference resolution. A page tab's offset is relative to its inventory window's origin. An anchor's offset is relative to the root it produces: the window origin for anything that locates a window, the UI block's top-left (so plain screen coordinates on a 16:9 capture) for the Storage button. The character inventory page tabs are 74x48; the storage ones are narrower and were cropped at 60x44. Template sizes divisible by 4 scale cleanly to 1080p; a text template that is not loses several points of match score. A size that does not is cut down to whole pixels before matching, so it costs a fraction of its border rather than its accuracy (see `Pipeline.md`); the sort button, at 38x38, is the worst case of the set at 1080p, where 28.5 becomes 28.

Template **names are a contract** with the Rust constants: anchor and page-tab names are looked up by string, and a missing name panics. The tag strings are matched literally too.

## How it is produced

The file is built from two sources, in this order:

1. **Icons, from Python.** `templates/items.json` is the item table: one row per in-game item with its tooltip `title`, its `icon` (a file in `templates/Icons`) and the `rarity` background. `templates/chests.json` is the chest table, written from the game's files together with the art in `templates/ChestIcons` (`Game files.md`); each of its "top" rows makes a template named `<icon>@<rarity>`, and a slot with such an icon is a chest. `templates/make_msg_pack.py` composites each distinct icon over its background and replaces the Icon templates in `public/ScannerConfig.msgpack` in place, leaving every other template alone. Rows may share an icon (books of one kind, a material and its event or roster-bound namesakes), and a row without an icon is only a title the tooltip reader accepts. A row's optional `confidence` becomes the template's required confidence in place of 0.95, and has to be the same on every row of an icon. The book rows have 0.93: at 1080p their slots score 0.944 to 0.953, each closest to the right kind of book, while the purple books stay under 0.85. A book row's `body` is its gear and honing levels as the tooltip's description gives them. Both tables are compiled into the scanner crate.
2. **Anchors, appended in the app.** The Setup sub-page of a character adds to that config. With a live screen share running, the developer marks a rectangle, names and tags it, and Rust crops and normalises it from the current frame. The page lists the entries for editing and reordering, and downloads the combined result as msgpack.
3. **Storage page tabs and window anchors, one-off.** The storage page tabs, the two storage windows' top-right icon groups and the inventory's search button were cropped from the 1440p screenshots by throwaway scripts that applied the same brightness normalisation (for setting 70), rather than captured in the app. Capturing them in the app works too, but the app stores screen positions, so the offsets must then be edited to be window-relative.
4. **Manual placement.** A config downloaded from the Setup page is copied into `public/` by hand. `templates/ScannerConfig.msgpack` is a leftover from the older flow and is no longer written.

## Where each anchor template was cut from

All 22 anchor-tagged templates are cut from the stills in `scripts/brightness/1440p raw`, at whole pixels, and normalised from setting 70 with the gamma law. The stills carry no record of their setting; 70 is read off the sort button's flat face, which is level 62 in every one of them and 62.4 at setting 70 on the 1080p captures (one level is about 1.5 settings).

- `storage page 1.png` to `storage page 4.png` (pet menu): the storage page tabs, the pet Storage button, and the two storages' top-right icon groups. Each tab's active template is from the still with that page open, its inactive one from another.
- `storage npc.png`: the NPC Storage button.
- `inventory top left.png`: the sort button, the search button, character page 1 active and page 2 inactive.
- `pet icon.png`: character page 2 active and page 1 inactive.
- There is no 1440p still with roster page 2 open, so "Roster page 1 inactive" and "Roster page 2 active" are character-storage tab pixels, which look the same.

The sort button and the four character page tabs used to be captures made in the app from some other frame; the stills differ from those by 0.3 to 1.8 levels on average.

## Rarity backgrounds

`templates/Backgrounds/<rarity>.webp` is what a slot of that rarity shows behind the art. **The ancient one was wrong**: 20 to 38 levels too dark, most at the bottom, so everything ancient scored 0.93 to 0.95 and was not read, on lossless stills too. That was the Level 4 scrolls and every ancient chest. It was rebuilt from the two 1440p recordings of 2026-10-08:

- `slot_scores` with `DUMP` wrote 20 ancient slots (scrolls, bars of gold, chests), brightness-normalised.
- For each, the pixels the art leaves bare (alpha under 3 and two pixels clear of it, digits excluded) are the background. The median over the slots covers 1,509 of the slot's 3,721 pixels, all round the edge.
- A cubic in x and y was fitted to those per channel, dropping outliers over 8 levels (a sparkle in one icon); the residual is 3 levels. The centre is extrapolated and under the art anyway.

Ancient slots now score 0.966 to 0.98, at 1080p and 1440p. The same fit for relic, legendary, epic and rare came out within 3 to 8 levels of the stored files, which is the fit's own error, so those were left. The script was a throwaway.

## Brightness calibration

The game's brightness slider changes every pixel, so a template captured at one setting will not match at another. The approach:

1. Estimate the player's brightness setting from the mean brightness of a matched anchor, using a fitted curve per anchor.
2. Remap each observed crop to a fixed reference brightness before comparing.

The fitted curves are per-variant constants in the scanner crate. A variant can also have none, and then says nothing about the brightness; the icon groups have none (see `Pipeline.md`).

The current curves of the sort button, the search button and the NPC Storage button were fitted to the mean of the matched patch at all 21 settings, over both sets of captures (`scripts/brightness/inputs`, the lone inventory, and `scripts/brightness/inputs storage`, the storage layout at the NPC; both 1080p). The means came from a temporary print in the anchor search, so there is no script that reproduces them as is. The pet menu's Storage button has no such captures and keeps its model-derived curve.

`scripts/brightness/calibrate_anchor.py` is defunct and kept for reference: the crops its sweep mode reads were dumped by the `crop_icons` bin, which was removed with the scanner's debug info. It printed a line to paste into an anchor's entry, in two modes:

- **From the model (default, no screenshots).** A stored template is already normalised, so the script runs the brightness model backwards on it to get the anchor's mean at every setting, then fits the curve. Run it with anchor names, or with none for every anchor in `public/ScannerConfig.msgpack`. Only the pet Storage button still uses this. It was off by up to 4 settings for the NPC button and by 9 for the icon groups at 1080p.
- **From a sweep (`--sweep`).** Fits real crops instead. `scripts/brightness/inputs` holds screenshots of the same scene at each setting, and `crates/scanner/src/main.rs` (a native harness with hardcoded local paths, not part of the app) runs the scanner over them and dumps the crops. This is how the model itself was calibrated. The dump reads `scripts/brightness/inputs` only.

For the character inventory anchors the two modes agree to within about 5 settings at the dark end and 1 at the bright end.

Calibrate on the resolution that matters most: a curve fitted at 1080p reads a button 2 to 3 settings high at 1440p.

The script's greyscale weights and the gamma ratio must match the Rust side for the fit to be valid. The model is described in `Pipeline.md`.

## Keeping things in sync

- **Where the art comes from.** The templates there now came from lostarkcodex.com; new ones are cut lossless from the game's icon sheets by the scripts in `scripts/game_files` (`Game files.md`). The pink books are `use_12_218` (weapon) and `use_12_219` (armor), the purple ones `use_7_69` and `use_7_70`; the site had one lossy copy of each. Both book templates are those files unchanged.
- Adding an item means putting its art in `templates/Icons`, adding a row to `templates/items.json` and running `make_msg_pack.py`. A new material also changes which chests matter, so run `scripts/game_files/chests.py` first. The icon's file name is the template name the rest of the app sees. Rebuild the wasm afterwards, since the table is compiled in.
- Adding or renaming an anchor means updating its entry in the Rust anchor list and rerunning brightness calibration for it. A bound must be at least the size of its template; slack beyond that is free, because the match position sets the root. The same template may be used by several anchors, but a piece of UI that is greyed out or recoloured in another context needs its own template.
- The TypeScript interfaces describing scanner structs are hand-written copies of the Rust ones and are not checked against them.
