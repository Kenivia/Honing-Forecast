# Game file scripts: what was checked

What was checked about the tables `Game files.md` builds, and what they replaced. Only read this when trying something new there.


- **`binds` against the tooltips** read on the suite (2026-10-10), by title, for titles bound one way in the table: all 165 bound reads agree (33 character, 132 roster). The 10 that do not are "Tradable" reads of gear, bracelets, engraving recipes and a quest item, none of them a chest.
- **`content_binds` counted** against their chest (October 2026): 97 are looser (44 tradable and 41 roster bound in a character-bound chest, 12 tradable in a roster-bound one), 538 are character bound in a roster-bound chest, the rest the same or a currency.
- **Against the codex**: the amounts are the game's, and the cases the codex had wrong or empty (the shard pouches at five times too much, "Honing Support Selection Chest II" with nothing in it) come out right. How the table did against the recordings is in `Tooltips tried.md`.
- `chests.json` went from 258 KB to 714 KB with the irrelevant rows.
- `find_icon.py` on the codex's own copy of the weapon book put the right icon first (33.3 against 68.3; lower is closer).
