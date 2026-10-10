# Game file scripts

`scripts/game_files` builds the scanner's chest table and icon art from the installed game's own files. Nothing comes from lostarkcodex.com any more; `scripts/codex` held the scripts that asked it, and what is left there is old downloads, ignored and safe to delete.

1. `extract.py` pulls the tables and the icon sheets out of the game, once per game patch.
2. `chests.py` writes `templates/chests.json` and `templates/ChestIcons`: every chest that opens to something the scanner counts.
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

- **`Item`** (130,390 rows): `Name` and `Desc` are text keys, `Grade` is the rarity (0 common to 7 sidereal; 9 is sidereal too), `Icon` and `IconIndex` name the art, `BindType`/`BindTarget`/`EnableMarket` say how it is bound. `Tier`, `ExpireDeadline` (`2026-10-14-10`), `ReUseBalanceLevel`, `GainMoneyType` and `UseGainCount` are what `chests.py` reads besides.
- **`GameMsg`**: one table per language, `GameMsg_English` with `KEY` and `MSG`. The text has `<FONT>` markup.
- **`RandomBoxBase`**: one row per chest, keyed by the item id (a few keys are of no item). `Type` is how it opens: 3 select one, 2 random, 0 and 1 obtain all (a few dozen of those say otherwise in `ComponentDesc`). The contents are under `RandomBoxEntityId`, or else under `DropIndex`.
- **`RandomBoxEntity`**: the choices, one row each: `NormalId` and `NormalMinCount`.
- **`DropBase`, `DropEntity`**: `DropBase` by `DropIndex`, then `DropEntity` by its `EntityIndex`. **The amount is `NormalMinCount` times `DropBase.Repetition`**: Destiny Shard Pouch (S) is 200 shards five times over.
- **`Money`**: a content with no `NormalId` is a currency, `NormalMoneyType` into here (18 is Destiny Shard, 14 Honor Shard, 2 gold, 1 silver).

Contents that differ by class sit in one column per class with `NormalId` 0; `game.box` leaves them out and counts them. Rows carry `ClassifyType`/`ClassifyIndex`, which gate them on an event or a piece of content; nothing here reads them.

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
- **What opens.** Everything in `RandomBoxBase`, and bars of gold: an item with `GainMoneyType` 1 (silver) or 2 (gold) and a `UseGainCount` is written as a chest that obtains all of that much. A bar has no chest panel on its tooltip; its slot is known by its icon alone.
- **A row** is `id`, `title`, `icon` (the art's name), `rarity`, `kind`, `level`, `extra`, `top` and `contents`, each content being title, amount and the id of the chest it is, when it is one of this table. "(Bound)" is cut off titles.
- **`level`** is `Item.ReUseBalanceLevel`, the item level the chest asks for. The game writes it across the slot (`Pipeline.md`), and 223 of the chests have one.
- **`extra`** counts the contents that differ by class, which are left out; the tooltip still has a row for the player's class, so the reader allows that many rows it cannot place. 8 chests.
- **Items that read the same are one row** under the smallest id: same title, icon, rarity, kind, level and contents. Mostly these differ in how they are bound. Chests are compared after the chests inside them were merged, so it goes round until nothing merges.
- **`top`** rows can sit in a slot and get an icon template: they have an English name, are not past `ExpireDeadline` on the day the script runs, and their `Tier` is 0 or 4 (1 to 3 are older tiers; most chests have 0). The other rows are only there for being inside a top one, and carry no icon.
- **Left out by name**: a chest with "Cube" or "Engraving" in its title (`UNWANTED`) is dropped before anything else, so a chest that only matters through one is dropped too. That is two chests today.
- **Left out by art**: a chest drawn with one of `UNWANTED_ICONS` is dropped the same way. Five icons, ten chests: Splendid Pouch, Mariner's Treasure Chest, the two Act 4 Denouement Clear Event Chests and the six Honing Support Material pouches.
- **Numbers** (October 2026): 932 rows, 925 of them top; 141 pieces of art making 221 templates with their rarity. 138 rows are gold alone and 102 silver alone.
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
