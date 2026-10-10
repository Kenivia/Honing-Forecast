# Game file scripts

`scripts/game_files` builds the scanner's chest table and icon art from the installed game's own files. Nothing comes from lostarkcodex.com any more; `scripts/codex` held the scripts that asked it, and what is left there is old downloads, ignored and safe to delete.

1. `extract.py` pulls the tables and the icon sheets out of the game, once per game patch.
2. `chests.py` writes `templates/chests.json` and `templates/ChestIcons`: every chest that opens to something the scanner counts, and every other chest drawn like one of those.
3. `python templates/make_msg_pack.py` and `pnpm run wasm`, as for any change to the tables.

`find_icon.py` is for looking: it says which icon a slot in a capture is drawn with.

## What is needed, and where it lives

None of this is tracked. `game_file_reader/` is ignored whole.

- **lostark-explorer** (github.com/Poyoanon/lostark-explorer), unpacked at `game_file_reader/lostark-explorer-main/lostark-explorer-main`. Its Core library holds the NA/EU keys and reads `.lpk`. Its CLI only inspects and patches, so `scripts/game_files/extract/` is a twenty-line C# program on top of Core that writes entries out. Core targets .NET 8 and builds with the .NET 10 SDK.
- **The Lost Ark build of umodel** (UE Viewer) at `game_file_reader/umodel/umodel_lostark_v7.exe`, with `SDL2_64.dll` beside it. The build is linked from the first page of the Lost Ark thread on gildor.org (topic 3055), a Google Drive zip with only the exe. The stock umodel does not open Lost Ark packages. It is 64-bit and complains that `SDL2.dll` is missing, but what it loads is the x64 `SDL2.dll` from libsdl's releases **renamed to `SDL2_64.dll`**.
- The game at `C:\Program Files (x86)\Steam\steamapps\common\Lost Ark`, or the folder given to `extract.py`. Reading it needs no elevation.

Claude Code will not run a downloaded exe on its own: `umodel_lostark_v7.exe` is on the allow list in `.claude/settings.json`.

## What extract.py writes

About 15 s in all.

- **`scripts/game_files/tables/`** (630 MB): seven of the 990 SQLite tables in `EFGame/data2.lpk`, decrypted, and `IconInfo.loa` from `data3.lpk`. The scripts open the `.db` files as they are.
- **`scripts/game_files/atlas/`** (760 MB): the 1,541 textures of the 22 `EFUI_IconAtlas_*` packages as PNG, in `<package>/Texture2D/<sheet>.png`. umodel takes a package by its real name only with `-nameresolve`; on disk the files have scrambled names.

Vite watches the whole project, and a folder of thousands of files being written had the dev server using two cores. `scripts/game_files` and `game_file_reader` are in `server.watch.ignored` in `vite.config.ts`.

## The tables

Every table has `PrimaryKey` and `SecondaryKey`; text columns hold keys into `GameMsg`.

- **`Item`** (130,390 rows): `Name` and `Desc` are text keys, `Grade` is the rarity (0 common to 7 sidereal; 9 is sidereal too), `Icon` and `IconIndex` name the art, `BindType` and `BindTarget` say how it is bound: 0/0 tradable, 2/1 bound to character, 2/2 bound to roster (`BindType` 1 binds when equipped, skins only; `EnableMarket` is whether it can be listed). `Tier`, `ExpireDeadline` (`2026-10-14-10`), `ReUseBalanceLevel`, `GainMoneyType` and `UseGainCount` are what `chests.py` reads besides.
- **`GameMsg`**: one table per language, `GameMsg_English` with `KEY` and `MSG`. The text has `<FONT>` markup.
- **`RandomBoxBase`**: one row per chest, keyed by the item id (a few keys are of no item). `Type` is how it opens: 3 select one, 2 random, 0 and 1 obtain all (a few dozen of those say otherwise in `ComponentDesc`). The contents are under `RandomBoxEntityId`, or else under `DropIndex`.
- **`RandomBoxEntity`**: the choices, one row each: `NormalId` and `NormalMinCount`.
- **`DropBase`, `DropEntity`**: `DropBase` by `DropIndex`, then `DropEntity` by its `EntityIndex`. **The amount is `NormalMinCount` times `DropBase.Repetition`**: Destiny Shard Pouch (S) is 200 shards five times over.
- **`Money`**: a content with no `NormalId` is a currency, `NormalMoneyType` into here (18 is Destiny Shard, 14 Honor Shard, 2 gold, 1 silver).

Contents that differ by class sit in one column per class (64 of them, each ending in `Id`) with `NormalId` 0: a filled column is a class that gets the row, and its id is what that class gets. `game.box` returns them apart, each with every title it has. Rows carry `ClassifyType`/`ClassifyIndex`, which gate them on an event or a piece of content; nothing here reads them.

The honing tables are in the same archive and are not extracted yet: see "Honing data in the game files" in `Calculator core/Domain model.md`.

## The icons

An icon is named `<Icon>_<IconIndex>` in lower case (`use_12_91`), the name the codex used too. `IconInfo.loa` lists all 45,286 with the sheet each is on and its rectangle there. **The index is not a position**: the sheets are packed, `use_12_91` is the 30th cell of `use_12`, and `money_15` is on `money_0`.

- The file is a 16-byte header with the count at offset 12, then per icon: name (`Use_12_91.png`), sheet, x, y, width, height, language, 8 zero bytes. A string is a 4-byte length and that many bytes ending in a NUL; an empty one is the length alone.
- Item icons are 64 px on 1024 px sheets, or 128 px (`shop_icon_*`).
- An icon with text on it has a sheet per language, named `<icon>_usa_english`; `game.icon_art` takes that one when there is one.
- 17,763 of the 17,867 icons that items name are there. The art is lossless, with the same alpha the codex's had.

## chests.py

`python scripts/game_files/chests.py` writes the whole table; nothing is picked by hand. A chest is in it when it opens, through any number of chests inside it, to a title of `templates/items.json`. So adding a material there (or an alias of one) and running this again brings in the chests that hold it.

- **Gold and silver.** A chest that reaches no material is in only when everything it opens to, all the way down, is gold alone or silver alone (and nothing of it differs by class). Silver with a potion, or gold with silver, is out. A chest that is in for a material still lists the gold and silver it holds.
- **What opens.** Everything in `RandomBoxBase`, and bars of gold: an item with `GainMoneyType` 1 (silver) or 2 (gold) and a `UseGainCount` is written as a chest that obtains all of that much. A bar has no chest panel on its tooltip; its hover is told by its title (`Tooltips.md`, "Bars of gold").
- **A row** is `id`, `title`, `icon` (the art's name), `rarity`, `kind`, `level`, `extra`, `top` and `contents`, each content being title, amount and the id of the chest it is, when it is one of this table. "(Bound)" is cut off titles.
- **`binds`** is how the row's items are bound, as the scanner's `Tradability` names. A row is several items, so it can be several ways: of the 925 top rows that count, 461 are character bound only, 446 roster bound only, 15 either (gold coins, some event gold bars, "Honing Support Selection Chest II") and 3 all three (Destiny Shard Pouch S, M and L). Only the items that can sit in a slot count, or all of them for a row none of whose can; irrelevant rows have none. The scanner takes a chest's tradability from it when no tooltip gave one (`Tooltips.md`, "Tradability").
  - Checked against the tooltips read on the suite (2026-10-10), by title, for titles bound one way in the table: all 165 bound reads agree (33 character, 132 roster). The 10 that do not are "Tradable" reads of gear, bracelets, engraving recipes and a quest item, none of them a chest.
  - By slot template: 175 of the 221 are bound one way over every chest that counts, and 218 in roster storage.
  - Materials gain little: the tradable, bound and "(Roster)" items of one material share an icon.
- **`content_binds`** is how each content is bound, in the order of `contents`: a content is an item with a `BindType` and `BindTarget` of its own. Only "RosterBound" and "Tradable" are written, and nothing for a currency or a content bound to a character, which is never looser than its chest; the field is left out when nothing is.
  - **Gold is the exception, and is hard-coded.** Gold comes tradable, roster bound or character bound like a material, the tables do not say which a chest gives (it is a currency, with no item behind it), and telling by the tooltip was judged not worth it. So a "Gold" content always has an entry, by the chest's art: "CharBound" for the bars of gold (`BOUND_GOLD_ICONS`: `use_13_34` to `use_13_38`, 86 contents), "Tradable" for every other chest (42: the "Gold Chest"s, the coins, the sacks, and the "10,000 Gold Bars" drawn with `use_11_212`). The user's rule, 2026-10-11; nothing was measured against tooltips. Where the items of a row differ, it is the tightest of them. The manifest uses it (`Manifest.md`); the scanner does not read it.
  - Counted contents against their chest (October 2026): 97 are looser (44 tradable and 41 roster bound in a character-bound chest, 12 tradable in a roster-bound one), 538 are character bound in a roster-bound chest, the rest the same or a currency.
  - The looser ones are all in obtain-all and random chests. Nine obtain-all chests give tradable things only (the Emergency, Harvest and Abundance Honing Materials Chests, S to L), 25 give into two bands ("Raid: Argeos", the Achievement Chests). No select-one has an option, however deep, that is looser than the chest.
  - One list for a row is exact: over every item of every row it gives the band the item's own contents would. Rows that list the same things would differ in one case only, which needs a character-bound chest neither has.
- **`level`** is `Item.ReUseBalanceLevel`, the item level the chest asks for. The game writes it across the slot (`Pipeline.md`), and 223 of the chests have one.
- **`extra`** counts the contents that differ by class, which are left out; the tooltip still has a row for the player's class, so the reader allows that many rows it cannot place. 8 chests.
- **Items that read the same are one row** under the smallest id: same title, icon, rarity, kind, level and contents. Mostly these differ in how they are bound. Chests are compared after the chests inside them were merged, so it goes round until nothing merges.
- **`top`** rows can sit in a slot and get an icon template: they have an English name, are not past `ExpireDeadline` on the day the script runs, and their `Tier` is 0 or 4 (1 to 3 are older tiers; most chests have 0). The other rows are only there for being inside a top one, and carry no icon.
- **Irrelevant by name**: a chest with "Cube" or "Engraving" in its title (`UNWANTED`) never counts as opening to a material, and neither does a chest that only matters through one.
- **Irrelevant rows** (`irrelevant: true`, 1,047 of them): every other box that is drawn with the art and rarity of a top row, so that a slot holding one can be told from the chest it looks like (`Slot grid.md`). It has to have an English name and not be expired; its tier does not matter. Its contents are what its tooltip lists, with no chest ids. Rows that read the same are one. 76 of the 221 templates have at least one.
- **`by_class`** (45 irrelevant rows): the chest has contents that differ by class. Its contents then hold every title any class has, one entry each, and the reader only asks that each row of the tooltip be one of them (`Tooltips.md`). Relevant rows keep `extra`.
- **A card is listed without "Card"**: the tooltip says "Lumencaligo" for the item "Lumencaligo Card" (`Item.Category` 32000).
- **Not in the table: chests the client has no contents for.** 128 live items on 29 of the templates are chests by category and are not in `RandomBoxBase`: old gear chests, "Raid: Ur'nil", the "Illusion Gift Chest" and "Cryptic Treasure Chest" that give gold, "Weekly Purification Mission Reward Chest III". Every one of the 789 tables was searched for their ids (2026-10-10): 125 are in none, so the server decides what they give, and the gold ones only say so in their description. Left out on purpose: such a slot reads as the chest it is drawn like, and is set by hand.
- **Not in the table either**: an irrelevant box with nothing listed, and the relevant boxes of tiers 1 to 3.
- **Left out by art**: a chest drawn with one of `UNWANTED_ICONS` is dropped the same way. Five icons, ten chests: Splendid Pouch, Mariner's Treasure Chest, the two Act 4 Denouement Clear Event Chests and the six Honing Support Material pouches.
- **Numbers** (October 2026): 1,979 rows: 932 that count, 925 of them top, and 1,047 irrelevant; 141 pieces of art making 221 templates with their rarity. 138 rows are gold alone and 102 silver alone. The file is 714 KB, from 258.
- **The art** goes to `templates/ChestIcons/<icon>.png`, 64 px, cut lossless from the sheets. `make_msg_pack.py` names each template `<icon>@<rarity>`.

Seven groups of chests list exactly the same rows and still open differently, because a chest inside them is another one under the same title (the two "Trailblazer Supplies Chest: Crucible Level 3", say). Those cannot be told apart by their tooltip, and where their icon is the same too they end up "Not included" in the manifest.

The amounts are the game's: the cases the codex had wrong or empty (the shard pouches at five times too much, "Honing Support Selection Chest II" with nothing in it) come out right. How the table did against the recordings is in `Tooltips.md`.

### Aliases in items.json

A material goes by other titles that are drawn alike: "[Event] Artisan's Metallurgy: Level 1", "Destiny Leapstone (Roster)", "[Event] Metallurgy: Hellfire [11-14] - Bound". Each has its own row in `templates/items.json` with the material's icon, rarity and label. They were found by listing the items that share an icon and a rarity with a known material and have another title. "Enhanced Metallurgy: Hellfire [19-20]" shares the book's art too and was left out, not knowing what it is.

## find_icon.py

`python scripts/game_files/find_icon.py <crop.png>`, or a capture with `--rect x,y,size`, a recording with `--frame`, and `--brightness` for the in-game setting. Ranks every item icon against one slot and prints the best with the items drawn with each; `.tmp/find_icon.png` shows the crop and the matches.

- The crop is a whole slot cut roughly. It is brought to the art's brightness with the gamma law, and each icon is scored on the closest 75% of its opaque pixels, so the slot's background, its number and the cursor do not count. The best 300 are tried again up to 3 px off.
- The icons are cut from the sheets once into `icons.npz` (about 10 s), and again whenever their number changes. A cached run takes about 5 s.
- An icon with fewer than 1,000 opaque pixels is never a match, since it would match anything. 5,103 of the icons are under that, Destiny Leapstone (976) among them.
- Not yet run on a real capture since the move from the codex's art; on the codex's own copy of the weapon book the right icon came first (33.3 against 68.3; lower is closer).
