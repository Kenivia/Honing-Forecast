# Game file scripts

`scripts/game_files` turns something seen in a capture into rows of the scanner's tables, using the installed game's own files. Nothing comes from lostarkcodex.com any more; `scripts/codex` held the scripts that asked it, and what is left there is old downloads, ignored and safe to delete.

From a slot cut out of a capture to a chest in the tables:

1. `extract.py` pulls the tables and the icon sheets out of the game, once per game patch.
2. `find_icon.py` finds which icon a slot is drawn with, and so which items it can be.
3. `chest.py` writes a chest, and the chests inside it, into `templates/chest.json` and `templates/inner_chests.json`.

Then `python templates/make_msg_pack.py` and `pnpm run wasm`, as for any change to those tables.

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

- **`Item`** (130,390 rows): `Name` and `Desc` are text keys, `Grade` is the rarity (0 common to 7 sidereal; 9 is sidereal too), `Icon` and `IconIndex` name the art, `BindType`/`BindTarget`/`EnableMarket` say how it is bound.
- **`GameMsg`**: one table per language, `GameMsg_English` with `KEY` and `MSG`. The text has `<FONT>` markup.
- **`RandomBoxBase`**: one row per chest, keyed by the item id. `Type` is how it opens: 3 select one, 2 random, 0 and 1 obtain all (a few dozen of those say otherwise in `ComponentDesc`, so `chest.py` prints the item's description). The contents are under `RandomBoxEntityId`, or else under `DropIndex`.
- **`RandomBoxEntity`**: the choices, one row each: `NormalId` and `NormalMinCount`.
- **`DropBase`, `DropEntity`**: `DropBase` by `DropIndex`, then `DropEntity` by its `EntityIndex`. **The amount is `NormalMinCount` times `DropBase.Repetition`**: Destiny Shard Pouch (S) is 200 shards five times over.
- **`Money`**: a content with no `NormalId` is a currency, `NormalMoneyType` into here (18 is Destiny Shard, 14 Honor Shard, 2 gold, 1 silver).

Contents that differ by class sit in one column per class with `NormalId` 0; `game.box` leaves them out. Rows carry `ClassifyType`/`ClassifyIndex`, which gate them on an event or a piece of content; nothing here reads them.

The honing tables are in the same archive and are not extracted yet: see "Honing data in the game files" in `Calculator core/Domain model.md`.

## The icons

An icon is named `<Icon>_<IconIndex>` in lower case (`use_12_91`), the name the codex used too. `IconInfo.loa` lists all 45,286 with the sheet each is on and its rectangle there. **The index is not a position**: the sheets are packed, `use_12_91` is the 30th cell of `use_12`, and `money_15` is on `money_0`.

- The file is a 16-byte header with the count at offset 12, then per icon: name (`Use_12_91.png`), sheet, x, y, width, height, language, 8 zero bytes. A string is a 4-byte length and that many bytes ending in a NUL; an empty one is the length alone.
- Item icons are 64 px on 1024 px sheets, or 128 px (`shop_icon_*`).
- An icon with text on it has a sheet per language, named `<icon>_usa_english`; `game.icon_art` takes that one when there is one.
- 17,763 of the 17,867 icons that items name are there. The art is lossless, with the same alpha the codex's had.

## Why chests are still picked by hand

**One title is several chests.** "Destiny Destruction Stone Pouch" is three items holding 200 stones, 31,500 stones, or a choice of 75 stones and 150 Refined Obliteration Stones. An icon is many items too. `chest.py` lists every candidate and writes nothing until `--id` picks one, unless they all agree.

The amounts themselves are the game's: the cases the codex had wrong or empty (the shard pouches at five times too much, "Honing Support Selection Chest II" with nothing in it) come out right. What a chest in a slot holds is still read off the tooltip.

## find_icon.py

`python scripts/game_files/find_icon.py <crop.png>`, or a capture with `--rect x,y,size`, a recording with `--frame`, and `--brightness` for the in-game setting. Ranks every item icon against one slot and prints the best with the items drawn with each; `.tmp/find_icon.png` shows the crop and the matches.

- The crop is a whole slot cut roughly. It is brought to the art's brightness with the gamma law, and each icon is scored on the closest 75% of its opaque pixels, so the slot's background, its number and the cursor do not count. The best 300 are tried again up to 3 px off.
- The icons are cut from the sheets once into `icons.npz` (about 10 s), and again whenever their number changes. A cached run takes about 5 s.
- An icon with fewer than 1,000 opaque pixels is never a match, since it would match anything. 5,103 of the icons are under that, Destiny Leapstone (976) among them.
- Not yet run on a real capture since the move from the codex's art; on the codex's own copy of the weapon book the right icon came first (33.3 against 68.3; lower is closer).

## chest.py

`python scripts/game_files/chest.py "<title>"`, or a capture with `--rect` like `find_icon.py`, which takes the closest icon and every item drawn with it. Plain chest art is shared widely (the brown chest `use_4_225`), so `--title` with a part of the name narrows a capture's candidates first.

- **`chest.json`** gets the chest's title, icon and rarity. The icon is the template in `templates/Icons` that is this art, else a new PNG named after the chest (`--icon` names it). The templates already there are lossy copies, so "is this art" is a mean difference under 15 over alpha-weighted pixels: the same art scores about 5 and other art 30 or more. A title already there is left alone, icon and all.
- **`inner_chests.json`** gets a row for every chest among its contents, and theirs. `--inner` writes the chest itself there and skips `chest.json`. Only contents the scanner knows are kept (titles in `templates/items.json`, and chests holding some); the rest are printed as left out, and a chest holding nothing known gets no row. A row with only a title is replaced; a row with `items` is kept.
- Both files are written one row a line.
