# Manifest

What the scan amounts to: materials per ownership band, plus the chests still to be opened, merged with the pages saved by earlier exports and with the other characters of the roster. It sits between the scanner and the rest of the site, under the slot grid and dashboard on the scanner page. Nothing in it is opened: every chest is passed on whole, with both tiers in it, and how each stack is opened is a state the calculator core decides (`Calculator core/Domain model.md`, "Chests"). It reaches the calculator only through the export button. It is read-only; corrections are made on the slots before export, or on the budgets and chests of the calculator after it.

Code: `Manifest.ts` (build logic, `live` and `manifest` computeds, the roster cover, `export_manifest`), `Manifest.vue` (the page), both in `InventoryScanner`. Saved pages, their merging and the budget snapshots are in `Utils/Scans.ts`. The chest types, `merge`, `to_serca` and `process_chests` are in `Utils/Chest.ts`, and the two chest panels are `Common/ChestPanels.vue`, shared with the calculator page. What was tried and dropped is in `Manifest tried.md`.

## Building it

- **Per page.** `live` keeps what each page holds apart (`PageContent`, keyed `"<window> <page>"`), and every currency is a page of its own (`CURRENCY_PAGES`): `Currency <n>` for its roster-wide bands, `CharCurrency <n>` for its char-bound one. A page is seen once it was open in game during this capture (`seen_pages` in `ScanStore.ts`, from each result's `pages`) or anything on it counts; a currency once it is good.
- **From slots, not the chest list.** A chest entry can stand for several stacks, so counts come from the slots. Only slots that are Good (assumed tradability included) or edited by hand count. Every other recognised slot goes into `missing`, with its place and why.
- **An edited slot never counts as missing.** No amount is a single item, and no tradability is what is assumed for its window; with neither it is left out. An edited chest slot whose chest was never read goes under "Not included".
- **A chest slot** is resolved with `chests_for` (the same lookup the dashboard uses), preferring the chest whose stacked amount is the slot's. With no chest read, it is whatever the slot's icon can only be (the result's `variants`), which covers bars of gold and the chests that are alone in their art.
- **Several chests have to open alike.** A read chest, or a slot's icon, can be several rows of the table. `settle` takes them as one when they open to the same things all the way down, gold bound the same way (`opens_to`, in any order). When they do not, because a chest inside them is another one under the same title, the stack goes under "Not included".
- **What is inside a chest takes the chest's band, or its own where that is looser** (`content_binds` of `chests.json`, `Game files.md`): a tradable material from a character-bound chest is tradable ("Emergency Honing Materials Chest (S)"), and a character-bound one from a roster-bound chest is roster bound, since the user picks the character it is opened on. Gold too: a roster-bound bar of character-bound gold gives roster-bound gold, since the bar can be moved and opened on any character (the table's band for gold is in `Game files.md`). A currency with no entry, or anything the table does not say, takes the chest's. A chest inside takes its band the same way and hands it on.
- **An obtain-all chest is listed once for each band it gives into**, under that band and not its own: "Raid: Argeos", character bound, is one row of tradable stones and one of character-bound leapstones. Each row has its own count and `x`.
- **A select-one's options all take the chest's band.** No select-one of the table has a looser option or gold, so `alternatives` does not look.
- **An obtain-all chest is a chest with one option**, the bag of its contents, in the same list as the others. An obtain-all inside one is part of that bag; any other chest inside is opened in turn, so an obtain-all of select-one chests shows as those select-one chests. Its contents are **not** added to the material cells.
- **Select-one**: each option is a bag of `label -> amount`. A chest listed as an option is flattened: an obtain-all into its contents, a select-one into its own options. An option that is no material and no chest of the table (Solar Grace, an astrogem chest) or a chest of chance is not offered, and the chest's other options go through. Chests of one band and kind that give the same are one stack (`add_stack`): counts summed, titles joined with " / ". The order the options, or the things in an option, were listed in does not matter; options are kept sorted.
- **`obtain_all` says which kind the scanner found.** A select-one with one option left is no choice, but it stays a select-one (`obtain_all: false`), as does a chest added by hand. The user deletes it if they would take the option that was not offered.
- **No chest holds a chest.** An option is `label -> amount`, so nesting cannot be passed on. The table nests five deep (October 2026: 21 select-one in select-one, 83 obtain-all in select-one, 104 select-one in obtain-all among the top rows) and all of it is flattened as above. The one hole: an obtain-all option that holds a select-one with a real choice would lose that part in `alternatives`; no chest of the table is like that.
- **Random** chests are listed apart, and chests that could not be settled under "Not included"; neither goes further.
- **Currencies** come from the scan result's `currencies`, not from slots (`Currency.md`), with their bands: silver roster bound, shards character bound, gold as its tooltip splits it. They add to what chests give. One that is not good yet is listed as missing, under "Currencies", and its saved value is used.
- **Irrelevant chests are nothing.** A slot the scanner marked irrelevant is skipped like any other, and a chest slot set by hand whose chests are all `irrelevant` rows of the table opens to nothing.

## Names and inner chests

- `scanner_setup/templates/items.json` rows carry `label`, the material label in `Constants.ts`. Slots map by `icon`, the contents of chests by `title`. An icon that rows with different labels share (the books) maps to nothing: such a slot counts under its own `label`, or, set by hand, under the material it was set to (`SHARED_ICONS`). Gold and Silver are never scanned from a slot; a chest's "Gold" and "Silver" contents count under those labels.
- **What a chest holds comes from `scanner_setup/templates/chests.json`** (`Game files.md`), by id. A content is a title, an amount and, when it is a chest of the table itself, that chest's id, so a chest inside a chest is opened exactly, however deep, and its title is never matched. The old `inner_chests.json`, filled by hand per title, is gone.
- Rust reads the same two files. A change there needs `pnpm run wasm` before the scanner reads it.

## Saved pages and other characters

Pages persist (`scan_pages` in the roster config, `ScanPage` in `Scans.ts`): an export saves every page it saw, replacing the saved one with the same roster, owner, window and page. Roster storage and the roster-wide currency pages belong to the roster (`char_id` null), the rest to the character (`ROSTER_WINDOWS`). `by` is the character it was scanned on.

- **A page not seen now is taken as last exported** (`own_pages`), the roster's and this character's. As soon as its page is open in game, the live one replaces it, complete or not; what it misses is in the missing list.
- **The other characters of the roster** (`other_pages`) add their saved pages' roster-bound and tradable bands and chests. Their char-bound bands stay theirs.
- **Accepted:** an item moved to a page not shown again, or to another character, is counted twice. The user is told which pages are saved (the tab marker in `Slot grid.md`, and the "From earlier exports" card) and is expected to show what changed.
- Pages of a deleted character or one moved to another roster are left in the save and ignored.

## Roster storage covered

Roster storage is shared, so it need not be shown again for every character. `roster_covered` says it is taken wholly from its saved pages, and while it is, everything read there is ignored (not counted, not missing, not saved). It is covered by default when a roster page was ever saved and the roster's roster-bound and tradable budgets are as the latest snapshot has them (`roster_unchanged`; the bound budgets are not compared). `roster_cover` in `ScanStore.ts` is the user's say either way, reset by an export and by a wipe; a scan moved to another character of the same roster keeps it. The grid shows it (`Slot grid.md`).

## Export

`export_manifest`, in this order:

1. Every missing slot (one with an address) is set irrelevant by an edit, so it counts as nothing and is saved that way.
2. The manifest is read, **before** the first snapshot: the snapshot can make roster storage covered by default and drop what was just read there.
3. A "Before export" snapshot of the roster (below).
4. The character's bound budget is overwritten with its own char-bound band; the roster's roster-owned and tradable columns with the merged roster-bound and tradable bands (other characters included); the free-tap budget with the sum of the three; a cell nothing holds is 0. Char-bound chests go to `profile.chests`, the rest to `roster_chests[roster_id]`, unconverted.
5. The pages seen are saved, then an "After export" snapshot, and the cover goes back to its default.

## Snapshots

`snapshots` in the roster config: the whole roster's budgets at a time (`Snapshot`: roster-owned, tradable and roster chests, and per character id its name, bound and free-tap budgets and chests, non-zero values only). Taken before and after each export, and by hand on the Snapshots page; one equal to the roster's latest is not added. They are of the budgets, not the scan, so the user's edits and auto-deducted costs between two exports show as the difference from one's "after" to the next's "before". `snapshot_diff` gives per label the change of char bound (one character or all), roster bound, tradable and free taps, and the chest stacks whose count changed (by key). The Snapshots page (`SnapshotsView.vue`, `/snapshots`) lists them with From and To radios (To can be now) and a delete each.

## The page

- Hidden behind a warning while any slot is missing; the user can open it anyway, and those slots are simply not in it (and set irrelevant by an export). Under the warning is the list "Missing slots": where each one is (`slot_place`), what it holds and the scanner's reason for it.
- Two tabs, one per tier, for the materials. A material shared between tiers shows on both with one value. The chest panels do not follow the tab: every chest shows, as merged.
- **Two chest panels side by side**, "Selection chests" (`obtain_all` false, a select-one with one option left included) and "Obtain-all chests" (strictly those), with the same rows: count, title, band, contents.
- **Amounts are read-only text**, what the export writes. There are no overrides.
- **"From earlier exports"** lists the saved pages in use (who scanned them and when) and the other characters whose roster-bound and tradable things are added.
- **Conversion toggles**, one per band, kept in `convert`. The char-bound one is locked off for a T4 character and defaults on for a Serca one; they reset when the character (by id) or its tier changes. Conversion runs on the merged amounts at the `CONVERTS_FROM` ratio: the T4 remainder stays, the gain lands on the Serca label, and tradable comes out roster-bound. The gain shows as "+c from T4". Saved pages are stored unconverted.
- **The toggles are for materials only.** Chests keep their T4 materials whatever the toggles say; they are converted on the calculator page, for the character's tier (below).

## On the calculator page

`process_chests(chests, tier)` in `Utils/Chest.ts` turns the character's chests and its roster's (`active_chests`) into what the calculator shows and the payload sends. Nothing of it is stored. Per chest:

- **Serca character**: every T4 material becomes the Serca one at the `CONVERTS_FROM` ratio, unrounded (the panel shows at most one decimal, so whole results show whole). A chest with anything converted is roster bound at best, all of it: a stack has one band.
- Labels of the other tier are dropped, and gold of a tradable chest (never counted). An option left empty is gone, which hides every Serca option from a T4 character.
- **An option that another is at least as good as, label for label, is gone.** Of two alike the first stays.
- A chest with no option left is gone. Chests that come out with the same band and options, in any order, are one stack (`add_stack` again): counts summed, titles joined. The key is the band and the options, and it is what `chest_states` is keyed by.

The panels there are "Selection chests" (several options left) and "Obtain-all and trivial selection chests" (one). `tests/chests.test.ts` covers this.

**"Edit chests"** above them swaps in `ChestEditor.vue`: the stored chests, unprocessed, with a count input and an `x` each and the "Add a chest" form (count, optional name, band, options of one material of the character's tier each). A char-bound chest is added to the character, any other to the roster. `tests/scans.test.ts` covers the merging, the cover and the snapshots.
