# Market scanner

A native program that drives the game's Market and Auction House with synthetic input and reads them off the screen, to replace loa-buddy as the site's price source (loa-buddy stays the fallback) and to collect what nobody offers (accessories, gems). Local only for now: it writes SQLite; the upload to the Worker and the site viewer are the next milestone. One region (NAE), on the character Kenicards. What was tried and failed is in `Overview tried.md`.

Code: `crates/market` (`hf-market`), Windows only. It reuses `hf-scanner`'s recogniser for text (`ocr.rs`).

## Running

- `cargo run --release -p hf-market --bin market_scan -- [--gap 60] [--once] [--only materials,gems,accessories] [--db PATH]`: a pass, then waits `--gap` minutes after it ends, again, until F10 is held or Ctrl+C. Started and stopped by hand; the game must be running, logged in, full screen at 1920x1080 on the primary layout, and the PC is not usable meanwhile (the game is kept in front). Writes `ignored/market/market.sqlite`, logs to `ignored/market/scan.log`, keeps screenshots of what it could not read in `ignored/market/debug/`.
- A pass takes about 4 min of materials, 1.5 of gems and 8 of accessories, plus up to 6 min waiting out the hourly notice.
- `market_ctl "<commands>"`: drive and read by hand; the command language is in `ctl.rs` (click, scroll, key, type, shot, `ocr`, `num`, `ref`, `looks`).
- `market_mcp`: the same as an MCP server (`lostark-market` in `.mcp.json`; tools `screenshot` and `run`). Build first: it runs `target/release/market_mcp.exe`.
- `input_test`: which ways of sending input the game takes (below). `glyph_eval`: scores the number reader on `ignored/market/dataset/labels.json` (102 labelled numbers over every number style; 102 right). `cap_dump`: frames of an `.hfcap` as PNGs.
- Never build while `market_scan` runs: Windows locks the exe. Use `CARGO_TARGET_DIR=<scratch>`.

## Input and capture

- Plain `SendInput` works, not elevated (2026-11-20): absolute moves land on the exact pixel, clicks with or without delays, and keys as virtual keys, scan codes or unicode characters. `LOSTARK.exe` is not elevated. Text is typed as unicode (case and spaces kept, which the search boxes need). Should injected input ever be refused, the fallback is a USB HID device (Pi Pico) fed over serial.
- Every action first brings the game to the front (`Game::focus`, AttachThreadInput then SetForegroundWindow) and exits if F10 is held. Screenshots are GDI captures of the game's monitor (`xcap`); full screen works.
- All coordinates are 1080p client pixels, measured on this machine; nothing scales.

## Getting around

- **Alt+Y toggles the Market and always opens it fresh**: Market tab, Search Market, class = own class, All Tiers, category tree collapsed. Every section starts from that (`fresh_market`), which is the recovery from any state too (Esc if Alt+Y does not open it).
- **State is checked by reference crops, not text** (`refs/<name>@x_y.png`, max-channel grey; `looks` = mean difference under 14 at the best of five 1 px shifts; a match reads 0.0, one row off reads 32). Window titles, headers, the Buy button, picked dropdown values, typed option keywords. A new one is made with `market_ctl "ref name x y w h"` once the screen shows the right thing.
- **Dropdowns are picked and checked, and picked again when the check fails** (`pick`): the first pick after Alt+Y often does not take. Lists that scroll are scrolled to the top first and then down a known number of rows (one notch is one row, 29 px).
- **The hourly notice** (chaos gate, 3 min before and on the hour) can cover the window's top: nothing starts in minutes 56 to 1 (`wait_safe_time`).
- **Log-in messages** (yellow, bottom middle) can cover a row: a value that does not parse is read again from a new screenshot after 2.5 s, up to four times (`steady`).
- **Never buy**: the Market's Buy (1580, 926) is pressed only when the Market header and the Buy button both match their crops (the Auction House's Bid/Buy sits on the same spot), and only to open the purchase window, which is closed by its X. Enter is only pressed in the Auction House search box.

## Reading

- **Numbers are read by the game's font, not the recogniser** (`glyph.rs`): split at empty columns, a glyph under half the line's height is a separator ('.') or dash, every other is compared with each digit drawn in `public/fonts/English.ttf` (ExuberanceF) with `ab_glyph`, sized to the line's digit height (5 sizes ±6%), at quarter-pixel offsets and ±1 px, against the crop scaled between its background and its brightest ink. The best has to beat the next digit by 1.25x, else '?'. Colour and size of the text do not matter (white, gold, blue, yellow-green; table, window, tooltip). Separators are told by the column: `parse_number` knows which columns have one decimal.
- **Items are told by their name block in the purchase window** (`names/<name without ':'>.png`, the icon and both lines, 795..1090 x 205..255), compared over ink pixels only (`ink_distance`, the window is translucent): the same item reads 0.0, the closest two items (Tremor [12-15] and [16-19]) 4.6. Accept under 2.5 and 2 clear of the next. An item with no crop is named by the recogniser and its crop kept in `ignored/market/unknown_names/` to be filed by hand. "Destiny Crystallized Guardian/Destruction Stone" are cut to the same text; only the icon tells them apart.
- **Gem rows likewise** by `gems/<Lv. N Kind Gem>.png` (the name text of a row, 612..762); every row's nearest crop must be the gem searched.
- **Text that is read** (accessory names, the trades line, time left) goes through the recogniser at up to four margins until two reads agree (`read_box`). It is stored as read; nothing depends on it.

## What a pass reads

- **Materials**: Tier 4, Enhancement Material > "Honing Materials" (11 items) and "Additional Honing Materials" (21), every page ("1/2" under the list, read by the font). Each row: Buy opens the purchase window: avg day price, recent price, available quantity, and the price tiers (four shown, a notch down shows the fifth; the list holds five, which sum to the available quantity). Hovering the avg day price shows "Average price for the last 5 days", today first. An item with no trades today ("-") has no hover. 32 items in about 4 min.
- **Gems**: Auction House, class All, the search box with "Lv. N Doomfire Gem" (damage) / "Lv. N Blazing Gem" (cooldown), N from 10 to 5, Enter; the first page (10), sorted by buy-now. T4 is Doomfire and Blazing; Annihilation and Crimson Flame are T3.
- **Accessories**: Advanced Search, Reset, then category, class All, Ancient, Tier 4 and two "Refinement Bonus" rows with an option each, min = max at one level. Groups: necklace dps (Additional Damage, Outgoing Damage Increase), necklace support (Brand Power, Increased Serenade/Piety/Harmony Meter Gain), earrings dps (Atk. Power %, Weapon Power %), ring dps (Crit Rate, Crit Damage), ring support (Ally Atk. Power Enhancement Effect, Ally Damage Enhancement Effect); support earrings have nothing of interest. Level pairs high/high, mid/mid, high/mid, mid/high, high/low, low/high: 30 searches. The values (low/mid/high) are what the min/max lists offer for Ancient T4, in `GROUPS`. A group's settings stay between searches; only the levels change, after a check of the rest.
- **Search results** are taken when they differ from the search before (a search that did not go through leaves the old rows), when buy-now ascends (else the header is clicked), and nothing only after two empty reads.

## Storage

`db.rs`: `scans` (one per pass), `materials` (name, raw name, bundle, avg day, recent, available), `depth` (tier 0-4, price, quantity), `history` (days ago 0-4, avg), `searches` (kind gem/accessory, item, category, gem level, level a/b, result count) and `listings` (rank, name, trades, quality, time left, min bid, buy now).

## Open

- Uploading scans to the Worker (D1) and the viewer on the site, with loa-buddy as fallback.
- Accessory listings record the two options searched, not the third option (it is only on the hover tooltip).
- The recogniser could be swapped for PP-OCRv5 mobile English (Apache-2.0, 7.8 MB ONNX): with a tight horizontal crop it read all 102 dataset numbers; it converts to `.rten` (`rten-convert --no-infer-shapes`), rten inference not verified.
- Other regions and other resolutions are not handled.
