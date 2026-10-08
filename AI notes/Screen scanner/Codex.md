# Codex scripts

`scripts/codex` turns something seen in a capture into rows of the scanner's tables, using lostarkcodex.com. The scripts are tracked; what they download (`icons/`, `items.json`, `icons.npz`) is not.

From a slot cut out of a capture to a chest in the tables:

1. `fetch_items.py` pulls the site's item list, once.
2. `fetch_icons.py` pulls the icons it names.
3. `find_icon.py` finds which icon a slot is drawn with, and so which items it can be.
4. `chest.py` writes a chest, and the chests inside it, into `templates/chest.json` and `templates/inner_chests.json`.

Then `python templates/make_msg_pack.py` and `pnpm run wasm`, as for any change to those tables.

## What the site has

- **The item list**: `query.php?a=items&l=us`, one 60 MB response with all 130,104 items. `fetch_items.py` keeps id, title, rarity and icon name of each in `scripts/codex/items.json` (9 MB).
- **Icons**: `/icons/<name>.webp`, 64 px lossy WebP (128 px for `shop_icon_*`). A name the site has no art for gets an 812-byte placeholder, which `fetch_icons.py` recognises by asking for a name that cannot exist. The files are kept flat under the site's names (`use_12_218.webp`). The list names 17,869 icons and the site has art for all but 106; the folder holds 23,275, since an earlier downloader counted file names upward and also got 5,512 that no item uses.
- **What a chest holds**: `query.php?a=drop&t=box&id=<item id>`, each content with its item id and amount.
- **How a chest opens** is only in the description on its item page ("Contains ... of your choice", "Choose and obtain one type", "A chest that grants all its contents"). `chest.py` reads select-one from "choice", "choose" or "select", random from "chance" or "random", else obtain-all.

## Why the result is reviewed

- **One title is several chests.** "Destiny Destruction Stone Pouch" is three items holding 200 stones, 31,500 stones, or 75 stones and 150 Refined Obliteration Stones. An icon is many items too (56 for the weapon book). `chest.py` lists every candidate and writes nothing until `--id` picks one, unless they all agree.
- **Amounts can be wrong.** The contents of Destiny Shard Pouch (S), (M) and (L) come back as 5,000, 10,000 and 15,000 while their own descriptions and the game say 1,000, 2,000 and 3,000. `chest.py` flags a chest whose description says "obtain N" with an N its contents do not have, and never overwrites a row that is already there.
- **Some items come back empty**, one of the two "Honing Support Selection Chest II" among them.

So the codex is trusted for icons and titles, and for the contents of inner chests only after a look at the diff. What a chest in a slot holds is never taken from it: that is read off the tooltip.

## find_icon.py

`python scripts/codex/find_icon.py <crop.png>`, or a capture with `--rect x,y,size`, a recording with `--frame`, and `--brightness` for the in-game setting. Ranks every icon against one slot and prints the best with the items drawn with each; `.tmp/find_icon.png` shows the crop and the matches.

- The crop is a whole slot cut roughly. It is brought to the art's brightness with the gamma law, and each icon is scored on the closest 75% of its opaque pixels, so the slot's background, its number and the cursor do not count. The best 300 are tried again up to 3 px off.
- The icons are read once into `icons.npz` (about 25 s), and again whenever their number changes. A cached run takes about 5 s.
- On the two book slots of the 22-55-45 recording the right icon came first with a clear lead (55.5 against 70.1, and 62.7 against 73.9; lower is closer).

## chest.py

`python scripts/codex/chest.py "<title>"`, or a capture with `--rect` like `find_icon.py`, which takes the closest icon and every item drawn with it. Plain chest art is shared widely (101 items for the brown chest `use_4_225`, each costing two requests), so `--title` with a part of the name narrows a capture's candidates first. On the 22-55-45 recording's unlisted chest (frame 240, rect 310,348,47) that gave three candidates, one of them "Metallurgy: Hellfire Selection Chest" with the 5, 2 and 1 books its tooltip reads as; it was not added to the tables.

- **`chest.json`** gets the chest's title, icon and rarity. The icon is the template in `templates/Icons` whose file is that art, else a new file named after the chest (`--icon` names it). A title already there is left alone, icon and all.
- **`inner_chests.json`** gets a row for every chest among its contents, and theirs. `--inner` writes the chest itself there and skips `chest.json`. Only contents the scanner knows are kept (titles in `templates/items.json`, and chests holding some); the rest are printed as left out, and a chest holding nothing known gets no row. A row with only a title is replaced; a row with `items` is kept.
- Both files are written one row a line.

## The dev server and the icon folder

Vite watches the whole project, and this folder is about 20,000 files. Renaming them all and then downloading into it had the running dev server using two cores and taking 15 s a page for as long as the download ran. `scripts/codex/icons` is now in `server.watch.ignored` in `vite.config.ts`.
