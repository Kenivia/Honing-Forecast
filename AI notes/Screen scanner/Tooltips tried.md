# Hover tooltips: don't try these

Warnings behind `Tooltips.md`: what failed or was ruled out in reading the tooltip. Only read this when trying something new on tooltip reading. Recordings and captures are named by their time; they are in `ignored/recordings`.

## Finding the title bar

- Don't tell a window header from a tooltip by pixels around the bar. The tooltip's left margin being the body colour fails under a large cursor, which sits exactly there (lost a chest at 1080p). The bar's colour going on to its left is also true of a tooltip level with a header (lost one at 1440p). `WINDOW_WIDTHS` is measured from recordings instead: at 1440p the pet menu's "Personal Storage" header passed every test, and with one bar looked for, no tooltip was read at all.
- Don't look for the bar over the whole frame: 4 ms at 1080p and 10 ms at 3440x1440 on every scan. The bound around the windows also dropped four false tooltips outside any window, and was 1.0 ms against 4.4 in the browser.
- Don't look at a few slot columns around the last place before the whole bound: same output, 5% faster at 3440x1440 only.
- Don't accept half the title's width for every row of a bar (it was done for the cursor's sake): the header piece beside a tooltip was taken instead of it for whole hovers (two on 21-42-32). The nearest look-alike, a header cut off by the fifth column's tooltip, is 40 px short of a title, so any slack up to there works; 3 to 40 give the same chests, and only hovers of 1 to 3 frames on the largest-cursor recording differ (12 lost at 3, 7 at 16, 1 at 40).

## How much is read

- Don't scan on for as long as lines are held for the highlight: the 54 more frames scanned on 22-55-45 moved which frames the slots are read on, enough for a slot scoring 0.951 against the limit 0.95 never to be recognised. Holding is cheap: of the tooltips that end up writing a slot or a chest 90 to 100% are of interest, of those that write nothing three quarters are not.
- Don't read a title for a tooltip with a chest panel: it matched nothing and was a fifth of all the reading.

## Titles

- Don't end the name at the first thin gap: a hyphen is too thin to count as a coloured column and "Hellfire [15-18]" was cut there.
- Don't call a pixel coloured at a quarter of its brightest: ancient's thin tan strokes had none, and the "l" and "d" of "100,000 Gold Bars" made a gap wider than a word, so the name was cut to "100,000 Go".
- Don't cut a title at one margin: "300,000 Gold Bar Pile" came back as "3300,000" at 5 and right at 4. A lossless still gives the same crop every time, so re-hovering alone does not help.
- Don't demand identical numerals: a stack of "Metallurgy: Hellfire [11-14]" whose title only ever read "(1l-14]" was never told.
- Don't take a lower similarity than 0.9, or none between rivals: "Destiny Destruction Stone Pouch" is 0.86 similar to "Destiny Destruction Stone".

## Amounts

- Don't read the number from one crop: 65 came back as 665 at one margin, 440 for 40 at another. The two-crop rule gave no wrong amount on all 1,690 tooltip frames of two recordings.
- Don't take the first candidate line when there is no pair: a cursor further in than its usual reach joined the description and the stacked line, leaving the total as the only candidate (a chest of 28 with 40 owned was stored as 40).
- Don't take any line with a yellow end for the stacked line: "Owned Phoenix Plume(s) x1529" (248 px in) and lines with the number 142 to 148 px in, above the amounts on some chests, were read as amounts.

## Books

- Don't take only the first digit run of a level line: the wide "11" comes back as "Honing Lv. 1 1", which counted as level 1, so as no book, and left a stack unrecognised.
- Don't rely on the title for which book: on the 439 book frames of 22-55-45 the description names the book on 437 and the title on 311 (neither ever wrongly), and the two-line "[Event] Metallurgy: Hellfire [11-14]" never read.

## Tradability and icons

- Don't let a merged bind line and "Untradable" read as tradable.
- Don't tell the hovered item by the large icon alone: with a large cursor level with the slot most of the 64 px icon is covered and several template items came back unidentified, scoring the closest 75% of pixels not being enough. The title recovered them all.
- The icon's limits: the right template measures 26 to 42 and the next best 50 and up, while a tooltip of something without a template has nothing under 90. Don't compare the icon on chest-panel tooltips or keep trying past six tries: skipping them halves the comparisons and changes nothing that is read.

## The hovered slot

- Don't take the lit slots first: a hover of one scan on a pouch had only the slot above it lit, which got its 50 over its own 107 (1440p capture of 2026-10-11).
- Don't pick the slot level with the tooltip without checking it was lit: a pushed-up tooltip came within 2 px of row 7's top and got the data of row 10 (stacks of "Metallurgy: Hellfire [19-20]" x3, 15-53 capture; row 10 was the lit one).
- Don't tie the only candidate unconditionally: "Processed Chaos Stone Chest (Weapon)" is two items with the same row and different art, one with no template, and its amount of 10 went onto a stack of 25 drawn the other way (21-31-53). A book's 18 went onto a stack of 23 before its own slot was recognised.
- Don't trust a slot's own count: "100" came back as "00" under the cursor.
- Don't untie rivals by the alternatives a slot may also be: it took away four rightly tied chests on 21-31-53. The case it is for: hovering "Metallurgy: Phantom [13-15]" x4 before its slot was known put its 4 and its chest on the "Blood [8-10]" x1 one row down (2160p capture of 2026-10-11).
- Don't take only the page lit right now for the slots beside a tooltip: a chest in a top row went nowhere, its tooltip covers its own window's tabs.

## Chests

- Don't require a chest's contents to be exactly the rows: the tooltip stops at 10 rows ("Offensive Battle Item Chest" holds 12). Nine of the 52 rows of "Legendary Class Engraving Recipe Selection Chest" were read, which used to need exactly 10 and fell to a pouch that lists by class.
- Don't count any chest fitting in part: "Bracelet Bonus Rework Ticket x3", the one row of a chest not in the table, came out as "Clear: Phantom Manifestor Brelshaza", which counts; two gems as "T4 Gem Chest (Lv. 1-3)".
- Don't compare a roman numeral as read. Over the suite some 70 reads of one came back as "II", "I", "?I", "?", "Ill", "!", "VI", "IV", glued to the word before it or not. Comparing the name as it is lets the nearest tier win ("II" read as "I" picks the wrong chest with full confidence); a longer list of look-alike characters fails on the next read; cutting the last word when all its characters are of `ivxl!|?1` missed "Una+1 Exchange Silver Pouch!" (no such last word). Not tried: counting the strokes in the row's pixels, which would tell the one group of chests nothing tells apart now.
- Don't take the kind from the header's yellow phrase, or filter by it: its shape varies, and "Ancient Bracelet Chest" x206 (Random) read its header as obtain all, so only obtain-all chests kept their votes and it was stored with no slot.
- Don't count a few yellow pixels as the header phrase: an item's gold border showing through a black stretch of the body passed for a panel with an "all" header, and "Serca Red" x1240 never got its tradability.
- Don't expect every chest tooltip to be in the table: before the irrelevant rows 126 of 242 chest tooltips on four recordings matched, and the other 116 fitted 106 irrelevant chests and none that counts.
- Don't tell a bar's gold band by art or by the smallest id of same-titled bars: "20,000 Gold Chest" x13 and "50,000 Gold Chest" read as character-bound gold, and "Thick Gold Bar" x5 as tradable. Don't tell them by the description's colours alone either: it splits 5 of the 10 mixed title groups but not character from roster bound, which only line widths would.
- Don't match the chest rows' icons: measured with the proof of concept's matcher it agreed with the text on 812 of 817 rows and never rescued one.

## Brightness and colour

- Don't expect the colour tests to survive a wrong estimate: with the estimate forced on the recordings at 60 everything reads from 56 to 61, two short chest hovers are lost from 62, and a quarter of the chests by 66.
- Don't tell yellow by red above 190: most yellow pixels of a recording at 100 did not reach it.
- Don't put the chest panel's black limit at 10: a recording at 100 had the panel start among the amount lines.
- Don't expect exact colour tests to hold on hovers of a few frames of a compressed recording: the title bar's blue-grey test measured exactly its old limit on two hovers and failed in bands about every two settings, and one 9-frame hover's yellow "all" has one pixel that counts as yellow at 59 and none from 59.5.
- Don't expect setting 5: bar and body are one level apart in every channel.

## Waiting for the slot

- Don't end a hover as soon as its last texts are in: with every text back the scan after it was sent, an Armor Book hover was read at scan 11 and its slot only recognised at scan 12, so it wrote nowhere. Don't wait until the slot is recognised either: it can wait forever on a slot that never matches, hence a fixed count of scans.
