# Hover tooltips: what was tried

History behind `Tooltips.md`: the cases each rule was made for, measurements, and what was dropped. Only read this when trying something new on tooltip reading. Recordings and captures are named by their time; they are in `ignored/recordings`.

## Finding the title bar

- **Window headers.** At 1440p in the pet menu "Personal Storage" passed every test on every frame, and since only one bar is looked for, no tooltip was ever read (both 1440p recordings of 2026-10-08: nothing at all before the header test). `WINDOW_WIDTHS` was measured on those recordings. Two tests on the pixels were tried first and dropped. The tooltip's left margin being the body colour fails under a large cursor, which sits exactly there (lost a chest at 1080p). The bar's colour going on to its left is also true of a tooltip that is level with a header (lost one at 1440p).
- **Where the bar is looked for.** Scanning the whole frame took 4 ms at 1080p and 10 ms at 3440x1440 on every scan. The bound around the windows dropped four false tooltips that were outside any window (titles "No source selected", "A", "K" and none) and changed nothing else that is read. On 21-31-53 seven fewer frames are scanned, which moves which frames the slots are read on: 20 slot scores move by up to 0.01, one slot's count reads "22" where it read "2", another "-1247" where it read "1247". In the browser on 17-28-55 the search is 1.0 ms on average and 0.1 at the median, against 4.4 and 4.5. Natively, with the suite's four recordings at once, its total is 1.3 s against 4.4 on 17-28-55 and 3.6 s against 14.4 on 21-42-32. A version that looked three slot columns to either side of the last place before the whole bound gave the same output and was 5% faster at 3440x1440 only, so it was left out.
- **The whole width.** Half the width used to be enough for every row, for the cursor's sake. That was a real loss: on 21-42-32 the header piece was taken instead of the tooltip for the whole of two hovers ("Arriving Battle Earrings", frames 3605 to 3611, and "Epic Astrogem Chest", 3720 to 3747) and for the first two frames of a third; on 21-31-53 for a hover of two frames. With the last place tried first it would have been kept for as long as the tooltip stayed. A header cut off by the tooltip of its window's fifth column is 368 to 370 of 409 in all three windows, by the sixth column's 26 too long. Tried at 3, 5, 8, 16 and 40 screen pixels: the chests, the slots' tooltip data and every status are the same at all of them, and the same as with half. What differs is hovers of one to three frames on the largest-cursor recording (22-55-45): of its 1,628 tooltip frames 12 are lost at 3, 7 at 16 and 1 at 40. Five 1440p stills that showed a tooltip where there is none now show none.

## How much is read

- With unlimited OCR the read limits read the recordings exactly as reading every frame did.
- No title for a tooltip with a chest panel: that was a fifth of all the reading.
- **Holding lines until a slot is lit.** Scanning on for as long as lines were held was tried first: the 54 more frames scanned on 22-55-45 were enough for a slot that scores 0.951 against the limit of 0.95 never to be recognised. Measured with every line read at once: of the tooltips that end up writing a slot or a chest, 90 to 100% are of interest (110 of 112 on the fast-hovers capture, 105 of 120 on 22-55-45); of those that write nothing, three quarters by strip width are not.
- **Building strips on every frame**: every frame of an unsettled hover used to build all its strips, and all but a few were thrown away. Stopping that took the mean wasm call from 22.2 to 20.0 ms at 1080p and from 17.7 to 13.7 at 3440x1440 in the browser.

## Titles

- "Hellfire [15-18]" used to be cut at the hyphen.
- **The colour test at a quarter**: the "l" and "d" of "100,000 Gold Bars" made a gap wider than a word's, and the name was cut to "100,000 Go" (2026-10-10 19-22-07 capture).
- **Margins**: "300,000 Gold Bar Pile" came back as "3300,000" at 5 and right at 4. The cost is up to two more title reads for such a hover.
- **Numerals had to be identical until 2026-10-12.** A stack of "Metallurgy: Hellfire [11-14]" whose title only ever read "(1l-14]" was never told (15-53 capture, roster page 2, row 7); it now is. Over the suite one more title is read: the "50,000 Gold Chest" of 21-31-53 that read "550,000".

## Amounts

- 65 read as 665 at one margin, 440 for 40 at another. On all 1,690 tooltip frames of the two recordings the two-crop rule gave no wrong amount.
- `STACKED_NUMBER_LEFT` was measured at 165 to 173 over the suite at UI heights 720 to 2160.
- **The first candidate used to be taken whatever it was** when there was no pair. A cursor further in than its usual reach (in the description, 173 px in at 1080p) joined the description and the stacked line into one block, which left the total as the only candidate: a chest of 28 with 40 owned was stored as 40 by a hover of two scans, over the 28 a longer hover had read (2026-10-10 18-47-43 capture, roster storage page 2, 9,5). It now reads nothing on that hover and ends at 28.
- "Owned Phoenix Plume(s) x1529" (248 px in) was read as that stack's amount, which is 769 and still unread, since the stacked line is absent or covered there; and a line with its number 142 to 148 px in, above the amounts on some chests, took the stacked line's place, alone or as the first of a pair. Two such hovers on the 19-56-53 capture now read their amount. None of these had a slot, so nothing else in the suite changed.

## Books

- "Honing Lv. 1 1" used to count as level 1 and so as no book. That left an "[Event] Tailoring: Hellfire [11-14]" stack unrecognised on a live capture (2026-10-10 18-47-43, roster storage 0,2): three hovers of one or two scans each, the description read right apart from that space, and the two-line title never (once "[1l-14]", then the cursor over its start).
- On the 439 book frames of the 22-55-45 recording the description names the book on 437 and the title on 311, neither ever wrongly. The two-line "[Event] Metallurgy: Hellfire [11-14]" never read. A pushed-up tooltip was not in the recording.

## Tradability and icons

- A merged bind line and "Untradable" used to read as tradable.
- **Chest tradability from the table**: 129 slot lines over the suite became good unhovered, with nothing else changed. Of 335 tooltips read in roster storage on the suite none was character bound.
- **The tooltip icon**: on the 2026-10-07 recording the right template measures 26 to 42 and the next best 50 and up, while a tooltip of something without a template has nothing under 90. About 2.5 ms a frame it runs. Skipping chest panels and capping at six tries halves the comparisons on the recordings (down by two thirds where most hovers are chests) and changes nothing that is read.

## The hovered slot

- **Taking the lit ones first was tried**: a hover of one scan on a pouch had only the slot above it lit, which got its 50 over its own 107 (1440p capture of 2026-10-11).
- **Rule 2's case**: two stacks of "Metallurgy: Hellfire [19-20]" x3 and one of another book in roster page 2's first column. The tooltip of row 10 was pushed up to within 2 px of row 7's top, and with the hints over a dark background nothing said it was pushed up. Row 7 got the data and row 10 stayed unread (15-53 capture). Rows 9 and 10 were lit, row 7 never; of the lit ones only row 10 shows 3.
- **Rule 1's case**: "Processed Chaos Stone Chest (Weapon)", which is two items with the same row and different art, one of them (`use_12_24`) no template: its amount of 10 went onto the stack of 25 drawn the other way three rows down (21-31-53 and two captures), and a book's 18 onto a stack of 23 before its own slot was recognised (18-47-43 capture).
- **Rule 4's case**: "100" came back as "00" under the cursor.
- **Untying rivals**, seen at 2160p (2026-10-11 capture): hovering "Metallurgy: Phantom [13-15]" x4 before its slot was known put its 4 and its chest on the "Blood [8-10]" x1 one row down, which was lit by the cursor's tail, and replaced the chest read for that slot a moment before. Going by alternatives as well as the slot's own icon took the slots of four rightly tied chests away on 21-31-53.
- **The page**: with only the page lit right now, a chest in a top row went nowhere.
- **Writes**: a short fading-in hover used to blank what a longer hover of the same slot had written.

## Chests

- **Rows that do not list everything.** "Offensive Battle Item Chest" holds 12, "Legendary Rapport Selection Chest" 14. Nine rows of the 52 of "Legendary Class Engraving Recipe Selection Chest" were read, which used to need exactly 10 and fell to a pouch that lists by class (14-58 capture).
- **Any chest fitting in part was tried and is wrong.** "Bracelet Bonus Rework Ticket x3", the one row of a chest that is not in the table, came out as "Clear: Phantom Manifestor Brelshaza", which counts; "Epic Combat Engraving Recipe Selection Pouch" as a "Lucky Random Chest" that lists the relic pouch; two gems as "T4 Gem Chest (Lv. 1-3)". With the two conditions all of those are gone on the seven captures of 2026-10-11. The battle item case that is left is on five of the suite's inputs.
- **Roman numerals.** Over the suite some 70 reads of one came back as "II", "I", "?I", "?", "?l", "Il", "Ill", "?II", "?lI", "?ll", "?l!", "!", "VI", "IV", glued to the word before it or not ("(S)I", "Pouch!"). **Before**, the name's last word was cut off when all its characters were of `ivxl!|?1`, and whether there was one had to agree. "Una+1 Exchange Silver Pouch!" has no such last word, so the "Una +1 Reward Exchange Chest" matched nothing (15-53 capture, roster page 1); it is read now. **Tried and dropped**: comparing the name as it is (the nearest tier then wins, and "II" read as "I" picks the wrong chest with full confidence); a longer list of look-alike characters (the next read has another). **Not tried**: counting the strokes in the row's pixels, which would say which tier it is. It would tell one group of chests apart that nothing tells apart now, so it is not worth its weight yet.
- **The header.** The kind used to come from the shape of the header's yellow phrase.
- **"Obtain one ... at random".** On capture 05-53-51 (1080p) "Ancient Bracelet Chest" x206 (Random, `use_2_10@ancient`, irrelevant) read its row right, but the header read as obtain all and only the ObtainAll chests listing "Radiant Savior Bracelet x1" kept their votes ("T4 Ancient Bracelet Chest", other icons). The chest was stored for the column with no slot, and the slot stayed at "hover it". With random chests let through under an obtain all header, the slots pick 65811040 and the slot shows as irrelevant. Nothing else in the suite changed.
- **The 8 px phrase.** A material's tooltip over an item with a gold border had 4 yellow pixels in two columns showing through under a black stretch of the body, which passed for a panel with an "all" header, so "Serca Red" x1240 never got its tradability (15-53 capture).
- **Tooltips of no chest in the table.** Before the irrelevant rows, 126 of the 242 chest tooltips on the four recordings matched; a Python port of the reader over the other 116, against every box of the game, fitted 106 to an irrelevant chest and none to one that counts. With the rows in, 14 irrelevant chests are stored on the recordings (`Slot grid.md`).
- **The slot of a chest**: the first guess of a slot used to stick when the rows came back before the amount.
- **Bars of gold.** Before, every bar stayed at "hover for tradability" whatever was hovered (2026-10-10 19-22-07 capture, ten stacks). On that capture all ten now come out with the amount and tradability the user had set by hand. On the 21-31-53 recording, the same roster compressed, nine of the ten do; the tenth is level with its slot and the cursor is over its first digit ("0,000 Gold Chest", "550,000"). The suite's stills and every other recording's slots and chests were as before; two more titles read at the new margins and one fewer.
- **How bars' gold is bound** (2026-10-19). Before, it was by art (`Game files tried.md`), and same-title bars were one row under the smallest id, so the 16-29-47 capture had its "20,000 Gold Chest" x13 and "50,000 Gold Chest" stacks as character-bound gold, and 16-51-17's "Thick Gold Bar" x5 as tradable. Telling them by the description's colours alone (pale yellow against tan lines, no OCR) was looked at: it splits 5 of the 10 mixed title groups, but not character from roster bound ("The obtained Gold is Character-bound." against "Roster-bound."), which only line widths would. Reading the description was chosen. With it, on the suite against HEAD built with the same day's `chests.json`: the stills are unchanged, every bar hover that stayed up a few scans reads its description exactly (eight on 16-29-47, one on 16-51-17, one each on three older captures), and only the bars' chests and slots change. One 20,000 Gold Chest hover of two scans on 10-23-03 gets no description and its slot now asks for a hover. Of the 16-29-47 stacks, the two "10,000 Gold Bars" are different items (x11 deposit, x9 roster bound).
- **The rows' icons.** Matching the row's 30 px icon was measured with the proof of concept's matcher: it agreed with the text on 812 of 817 rows and never rescued one.

### Results

Natively, every frame, on the four recordings:

| Recording | Before | With the table |
| --- | --- | --- |
| 1080p, 2026-10-07 17-28-55 (1,150 frames) | 14 chests | 14, the same ones |
| 1080p, 2026-10-07 22-55-45 (3,634 frames) | 24 | 26: the same 24, a chest of silver and a book chest that was not in the old table |
| 3440x1440, 2026-10-08 21-42-32 (4,167 frames at 60 a second) | 0 (no tooltip found) | 36 |
| 3440x1440, 2026-10-08 21-31-53 (8,639 frames) | 0 | 29 |

- Every chest stored is a row of the game's table with every count agreeing, so a misread cannot make one up; what was checked is that nothing is lost and that each sits with its slot. Of the 105, all but one have a slot in their column drawn with one of their icons, and all but one of those show their amount. The one without a slot was under tooltips whenever its page was up; the one amount is a slot's own count misread ("00" for 100).
- The same-column case: roster page 2 at 1080p has "Honing Leapstone Selection Chest VIII" x86 and "IX" x70 one above the other with the same icon and pushed-up tooltips. Both come out as their own row (VIII has a third option the old table did not know) and each slot is tied to its own.
- Fed every second frame (and the 1440p ones every fourth, with OCR held to two lines a frame as in the browser), the 1440p recordings give the same 36 and 29. The 1080p ones lose one chest each, both hovers of a single frame.
- In the browser (headless Chromium, the 17-28-55 and 21-42-32 recordings at half speed): the same 14 and 36 chests, and the manifest opens them through to materials (`Manifest.md`).

Overall at the time: all nine tooltip stills read correctly (title, amount, bind kind; slot wherever the item has an icon). On the largest-cursor recording (1777 frames, H.264) a tooltip is found on 1160 frames in 134 hovers; 40 hovers get an accepted title, 27 of them a slot; 26 slots end up with tooltip data, and all 15 in roster storage match the annotated amounts and the known tradable list.

## Brightness and colour

- With the estimate forced on the recordings at 60, everything reads from 56 to 61, two short chest hovers are lost from 62, and a quarter of the chests by 66.
- Yellow used to be told by red above 190, which most yellow pixels of a recording at 100 did not reach.
- With the chest panel's limit at 10 a recording at 100 had the panel start among the amount lines, losing the amounts and the chest.
- On lossless captures of one chest tooltip at all 21 settings (`scanner_setup/brightness/inputs storage`), everything reads at every setting but 5, natively; in the browser 0, 20 and 100 were checked. The recording at 100 reads 24 chests.
- **Edge cases of compressed recordings.** The title bar's blue-grey test measured exactly its old limit of 4 on two hovers of 5 and 8 frames and failed in bands about every two brightness settings. On one 9-frame hover the yellow word of the chest header ("all") has a single pixel that counts as yellow at brightness 59 and none from 59.5 up, so natively that chest is not read at the true brightness of 60.
- The 1440p example was taken at another setting and every colour is shifted a little against the 1080p ones.

## The proof of concept

The approach that worked in Python, before the scanner had it:

1. **Is there a tooltip**: find rows with a flat run of the title colour that ends at a sharp right edge, at least half the title width long, on several consecutive rows. The tooltip's left edge is the run's right end minus 307. Confirm with the right margin below it: title colour for 25 to 80 rows, then the body colour. On lossless input "flat" can mean identical neighbouring pixels with no colour constant at all; the colour checks that reject other windows' grey bars are that the title is blue-grey and the body is the expected fraction of it.
2. **What is hovered**: compare the 64 px icon at its fixed offset against the raw icon art in `scanner_setup/templates/Icons`, using the art's own alpha as the mask and scoring only the closest 75% of pixels so a cursor over part of it does not matter. The existing slot templates do not work here because the tooltip draws the icon over a different background. The slot follows from placement: the column from the tooltip's edge, the row from its top when it was not pushed up, otherwise the slot in that column that was already known to hold that item.
3. **Where the text is**: split the body into text lines by row projection, using only the columns the cursor cannot reach. An amount line is a short line outside the black panel whose yellow reaches the line's right end and has white just before it; stacked and total are the first such pair one line apart. Untradable is a red right-aligned line above the amounts. Bound to Character or Roster is the line above that, judged by where it ends. Chest rows are the tall lines inside the black panel.

Results on stills: all nine tooltip stills parse (1080p and 1440p, one and two line titles, both sides, all three chest kinds), and none of the 32 screenshots without a tooltip trigger it.

Results on the largest-cursor recording (59 s, 143 hovers, roster storage page 2 annotated): 27 of the 28 distinct amounts in roster storage were read, none wrongly; the 28th was never detected as hovered. Tradable against roster bound agreed with the tooltip text for every item read. One hover was lost because the cursor covered the top-left of the title and the bar was located 4 px off.

Icon identification is the weak part with a large cursor: when the tooltip is level with the slot the cursor covers most of the 64 px icon, and several template items came back unidentified. Scoring only the best 75% of pixels is not enough.

**Reading the title is the better identifier.** The title bar is OCR'd, the `[X n]` count and "(Bound)" are dropped, and the rest is fuzzy-matched against a table of in-game names (`ITEM_TITLES` in the proof of concept). On that recording a title was read on 136 of 143 hovers, every hover of a template item was identified (17 by title, 16 by icon, the two agreeing wherever both answered), and the title recovered both items the icon lost to the cursor. Three rules came out of it:

- Similarity has to be at least 0.92. "Destiny Destruction Stone Pouch" is 0.86 similar to "Destiny Destruction Stone".
- Digits must match exactly. "Level 4" is 0.96 similar to "Level 3".
- Every title line is centred in the bar. If a line's left and right gaps differ, something is over it, and a covered start can turn one valid name into another ("Great Destiny Leapstone" into "Destiny Leapstone"). Frames with centred lines outvote the rest. The test is conservative: most off-centre frames still read correctly.

The OCR used inserts stray letters now and then ("Destiny D Destruction Stone"), which the fuzzy match absorbs. The title count is not reliable and is absent on single items. The scanner since moved to 0.9 similarity and numerals by rivals.

## Waiting for the slot

- **Until 2026-10-19 a hover ended as soon as its last texts were in**, slot or not. Live it worked because OCR was slow; a re-run of `10-18` with every text back the scan after it was sent read hover 1 (an Armor Book, scans 9 and 10) at scan 11 while its slot was only recognised at scan 12, so it wrote nowhere and the slot asked for a hover. Waiting for a fixed count of scans was chosen over waiting until the slot is recognised, which could wait forever on a slot that never matches.

