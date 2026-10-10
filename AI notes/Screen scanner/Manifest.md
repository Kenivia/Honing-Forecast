# Manifest

What the scan amounts to: materials per ownership band, plus the select-one chests still to be opened. It sits between the scanner and the rest of the site, under the slot grid and dashboard on the scanner page. It does not write into the calculator's state yet; that is the next step, where chest opening becomes a state decided up front like the special state.

Code: `Manifest.ts` (build logic, `scanned` and `manifest` computeds), `Manifest.vue` (the page), both in `InventoryScanner`. Overrides live in `ScanStore.ts` so a new capture wipes them. Everything is session-only.

## Building it

- **From slots, not the chest list.** A chest entry can stand for several stacks, so counts come from the slots. Only slots that are Good (assumed tradability included) or edited by hand count. Every other recognised slot goes into `missing`, with its place and why.
- **An edited slot never counts as missing.** No amount is a single item, and no tradability is what is assumed for its window; with neither it is left out. An edited chest slot whose chest was never read goes under "Not included".
- **A chest slot** is resolved with `chests_for` (the same lookup the dashboard uses), preferring the chest whose stacked amount is the slot's. With no chest read, it is whatever the slot's icon can only be (the result's `variants`), which covers bars of gold and the chests that are alone in their art.
- **Several chests have to open alike.** A read chest, or a slot's icon, can be several rows of the table. `settle` takes them as one when they open to the same things all the way down (`opens_to`, in any order). When they do not, because a chest inside them is another one under the same title, the stack goes under "Not included".
- **What is inside a chest takes the chest's band, or its own where that is looser** (`content_binds` of `chests.json`, `Game files.md`): a tradable material from a character-bound chest is tradable ("Emergency Honing Materials Chest (S)"), and a character-bound one from a roster-bound chest is roster bound, since the user picks the character it is opened on. A currency, or anything the table does not say, takes the chest's. **Gold never does**: it goes into the band the table gives it by the chest's art, character bound from a bar of gold and tradable from anything else, whatever band the chest is in (`Game files.md`). A chest inside takes its band the same way and hands it on.
- **An obtain-all chest is listed once for each band it gives into**, under that band and not its own: "Raid: Argeos", character bound, is one row of tradable stones and one of character-bound leapstones. Each row has its own count and `x`.
- **A select-one's options all take the chest's band.** No select-one of the table has a looser option or gold, so `alternatives` does not look.
- **Obtain-all** chests are their contents, and are listed in `opened` with one bag as their only option. An obtain-all inside one is part of that bag; any other chest inside is opened in turn, so an obtain-all of select-one chests shows as those select-one chests. `manifest` sums the listed ones into `extra`, which is added to the cells.
- **Select-one** chests are the only ones passed on. Each option is a bag of `label -> amount`. A chest listed as an option is flattened: an obtain-all into its contents, a select-one into its own options. An option that is no material and no chest of the table (Solar Grace, an astrogem chest) or a chest of chance is not offered, and the chest's other options go through. Identical chests in one band are grouped and their counts summed.
- **A select-one with one option left** is no choice: it goes into `opened` with the obtain-all chests and counts as that option. The user deletes it if they would take the option that was not offered.
- **Random** chests are listed apart, and chests that could not be settled under "Not included"; neither goes further.
- **Irrelevant chests are nothing.** A slot the scanner marked irrelevant is skipped like any other, and a chest slot set by hand whose chests are all `irrelevant` rows of the table opens to nothing.

## Names and inner chests

- `templates/items.json` rows carry `label`, the material label in `Constants.ts`. Slots map by `icon`, the contents of chests by `title`. An icon that rows with different labels share (the books) maps to nothing: such a slot counts under its own `label`, or, set by hand, under the material it was set to (`SHARED_ICONS`). Gold and Silver are never scanned from a slot; a chest's "Gold" and "Silver" contents count under those labels.
- **What a chest holds comes from `templates/chests.json`** (`Game files.md`), by id. A content is a title, an amount and, when it is a chest of the table itself, that chest's id, so a chest inside a chest is opened exactly, however deep, and its title is never matched. The old `inner_chests.json`, filled by hand per title, is gone.
- Rust reads the same two files. A change there needs `pnpm run wasm` before the scanner reads it.

## The page

- Hidden behind a warning while any slot is missing; the user can open it anyway, and those slots are simply not in it. Under the warning is the list "Missing slots": where each one is (`slot_place`), what it holds and the scanner's reason for it.
- Two tabs, one per tier. A material shared between tiers shows on both with one value. A chest shows on a tab when one of its options holds a material of that tier.
- **Two chest panels side by side**, "Selection chests" and "Obtain-all chests" (`opened`), with the same rows: count, title, band, contents.
- **Editing is behind the "Edit manifest" checkbox**, off by default and not kept. Off, the cells and chest counts are disabled and the delete buttons and the card for adding a chest are hidden.
- **Deleting a chest.** Every chest of both panels has an `x` while editing. Its key goes into `deleted_chests` in `ScanStore.ts`, and `manifest` leaves it out, so its contents leave `extra` too. There is no undo short of a new capture.
- **Adding a chest by hand.** The "Add a selection chest" card under the chest list takes a count, an optional name, a band and options of one material each (those of the tier shown). It goes into `added_chests` in `ScanStore.ts`, joins the scanned chests in `manifest` (so it converts and takes a count override like them), and is wiped by a new capture like the overrides. It stays in the selection panel even with one option.
- **Cells are inputs.** A cell shows the override, else what the slots hold, else nothing with a `?` placeholder. Typing sets an override (arithmetic works); emptying it goes back to the scan. What the obtain-all chests add is a read-only "+k from chests" beside the input and is added on top of an override; the way to change it is the chest's count or its `x`. Chest counts can be overridden the same way.
- **Conversion toggles**, one per band, kept in `convert`. The char-bound one is locked off for a T4 character and defaults on for a Serca one; they reset when the character or its tier changes. Conversion runs after overrides, at the `CONVERTS_FROM` ratio: the T4 remainder stays, the gain lands on the Serca label, and tradable comes out roster-bound. The gain shows as a read-only "+c from T4" beside the Serca input, so the way to change it is to override the T4 cell.
- Chest options convert too, without rounding, since a count multiplies them later. The chest keeps its own band, which is not settled for a converted tradable chest.
