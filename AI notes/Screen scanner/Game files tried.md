# Game file scripts: don't try these

Warnings behind `Game files.md`: what is not in the client, and what was ruled out for the tables it builds. Only read this when trying something new there.

## Not in the client

- **Chaos dungeon and guardian raid drop rates are not in the client** (2026-10-25, all 990 tables of the pre-update snapshot). 105 tables ship empty, `ChaosDungeonTier` and `ChaosDungeonSchedule` among them (columns only, `Reward_Item` too), and `PCDrop`, `DropBoost`, `ItemSeasonSmartDrop`. Guardians are `RaidCommon` (`RaidType` 1), all with `RewardPropId` 1081035, but `PropDrop` keeps only its `Classify*` columns, so nothing leads to their loot.
  - **`DropBase.Ratio` is a real chance** (0.005, 0.375, 50; about 12k of 39.6k rows under 100), but those rows are old field and NPC drops: rune stones, potions, maps, life-skill materials, cards, recipes. `NpcDrop` maps NPCs to drops (669 of its 775 drop ids are in `DropBase`). `DropEntity.DisplayRatio` is set on 67 rows only.
  - **Every T4 honing material drop has `Ratio` 100**: 87 `DropBase` rows, all `ClassifyType` 42 (raid gates, bonus chests, chests). Besides those, one NPC: field boss Sevek Atun (`NpcDrop` 300990, drop 70160500), Destiny stones and leapstones in item-level bands 1640 to 1750+, each `Repetition` 5 with a min/max count (1700-1719: 213-288 Destruction Stones).
  - So chaos and guardian rates have to come from community data or our own captures.
- **Mari's Secret Shop and the cash shop listings are not in the client either** (2026-10-25). Only their UI text is (`sys.ingameshop.*` in `GameMsg`, "Mari's Secret Shop", "The product list has been refreshed"). No table, XML or other archive holds the products. `PaidProduct` is crystal prices of paid services (no item targets), `Shop`/`ShopNpc` the NPC vendors, `DirectShop` the event and guild exchange shops, `GoldShopEntity` empty. The server sends the shop's stock.
- **Don't look for the contents of old gear chests, "Raid: Ur'nil", the "Illusion Gift Chest", "Cryptic Treasure Chest" or "Weekly Purification Mission Reward Chest III"**: 128 live items on 29 templates are chests by category and not in `RandomBoxBase`, and 125 of them are in none of the 789 tables searched for their ids (2026-10-10). The server decides what they give; the gold ones only say so in their description.
- **Don't look for the small coin icons in the icon atlases**: only the large item art of the currencies is there (`Game files.md`, "Money").

## Ruled out

- **Don't take the codex's chests or amounts**: it had the shard pouches at five times too much and "Honing Support Selection Chest II" with nothing in it. The game's tables (amount times `Repetition`) are right on both.
- **Don't tell a bar's gold band by its art** (a bar's gold was "CharBound" for `use_13_34` to `use_13_38` and "Tradable" for every other chest, nothing measured): on the 16-29-47 capture the "Gold Chest"s (said tradable) give roster-bound gold and the 200,000 / 300,000 piles (300,000 said character bound) give deposit gold. `Item.Category` is right.
- **Don't trust the community sheet for Serca Normal's bonus chest price** ("Raids, Dungeons and Guardians Rewards.xlsx" in `ignored/`, September 2026): 4,160 / 6,080 in the game against 4,480 / 6,720 in the sheet. Every other price and amount of the T4 raids agrees with the game.
- **Don't put the Conquest Campaign Upgrade Chests (`use_11_161` to `163`) back in the table**: ten skin chests share their art and rarity, so a skin chest that was never hovered ended NeedHover and failed the e2e check on the 10-20 capture (hand-set empty slot CharStorage 0 3,0). One-time catch-up rewards are not worth a hover on every skin chest, so their art is in `UNWANTED_ICONS`.
- **Don't compare scanner suite runs against the whole of `target/scanner-suite/<run>`**: it keeps outputs of recordings no longer in `ignored/recordings`, so compare only those of current inputs.
