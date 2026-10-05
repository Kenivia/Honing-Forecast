# Hover tooltips

How the item tooltip is read. The scanner reads it in `crates/scanner/src/tooltip`, as the last step of every scan. The research behind it was done in Python (`scripts/tooltips/tooltip_poc.py` and `video_report.py`), which still run and also cover things the scanner does not do (matching the tooltip's icon, chest contents).

## What the scanner does with it

Each frame: find the title bar, read the title, find the "Amount Stacked" number and the bind lines, work out which slot is hovered, and write the amount and tradability onto that slot.

- **Hover.** A tooltip that stays within 4 px (at 1080p) is one hover, kept in the scanner state across frames (it survives 4 missed frames). Title, amount and tradability are voted on over the hover's frames. OCR stops for a hover once three reads agree.
- **Title.** The bar is split into text lines by hand and each line goes to the recogniser on its own, inverted to dark on white and resized to the 64 px line height the recogniser works at. The name is in the rarity colour and the "[X n]" count after it is white, so a coloured name is cropped to its coloured part and everything after it is skipped. This matters: with the count attached the recogniser returns "Pouch III [X" without the III, which is a different valid item. White (common) names keep the count and it is cut off as text. "(Bound)" is cut the same way.
- **Accepted titles** are the `title` values in `templates/items.json`, compiled into the crate. A read has to be at least 0.9 similar (normalised Levenshtein) and 0.05 clear of the next title, with identical digits; an exact read always counts, since "Pouch II" and "Pouch III" are closer than that margin. A hover's title is the canonical string from the table, or nothing.
- **Hovered slot.** Among remembered slots on the active pages that hold this title's icon, in the column the tooltip sits against, level with or below the tooltip: the only one, else the one level with the tooltip, else the only one whose own count equals the tooltip's. Titles without an icon never resolve to a slot.
- **Result.** `hover` in the scanner state has the voted title, amount, tradability and slot. The slot gets `tooltip_amount` and `tradability`, which it keeps while it holds the same icon. The slot's own OCR'd count is left alone; the tooltip amount is the full number where the slot shows 9999 or is misread.

Colours are compared after brightness normalisation, so the constants in the Rust code are a few levels below the on-screen values quoted below. Pixel constants there are 1440p and scaled by the scanner's scale factor like everything else; the sizes in this note are the 1080p measurements, which are 3/4 of them.

`cargo run --release --bin tooltip_test` is the native harness: it runs the whole scanner over stills, or over a recording piped in as raw frames by `scripts/tooltips/dump_frames.py`, and prints each hover and every slot that got tooltip data.

Results: all nine tooltip stills read correctly (title, amount, bind kind; slot wherever the item has an icon). On the largest-cursor recording (1777 frames, H.264) a tooltip is found on 1160 frames in 134 hovers; 40 hovers get an accepted title, 27 of them a slot; 26 slots end up with tooltip data, and all 15 in roster storage match the annotated amounts and the known tradable list. The tooltip step takes about 13 ms a frame natively. It has not been run in the browser yet.

All sizes below are 1080p pixels and scale with the UI height. Colours are at the brightness of the 1080p examples; the 1440p example was taken at another setting and every colour is shifted a little, so production code should normalise brightness first.

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

- **A wrong slot is possible when the hovered slot was never seen.** The cursor hides the hovered slot. If that slot was never recognised and another stack of the same item sits lower in the same column, that other stack is the only candidate and gets the tooltip's data.
- **Two stacks of one item in a column** resolve only if the tooltip is level with one of them or the slot's own count was read correctly. The two leapstone selection chests in the recording stayed unresolved for this reason.
- **Chests that share an icon** (II and III of the same pouch) are one slot template. Only the title tells them apart, and the slot remembers the icon, not the title.
- **The tolerant title-colour match gives an occasional false tooltip** on a screenshot without one (one of the storage stills). Nothing is written, because no title matches.
- **The recordings are H.264**, which breaks exact colours. The scanner matches the title colour with a tolerance of 8 either way. Whether live screen-share frames behave better has not been checked.
- **Character against Roster bound** is told apart only by how far the bind line extends (English text). When the cursor hides the line the frame casts no vote.
- **Titles are English only.**
- The recogniser returns `?` for characters it does not know, the bracket of the count among them.
- Chest contents and the tooltip's own icon are not read by the scanner. Chests are meant to be identified by title, with their contents hard-coded later.
