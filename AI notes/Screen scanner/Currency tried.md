# Currencies: what was tried

What was measured on the currency reader (`Currency.md`), and the cases behind its rules. Only read this when trying something new there.

## The cases behind the rules

- **Contrast.** An icon showing faintly through a near-opaque tooltip still scored 0.978: the gold tooltip hovered from the bar covers the shards, and their number read as garbage.
- **Tint.** The gold coin's template scores 0.92 on the silver coin. Once the bar's gold was remembered on the silver coin, it was looked for there from then on and its tooltip never found.
- **The coin rim.** Cut to the face alone, the coins scored 0.86 to 0.90 where the game draws the UI at 1080 lines, and the bar's silver was never found. With the rim: 0.96 to 0.99.
- **Ink above the median.** At 1080 lines the thin "1" of a dim green "15,204,181" peaks under 90, so with a fixed level the number was cut to "81".
- **Three reads at once.** With one read out at a time a hover gave one vote, and gold took three hovers.
- Captures made before the currencies differ from scan 0, since the digest includes them.

## Margins and digit groups (2026-10-19)

- **The case.** Three captures at native 1920x1080 (`ignored/recordings`, 2026-10-19 04-13, 04-14, 04-16). With the old symmetric margins (4, 3, 5, 2, 6, 7 px at 1440p) gold's "Owned" read ",138,638" three times and "3,138,638" twice, so the wrong value settled and the tooltip did not add up; shards read "15,275,481" twice, "15,275,48" and "5,275,48", and disagreed (the third capture got lucky). Silver was fine.
- **Not the background, not the cut.** The crops held every digit on a clean dark background. The recogniser gives the same misread for the same crop every time.
- **At 0.75 scale the six margins rounded to 3, 2, 4, 2, 5, 5 px**: four distinct crops, so a still screen had four votes, not six.
- **Margin grid** (left and right 0 to 6 px, 1080 lines, real recogniser): on the seven lines of those captures, left 0 to 2 and right 0 to 3 read all of them; left 3 lost "Owned"'s first digit; from 4 px most lines lost one. On the forced 21:9 still ("currency breakdown default position 21 by 9") left 0 to 4 was clean for all three totals. Other resizing filters (Lanczos, linear, nearest, 2x then 64), white padding, vertical padding and stretching the background to white each fixed some lines and broke others.
- **Groups.** First tried as exactly three digits after the first group of one to three: on that 21:9 still the recogniser misses a separator ("3.423587", "341815830") often enough that gold disagreed. Relaxed to multiples of three.
- **Result**, against the committed code (`pnpm scanner-suite` plus the stills and captures in `1440p raw/silver gold shard`, which the suite does not reach): the three captures read all of silver, gold with its split and shards on their first three reads. The 2160p stills had settled silver at 24,339,250 (first digit dropped) and now read 324,339,250. 1080p gold, 720p silver and one hover still's silver went from disagreeing to read. The currency stills and the 2026-10-17 captures read as before, some with more reads. One change for the worse is only apparent: on "Char Storage rightmost tradable lv 3 armor scroll x38" a tooltip covers silver up to "598", which used to settle as 598 and now disagrees ("598", "5598"). A number cut off by a tooltip is not handled.

## The tooltip over the total (2026-10-19)

- **The case.** Capture 05-56-28 (1080p): the inventory's gold is half covered at the start and then hovered, so its total only ever reads "428", which settled. The tooltip read 3,309,428 in full, and gold was an error ("adds up to 3309428, but 428 is shown"). Now gold is the tooltip's sum and the total is ignored. Nothing else in the suite or the currency stills changed.

## Measured (2026-10-17)

- The twelve 1440p stills in `scanner_setup/brightness/1440p raw/silver gold shard` (setting 50, 16:9 and forced 21:9, window in its default place and moved, tooltip left and right of the inventory, from the bar and from the window) all read what `note.txt` says, wherever the currency is on screen. The moved window is not read, as intended.
- The capture there was recorded with "Forced 21:9" off. Replayed with `capture_replay --forced-21-9 --reread --no-edits`, all three and gold's split are read.
- Template scores on those stills: 0.997 to 1.0 for the right icon; other icons of the bar 0.85 at most; silver against gold 0.87.
- The three captures of 2026-10-17 08:27 to 08:35 there (forced 21:9 on, so 1080 lines): replayed with `--reread --no-edits`, everything is read, gold on its first hover. The one without gold on the bar and never hovered stays at "needs a hover", as it should.
- Not tried: other resolutions, brightness settings, a lossy recording, the inventory's tooltip when the inventory is high enough not to push it up.
