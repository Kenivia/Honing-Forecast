# Currencies: what was tried

Warnings for the currency reader (`Currency.md`): what failed or was ruled out. Only read this when trying something new there.

## Matching and reading

- **Don't match an icon on score alone**: template matching ignores contrast, and an icon showing faintly through a near-opaque tooltip still scored 0.978 (the gold tooltip hovered from the bar covers the shards, and their number read as garbage).
- **Don't match on grey alone, or remember a match that fails the tint check**: the gold coin's template scores 0.92 on the silver coin, and once the bar's gold was remembered on the silver coin it was looked for there from then on and its tooltip never found.
- **Don't cut the coin templates to the face**: at 1080 lines they scored 0.86 to 0.90 and the bar's silver was never found; with the rim 0.96 to 0.99.
- **Don't use a fixed ink level for text**: the thin "1" of a dim green "15,204,181" peaks under 90 at 1080 lines and the number was cut to "81". Use a step above the strip's median.
- **Don't send one read at a time**: a hover gave one vote and gold took three hovers. Send three at once.

## Margins and digit groups

- **Don't use wide or symmetric margins.** At native 1080p the old (4, 3, 5, 2, 6, 7 px at 1440p) margins made gold's "Owned" read ",138,638" three times (a wrong value settled) and shards read "15,275,481" twice with "15,275,48" and "5,275,48" besides. The recogniser misreads the same crop the same way every time and the background was clean, so it is the margin, not the cut. Left 0 to 2 and right 0 to 3 px read every line; from 3 px left, or 4 on, lines lost a digit.
- **Don't assume margins stay distinct once scaled down**: at 0.75 scale six margins rounded to four distinct crops, so a still screen had four votes. A repeat is widened instead.
- **Don't try other resizing filters (Lanczos, linear, nearest), white or vertical padding, or a white stretched background to fix dropped digits**: each fixed some lines and broke others.
- **Don't require exactly three digits per group after the first**: the recogniser misses a separator often enough ("3.423587", "341815830") that gold disagreed on forced 21:9. Multiples of three.
- **Don't count whitespace in a read**: the recogniser spaces out "11" ("3,1 13,003"), which never voted, and gold ended as "Its reads disagree" on every margin.
- **Known gap**: a number cut off by a tooltip is not handled. Silver covered up to "598" used to settle as 598 and now disagrees ("598", "5598").

## Gold

- **Don't trust the total over the tooltip's sum**: when the inventory's gold is half covered it only ever reads "428", which settled and made gold an error ("adds up to 3309428, but 428 is shown") while the tooltip read 3,309,428 in full. Gold is the tooltip's sum once its lines settled.

## Not tried

Other resolutions and brightness settings than those of the stills and captures in `scanner_setup/brightness`, a lossy recording, and the inventory's tooltip when the inventory is high enough not to push it up.
