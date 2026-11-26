# The Lost Ark client

What the screen scanner (`Screen scanner/`) and the market scanner (`Market scanner/`) both depend on: how to drive the game, how it draws what we read, its life cycle, and what to redo when it changes. How each scanner is developed and tested is in `Screen scanner/Developing and testing.md` and `Market scanner/Developing and testing.md`.

## Rules

- **Never press Buy, Bid or Confirm** in the Market, the Auction House or the shop. The market scanner opens the purchase window only to read it (`market.rs`) and never touches the exchange's purchase panel (`exchange.rs`).
- **Ask before using the game** when the PC may be in use, and **leave it as found**. The author keeps 16:9 (Force 21:9 off), HUD Size 90% and the inventory open on this PC.
- The update signing key (`ignored/market/update_signing_key.pem`) is never printed or copied anywhere (`Market scanner/Remote.md`).

## Driving it

| Server (`.mcp.json`) | Drives | Use |
| --- | --- | --- |
| `lostark-market` | the game on this PC, directly | inventory-scanner work, exploring screens by hand |
| `lostark-home` | this PC through the relay and its `market_agent` | testing the remote path |
| `lostark-borry` | the friend's PC (`borry-laptop`) through the relay | that machine only; `run` works only while its agent is idle |

- All three are `target/release/market_mcp.exe` (`--remote <machine>` for the relay ones). Its tools are `screenshot` (a crop, shrunk, for looking) and `run`, a script in the command language of `crates/market/src/ctl.rs` (click, key, type, scroll, wait, `shot`, `ocr`, `num`, `ref`, `looks`). The remote ones add `control`, `exec` and `files` (`Remote.md`). `market_ctl "<script>"` runs the same language from a shell.
- **Coordinates are 1080p client pixels** everywhere: the game is full screen at 1920x1080 on every machine the market scanner runs on. Nothing in the market scanner scales. The inventory scanner reads any resolution.
- **Input** is plain `SendInput`, not elevated (`input.rs`): moves land on the exact pixel, text is typed as unicode. Every action first brings the game to the front and stops if F10 is held (`game.rs`). If injected input were ever refused, the fallback is a USB HID device.
- **Screenshots** are GDI captures of the game's monitor (`xcap`, `shot.rs`). **The cursor is not in them**, though a browser screen share has it. So stills and `game_capture` recordings never show what a cursor covers.
- A Claude Code session holds `market_mcp.exe`, so a build that relinks it needs the sessions closed or the server disabled (`/mcp`). Other bins build fine beside it. Never build a bin while it runs: Windows locks the exe (use `CARGO_TARGET_DIR=<scratch>`).

## Getting around

Positions of everything the market scanner clicks are in its code (`ui.rs`, `market.rs`, `auction.rs`, `exchange.rs`, `region.rs`). Each step is checked by a reference crop. The rest are for driving by hand:

- **Esc** closes the open windows first, then opens the Game Menu. With a popup up (the Event Notice at server select), Esc opens "Exit Game" instead.
- **Settings** is at (1022, 742) in the Game Menu. Under Video are Resolution, Screen and the **Force 21:9 Aspect Ratio** checkbox (1176, 452, scrolled to the top). Under Gameplay > Display Settings (578, 491) are the **HUD Size** radios 80/90/100/105/110% at x 968, 1053, 1139, 1224, 1309, y 541. The buttons are Apply (1088, 812), OK (1193, 812) and Cancel (1298, 812).
- **Every Apply or OK asks about the unbound hotkeys** on these accounts; its OK is at (919, 613) and saves the settings as they are. A settings hover tooltip can cover the next control, so move the cursor off first.
- **At forced 21:9** the picture is 1920x810 at y 135 to 945, and a 16:9 UI point moves to (240 + 0.75 x, 135 + 0.75 y). Windows docked right stay at the picture's right edge, not the 16:9 block's.
- **Alt+Y** toggles the Market, always fresh: Market tab, own class, All Tiers. **F9** opens the Lost Ark Shop; it is a per-character keybind and has to be set on every new character.
- **The Stronghold** is where the market characters wait. Its background stays steady behind the translucent windows, and the hourly chaos-gate notice does not show there. A character's first visit opens a "Guide" window; Esc closes it.

## How it draws

These facts are true of the client, not of one reader, and both scanners lean on them.

- **The UI is laid out in a 16:9 block centred in the picture and scaled continuously** with its height. Nothing snaps to whole pixels, so an offset that depends on the resolution is ours, not the game's. The HUD and windows the game keeps "inside the render area" (the currency window, the lone inventory) hug what is rendered, which is wider than the block at 21:9 or on an ultrawide screen.
- **HUD Size** scales the HUD (the currency bar at the top) from the render area's corner, and nothing else.
- **The brightness setting is a display gamma** applied before blending (`Screen scanner/Pipeline.md`, "Brightness"). The inventory scanner estimates it and normalises; the market scanner's crops were taken on the machines' own setting.
- **Windows and tooltips are translucent.** What shows through changes their pixels, so crop matches either mask the background out (alpha in a reference crop, ink pixels only) or happen over a steady one.
- **Numbers are drawn in ExuberanceF**, from `font.lpk` (`Binaries/Fonts/<Language>/FontMap.xml`), with monospaced digits. `public/fonts/English.ttf` is cut from it. Both readers draw it themselves rather than learn shapes from captures (`Further learnings/How the game draws.md`).
- **Text in the serif gold UI font** (titles, headers) reads badly with any recogniser; it is checked by reference crops.
- **Players can swap the game's font files**, so the font is not the client's language.

## Life cycle

- **Launching**: through Steam (`steam://rungameid/1599340`); Steam itself is logged in by hand. The process and a 1152x720 splash come first, and the window can stay hidden for minutes (about 4 on a new account's first launch, when a "Game Entry Narration Assistance" screen also comes before server select). `market_agent` waits up to 30 minutes for character or server select (`Game::try_find`).
- **"Cannot connect to the server. [EAC Offline]"** on Launch can happen now and then and is not only the network. It drops to server select with the Event Notice up, and only a restart clears it.
- **Maintenance** (seen on a Wednesday from about 12:00 UTC, both regions at once): a client launched then sits at server select under an "Unavailable Region … [OK]" popup (crop `region_closed`). Not seen yet: what a client in game sees when maintenance starts, the "under maintenance" screen right after it ends, and a login queue.
- **The attendance window** ("Login Events") opens over the Stronghold on the first log-in after 06:00 server time; Esc closes it.
- **Days**: the Market's 5-day averages roll over at 00:00 UTC, not at the 10:00 UTC reset. That was checked from a PC whose clock is UTC; `borry-laptop` (UTC+8) can tell whether it is really local midnight.
- **The PC must stay awake with the screen on.** A display that turns off wakes only to the Windows sign-in page, and the game cannot be brought to the front again. Sleep is not an option either (Modern Standby). `market_agent` holds the display awake while scanning.
- **Machine quirks** go in profile crops, not code: an unactivated Windows draws an "Activate Windows" watermark over the bottom right (where the Market's Buy button is), so that machine has its own `buy_button` crop.

## When the game changes

A patch can change three things: the tables and art, how a screen looks, and what it says. Check each:

1. **Tables and art** (every patch). Run `scanner_setup/game_files/extract.py`, keep a snapshot as `ignored/game_files_<export date>/`, and diff its tables with the last snapshot (`dump_all` in the oldest one decrypts whole archives). Then re-run every generator in the order in `Screen scanner/Game files.md` ("After a patch"): chests, decoys, the config, the choices and income data, the honing constants, the icons, and the market's item art (`crates/market/tools/market_items.py`). `git diff` on the generated files is the change list.
2. **Fonts**: `fonts.py`, then `glyph_eval` (market) and `scanner-check` (slots).
3. **A screen that looks different.** Symptoms are a market unit failing every try with the same screenshot in `ignored/market/debug/`, or the inventory scanner finding no window or tab. Take the crop again where the screen now shows the right thing: `market_ctl "ref <name> x y w h"` for the market (`Market scanner/Developing and testing.md`), `anchors.py add/locate` for the inventory scanner's anchors and tabs (`Screen scanner/Config and calibration.md`). Move a click whose button moved in the code that clicks it.
4. **Tooltip and text changes** (new lines, other colours, other wording): `Screen scanner/Tooltips.md`, then a capture of the new tooltip as a test (`Screen scanner/Developing and testing.md`).
5. **Then prove it**: `pnpm scanner-check` and a suite `cmp` against the last reference run, the e2e `CHECKED` captures, and a `market_scan --once --quick` round with `--strips` checked by `verify_strips.py`.
