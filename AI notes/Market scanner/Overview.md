# Market scanner

A native program that drives the game's Market, Auction House and currency exchange with synthetic input and reads them off the screen. It is the site's own price source (loa-buddy stays the fallback), and it collects what nobody offers (gems; accessories, paused). It writes SQLite and uploads each pass to the site's Worker; the site prices each material by the newer of our lowest listing and loa-buddy's (`/prices`, `Frontend/External services.md`).

Two regions, both scanned each round: NAE on Kenicards (Thaemine) and EUC on Kenimommy (Arcturus) on this PC, each parked in its Stronghold. How to drive the game, how it draws and its life cycle are in `Lost Ark client.md`; how to develop and test this is in `Developing and testing.md`; other PCs, the relay and Discord in `Remote.md`. What failed is in `Further learnings/Overview tried.md`.

Code: `crates/market` (`hf-market`), Windows only. Each module starts with a comment saying what it does; the bins' switches are in `--help`.

## Layout

- **Built into every exe** (`assets.rs`, about 20 MB): the reference crops (`refs/`), the purchase-window name blocks (`names/`), gem row names (`gems/`), the item art (`items/`), the PP-OCRv5 model and the digit font. Each template folder is overridden file by file from the profile folder (a file's key is its name without `@x_y` and extension): the crate's own folders in dev, so a new crop works without a rebuild, and `profile/` beside the exe in the portable layout.
- **The data folder** holds logs, the database, `debug/`, `strips/`, `upload.json` and the relay configs: `ignored/market/` in dev, `data/` beside the exe otherwise.
- **The text model** is not tracked: `rec.onnx` and `dict.txt` from Hugging Face `monkt/paddleocr-onnx` (`languages/english`) go in `ignored/market/models/en_PP-OCRv5_mobile_rec/` before building.

## Running

- `pnpm market-scan [--gap N] [--once] [--quick] [--strips]` runs `market_scan`: a pass of each region (one upload each), then a gap after it ends, until F10 is held or Ctrl+C. F10 stops at the next action and still stores and uploads what the pass read (`run.rs`). The default sections are materials, gems and exchange; `--only accessories` runs the paused accessories, whose code and crops are kept. `--upload-only` sends passes not sent yet.
- `pnpm market-agent` runs `market_agent --dev`: the same passes on a schedule, controlled through the relay or its console (`Remote.md`).
- The game must be logged in, full screen at 1920x1080, with the PC left alone meanwhile (the game is kept in front).
- A pass takes about 7 min of materials, 1.5 of gems, 10 s of exchange (8 min of accessories). `--quick` runs every path in a few minutes.
- Uploading needs `upload.json` (`{"url", "token"}`); without it nothing is sent.

## Getting around

- **Steps wait for the screen, not a fixed time** (`ui::wait`, `ui::act`): after a click the screen is polled every 0.25 s until it shows what the step should bring (a crop, a title, the next page number), up to a few seconds. A click that is harmless to repeat is repeated. `act` can delay before each try, for screens that show before they take clicks. Lists whose contents come after their title get a short settle.
- **Units of work are tried again from a reset UI** (`ui::unit`): a Market section, each gem or accessory search, the exchange, a region switch. A failed try keeps a screenshot (`ui::fail` takes it at the failing step, before any cleanup key), closes the Market and the shop (`ui::reset`), and waits 5, 20, then 60 s. After four failures the unit is left out of the pass, not stored as empty. Every failed try that has another after it is sent as an `error` event. Reads that degrade without failing (the 5-day hover, `steady`) are only logged. The user wants this general, with no per-screen special cases.
- **State is checked by reference crops, not text** (`refs/<name>@x_y.png`, `ui::looks`): max-channel grey, optionally with alpha so translucent parts are not compared, accepted under a mean difference of 14 at the best of five 1 px shifts. A match reads about 0.0, one row off about 32.
- **Dropdowns are picked and checked, and picked again** (`pick`): the first pick after Alt+Y often does not take. Lists that scroll go to the top first, then down a known number of rows (one notch is one row).
- **A fresh Market sorts by lowest price**, which reorders while pages are read, so each list is sorted by Rank first (`rank_sorted`).
- **Switching region** (`region.rs`; its header lists the steps). Each step is checked by a crop. The server row and the character's slot are clicked where their crops were taken, so another machine needs only its own `char_<region>` and `server_<region>` crops. Enter on server select ignores clicks for about 3.5 s after the region prompt (`act` waits 3 s). The Stronghold wait also takes the attendance window and closes it. A switch that fails every try panics the round, so the agent restarts the game instead of going on from a strange screen. In-game "Switch Servers" lists only the current region's servers.
- **The shop** (F9; its first open can take up to 60 s): only Currency Exchange and Leave are pressed. The mouse waits at (1900, 600), away from the bottom bar's tooltips. The tables draw in after the title, so the first read waits 1.5 s.
- **Never buy**: the Market's Buy is pressed only when the Market header and the Buy button both match their crops (the Auction House's Bid/Buy sits on the same spot), only to open the purchase window, which is closed by its X. Enter is pressed only in the Auction House search box.
- **Something covering a value**: a value that does not parse is read again from a new screenshot after 2.5 s, up to four times (`steady`).

## Reading

- **Numbers by the game's font, not a recogniser** (`glyph.rs`). The line is split at empty columns. A glyph under half the line's height is a separator or a dash. Every other glyph is compared with each digit drawn in ExuberanceF at the line's digit height, at quarter-pixel offsets and ±1 px, against the crop scaled between its background and its brightest ink. The best digit has to beat the next by 1.25x, else '?'. Colour and size do not matter. `parse_number` knows which columns have a decimal.
- **Items by their name block** in the purchase window (`names/`, the icon and both lines), compared over ink pixels only (`ink_distance`, the window is translucent): accepted under 2.5 and 2 clear of the next.
- **Else by their icon** (`icons.rs`): the game's art for every item listable in the material categories (`items/`, from `market_items.py`), by normalised cross-correlation over the art's opaque pixels at ±1 px. Icons close to the best are kept, and the name read decides between them (its numerals must be the item's; a name cut short is compared with as much as was read). An item nothing names is kept in `ignored/market/unknown_names/`, stored under the known name its read matches at 90% (else as read), and sent as an `error` event with the screen, once per read (`report_unknown`).
- **The name block** also gives the bundle ("[Sold in bundles of N units]"); a wrapped name's second line ("[19-20]") is not one.
- **Gem rows** by `gems/<Lv. N Kind Gem>.png`: every row's nearest crop must be the gem searched.
- **Text by PP-OCRv5 mobile English** (`pp.rs`, rten): grey, cut to the largest run of ink columns (a table's divider is not text), padded, inverted, 48 px high, CTC. It writes no spaces, so names are compared without them.
- **The 5-day hover's rows are found by their coins**: a "-" day's row is shorter.

## What a pass reads

- **Materials** (`market.rs`): Enhancement Material > All at Tier 4 and Trade Skill > All, every page (the "1/4" indicator, read by the font). Per row, Buy opens the purchase window: average day price, recent price, available quantity and the five price tiers. Hovering the average shows the last 5 days, today first; an item with no trades today ("-") has no hover.
- **Gems** (`auction.rs`): the Auction House search box with "Lv. N Doomfire Gem" / "Lv. N Blazing Gem", N from 10 to 5, the first page sorted by buy-now. EUC's Auction House is thin for high gems, so short lists there are real.
- **Accessories** (paused): Advanced Search for ancient T4 with two "Refinement Bonus" options at one level each, 5 groups x 6 level pairs (`GROUPS`). The third option is only on the hover and is not recorded.
- **Currency exchange** (`exchange.rs`): the Information page's Royal Crystal sale offers, Crystal purchase offers and recent prices, by the font ('.' is the shop's thousands separator). Empty rows are skipped, not the end.
- **Search results** are taken when they differ from the search before (a search that did not go through leaves the old rows) and buy-now ascends. Nothing is taken only after four empty reads 2.5 s apart, since a slow answer would show up in the next search.

## Upload and storage

- After each pass, `upload::payload` builds one JSON: `{version, region, machine, started, finished, sections, points}`. `sections` holds everything read, by kind, as the structs serialise, each read with `at` (UTC). A new kind of data is a new section; the Worker does not look inside. `points` are `[key, day, value]` for the daily history (`material/<name>/avg|lowest|recent`, `gem/...`, `exchange/...`).
- **The 5-day averages are of complete days**, rolling over at 00:00 UTC, so the first is dated the day before the scan (`Lost Ark client.md`, "Days").
- A stopped pass uploads what it read; one that read nothing is not sent. The payload is kept in `scans.payload` and sent after the pass; a failed send is retried after the next pass (`send_pending`), and a pass sent twice is stored once.
- `db.rs`: `scans`, `materials`, `depth`, `history`, `searches`, `listings`. The exchange is only in the payload.

## Open

- The site reading more of the uploads: prices other than the lowest listing, as a toggle on the market page. Gems are shown (lowest buy-now only).
- Recovering from a lost connection while the agent is not running: `ui::reset` only closes the scanner's own windows.
- Other resolutions are not handled.
