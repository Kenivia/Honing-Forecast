# Testing against the live game

The inventory scanner can be worked on against the game running on this PC: drive it with the `lostark-market` MCP, take stills and debug captures of it natively, and hold the scanner to known answers with `scanner_check`. No browser is involved: the native bins run the same scanner the page does (`Pipeline.md`, "Debug captures").

## Folders

| Folder | What |
| --- | --- |
| `ignored/live/` | working files: stills and captures being looked at |
| `ignored/scanner_tests/recordings/` | debug captures kept as tests (the page's `.hfcap` files and `live-*.hfcap` made here); `pnpm scanner-suite`, `scanner-profile` and the e2e tests read them from here |
| `ignored/scanner_tests/stills/<folder>/` | stills kept as tests, a folder per subject (`hud_size/`); the suite runs each folder as `local-<folder>` |

All of `ignored/` is git-ignored, so these are on this PC only. Move a file into `scanner_tests` only once what it must read is known (below).

## Driving the game

- `lostark-market` (`.mcp.json`, `market_mcp`) is this PC's game: `screenshot` (crop and shrink in client pixels) and `run` (`click`, `move`, `key`, `scroll`, `wait`, ...; `Market scanner/Overview.md`). Coordinates are 1080p client pixels. Never click Buy, Bid or Confirm in the Market.
- `screenshot` is for looking; it is shrunk and lands in the session's tool results. Stills to keep come from `game_capture` (below).
- **Settings**: Esc closes the open windows first, then opens the Game Menu; Settings is at (1022, 742). Video (Resolution, Screen, **Force 21:9 Aspect Ratio** checkbox at (1176, 452) once scrolled to the top) and Gameplay > Display Settings (578, 491), whose UI Display Settings row has **HUD Size** radios 80/90/100/105/110% at x 968, 1053, 1139, 1224, 1309, y 541. Apply is at (1088, 812), OK (1193, 812), Cancel (1298, 812).
- **Every Apply or OK asks "There are 2 unbound hotkeys. Do you want to save without binding the keys?"** on this account; OK is at (919, 613). That only saves the settings as they are.
- **At forced 21:9** the picture is 1920x810 at y 135 to 945, and a point of the 16:9 UI moves to (240 + 0.75 x, 135 + 0.75 y): HUD radios at y 541, x 966, 1031, 1095, 1158, 1221; Apply (1056, 744); the warning's OK (929, 595); Settings in the Game Menu (1006, 691). Windows docked right (the inventory) sit at the right edge of the 21:9 picture, not of the 16:9 block: the inventory's gold coin at (1862, 742).
- A settings hover tooltip can cover the next control; move the cursor off first.
- Leave the game as it was found (the author keeps 16:9 and HUD 80%, inventory open).

## Stills and captures

`pnpm game-capture still <file.png>` and `pnpm game-capture record <file.hfcap> [--seconds N]` (`crates/market/src/bin/game_capture.rs`; or `target/release/game_capture.exe` once built).

- A still is the client area at full resolution, as the scanner wants it.
- A recording is a real `HFCAP005` capture (start record, then a frame every 33 ms or as fast as the screenshot allows, about 17 fps and 7 MB/s with the world moving): `capture_replay`, `capture_edit` and `scanner_check` take it like one from the page. It has no end record; `capture_edit` adds one. It stops at `--seconds`, on F10, or when `<file>.stop` appears, so the usual loop is: start `record` in the background, drive the game with `run`, `touch <file>.stop`, wait for the file to go, then `capture_replay`.
- **The cursor is not in the frames** (the screenshot does not draw it; a screen share does).

## Known answers

`pnpm scanner-check [-- <part of a file name>...]` (`crates/scanner/src/bin/scanner_check.rs`) runs every file listed in an `expected.json` under `ignored/scanner_tests` (three at a time, a capture is held whole in memory) and exits 1 if one is wrong. About a minute for all.

```json
{"hud 90 bar gold hover.png": {"Silver": [0, 362272254, 0], "Gold": [280000, 0, 3960769]}}
```

- Each listed currency must be Good with those bands (char bound, roster bound, tradable); unlisted ones are not checked.
- A capture's hand edits (`capture_edit`) are checked as `capture_replay --check` does; list a capture with `{}` to check only those.
- Only list what was checked by eye against the frames or the game (`capture-truth`: a reading is not an answer).
- `pnpm scanner-suite` still says whether anything read changed; `scanner_check` says whether what is known reads right.

## On file (2026-11-24)

- `stills/hud_size/`: `hud <80|90|100|105|110> settings[ 21 by 9].png` (the settings window open, nothing hovered) and `hud <n> bar gold hover[ 21 by 9].png` (the bar's gold hovered), 16:9 and forced 21:9, 1080p. The 16:9 `settings` ones were taken by the author, the rest here.
- `recordings/live-2026-11-24T06-24-hud110-bar-gold.hfcap` (16:9) and `live-2026-11-24T06-40-hud110-21-by-9-bar-gold.hfcap` (forced 21:9): HUD 110%, the bar's gold hovered, then the inventory's.
- Listed with hand edits only: the three `CHECKED` captures of `Browser harness/Screen scanner.md`. With currencies: `honing-forecast-2026-11-24T07-22-59-049Z` (a user's, HUD 100%, a one-line "Gold Owned").
