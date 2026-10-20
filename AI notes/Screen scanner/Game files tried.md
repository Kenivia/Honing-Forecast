# Game file scripts: what was checked

What was checked about the tables `Game files.md` builds, and what they replaced. Only read this when trying something new there.


- **`binds` against the tooltips** read on the suite (2026-10-10), by title, for titles bound one way in the table: all 165 bound reads agree (33 character, 132 roster). The 10 that do not are "Tradable" reads of gear, bracelets, engraving recipes and a quest item, none of them a chest.
- **`content_binds` counted** against their chest (October 2026): 97 are looser (44 tradable and 41 roster bound in a character-bound chest, 12 tradable in a roster-bound one), 538 are character bound in a roster-bound chest, the rest the same or a currency.
- **Against the codex**: the amounts are the game's, and the cases the codex had wrong or empty (the shard pouches at five times too much, "Honing Support Selection Chest II" with nothing in it) come out right. How the table did against the recordings is in `Tooltips tried.md`.
- `chests.json` went from 258 KB to 714 KB with the irrelevant rows.
- `find_icon.py` on the codex's own copy of the weapon book put the right icon first (33.3 against 68.3; lower is closer).
- **Bonus chests against the community sheet** ("Raids, Dungeons and Guardians Rewards.xlsx" in `ignored/`, September 2026, its "Raid Info" and "Raid extra loot" tabs; 2026-10-20, on the client before the October update and again after): every price and every amount of the T4 raids agree (Horizon Cathedral, Final Day, Armoche, Mordum, Serca Hard and Nightmare), accessories, stones, bracelets and cores too, but Serca Normal's price, 4,160 / 6,080 in the game against 4,480 / 6,720 in the sheet. The chests were found by searching every table for the sheet's prices.
- **Gold by art** (2026-10-11 to 10-19): a bar's gold was "CharBound" for `use_13_34` to `use_13_38` and "Tradable" for every other chest, nothing measured. `Item.Category` replaced it: on the 16-29-47 capture the "Gold Chest"s (said tradable) give roster-bound gold and the 200,000 / 300,000 piles (300,000 said character bound) give deposit gold.
