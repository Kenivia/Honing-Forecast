# Currencies: what was tried

What was measured on the currency reader (`Currency.md`), and the cases behind its rules. Only read this when trying something new there.

## The cases behind the rules

- **Contrast.** An icon showing faintly through a near-opaque tooltip still scored 0.978: the gold tooltip hovered from the bar covers the shards, and their number read as garbage.
- **Tint.** The gold coin's template scores 0.92 on the silver coin. Once the bar's gold was remembered on the silver coin, it was looked for there from then on and its tooltip never found.
- **The coin rim.** Cut to the face alone, the coins scored 0.86 to 0.90 where the game draws the UI at 1080 lines, and the bar's silver was never found. With the rim: 0.96 to 0.99.
- **Ink above the median.** At 1080 lines the thin "1" of a dim green "15,204,181" peaks under 90, so with a fixed level the number was cut to "81".
- **Three reads at once.** With one read out at a time a hover gave one vote, and gold took three hovers.
- Captures made before the currencies differ from scan 0, since the digest includes them.

## Measured (2026-10-17)

- The twelve 1440p stills in `scanner_setup/brightness/1440p raw/silver gold shard` (setting 50, 16:9 and forced 21:9, window in its default place and moved, tooltip left and right of the inventory, from the bar and from the window) all read what `note.txt` says, wherever the currency is on screen. The moved window is not read, as intended.
- The capture there was recorded with "Forced 21:9" off. Replayed with `capture_replay --forced-21-9 --reread --no-edits`, all three and gold's split are read.
- Template scores on those stills: 0.997 to 1.0 for the right icon; other icons of the bar 0.85 at most; silver against gold 0.87.
- The three captures of 2026-10-17 08:27 to 08:35 there (forced 21:9 on, so 1080 lines): replayed with `--reread --no-edits`, everything is read, gold on its first hover. The one without gold on the bar and never hovered stays at "needs a hover", as it should.
- Not tried: other resolutions, brightness settings, a lossy recording, the inventory's tooltip when the inventory is high enough not to push it up.
