# Currencies

Silver, gold and Destiny Shards are read apart from the slots, in `crates/scanner/src/currency.rs`, as the last step of every scanned frame. The page shows them in a bar above the slot grid (`CurrencyBar.vue`) and the manifest adds them to its cells (`Manifest.md`).

## Where they are read

A total is the number just left of its currency's icon. The icon is template-matched inside a small bound placed from an origin, and the number is cut from the columns left of it up to the first wide gap. Three places, each with its own origin:

| Place | Origin | Currencies | Icon templates |
| --- | --- | --- | --- |
| Inventory | the character inventory's window origin, lone or in storage | silver, gold | `Silver coin`, `Gold coin` |
| Currency window | the `Currency` anchor's root | silver, gold, shards | the same, and `Destiny shard` |
| Bar at the top of the screen | the game area's top left (`ScreenInfo.corner`) | all three, anywhere along it | `Bar silver coin`, `Bar gold coin`, `Bar destiny shard` |

- **Bands.** Silver is always roster bound and shards character bound. Gold comes as tradable, roster bound, character bound and deposit, and only its tooltip says how much of each. Deposit is gold bought on the currency exchange that is held back for a few days, and counts as tradable.
- **The currency window is assumed to be where the game first puts it**, and its rows where they start, since it can be moved and its sections collapsed. Its anchor (`Basic currency header`, `Currency window tab`) is only searched at that spot, and only while the storage layout is closed. The window is see-through, so neither variant has a brightness curve. A window that was moved is simply not read.
- **The bar is hidden while the storage layout is open** and is not looked at then. The user picks which currencies it shows and in what order, so its icons are searched along its whole length, at the place they were last found first.
- **The game area's corner, not the 16:9 block.** The HUD and the currency window hug the left edge of what the game renders, which at forced 21:9 or on a native ultrawide is left of the UI block the windows are placed in. `ScreenInfo.corner` is that edge, and `Bound::Corner` places from it.
- **An icon has to show its contrast.** Template matching ignores contrast, and an icon showing faintly through a near-opaque tooltip still scored 0.978: the gold tooltip hovered from the bar covers the shards, and their number read as garbage. A match whose grey spread is under 0.4 of its template's is dropped (`MIN_CONTRAST`).
- **And its tint.** Matching is on grey, and the gold coin's template scores 0.92 on the silver coin. So the share of red over blue (`tint`: 0.35 for gold, 0.04 for silver, at either scale) has to be within 0.15 of the template's. A match failing either check is not remembered as where the icon was: once the bar's gold was remembered on the silver coin, it was looked for there from then on and its tooltip never found.
- **The coin templates include the rim and 2 px around it** (24 px in the inventory and the window, 20 px on the bar). Cut to the face alone, they scored 0.86 to 0.90 where the game draws the UI at 1080 lines, as it does with forced 21:9 on a 16:9 1440p screen (letterboxed to 2560x1080), and the bar's silver was never found. With the rim: 0.96 to 0.99. The number is looked for from 2 px (1440p) left of the template.
- **Text is what is 40 brighter than the median of its strip** (`INK_ABOVE`), not a fixed level: the window is see-through, and at 1080 lines the thin "1" of a dim green "15,204,181" peaks under 90, so the number was cut to "81".
- Nothing here needs the brightness estimate: icons are matched as captured, text is told from its own strip, and lines go to OCR with the estimate, or 50 when there is none. So the bar alone reads with no window open.

## Gold's tooltip

- **It sits at a fixed offset from the gold icon it was hovered from**, whatever the number's length (`PlaceSpec.tooltip`): 40 px right or 548 px left of the inventory's coin, 52 px right of the currency window's and 59 px right of the bar's. The window and the bar always have it on the right. A tooltip pushed up by the bottom of the screen has its title at 520 (UI, from the game's top) every time. So its title (`Gold tooltip title`) is only looked for at those spots, with 10 px of slack, and has to be yellow: the white line "Bound Gold cannot be used..." matches the same words.
- **The cursor is over the coin while the tooltip is up**, so the coin cannot be found then. The spots are taken from where the coin should be (inventory, window) or where it was last found (bar). The stills have no cursor, and this was only seen on the capture.
- **Six lines are read** at fixed offsets from the title: the "Gold Owned", "Roster-bound Gold owned" and "Character-bound Gold" amounts, each followed by "/99,999,999,999", and "D+1" to "D+3" of the deposits. A band is the digits before the slash; without one, the cap is cut off the end. A deposit line is read whole and taken after its colon: a lone "0" read badly.
- **Tradable is "Gold Owned" plus the three deposits.** On the stills: 2,998,587 + 0 + 0 + 20,000 tradable, 5,000 roster bound, 400,000 character bound, which is the 3,423,587 shown.

## Votes

- **Everything is a `Field`**: a currency's total, pooled over the three places, or one tooltip line. Each read is one line, voted on by its value; three agreeing settle it for the session and stop its reading (the scan is a snapshot).
- **Three reads go out at once per field**, each cut with the next of six margins (`MARGINS`), since the recogniser doubles or drops a digit at some widths; the next three only once all are back. A tooltip is often up for under half a second, and its long lines take that long to come back: with one read out at a time a hover gave one vote, and gold took three hovers. A crop seen before is the same OCR id and does not vote again, so a still screen gets six distinct reads at most.
- **Six reads without three agreeing is an error**, shown as "Its reads disagree". So is a tooltip whose parts do not add up to a total read elsewhere.
- **A still screen is scanned on** while something unsettled is on it or a read is out (`currency_unfinished`, in `worth_scanning`), or the reads would stop at the quiet-scan limit.
- Gold is good once all six lines settled; it needs a hover while none was seen. Silver and shards are good once their total settled.

## Results

The scan result's `currencies` lists each with its status (the slots' `SlotStatus`), a reason, its value and its bands (char bound, roster bound, tradable). They count as missing in the manifest until good. The capture digest includes them, so captures made before them differ from scan 0.

## Measured (2026-10-17)

- The twelve 1440p stills in `scripts/brightness/1440p raw/silver gold shard` (setting 50, 16:9 and forced 21:9, window in its default place and moved, tooltip left and right of the inventory, from the bar and from the window) all read what `note.txt` says, wherever the currency is on screen. The moved window is not read, as intended.
- The capture there was recorded with "Forced 21:9" off. Replayed with `capture_replay --forced-21-9 --reread --no-edits`, all three and gold's split are read.
- Template scores on those stills: 0.997 to 1.0 for the right icon; other icons of the bar 0.85 at most; silver against gold 0.87.
- The three captures of 2026-10-17 08:27 to 08:35 there (forced 21:9 on, so 1080 lines): replayed with `--reread --no-edits`, everything is read, gold on its first hover. The one without gold on the bar and never hovered stays at "needs a hover", as it should.
- Not tried: other resolutions, brightness settings, a lossy recording, the inventory's tooltip when the inventory is high enough not to push it up.

## Tools

- `print_statuses` (`tooltip_test`, `capture_replay`) prints each currency and every field's votes.
- `tooltip_test` takes forced 21:9 from a still's name ("21 by 9"), as `anchor_locate` does; `capture_replay --forced-21-9` overrides a capture's start record.
