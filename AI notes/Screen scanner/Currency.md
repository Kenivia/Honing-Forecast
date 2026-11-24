# Currencies

Silver, gold and Destiny Shards are read apart from the slots, in `crates/scanner/src/currency.rs`, as the last step of every scanned frame. The page shows them in a bar above the slot grid, in the same card (`CurrencyBar.vue`) and the manifest adds them to its cells (`Manifest.md`).

Things that failed here are in `Currency tried.md`; read it only when trying something new.

## Where they are read

A total is the number just left of its currency's icon. The icon is template-matched inside a small bound placed from an origin, and the number is cut from the columns left of it up to the first wide gap. Three places, each with its own origin:

| Place | Origin | Currencies | Icon templates |
| --- | --- | --- | --- |
| Inventory | the character inventory's window origin, lone or in storage | silver, gold | `Silver coin`, `Gold coin` |
| Currency window | the `Currency` anchor's root | silver, gold, shards | the same, and `Destiny shard` |
| Bar at the top of the screen | the game area's top left (`ScreenInfo.corner`) | all three, anywhere along it | `Bar silver coin`, `Bar gold coin`, `Bar destiny shard` |

- **Bands.** Silver is always roster bound and shards character bound. Gold comes as tradable, roster bound, character bound and deposit, and only its tooltip says how much of each. Deposit is gold bought on the currency exchange that is held back for a few days, and counts as tradable.
- **The currency window is assumed to be where the game first puts it**, and its rows where they start, since it can be moved and its sections collapsed. Its anchor (`Basic currency header`, `Currency window tab`) is only searched at that spot, and only while the storage layout is closed. The window is see-through, so neither variant has a brightness curve. A moved window is simply not read.
- **The bar is hidden while the storage layout is open** and is not looked at then. The user picks which currencies it shows and in what order, so its icons are searched along its whole length, at the place they were last found first.
- **The bar is drawn at the HUD Size setting** (Gameplay > Display Settings: 80, 90, 100, 105 or 110%), scaled from the game area's corner, and its templates were cut at 90%. Its icons are matched at each of the five sizes over the UI height (`HUD_SCALES`, as `find_template_at` with a larger or smaller UI height), the best that passes, until one is found; that size (`Currencies.hud`) is then kept for the session, and the number beside an icon is cut at it too. A HUD changed mid-session is not followed. The gold tooltip from the bar is not scaled by it: its title moves by the coin's growth (17 in 1440p units from 80 to 110%), inside the slack below. The inventory and the currency window do not change with it.
- **Where the bar is small the coins score low**: forced 21:9 on a 1080p screen is a UI of 810 lines, and at 80% the coins are about 10 px; the right coin scored 0.84 to 0.89 there, and the other coin often better. The bar coins' limit is 0.8 (`anchors.py confidence`), and a match that fails the contrast or tint check below (the other coin) is searched past, left and right of it (`find_shown`).
- **The game area's corner, not the 16:9 block.** The HUD and the currency window hug the left edge of what the game renders, which at forced 21:9 or on a native ultrawide is left of the UI block the windows are placed in. `ScreenInfo.corner` is that edge, and `Bound::Corner` places from it.
- **An icon has to show its contrast.** Template matching ignores contrast, so an icon showing faintly through a near-opaque tooltip still scores high (the gold tooltip hovered from the bar covers the shards). A match whose grey spread is well under its template's is dropped (`MIN_CONTRAST`).
- **And its tint.** Matching is on grey, so the gold coin's template also scores high on the silver coin. The share of red over blue (`tint`) has to be close to the template's (`MAX_TINT_SHIFT`). A match failing either check is not remembered as where the icon was, or the icon would be looked for there from then on.
- **The coin templates include the rim and a margin around it.** The face alone does not match where the game draws the UI at 1080 lines (forced 21:9 on a 16:9 1440p screen is letterboxed to 2560x1080). The number is looked for from just left of the template.
- **Text is what is a fixed step brighter than the median of its strip** (`INK_ABOVE`), not a fixed level: the window is see-through, and a thin dim digit can be dim.
- Nothing here needs the brightness estimate: icons are matched as captured, text is told from its own strip, and lines go to OCR with the estimate, or 50 when there is none. So the bar alone reads with no window open.

## Gold's tooltip

- **It sits at an offset from the gold icon it was hovered from** (`PlaceSpec.tooltip`): right or left of the inventory's coin, right of the window's and the bar's (they never flip). Hovering the number brings it up too, and the cursor does not move it. Flipped left, its right edge is just left of the number, so it moves with the number's length: the offsets are for a seven digit number, and the title is looked for `LONGER_NUMBER` further left to `SHORTER_NUMBER` further right (391,947 put it 20 right of the offset). A tooltip pushed up by the bottom of the screen ends at the same height every time, so its title is at `PUSHED_UP_TITLE`, or a line lower when the tooltip is a line shorter (below). So its title (`Gold tooltip title`) is only looked for at those spots, with some slack, and has to be yellow: the white line "Bound Gold cannot be used..." matches the same words. Its limit is 0.8: at a UI of 810 lines it scores 0.84 (0.95 at 1080).
- **A short owned amount fits beside "Gold Owned:"** ("Gold Owned: 333,947/99,999,999,999" on one line; 2,998,587 and 3,361,001 wrap onto their own line), and the tooltip is then a line (`ONE_LINE_SHORTER`) shorter. Which it is goes by how far the line of that label runs (`LABEL_ONLY`); on one line every line after the title is read that much higher, and the owned amount is taken from the digits just before the slash.
- **The cursor is over the coin while the tooltip is up**, so the coin cannot be found then. The spots are taken from where the coin should be (inventory, window) or where it was last found (bar). Stills have no cursor, so only captures show this.
- **Six lines are read** at fixed offsets from the title: the "Gold Owned", "Roster-bound Gold owned" and "Character-bound Gold" amounts, each followed by "/99,999,999,999", and "D+1" to "D+3" of the deposits. A band is what is before the slash; a read without one does not vote. A deposit line is read whole and taken after its colon, since a lone "0" reads badly.
- **Tradable is "Gold Owned" plus the three deposits** (the total shown is that plus the roster- and character-bound lines).

## Votes

- **Everything is a `Field`**: a currency's total, pooled over the three places, or one tooltip line. Each read is one line, voted on by its value; three agreeing settle it for the session and stop its reading (the scan is a snapshot).
- **Three reads go out at once per field**, each cut with the next of six margins (`MARGINS`), since the recogniser drops a first or last digit at some widths; the next three only once all are back. A tooltip is often up for under half a second, about as long as its long lines take to come back. A crop seen before is the same OCR id and does not vote again, so a still screen gets six distinct reads at most.
- **The margins are small, and a left and right pair**: wide empty space makes the recogniser drop edge digits. Scaled down, two margins can round to the same pixels, which would be the same crop; a repeat is widened by a pixel on the right.
- **A read whose digit groups are broken does not vote** (`grouped`): every group after the first must be a multiple of three long and none empty. That is what a dropped first or last digit leaves ("15,275,48", ",138,638"); a missed separator ("3.423587") still counts, and spaces are dropped first (the recogniser spaces out "11": "3,1 13,003"). A dropped first digit that leaves whole groups ("5,275,481") cannot be told and is outvoted. A read that does not vote still uses up its margin.
- **Six reads without three agreeing is an error**, shown as "Its reads disagree".
- **A still screen is scanned on** while something unsettled is on it or a read is out (`currency_unfinished`, in `worth_scanning`), or the reads would stop at the quiet-scan limit.
- **Gold is its tooltip's sum** once all six lines settled, whatever total was read: a total can be half covered (the tooltip of a slot over the inventory's, or the cursor and the gold tooltip itself), reads as its visible end ("428") and settles. From then on gold is no longer looked for. Before that it needs a hover, and a total that disagrees is an error until the tooltip is read. Silver and shards are good once their total settled.

## Results

The scan result's `currencies` lists each with its status (the slots' `SlotStatus`), a reason, its value and its bands (char bound, roster bound, tradable). They count as missing in the manifest until good.

## Set by hand

- Clicking or hovering a currency in the bar shows it in the dashboard (`CurrencyDashboard.vue`), locked and followed like a slot. It has a Read and a "Correct it" column per band: all three for gold, roster bound for silver, char bound for shards.
- **Save edit** puts the bands in `currency_edits` (`ScanStore.ts`); **Remove edit** drops them. The scanner is not told and keeps reading. `shown_currency` is what the bar and the manifest use: an edited currency is good with its edit's bands, whatever was read.

## Tools

- `print_statuses` (`tooltip_test`, `capture_replay`) prints each currency and every field's votes.
- `pnpm scanner-check` holds the currencies of the stills and captures in `ignored/scanner_tests` to known bands (`Live game.md`): every HUD size at 16:9 and forced 21:9 (`stills/hud_size`), live captures at HUD 110%, and `honing-forecast-2026-11-24T07-22-59-049Z.hfcap` (a user's, HUD 100%, a one-line owned amount and a 6-digit total).
- The native bins detect forced 21:9 on stills like the page does; `capture_replay` takes it from the capture's start record.
