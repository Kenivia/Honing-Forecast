# Manifest

What the scan amounts to: materials per ownership band, plus the select-one chests still to be opened. It sits between the scanner and the rest of the site, under the slot grid and dashboard on the scanner page. It does not write into the calculator's state yet; that is the next step, where chest opening becomes a state decided up front like the special state.

Code: `Manifest.ts` (build logic, `scanned` and `manifest` computeds), `Manifest.vue` (the page), both in `InventoryScanner`. Overrides live in `ScanStore.ts` so a new capture wipes them. Everything is session-only.

## Building it

- **From slots, not the chest list.** A chest entry can stand for several stacks, so counts come from the slots. Only slots that are Good (assumed tradability included) or edited by hand count. Every other recognised slot goes into `missing`, with its place and why.
- **An edited slot never counts as missing.** No amount is a single item, and no tradability is what is assumed for its window; with neither it is left out. An edited chest slot whose chest was never read goes under "Not included".
- **A chest slot** is resolved with `chests_for` (the same lookup the dashboard uses), preferring the chest whose stacked amount is the slot's. With no chest read, it is whatever the slot's icon can only be (the result's `variants`), which covers bars of gold and the chests that are alone in their art.
- **Several chests have to open alike.** A read chest, or a slot's icon, can be several rows of the table. `settle` takes them as one when they open to the same things all the way down (`opens_to`, in any order). When they do not, because a chest inside them is another one under the same title, the stack goes under "Not included".
- **Everything inside a chest takes the chest's band**, whatever the row's "(Bound)" says: the user picks the character it is opened on.
- **Obtain-all** chests are their contents. They go into `extra`, the k of the "n+k" a cell shows. A chest inside one is opened in turn, so an obtain-all of select-one chests shows as those select-one chests.
- **Select-one** chests are the only ones passed on. Each option is a bag of `label -> amount`. A chest listed as an option is flattened: an obtain-all into its contents, a select-one into its own options. An option that is no material and no chest of the table (Solar Grace, an astrogem chest) or a chest of chance is not offered, and the chest's other options go through. Identical chests in one band are grouped and their counts summed.
- **Random** chests are listed apart, and chests that could not be settled under "Not included"; neither goes further.

## Names and inner chests

- `templates/items.json` rows carry `label`, the material label in `Constants.ts`. Slots map by `icon`, the contents of chests by `title`. An icon that rows with different labels share (the books) maps to nothing: such a slot counts under its own `label`, or, set by hand, under the material it was set to (`SHARED_ICONS`). Gold and Silver are never scanned from a slot; a chest's "Gold" and "Silver" contents count under those labels.
- **What a chest holds comes from `templates/chests.json`** (`Game files.md`), by id. A content is a title, an amount and, when it is a chest of the table itself, that chest's id, so a chest inside a chest is opened exactly, however deep, and its title is never matched. The old `inner_chests.json`, filled by hand per title, is gone.
- Rust reads the same two files. A change there needs `pnpm run wasm` before the scanner reads it.

## The page

- Hidden behind a warning while any slot is missing; the user can open it anyway, and those slots are simply not in it. Under the warning is the list "Missing slots": where each one is (`slot_place`), what it holds and the scanner's reason for it.
- Two tabs, one per tier. A material shared between tiers shows on both with one value. A chest shows on a tab when one of its options holds a material of that tier.
- **Adding a chest by hand.** The "Add a selection chest" card under the chest list takes a count, an optional name, a band and options of one material each (those of the tier shown). It goes into `added_chests` in `ScanStore.ts`, joins the scanned chests in `manifest` (so it converts and takes a count override like them), has a Remove button, and is wiped by a new capture like the overrides.
- **Cells are inputs.** A cell shows the override, else `n+k`, else nothing with a `?` placeholder. Typing sets an override (arithmetic works); emptying it goes back to the scan. Chest counts can be overridden the same way.
- **Conversion toggles**, one per band, kept in `convert`. The char-bound one is locked off for a T4 character and defaults on for a Serca one; they reset when the character or its tier changes. Conversion runs after overrides, at the `CONVERTS_FROM` ratio: the T4 remainder stays, the gain lands on the Serca label, and tradable comes out roster-bound. The gain shows as a read-only "+c from T4" beside the Serca input, so the way to change it is to override the T4 cell.
- Chest options convert too, without rounding, since a count multiplies them later. The chest keeps its own band, which is not settled for a converted tradable chest.
