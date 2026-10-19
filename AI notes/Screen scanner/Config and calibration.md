# Scanner config and calibration

The scanner recognises things by comparing screen crops with stored templates. This note covers where those templates and the brightness constants come from. All of it is developer tooling; end users do not calibrate anything. How the templates and curves got to where they are, and the older tools, are in `Config and calibration tried.md`; read it only when trying something new here.

## ScannerConfig.msgpack

`public/ScannerConfig.msgpack` is a list of templates. Each has raw RGBA pixels, a name, a tag, an offset rectangle, and an optional required confidence. The scripts also write a `normalized` flag, which nothing reads. The required confidence overrides the 0.9 pass limit of whichever comparison the template goes through (fixed-position or template matching); it is 0.8 on every page tab and null on everything else, and is set with the anchor script (below). The frontend fetches it at startup and hands it to Rust when the scanner starts. The OCR model is loaded the same way.

Three tags exist:

- **Icon**: one per material, a fixed-size square. Matched against inventory slots.
- **Decoy**: a look-alike of an icon that we do not count, made like an icon from `templates/decoys.json` and `templates/DecoyIcons` and named `~<icon>@<rarity>`. Matched against slots beside the icons (`Pipeline.md`, "Slots"); nothing else reads them.
- **Anchor**: fixed pieces of UI used to locate things, and page tabs. Offsets are at the reference resolution. A page tab's offset is relative to its inventory window's origin. An anchor's offset is relative to the root it produces: the window origin for anything that locates a window, the UI block's top-left (so plain screen coordinates on a 16:9 capture) for the Storage button. The character inventory page tabs are 74x48; the storage ones are narrower and were cropped at 60x44. Template sizes divisible by 4 scale cleanly to 1080p; a text template that is not loses several points of match score. A size that does not is cut down to whole pixels before matching, so it costs a fraction of its border rather than its accuracy (see `Pipeline.md`); the sort button, at 38x38, is the worst case of the set at 1080p, where 28.5 becomes 28.

Template **names are a contract** with the Rust constants: anchor and page-tab names are looked up by string, and a missing name panics. The tag strings are matched literally too.

## How it is produced

The file is built from two sources, in this order:

1. **Icons, from Python.** `templates/items.json` is the item table: one row per in-game item with its tooltip `title`, its `icon` (a file in `templates/Icons`) and the `rarity` background. `templates/chests.json` is the chest table, written from the game's files together with the art in `templates/ChestIcons` (`Game files.md`); each of its "top" rows makes a template named `<icon>@<rarity>`, and a slot with such an icon is a chest. `templates/make_msg_pack.py` composites each distinct icon over its background and replaces the Icon templates in `public/ScannerConfig.msgpack` in place, leaving every other template alone. Rows may share an icon (books of one kind, a material and its event or roster-bound namesakes), and a row without an icon is only a title the tooltip reader accepts. A row's optional `confidence` becomes the template's required confidence in place of 0.95, and has to be the same on every row of an icon. The book rows have 0.93: at 1080p their slots score 0.944 to 0.953, each closest to the right kind of book, while the purple books stay under 0.85. A book row's `body` is its gear and honing levels as the tooltip's description gives them. Both tables are compiled into the scanner crate.
2. **Anchors, from Python.** `scripts/anchor setup/anchors.py` edits the Anchor templates of `public/ScannerConfig.msgpack` in place and leaves the icons alone:
   - `add <still> <name> <x> <y> <width> <height> --offset <x> <y> [--brightness 70] [--confidence 0.8]` cuts a whole-pixel rectangle out of a 1440p still, normalises it from the setting the still was taken at with the gamma law, and adds it, or replaces the anchor of that name where it stands. `--offset` is what the config stores: where the template sits relative to the root it gives.
   - `locate <still> <name>` finds a stored anchor in a still and says how far apart the pixels are, which gives the rectangle to cut again and checks the still's setting.
   - `list`, `confidence <name> <value|none>`, `remove <name>`.
   - Every anchor is also kept as `scripts/anchor setup/anchors/<name>.png` for review (`export` writes them all).
3. `templates/ScannerConfig.msgpack` is a leftover from an older flow and is no longer written.


## Where each anchor template was cut from

The 22 older anchor-tagged templates are cut from the stills in `scripts/brightness/1440p raw`, at whole pixels, and normalised from setting 70 with the gamma law. The stills carry no record of their setting; 70 is read off the sort button's flat face, which is level 62 in every one of them and 62.4 at setting 70 on the 1080p captures (one level is about 1.5 settings).

- `storage page 1.png` to `storage page 4.png` (pet menu): the storage page tabs, the pet Storage button, and the two storages' top-right icon groups. Each tab's active template is from the still with that page open, its inactive one from another.
- `storage npc.png`: the NPC Storage button.
- `inventory top left.png`: the sort button, the search button, character page 1 active and page 2 inactive.
- `pet icon.png`: character page 2 active and page 1 inactive.
- There is no 1440p still with roster page 2 open, so "Roster page 1 inactive" and "Roster page 2 active" are character-storage tab pixels, which look the same.
- `silver gold shard/*.png` (2026-10-17, **setting 50**, read off the sort button with `locate`): the currency templates. `Basic currency header` and `Currency window tab` (the currency window's anchor, offsets from the game's top left) and `Destiny shard` from `currency breakdown default position .png`; `Gold coin`, `Silver coin` and the three `Bar ...` icons from `char inventory no hover.png`; `Gold tooltip title` from `hovered in inventory tooltip left.png`. The coins are cut with their rim and 2 px around it (`Currency.md`). The icons' offsets are not read: their bounds are in `currency.rs`. None has a brightness curve. The game's own files only have the large item art of these currencies (`Game files.md`), so they were cut from stills.


## Rarity backgrounds

`templates/Backgrounds/<rarity>.webp` is what a slot of that rarity shows behind the art. The ancient one was rebuilt from recordings (a cubic fitted per channel to the pixels the art leaves bare); the others are the stored files.

## Brightness calibration

The game's brightness slider changes every pixel, so a template captured at one setting will not match at another. The approach:

1. Estimate the player's brightness setting from the mean brightness of a matched anchor, using a fitted curve per anchor.
2. Remap each observed crop to a fixed reference brightness before comparing.

The fitted curves are per-variant constants in the scanner crate. A variant can also have none, and then says nothing about the brightness; the icon groups have none (see `Pipeline.md`).

The current curves of the sort button, the search button and the NPC Storage button were fitted to the mean of the matched patch at all 21 settings, over both sets of captures (`scripts/brightness/inputs`, the lone inventory, and `scripts/brightness/inputs storage`, the storage layout at the NPC; both 1080p). The means came from a temporary print in the anchor search, so there is no script that reproduces them as is. The pet menu's Storage button has no such captures and keeps its model-derived curve.

`scripts/brightness/calibrate_anchor.py` is defunct and kept for reference (`Config and calibration tried.md`). The pet menu's Storage button still has the curve it made from the brightness model.

Calibrate on the resolution that matters most: a curve fitted at 1080p reads a button 2 to 3 settings high at 1440p.

The script's greyscale weights and the gamma ratio must match the Rust side for the fit to be valid. The model is described in `Pipeline.md`.

## Keeping things in sync

- **Where the art comes from.** The templates there now came from lostarkcodex.com; new ones are cut lossless from the game's icon sheets by the scripts in `scripts/game_files` (`Game files.md`). The pink books are `use_12_218` (weapon) and `use_12_219` (armor), the purple ones `use_7_69` and `use_7_70`; the site had one lossy copy of each. Both book templates are those files unchanged.
- Adding an item means putting its art in `templates/Icons`, adding a row to `templates/items.json` and running `make_msg_pack.py`. A new material also changes which chests matter, so run `scripts/game_files/chests.py` first. The icon's file name is the template name the rest of the app sees. Rebuild the wasm afterwards, since the table is compiled in.
- Adding or renaming an anchor means updating its entry in the Rust anchor list and rerunning brightness calibration for it. A bound must be at least the size of its template; slack beyond that is free, because the match position sets the root. The same template may be used by several anchors, but a piece of UI that is greyed out or recoloured in another context needs its own template.
- The TypeScript interfaces describing scanner structs are hand-written copies of the Rust ones and are not checked against them.
