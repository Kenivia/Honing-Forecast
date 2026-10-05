# Hover tooltips

How the item tooltip is read. The scanner reads it in `crates/scanner/src/tooltip`, as the last step of every scan. The research behind it was done in Python (`scripts/tooltips/tooltip_poc.py` and `video_report.py`), which still run and also cover the one thing the scanner does not do (matching icons: the tooltip's own and the chest rows').

## What the scanner does with it

Each frame: find the title bar, read the title, find the "Amount Stacked" number and the bind lines, work out which slot is hovered, and write the amount and tradability onto that slot. If the tooltip has a chest panel, its kind and contents are read too and stored as a chest (see "Chests").

- **Hover.** A tooltip that stays within 4 px (at 1080p) and whose title bar has text in the same columns is one hover, kept in the scanner state across frames (it survives 4 missed frames). The title check matters: tooltips pushed up by the screen bottom land in the same place for neighbouring slots, and without it two items' votes were mixed. Title, amount and tradability are voted on over the hover's frames.
- **Reading is deferred.** The scan only cuts the text lines out and sends them off (see "OCR runs elsewhere" in `Pipeline.md`); the votes are cast when the texts come back, which can be after the tooltip has gone. So the hover keeps what only its frame could tell (the bar, the slots beside it, where a chest goes), and a hover that is over stays in the state until its last texts are in.
- **How much is read.** Title, amount and chest are each read until three reads agree, with one read of each out at a time: the next is sent when the last is back, so a hover asks for no more than OCR can deliver. On top of that a title is read three times and then only when it looks different (the cursor moved over it), and amount and chest at most six times. With unlimited OCR this reads the recordings exactly as reading every frame did.
- **The last frame.** While a read is out, the newest frame's lines are held, and sent when the hover ends. A short hover is over before its first read is back, and its first frame is the one most likely to be half faded in.
- **Title.** The bar is split into text lines by hand and each line goes to the recogniser on its own, inverted to dark on white and resized to the 64 px line height the recogniser works at. The name is in the rarity colour and the "[X n]" count after it is white, so a coloured name is cropped to its coloured part and everything after it is skipped. This matters: with the count attached the recogniser returns "Pouch III [X" without the III, which is a different valid item. White (common) names keep the count and it is cut off as text. "(Bound)" is cut the same way.
- **Accepted titles** are the `title` values in `templates/items.json`, compiled into the crate. A read has to be at least 0.9 similar (normalised Levenshtein) and 0.05 clear of the next title, with identical numerals (digits and roman numerals, compared as whole words); an exact read always counts. A numeral with an unknown character in it ("?II") matches nothing. A hover's title is the canonical string from the table, or nothing.
- **Amount.** The number is the yellow run at the end of the "Amount Stacked" line. The recogniser doubles or drops a digit when the crop is one pixel wider or narrower, and which crop does it differs from number to number (65 read as 665 at one margin, 440 for 40 at another). So the number is read from two plain crops with different margins, and when they disagree a third crop built from the yellow colour alone breaks the tie; with no two agreeing the frame gives no amount. On all 1,690 tooltip frames of the two recordings this gave no wrong amount. Compression can take the yellow out of a thin last digit, so the line may run a little past the yellow as long as what follows is not white.
- **Tradability** is "Tradable" only when no red right-aligned line is found *and* no lines were merged into one block above the amounts. A cursor beyond its usual reach merges the bind line and "Untradable" into one block, and that used to read as tradable.
- **Hovered slot.** Among remembered slots on the active pages that hold this title's icon, in the column the tooltip sits against, level with or below the tooltip: the only one, else the one level with the tooltip, else the only one whose own count equals the tooltip's. Titles without an icon never resolve to a slot.
- **Result.** `hover` in the scanner state has the voted title, amount, tradability and slot. The slot gets `tooltip_amount` and `tradability`, which it keeps while it holds the same icon. The slot's own OCR'd count is left alone; the tooltip amount is the full number where the slot shows 9999 or is misread.

Colours are compared after brightness normalisation, so the constants in the Rust code are a few levels below the on-screen values quoted below. Pixel constants there are 1440p and scaled by the scanner's scale factor like everything else; the sizes in this note are the 1080p measurements, which are 3/4 of them.

`cargo run --release --bin tooltip_test` is the native harness: it runs the whole scanner over stills, or over a recording piped in as raw frames by `scripts/tooltips/dump_frames.py`, and prints each hover and every slot that got tooltip data.

Results: all nine tooltip stills read correctly (title, amount, bind kind; slot wherever the item has an icon). On the largest-cursor recording (1777 frames, H.264) a tooltip is found on 1160 frames in 134 hovers; 40 hovers get an accepted title, 27 of them a slot; 26 slots end up with tooltip data, and all 15 in roster storage match the annotated amounts and the known tradable list. The tooltip step takes about 13 ms a frame natively. It has not been run in the browser yet.

All sizes below are 1080p pixels and scale with the UI height. Colours are at the brightness of the 1080p examples; the 1440p example was taken at another setting and every colour is shifted a little, so production code should normalise brightness first.

## Chests

A chest is whatever its tooltip lists. Chests with one name come with different contents ("Honing Support Materials Selection Chest: Crucible" exists with two and with five options) and there are too many variants to keep a table of, so neither the title, the slot icon nor the slot's number is used. The title is kept for display only. The structs (`Chest`, `ChestKind`, `ChestContent`) live in the scanner crate and in TypeScript, not in core.

- **Kind** comes from the yellow phrase of the panel's first line, as in the proof of concept: a short phrase is "obtain all", one that runs to the end of the line is "chance", otherwise "select one". Only "all" has a one-line header, which is how the header is skipped.
- **Rows.** Text lines are taken from just right of the icon column. Lines less than 10 px apart (1080p) belong to one row; the last line of a row is the "xN" count and the ones before it the name. Rows end at the first gap over 22 px, because over a dark background the body seems to run on into the shortcut hints.
- **Reading.** Each line goes to the recogniser on its own with 4 px of margin across and 2 px down, never reaching left of the text start. That margin is tuned: a wider one lets the icon's edge in and the read collapses, and white padding around the line was worse too. "(Bound)" is cut off the name and kept as a flag. The count is only read for rows whose name is known.
- **Known rows only.** A name is matched with the same rules as a title; rows that match nothing are dropped. A row that is *nearly* a known title (0.85) but not accepted makes the whole frame cast no vote, since it is more likely a misread than another item.
- **Chests inside chests** are listed by name ("Destiny Destruction Stone Pouch II x1"). The body font draws roman numerals as bare strokes and the recogniser returns "?I", "Il", "VIl", so these rows are matched ignoring the numeral and given the closest numbered title. The tier can be wrong; the plan is to ask the user what is in them.
- **Storing.** A chest with no known row is not stored. Otherwise it is stored from the first frame that reads, because the browser only gets one or two frames of a hover; later frames of the same hover outvote it.
- **Which chest it is.** The tooltip's column gives the inventory, page and column. The exact slot is known when the tooltip is level with a slot and was not pushed up. A new read replaces the chest last seen in that slot, else one in the same column with the same kind, contents, tradability and amount (a missing amount or tradability matches anything), else it is added.
- **Level or pushed up.** A pushed-up tooltip's shortcut hints end at the screen bottom. The hints box is 80% black and is followed by comparing its sides with the background just outside it, which only works where that background is bright. Over a dark one (the storage NPC's dialogue bar) the test says nothing and alignment with a slot row decides alone.
- **The title bar's top** is found up to 5 px low when something is over its first rows; its bottom is not. The slot test counts the top back from the bottom using the two known bar heights.

Results, checked by eye against frames. Smaller-cursor recording: all 25 hovered chests that hold something known are stored with the right kind, contents, amount and tradability. Largest-cursor recording: 25 stored, all right apart from one inner chest's tier. Feeding every 10th frame (3 a second) gives 25 of 25 and 21 of 23 with nothing wrong; every 30th gives 17 and 12, again with nothing wrong, the rest were never on a sampled frame. In the browser, with every frame scanned, both recordings give all 25 of the earlier native run's chests, compared entry by entry, plus one of them a second time without its amount (Chromium; Firefox was only run before the brightness change, 25 of 25 on the smaller-cursor recording). Slot positions were only spot-checked.

Matching the row's 30 px icon was measured with the proof of concept's matcher and left out. Where the text names a known item the icon agreed on 812 of 817 rows (the 5 are a hover boundary), and it never passed for an unknown item. But it never rescued a row either: a cursor that hides the name hides the icon first, and tiers of one chest share an icon. It would only earn its place for non-English clients.

## What a tooltip looks like

Top to bottom:

- **Title bar.** Fully opaque, one flat colour (30, 34, 39), 307 px wide. 34 px tall with a one-line title, 52 px with two lines. The item name is in the rarity colour, followed by a white `[X n]` that can wrap onto the second line. Above the text is a clean strip of the bar colour, 11 rows. A single-item stack has no `[X n]` at all.
- **Body.** About 97% opaque over (14, 19, 20), so its pixels sit 0 to 8 levels above that colour depending on what is behind.
  - Item icon: 64 x 64, the native icon art, 7 px in from the title's left edge and 17 px below the title bar, over a rarity gradient. Rarity and "Item Tier n" text to its right.
  - Optional right-aligned white line ("Required Item Lv. 1640").
  - Bound items only: a white left-aligned "Bound to Character" / "Bound to Roster" line, then a red right-aligned "Untradable". Tradable items have a white right-aligned "Tradable" instead and no bind line.
  - Description lines in pale yellow (228, 204, 130). "Consumed before tradable items." is in the bright amount yellow.
  - "Amount Stacked: n" and "Total Amount Owned: n": white label, number in opaque (255, 213, 0), with a thousands separator.
  - Red restriction lines, then cyan acquisition sources.
- **Chest contents** (boxes and pouches): a pure black (0, 0, 0) panel inset 5 px in the body, placed after the restriction lines. A header with one yellow phrase, then one row per item: a 30 px icon square 18 px in from the title's left edge, the name in rarity colour, and a white "xN" under the name.
  The header comes in three kinds, told apart by the yellow phrase alone: "Obtain [all] of the following items." (one short word), "You can [select and obtain 1] of the following items." (white text after it on the same line), "There is a [chance that you can obtain one] of the following items." (yellow runs to the end of the line).
- **Shortcut hints.** A separate, narrower box below, 80% black. Its lines depend on where the item is; nothing useful is in it.

## Placement

The tooltip sits against the hovered slot's column: its title starts 8 px right of the slot's right edge, or ends 12 px left of the slot's left edge when it flips to the left. Its top equals the slot's top, unless the screen bottom would cut it off, in which case it is pushed up and only the column is known. Most examples are pushed up.

## The cursor

Screenshots do not include the cursor, the recording does. It is customisable, so it is never a matching target. It sits over the hovered slot and, when the tooltip is on the slot's right, pokes into the tooltip's left edge. At the largest size it reaches about 90 px in: the left of the title bar, most of the icon when the tooltip is level with the slot, and the start of the bind and amount labels are covered. The hovered slot itself and its hover glow (a brighter inner ring about 6 px wide) are usually covered and must not be relied on. Whatever the cursor touches merges with it when the body is split into lines, so the header and amount lines are read only from columns 100 px and beyond.

Tooltips also fade in over a few frames, during which no colour matches.

## Approach that worked in the proof of concept

1. **Is there a tooltip**: find rows with a flat run of the title colour that ends at a sharp right edge, at least half the title width long, on several consecutive rows. The tooltip's left edge is the run's right end minus 307. Confirm with the right margin below it: title colour for 25 to 80 rows, then the body colour. On lossless input "flat" can mean identical neighbouring pixels with no colour constant at all; the colour checks that reject other windows' grey bars are that the title is blue-grey and the body is the expected fraction of it.
2. **What is hovered**: compare the 64 px icon at its fixed offset against the raw icon art in `templates/Icons`, using the art's own alpha as the mask and scoring only the closest 75% of pixels so a cursor over part of it does not matter. The existing slot templates do not work here because the tooltip draws the icon over a different background. The slot follows from placement: the column from the tooltip's edge, the row from its top when it was not pushed up, otherwise the slot in that column that was already known to hold that item.
3. **Where the text is**: split the body into text lines by row projection, using only the columns the cursor cannot reach. An amount line is a short line outside the black panel whose yellow reaches the line's right end and has white just before it; stacked and total are the first such pair one line apart. Untradable is a red right-aligned line above the amounts. Bound to Character or Roster is the line above that, judged by where it ends. Chest rows are the tall lines inside the black panel.

Results on stills: all nine tooltip stills parse (1080p and 1440p, one and two line titles, both sides, all three chest kinds), and none of the 32 screenshots without a tooltip trigger it.

Results on the largest-cursor recording (59 s, 143 hovers, roster storage page 2 annotated): 27 of the 28 distinct amounts in roster storage were read, none wrongly; the 28th was never detected as hovered. Tradable against roster bound agreed with the tooltip text for every item read. One hover was lost because the cursor covered the top-left of the title and the bar was located 4 px off.

Icon identification is the weak part with a large cursor: when the tooltip is level with the slot the cursor covers most of the 64 px icon, and several template items came back unidentified. Scoring only the best 75% of pixels is not enough.

**Reading the title is the better identifier.** The title bar is OCR'd, the `[X n]` count and "(Bound)" are dropped, and the rest is fuzzy-matched against a table of in-game names (`ITEM_TITLES` in the proof of concept, which only holds the names seen so far). On that recording a title was read on 136 of 143 hovers, every hover of a template item was identified (17 by title, 16 by icon, the two agreeing wherever both answered), and the title recovered both items the icon lost to the cursor. Three rules came out of it:

- Similarity has to be at least 0.92. "Destiny Destruction Stone Pouch" is 0.86 similar to "Destiny Destruction Stone".
- Digits must match exactly. "Level 4" is 0.96 similar to "Level 3".
- Every title line is centred in the bar. If a line's left and right gaps differ, something is over it, and a covered start can turn one valid name into another ("Great Destiny Leapstone" into "Destiny Leapstone"). Frames with centred lines outvote the rest. The test is conservative: most off-centre frames still read correctly.

The OCR used inserts stray letters now and then ("Destiny D Destruction Stone"), which the fuzzy match absorbs. The title count is not reliable and is absent on single items.

## Known limits

- **A tooltip that is only up for a few frames of a compressed recording sits at the edge of several colour tests**, and which side it falls on moves with the brightness estimate, because the brightness lookup rounds neighbouring levels together at places that shift with it. Two were found so far. The title bar's blue-grey test (blue minus red) measured exactly its old limit of 4 on two hovers of 5 and 8 frames and failed in bands about every two brightness settings; the limit is now 2 (other windows' grey bars measure -1 to 1). And on one 9-frame hover the yellow word of the chest header ("all") has a single pixel that counts as yellow at brightness 59 and none from 59.5 up, so natively that chest is not read at the true brightness of 60. Not seen on tooltips that stay up.
- **A chest can be listed twice**, once without its amount, when a brief hover's read lands after a longer one's.

- **A wrong slot is possible when the hovered slot was never seen.** The cursor hides the hovered slot. If that slot was never recognised and another stack of the same item sits lower in the same column, that other stack is the only candidate and gets the tooltip's data.
- **Two stacks of one item in a column** resolve only if the tooltip is level with one of them or the slot's own count was read correctly. The two leapstone selection chests in the recording stayed unresolved for this reason.
- **Chests that share an icon** (II and III of the same pouch) are one slot template. Only the title tells them apart, and the slot remembers the icon, not the title.
- **The tolerant title-colour match gives an occasional false tooltip** on a screenshot without one (one of the storage stills). Nothing is written, because no title matches.
- **The recordings are H.264**, which breaks exact colours. The scanner matches the title colour with a tolerance of 8 either way. Whether live screen-share frames behave better has not been checked.
- **Character against Roster bound** is told apart only by how far the bind line extends (English text). When the cursor hides the line the frame casts no vote.
- **Titles are English only.**
- The recogniser returns `?` for characters it does not know, the bracket of the count among them.
- The tooltip's own icon and the chest rows' icons are not read by the scanner.
- **A pushed-up chest tooltip over a dark background can get the wrong row** if its top happens to line up with a slot row (about 1 in 15), and would then replace the chest stored for that slot.
- **Two stacks of the same chest in one column** with the same amount, or one read before its amount was, are stored as one.
- **A chest row hidden by the cursor is silently missing** from the contents when the rest of the frame reads.
- **Tooltip reading costs OCR calls**: a chest frame is several times slower than a frame without a tooltip in the browser (about 0.16 s against 0.6 to 0.7 s in headless Chromium).
