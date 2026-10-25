# Game file scripts

`scanner_setup/game_files` builds the scanner's chest table and icon art from the installed game's own files. Nothing reads `scanner_setup/codex` (old downloads from lostarkcodex.com, git-ignored).

1. `extract.py` pulls the tables and the icon sheets out of the game, once per game patch.
2. `chests.py` writes `scanner_setup/templates/chests.json` and `scanner_setup/templates/ChestIcons`: every chest that opens to something the scanner counts, and every other chest drawn like one of those.
3. `decoys.py` writes `scanner_setup/templates/decoys.json` and `scanner_setup/templates/DecoyIcons`: what we do not count that looks like one of our templates.
4. `python scanner_setup/templates/make_msg_pack.py` and `pnpm run wasm`, as for any change to the tables.
5. `bonus_chests.py` writes `frontend/Data/bonus_chests.json`, the raids' bonus chests as choices for the Choices page ("Bonus chests" below). It needs only the tables.

Two more read the same tables through `game.py` but live in `scripts/game_files/`: `honing.py` (the Serca honing constants, "Honing data in the game files" in `Calculator core/Domain model.md`) and `paradise.py`, which writes `frontend/Data/paradise.json` ("Paradise" below).

`game.py` is what they share (`table`, `text`, `box`, `listed`, `icon_info`, `icon_art`, `load_index`). `find_icon.py` is for looking: it says which icon a slot in a capture is drawn with.

What was checked and what is not in the client is in `Game files tried.md`; read it only when trying something new here.

## What is needed, and where it lives

None of this is tracked. `ignored/game_file_reader/` is ignored whole.

- **lostark-explorer** (github.com/Poyoanon/lostark-explorer), unpacked at `ignored/game_file_reader/lostark-explorer-main/lostark-explorer-main`. Its Core library holds the NA/EU keys and reads `.lpk`. Its CLI only inspects and patches, so `scanner_setup/game_files/extract/` is a small C# program on top of Core that writes entries out. Core targets .NET 8 and builds with the .NET 10 SDK.
- **The Lost Ark build of umodel** (UE Viewer) at `ignored/game_file_reader/umodel/umodel_lostark_v7.exe`, with `SDL2_64.dll` beside it. The build is linked from the first page of the Lost Ark thread on gildor.org (topic 3055), a Google Drive zip with only the exe. The stock umodel does not open Lost Ark packages. It is 64-bit and complains that `SDL2.dll` is missing, but what it loads is the x64 `SDL2.dll` from libsdl's releases **renamed to `SDL2_64.dll`**.
- The game at `C:\Program Files (x86)\Steam\steamapps\common\Lost Ark`, or the folder given to `extract.py`. Reading it needs no elevation.

Claude Code will not run a downloaded exe on its own: `umodel_lostark_v7.exe` is on the allow list in `.claude/settings.json`.

## What extract.py writes

- **`scanner_setup/game_files/tables/`**: sixteen of the SQLite tables in `EFGame/data2.lpk`, decrypted, and `IconInfo.loa` from `data3.lpk`. The list is in `extract/Program.cs`; a script that needs another table has to add it there. The scripts open the `.db` files as they are (`game.table`).
- **`scanner_setup/game_files/atlas/`**: the textures of the `EFUI_IconAtlas_*` packages as PNG, in `<package>/Texture2D/<sheet>.png`. umodel takes a package by its real name only with `-nameresolve`; on disk the files have scrambled names.

Vite watches the whole project, so big folders are in `server.watch.ignored` in `vite.config.ts` (`scanner_setup/game_files`, `scanner_setup/brightness`, `scanner_setup/codex`, `ignored`, `crates`, `target`).

## The tables

Every table has `PrimaryKey` and `SecondaryKey`; text columns hold keys into `GameMsg`.

- **`Item`**: `Name` and `Desc` are text keys, `Grade` is the rarity (0 common to 7 sidereal; 9 is sidereal too), `Icon` and `IconIndex` name the art, `BindType` and `BindTarget` say how it is bound: 0/0 tradable, 2/1 bound to character, 2/2 bound to roster (`BindType` 1 binds when equipped, skins only; `EnableMarket` is whether it can be listed). `Tier`, `ExpireDeadline` (`2026-10-14-10`), `ReUseBalanceLevel`, `Category`, `GainMoneyType` and `UseGainCount` are what `chests.py` reads besides.
- **Expiry is in `Item` too.** `ExpireType` 0 is none, 3 a fixed date in `ExpireDeadline` (UTC by the one tooltip compared: "Expires: 10/14/2026 06:00"), 1 a duration in `ExpireTime` in seconds; 5 and 6 were not looked at. An item that expires has a clock on its slot (`Pipeline.md`, "Overlays"; seen for type 3 only, on "[Event] Artisan's Metallurgy: Level 4"). The expiring item is drawn with the plain one's art and differs in title by "[Event]", and one title is several rows with different deadlines, so a title says that a stack expires and not when. Only `chests.py` reads these columns, to leave out chests past their deadline.
- **`GameMsg`**: one table per language, `GameMsg_English` with `KEY` and `MSG`. The text has markup, which `game.text` strips.
- **`RandomBoxBase`**: one row per chest, keyed by the item id (a few keys are of no item). `Type` is how it opens: 3 select one, 2 random, 0 and 1 obtain all (a few of those say otherwise in `ComponentDesc`, which nothing reads). The contents are under `RandomBoxEntityId`, or else under `DropIndex`.
- **`RandomBoxEntity`**: the choices, one row each: `NormalId` and `NormalMinCount`.
- **`DropBase`, `DropEntity`**: `DropBase` by `DropIndex`, then `DropEntity` by its `EntityIndex`. **The amount is `NormalMinCount` times `DropBase.Repetition`**: Destiny Shard Pouch (S) is 200 shards five times over.
- **`Money`**: a content with no `NormalId` is a currency, `NormalMoneyType` into here (18 is Destiny Shard, 14 Honor Shard, 2 gold, 1 silver). Its `Icon` and `IconIndex` are the large item art (`etc_14` silver, `money_4` gold, `money_15` Destiny Shard). The small single coins the inventory, the currency window and the HUD draw are not among the icon atlases; they are probably in UI packages `extract.py` does not pull.

Contents that differ by class sit in one column per class (each ending in `Id`) with `NormalId` 0: a filled column is a class that gets the row, and its id is what that class gets. `game.box` returns them apart (`class_columns`), each with every title it has. Rows carry `ClassifyType`/`ClassifyIndex`, which gate them on an event or a piece of content; `chests.py` ignores them.

The honing tables (`ItemEnhanceCommon`, `ItemQualityOption`, `ItemEnhanceMaterial`) are read by `scripts/game_files/honing.py`: see "Honing data in the game files" in `Calculator core/Domain model.md`.

**Which client.** The tables and atlas are from the client after the October 2026 update (archives dated 2026-10-21), and every generated file is from it. The update added the Serca vambrace (ItemEnhanceCommon group 411102), the Serca books ("Tremor"), Paradise key rows and the tables `ClassItemMapping` and `ZoneEpicGateBossChallenge*`, and stored juice rates ten times finer (out of 100,000).

**Pre-update snapshot.** `ignored/pre_oct_2026_update_game_files/` holds the client as of 2026-10-19, before the update: raw `.lpk`s, every table decrypted, the extracted tables and atlas, and the tooling. Its `README.md` says what is where. Its `dump_all` decrypts any archive whole (all the tables in `data2.lpk`, about 40 s), which is how two clients were compared table by table.

## The icons

An icon is named `<Icon>_<IconIndex>` in lower case (`use_12_91`). `IconInfo.loa` lists them all with the sheet each is on and its rectangle there. **The index is not a position**: the sheets are packed, `use_12_91` is the 30th cell of `use_12`, and `money_15` is on `money_0`.

- The file is a 16-byte header with the count at offset 12, then per icon: name (`Use_12_91.png`), sheet, x, y, width, height, language, 8 zero bytes. A string is a 4-byte length and that many bytes ending in a NUL; an empty one is the length alone.
- Item icons are 64 px on 1024 px sheets, or 128 px (`shop_icon_*`).
- An icon with text on it has a sheet per language, named `<icon>_usa_english`; `game.icon_art` takes that one when there is one.
- A few of the icons that items name are not on the sheets. The art is lossless.

## chests.py

`python scanner_setup/game_files/chests.py` writes the whole table; nothing is picked by hand. A chest is in it when it opens, through any number of chests inside it, to a title of `scanner_setup/templates/items.json`. So adding a material there (or an alias of one) and running this again brings in the chests that hold it.

- **Gold and silver.** A chest that reaches no material is in only when everything it opens to, all the way down, is gold alone or silver alone (and nothing of it differs by class). Silver with a potion, or gold with silver, is out. A chest that is in for a material still lists the gold and silver it holds.
- **What opens.** Everything in `RandomBoxBase`, and bars of gold: an item with `GainMoneyType` 1 (silver) or 2 (gold) and a `UseGainCount` is written as a chest that obtains all of that much. A bar has no chest panel on its tooltip; its hover is told by its title (`Tooltips.md`, "Bars of gold").
- **A row** is `id`, `title`, `icon` (the art's name), `rarity`, `kind`, `contents` (title, amount and the id of the chest it is, when it is one of this table), and, when they apply, `level`, `extra`, `top`, `binds`, `content_binds`, `descs`, `irrelevant` and `by_class`. "(Bound)" is cut off titles.
- **Bars merge by how their gold is bound.** Items that read the same are one row, but bars whose gold is bound differently are not, though their title, art and rarity are the same ("50,000 Gold Chest" is four items, one per band). A bar row has `descs`, the English descriptions of its items, which is all that tells them apart on a tooltip (`Tooltips.md`, "Bars of gold").
- **`binds`** is how the row's items are bound, as the scanner's `Tradability` names. A row is several items, so it can be several ways: character only, roster only, either, or all three (the Destiny Shard Pouches). Only the items that can sit in a slot count, or all of them for a row none of whose can; irrelevant rows have none. The scanner takes a chest's tradability from it when no tooltip gave one (`Tooltips.md`, "A chest's tradability also comes from the game's table"). Materials gain little: the tradable, bound and "(Roster)" items of one material share an icon.
- **`content_binds`** is how each content is bound, in the order of `contents`: a content is an item with a `BindType` and `BindTarget` of its own. Only "RosterBound" and "Tradable" are written, and nothing for a currency or a content bound to a character, which is never looser than its chest; the field is left out when nothing is.
  - **Gold always has an entry.** It is a currency, with no item behind it, so its band is the chest's own: a bar's from `Item.Category` (`GOLD_BANDS`: 34300 plain, 34301 and 34302 deposit, all "Tradable"; 34303 "RosterBound", 34304 "CharBound"), which every gold item has and which agrees with every description ("Use to obtain 10,000 Character-bound Gold.", "The obtained Gold is Roster-bound.", "restricted for the first 3 days"). Any other chest's gold is "Tradable": their descriptions say nothing. The manifest and the slot status read it (`Manifest.md`).
  - The looser ones are all in obtain-all and random chests (the Emergency, Harvest and Abundance Honing Materials Chests give tradable things only; "Raid: Argeos" and the Achievement Chests give into two bands). No select-one has an option, however deep, that is looser than the chest.
  - One list for a row is exact: over every item of every row it gives the band the item's own contents would. Rows that list the same things would differ in one case only, which needs a character-bound chest neither has.
- **`level`** is `Item.ReUseBalanceLevel`, the item level the chest asks for. The game writes it across the slot (`Pipeline.md`).
- **`extra`** counts the contents that differ by class, which are left out; the tooltip still has a row for the player's class, so the reader allows that many rows it cannot place.
- **Items that read the same are one row** under the smallest id: same title, icon, rarity, kind, level, contents, `extra` and gold band. Mostly these differ in how they are bound. Chests are compared after the chests inside them were merged, so it goes round until nothing merges.
- **`top`** rows can sit in a slot and get an icon template: they have an English name, are not past `ExpireDeadline` on the day the script runs, and their `Tier` is 0 or 4 (1 to 3 are older tiers; most chests have 0). The other rows are only there for being inside a top one, and carry no icon.
- **Irrelevant by name**: a chest with "Cube" or "Engraving" in its title (`UNWANTED`) never counts as opening to a material, and neither does a chest that only matters through one.
- **Irrelevant rows** (`irrelevant: true`): every other box that is drawn with the art and rarity of a top row, so that a slot holding one can be told from the chest it looks like (`Slot grid.md`). It has to have an English name and not be expired; its tier does not matter, and it needs something listed. Its contents are what its tooltip lists, with no chest ids. Rows that read the same are one. Ones with `level` matter to the slot reader (`Tooltips.md`, "Irrelevant chests").
- **`by_class`** (irrelevant rows): the chest has contents that differ by class. Its contents then hold every title any class has, one entry each, and the reader only asks that each row of the tooltip be one of them (`Tooltips.md`). Relevant rows keep `extra`.
- **A card is listed without "Card"**: the tooltip says "Lumencaligo" for the item "Lumencaligo Card" (`Item.Category` 32000, `game.listed`).
- **Not in the table: chests the client has no contents for.** Chests by category with no `RandomBoxBase` row (old gear chests, "Raid: Ur'nil", the "Illusion Gift Chest" and "Cryptic Treasure Chest" that give gold) are left out on purpose: such a slot reads as the chest it is drawn like, and is set by hand (`Game files tried.md`). Also not in: an irrelevant box with nothing listed, and the relevant boxes of tiers 1 to 3.
- **Left out by art** (`UNWANTED_ICONS`): a chest drawn with one of its icons is dropped the same way. That is five quest-chest icons (`all_quest_02_*`) and `use_11_161` to `163`, the three Conquest Campaign Upgrade Chests (Normal, Hard, Nightmare: one-time catch-up rewards), whose art and rarity skin chests share, so a slot of them could never be told apart without a hover.
- **The art** goes to `scanner_setup/templates/ChestIcons/<icon>.png`, 64 px, cut lossless from the sheets. `make_msg_pack.py` names each template `<icon>@<rarity>`. Rows past their expiry date drop out each time it is run.

Some groups of chests list exactly the same rows and still open differently, because a chest inside them is another one under the same title (the two "Trailblazer Supplies Chest: Crucible Level 3", say). Those cannot be told apart by their tooltip, and where their icon is the same too they end up "Not included" in the manifest.

### Aliases in items.json

A material goes by other titles that are drawn alike: "[Event] Artisan's Metallurgy: Level 1", "Destiny Leapstone (Roster)", "[Event] Metallurgy: Hellfire [11-14] - Bound". Each has its own row in `scanner_setup/templates/items.json` with the material's icon, rarity and label. To find more, list the items that share an icon and a rarity with a known material and have another title. "Enhanced Metallurgy: Hellfire [19-20]" shares the book's art too and was left out, not knowing what it is.

The Serca books ("Metallurgy: Tremor [12-15]" and the like) are drawn with the Hellfire books' art (`use_12_218` weapon, `use_12_219` armour) on the ancient background, where the Hellfire ones are relic. A template is one icon on one rarity, so they have icons of their own, "Serca Weapon Book" and "Serca Armor Book" in `scanner_setup/templates/Icons`, cut lossless from the atlas, and a `body` ("Weapon 12-15") like the Hellfire rows.

## decoys.py

`python scanner_setup/game_files/decoys.py`, after `chests.py` and after any change to `items.json`: a decoy is whatever is not ours, so the list follows the tables. What the scanner does with it is in `Pipeline.md`, "Slots".

- **The pool** is every icon of a named item on every rarity some item has it on. The art and rarity of any item titled like a material of `items.json` are left out, since our material art is a lossy copy of the game's and would be its own decoy.
- **A decoy** is a pair that renders within `NEAR` (0.93) of one of our templates and under `SAME` (0.99, the same picture, which a duplicate of our art under another name is). Rendered as a template is at 1080p, 46x29 without its outermost pixels, and scored as the scanner scores a slot. The margin under the slot limit covers a slot scoring a little off what its template would (`Icon matching bench tried.md`).
- **A row** of `decoys.json` is `icon`, `rarity`, the template it is `like` and how `alike`. The art goes to `scanner_setup/templates/DecoyIcons/<icon>.png`.
- **A decoy is not known to be irrelevant, only not in our tables.** A chest that should count and is missing from `chests.json` becomes a decoy, and is then refused where before it was only unrecognised.
- Most of the pool can never sit in these inventories (mounts, sail glyphs); they do no harm.
- It reads the icons from `icons.npz`, which `find_icon.py` makes on its first run.

## Bonus chests

After clearing a raid gate the game offers a chest of more of its rewards for gold ("bonus chest"). It is no item: it is a drop, in the tables of every raid from Argos and Valtan to Serca.

- **`StagePerReward`**: a row per gate of one raid at one difficulty (`PrimaryKey`), the gate in `GateIndex`. The bonus chest is `PaidRewardDropIndex`; its price is the row of **`PaidActionCost`** with `PrimaryKey = PaidRewardType` (80, or 79 for the old legion raids) and `SecondaryKey = PaidRewardCostSecondaryKey`, the gold in `GameMoneyAmount`. `RewardDropIndex` is the free clear reward; `FirstClearRewardDropIndex` the first clear's.
- **The contents** are `DropBase` and `DropEntity`, read as for chests (amount times `Repetition`). A `DropEntity` group of several rows is one of them per repetition (accessories: necklace, earrings or ring; card packs: Legendary, Epic or Rare), with no odds in the client (`DisplayRatio` 0). A `DropBase` row with `RewardType` 1 and no entity is a smart drop (`SmartDropIndex` into `SmartDrop`, its `DropIndex` and `Fail_DropIndex`), which for these chests are Ark Grid cores of the player's class: one row per core. Every row is gated by a `ClassifyType` 42 key of the raid.
- **Names**: `ZoneEpicGate.StagePerRewardId` is the `StagePerReward` key. The raid is `EnterUIName`, or `GroupIdTitle` (Horizon Cathedral), on one row of the zone (`PrimaryKey`) only; the old legion raids have neither and are named by the first line of `EnterUIDesc`. Its `SecondaryKey` is the difficulty: 0 Normal, 1 Hard, 2 Nightmare, 4 Solo, 5 Extreme; Horizon Cathedral's levels are in `ZoneLevelDisplayName`.
- **Item levels**: the `ContentsRequirement` row whose `PrimaryKey` is the gate's `ZoneEpicGate.ClassifyIndex` has the item level a difficulty asks for in `OptimalItemLevelMin`. Its `Name` is a short title, "Witch of Agony, Serca [Normal]", "Armoche [Hard]".
- **`bonus_chests.py`** writes one choice per gate whose chest holds a material of `items.json` other than gold and silver (titles with " (Bound)" cut). Difficulties that sell the same chest for the same gold are one row, and one difficulty ("Normal / Solo"). A row is `id`, `group` (the raid), `title` ("Gate 1"), `difficulty`, `item_level`, `band` 0, `options` (`{}` and the contents with `Gold` negative, by label) and `others`, what else it holds as text. The materials are taken as char bound; their items are "(Bound)".

## Paradise

Paradise is "Hell" in the client (`TrinityInferno*`), the sekwahar "hell_drops" page's data. A run ends on a reward level, Base or Level 1 to 10; the game offers three of that level's main rewards and the player takes one, in Destiny with the additional rewards, or the Wealth ones when the run had Wealth. Netherworld has neither: its rewards come alone.

- **`TrinityInferno`**: a row per key item (`SecondaryKey`) that opens a season's Paradise (`PrimaryKey`: 4003 Destiny and 4004 Netherworld are season 4, 3003/3004 season 3). `InfernoType` is the mode: 1 and 2 Destiny ("Hell Key", "Hell Key of Destiny"), 3 and 4 Netherworld (Flame and Frost keys). The key's `ReUseBalanceLevel` is the item level. `InfernoRewardId` is the reward set. `ClassifyType` 19 rows are the keys sold all season; 21 and 35 are events.
- **`TrinityInfernoReward`**: a row per reward set (`PrimaryKey`) and level (`SecondaryKey` 1, 10, 20 ... 100: Base, Level 1 ... 10). `MainDropId1` to `20` are the main rewards with weights `Ratio1` to `20` (all 10, so the three offered are drawn evenly), `SubDropId` the additional rewards and `PlentyDropId` the Wealth ones, about ten times as much, given instead of them; both are 0 on every Netherworld set. Each is a drop, read as for chests; every row of them has `DropBase.Ratio` 100 and one entity.
- **Two sets for one item level**: 1730 has 1024 (T4 materials) and 1025 (Serca), both under "Hell Key V". The newest set of an item level is the highest id (what `paradise.py` takes; the sekwahar page agrees).
- **`paradise.py`** writes one choice per mode, item level and level from the newest two seasons' `ClassifyType` 19 keys (`InfernoType` 2 and 3). A row is a `Choice` with `names` (what each option is), `options` (the main reward plus the additional rewards, by label) and `wealth` (plus the Wealth ones instead; left out where there is no Wealth, so for all of Netherworld), `difficulty` the item level, `title` the level, `band` 0 (the materials are "(Bound)" and the gold bars, category 34304, character bound). A select-one chest of materials (the stone selection chests, the advanced honing support chests) is one option per material; gold bars are their gold. What is no material is an option that gives only the additional rewards.
- **No 1770 yet** (2026-10-21): no item asks for more than 1750, and the update left `TrinityInfernoReward` as it was. Its new key rows are events whose key items are not in `Item` yet.

## find_icon.py

`python scanner_setup/game_files/find_icon.py <crop.png>`, or a capture with `--rect x,y,size`, a recording with `--frame`, `--brightness` for the in-game setting (default 50) and `--top` for how many matches to print. Ranks every item icon against one slot and prints the best with the items drawn with each; `.tmp/find_icon.png` shows the crop and the matches.

- The crop is a whole slot cut roughly. It is brought to the art's brightness with the gamma law, and each icon is scored on the closest 75% of its opaque pixels, so the slot's background, its number and the cursor do not count. The best 300 are tried again a few px off.
- The icons are cut from the sheets once into `icons.npz`, and again whenever their number changes.
- An icon with fewer than 1,000 opaque pixels is never a match, since it would match anything (Destiny Leapstone is one).
- `decoys.py` uses its icon cache (`load_icons`).
