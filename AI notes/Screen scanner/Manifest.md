# Manifest

What the scan amounts to: materials per ownership band, plus the select-one chests still to be opened. It sits between the scanner and the rest of the site, under the slot grid and dashboard on the scanner page. It does not write into the calculator's state yet; that is the next step, where chest opening becomes a state decided up front like the special state.

Code: `Manifest.ts` (build logic, `scanned` and `manifest` computeds), `Manifest.vue` (the page), both in `InventoryScanner`. Overrides live in `ScanStore.ts` so a new capture wipes them. Everything is session-only.

## Building it

- **From slots, not the chest list.** A chest entry can stand for several stacks, so counts come from the slots. Only slots that are Good (assumed tradability included) or edited by hand count. Every other recognised slot is `missing`.
- **An edited slot never counts as missing.** No amount is a single item, and no tradability is what is assumed for its window; with neither it is left out. An edited chest slot whose chest was never read goes under "Not included".
- **A chest slot** is resolved with `chests_for` (the same lookup the dashboard uses), preferring the chest whose stacked amount is the slot's.
- **Everything inside a chest takes the chest's band**, whatever the row's "(Bound)" says: the user picks the character it is opened on.
- **Obtain-all** chests are their contents. They go into `extra`, the k of the "n+k" a cell shows. A chest inside one is opened in turn, so an obtain-all of select-one chests shows as those select-one chests.
- **Select-one** chests are the only ones passed on. Each option is a bag of `label -> amount`. A chest listed as an option is flattened: an obtain-all into its contents, a select-one into its own options. Identical chests in one band are grouped and their counts summed.
- **Random** chests, and chests whose contents are not known, are listed under "Not included" and go no further.

## Names and inner chests

- `templates/items.json` rows carry `label`, the material label in `Constants.ts`. Slots map by `icon`, chest rows by `title`. Books have no title there, so book slots are not counted yet. Gold and Silver are never scanned and stay `?`.
- A tooltip only names a chest inside a chest, so what it holds comes from `opens` (`kind` and `items`, title to amount) on its row in `templates/chest.json`. A row without `opens` is "contents unknown" when it turns up inside another chest. The two `Dummy ...` rows are placeholders for an obtain-all of select-ones.
- Rust reads the same two files and ignores the extra fields. A new title there needs `pnpm run wasm` before the scanner reads it.

## The page

- Hidden behind a warning while any slot is missing; the user can open it anyway, and those slots are simply not in it.
- Two tabs, one per tier. A material shared between tiers shows on both with one value. A chest shows on a tab when one of its options holds a material of that tier.
- **Cells are inputs.** A cell shows the override, else `n+k`, else nothing with a `?` placeholder. Typing sets an override (arithmetic works); emptying it goes back to the scan. Chest counts can be overridden the same way.
- **Conversion toggles**, one per band, kept in `convert`. The char-bound one is locked off for a T4 character and defaults on for a Serca one; they reset when the character or its tier changes. Conversion runs after overrides, at the `CONVERTS_FROM` ratio: the T4 remainder stays, the gain lands on the Serca label, and tradable comes out roster-bound. The gain shows as a read-only "+c from T4" beside the Serca input, so the way to change it is to override the T4 cell.
- Chest options convert too, without rounding, since a count multiplies them later. The chest keeps its own band, which is not settled for a converted tradable chest.
