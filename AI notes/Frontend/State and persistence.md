# State and persistence

The shape of the frontend state and how it is saved.

## Pages

- **Roster setup**: manage characters and rosters, set each roster's region, import characters.
- **Market & mats**: roster-owned and tradable materials, market prices.
- **Income is dev only** for now: its route and sidebar link exist only under `import.meta.env.DEV`; snapshotting still runs in every build.
- **Character page**: one per character, with sub-pages as child routes. The calculator sub-page is the main one: upgrade tickbox grids, material budgets and distribution graphs, the chest panels (the character's chests and its roster's, `active_chests`, as `process_chests` leaves them for the character's tier; "Edit chests" swaps in the stored ones, editable; `ChestPanels` is one collapsible card on the calc and scanner pages, the lists side by side, rows on a subgrid so their columns line up; the obtain-all list is hidden until its checkbox is ticked and the selection list takes two columns meanwhile; each list scrolls past `max-h-80`; open and shown are models Calc holds, so toggling Edit chests keeps them, none of it saved), whose selection panel also shows how each stack is provisionally opened (`ChestPanels`' `states`, the histogram's `state_bundle.chest_states`, so it follows the optimizer's interim states like the graphs do; per option how many chests, only for a state that fits, the same rule as `chest_gains`, else the options joined by "or"), optimizer controls, then the generated instructions. The costs distribution's owned column (bound, roster or tradable owned; the tickbox stays the bound column's trade tick), chance column and graph lines follow the character's `chance_band`, picked in the owned column's header: which lines show is derived from it (`annotations_shown` in `MaterialDist/DistLines.ts`), not stored. A tickbox grid's header click moves only the rows at the column's status (`click_column` in `Utils/StatusGrid.ts`); a row already past it is left alone. The other sub-pages are the guide, the scanner (listed last in the sidebar with a "beta" tag, the footer's `.new-tag` from `shared.css`; no banner, the scanner page's guide has the limitations and where to report issues; without `getDisplayMedia`, i.e. on mobile, the page shows a warning instead and downloads nothing until "Show scanner anyway", which `scanner_shown` in `ScannerUIutils.ts` keeps for the session, sidebar control panel included), Income (the roster's budget snapshots and income per week, `Screen scanner/Income.md`) and Choices (what a raid's bonus chest is worth, `Screen scanner/Choices.md`).
- **Change logs**.

The character page uses nested routes: `CharView` owns the sidebar and renders the sub-page through `<RouterView/>`, keying the calculator by profile id so a character switch remounts it.

## Two stores

The split is by lifetime, and it is what keeps the save file honest.

- **`useRosterStore`** (`RosterConfig.ts`) holds one `roster_config` object. Everything in it is persisted; there is no opt-out list.
- **`useRuntimeStore`** (`RuntimeState.ts`) holds everything session-only: the per-character worker bundles, the scanner's cropper bundle and frame source, `budget_snapshots`, `adv_cache`, and the in-flight fetch / details-update flags. None of it is ever written.

Data in `roster_config` lives at four scopes:

| Scope | Examples |
| --- | --- |
| Per character | Upgrade grids, keyed upgrades, bound material budgets, free-tap budget, chests, tier, band values, optimizer settings. |
| Per roster | Roster-owned materials, tradable materials, roster chests, region, the scanner's saved pages and the budget snapshots (both lists in the config, filtered by roster), manual roster-bound and tradable income, the Choices page's custom choice. |
| Per region | Market prices per bundle, the chosen bundle per material, the last fetched market response. |
| Global | UI flags, the active character index, last seen version, whether the scanner's beta banner was dismissed, whether snapshots are shared (`analytics`). |

The active character is set from the route, and most components read through `active_*` getters.

Every profile has an `id` (`crypto.randomUUID()`), and everything internal identifies a character by it, never by `char_name`. The name is only for display, the route, lostark.bible requests, and matching imported in-game characters to profiles. The route `/:characterName` is checked by one global `router.beforeEach` (`Router/index.ts`), so it runs on every navigation, param-only changes included: an unknown name is replaced by the first character's, keeping the sub-page, query and hash (change-log versions are checked there too). `CharView` then only maps the name to the active index (`profile_index_by_name`), and from there the active index and ids take over. Renaming the active character replaces the route with the new name, so a reload finds it. `new_char_profile` makes a fresh id and duplicating a character replaces the copied one; `reset_active_profile` keeps it, so the page is not remounted: "Reset this char" clears the character's worker results first, as `change_tier` does, since a Serca result read against T4 labels throws. An empty name becomes `Newchar`, numbered like any taken name (`format_char_name`).

Every tab saves the whole config, so the last to save wins. Nothing merges: `OtherTabBanner` only warns while another tab is open (a `BroadcastChannel` in `Utils/OtherTabs.ts`, each tab greeting the others once per page load and saying goodbye on `pagehide`). Both banners sit under the header, except on character pages, where `CharView` puts them at the top of the main area instead, since the fixed sidebar covers the space under the header.

Character and roster changes are store actions in `RosterConfig.ts` (`add_char`, `add_roster`, `duplicate_profile`, `move_profile`, `delete_profile`); `RosterView` only calls them. Each keeps the active character by id, not by index. Deleting the active character moves to the one before it (the next one if it was first). Deleting a character drops its own scan pages; deleting a roster's last character also drops the roster's owned columns, region, `roster_chests`, `roster_income`, `custom_choices`, and the scan pages and snapshots with its roster id. Duplicating makes a separate, identical character: its chests and income settings are copied too, so they count for both.

**Roster ids are uuids** (`crypto.randomUUID()`, since V10), like profile ids, so a roster is unique across saves and browsers (the analytics uploads key by it) and a new roster never inherits a deleted one's data. Nothing stores the roster order: `roster_ids` lists them in order of first appearance in `profiles`, and `add_profile` inserts after the roster's last character (`add_char`, `duplicate_profile` and the default all go through it or start grouped), so a roster's characters stay together and deleting one never reorders the rosters. `move_profile` only swaps within a roster. `from_saved` groups the loaded profiles the same way, keeping the active one. The "Roster N" labels are positions in `roster_ids`, not ids.

Runtime worker bundles are keyed by profile id. `App.vue` watches the list of ids and calls `sync_profiles`, which is the only place entries are created or dropped, so adds and deletes flow through one point and a rename keeps its workers.

## Conventions

- **Materials** are described once, by label, in `MATERIAL_TABLE` in `Constants.ts`: which tiers it exists in, graph colour, bundle sizes, fallback prices and icon. Everything else is derived from it, including `ALL_LABELS`, `TIER_MATERIALS`, `SHARED_LABELS`, `ALL_BUNDLE_KEYS`, `FALLBACK_PRICES` and `IconMap`. Never add a parallel array indexed by material row; look an attribute up by label.
- **The order of `MATERIAL_TABLE` is a contract.** Filtering it by tier produces that tier's cost-array row order, which Rust indexes by, so entries are interleaved (`Red`, `Serca Red`, `Blue`, `Serca Blue`, ...) such that each tier's projection matches its Rust constants. Inserting a material in the wrong place silently misaligns the payload.
- **Two cross-tier relations, and no others.** A label listed under several `tiers` is one material with one stored value, so nothing is ever synced between tiers and `SHARED_LABELS` is derived rather than hand-written. `CONVERTS_FROM` says a material can be made from a lower tier's at a fixed rate, which drives the effective price, the convert button and how a Serca character counts its T4 stock. A material and the one it converts to or from share their trade tick (`linked_labels`): `MaterialCell` sets both, and a load disables both when the save disabled either. Adding a tier means adding table rows, not new branches.
- **Prices are per bundle; everything else is per unit.** A material has a list of `bundle_sizes` (shards sell in 1000 / 2000 / 3000 bags, everything else has one size). `mats_prices` is keyed by `bundle_key(label, size)`, `selected_bundles` records which bundle the user buys, and `unit_prices` / `effective_unit_prices` produce the per-unit numbers the payload and UI use. A market fetch prices every bundle and selects the cheapest per unit.
- **`InputColumn`** is the runtime model behind every numeric input column: `values`, `upper_bound` and `enabled`, all keyed by label (or by bundle key, for prices). A column carries no row order; callers iterate the labels they want to show. `input_column_to_num` returns a label-keyed record. Inputs accept simple arithmetic; a value is clamped to 0 and the column's upper bound (`MAX_AMOUNT`, 99,999,999,999, the game's own gold cap, unless the column sets lower; 999,999,999 before 2026-10-23, which 1B+ silver hit), and anything unparseable is 0. Only the values and the disabled labels are saved; everything else is rebuilt from `Constants.ts`.
- **Nothing is indexed by material row.** The payload carries `material_labels` and a label-keyed `materials` map, and every per-material result comes back label-keyed, so `ALL_LABELS[tier]` is used for display order and to declare the row order to Rust, and for nothing else.
- **Keyed upgrades** are the canonical list of upgrades for a character, keyed by piece, level, kind and tier. They are derived from the tickbox grids and carry per-upgrade progress and the last optimizer state.
- **Tier handling assumes exactly two tiers.** The Rust side would accept more; the frontend's tier-switching logic would not.
- Worker bundles use `shallowReactive`, not refs, so their fields read as plain values whether or not they sit in a store, and large wasm results are never deep-wrapped.

## Persistence

`SavedConfig.ts` defines the saved shape and converts to and from it; `ConfigStorage.ts` owns localStorage and the migrations.

- One key, `HF_CONFIG_V10_COMPRESSED`, lz-string-compressed JSON, with the version **also inside** the payload. The key carries the version so a save is never read by code that predates it; the payload version drives the migration chain.
- One column per scope, not one per tier, so a material shared between tiers is stored once.
- Two rules govern the shape: nothing derivable from `Constants.ts` is stored, and material values are keyed by **label**, never by row index. Reordering, inserting or removing a material row is therefore not a save-shape change.
- `to_saved` / `from_saved` are explicit. A field that is not listed is not saved, so transient state cannot leak in by being added to the store.
- Numbers are saved as numbers. Saving locale-formatted strings corrupted values when the browser locale changed between sessions.
- Any store mutation triggers a debounced write (500 ms), and `App.vue` also writes on `pagehide` so the last edits survive closing the tab.
- On load, keys are tried newest-first; the first one that decompresses wins, every legacy key is then deleted, and `MIGRATIONS[v]` runs in sequence up to `STORAGE_VERSION`.
- A save that does not decompress (garbage can decompress to `null` rather than throw, which counts too), or that throws while migrating or loading, is copied to `<key>_BROKEN` before the defaults are used, since the next write overwrites the key. `broken_save` (`ConfigStorage.ts`) holds it, also when the copy was made by an earlier load, and `BrokenSaveBanner.vue` (placed like `OtherTabBanner`) offers **Download** (`broken_backup`: the decompressed save with its version added, which Backup → Import reads once the code can load it, or the raw string) and **Discard** (deletes every `_BROKEN` key). It is never loaded back by itself: by then the user may have a new save, and the choice is theirs. Only one copy is kept; a newer failure replaces it.
- If the copy does not fit in the quota, the save is held only in memory and `write_state` is paused until Discard, so the original key is not overwritten; the banner says so. Each reload fails the same way and pauses again.

**Before adding a migration, check what production runs.** Production is the `Production` branch on origin (`pnpm deploy` pushes main there): `git fetch origin Production` then `git show origin/Production:frontend/Stores/ConfigStorage.ts | grep STORAGE_VERSION`. If production is below the repo's `STORAGE_VERSION`, the current version is unreleased and nobody has a save in it: fold the change into the existing latest migration instead of starting a new version. Only bump when production already runs the repo's version.

**Changing the saved shape** (once production has the current version) means bumping `STORAGE_VERSION`, setting `STORAGE_KEY` to the new version's key, moving the old key to the top of `LEGACY_KEYS`, and adding one entry to `MIGRATIONS`. Adding a material no longer needs a migration at all: an unknown label in the save is dropped, a missing one falls back to the template value from `Constants.ts`.

A profile's `id` is saved and survives load, save, load unchanged. It also arrived during V8: a save without ids (or with an id an earlier profile already took) gets fresh ones in `profile_from_saved`, so there is no migration step for it.

A profile's `chests` are saved as the scanner exported them (title, band, count, options by label, `obtain_all`), with both tiers in them; what the calculator uses is derived on the page. They arrived during V8: V7 had none, so the migration writes an empty list and a save without the field loads as empty. Only char-bound chests are a profile's now; the rest are the roster's (`roster_chests`, keyed by roster id). `roster_chests`, `scan_pages` and `snapshots` also arrived during V8 and load empty from a save without them, V7 included (the migration goes through `from_saved`).

`roster_income`, `custom_choices` and a profile's `income_manual` / `bound_income` / `income_now` / `income_results` / `choice_levels` / `optimizer_cache` / `choice_results` / `choice_seed` arrived during V8 too and load empty or off. The last three are saved optimizer results (`Wasm contract.md`, "Saved results"); the two states are `markRaw` on load and on write, so the store's deep watch does not walk them. `scan_pages` and `snapshots` only grow with exports and are rewritten with the rest of the config on every change. If that gets slow, they are the part to move to a key of their own.

**Do not trust a pre-V8 column's own `keys` array.** Nothing before V8 read it, so it drifted out of order in real saves while `data` stayed positional against `ALL_LABELS`. `migrate_7_to_8` maps by position for that reason. A V7 column can also be shorter than its tier's labels (the enhanced and Serca books were appended after some V7 saves were made); `v7_merge_columns` leaves a missing row out rather than writing 0, so `from_saved` gives it the template value (its fallback price).

`migrate_8_to_9` repairs saves made before that fix: in every region's `mats_prices`, an enhanced or Serca book priced exactly 0 is reset to its fallback (any other value is the user's). That is all V9 changes; the shape is V8's. `tests/live_v8.test.ts` runs it on a real V8 save (`tests/v8 config`). `node tests/make_pasted_store.mjs [in] [out]` turns such a raw save into a devtools snippet that loads it; the raw string holds `"` and `\`, so it must be embedded via `JSON.stringify`.

`migrate_9_to_10` gives every roster a uuid: the per-roster records (owned columns, regions, `roster_chests`, `roster_income`, `custom_choices`) are re-keyed, and every profile's, scan page's and snapshot's `roster_id` mapped. Profiles are stably sorted by the old numeric id first, so the rosters show in the order they did (V9 sorted them by id). Tests use the strings `"0"`, `"1"` as roster ids, so `record[0]` still finds them; the default config's roster is a fresh uuid, so a test on it reads `active_profile.roster_id` or `roster_ids[0]`.

`migrate_7_to_8` **drops** V7's `optimizer_treatment_plan` rather than mapping it: two of its four values behaved identically, one was unreachable, and it is not worth carrying, so `band_values` is left out and `profile_from_saved` supplies the default. `histogram_treatment_plan` is a different setting — which ownership line the chance column reads — and becomes `chance_band`. `leftover_price` is dropped too; it was always zero.

`migrate_7_to_8` also collapses V7's per-tier columns into one. Where both tiers held a value for the same material, the character's **active tier wins** for its own budgets, and tier 0 wins for the roster-wide columns V7 kept in sync by a watcher. V7's separate `shard_infos` structure folds into Shards' three bundle prices plus `selected_bundles`.

V3 and V4 saves are no longer migrated; their keys are deleted on load.

## Cross-tier behaviour

`change_tier` converts nothing: budgets stay as they are, and a Serca character's T4 is converted when the payload is built. The upgrade-grid remap (`convert_apply_done_want`, through `PLUS_TIER_CONVERSION`, whose Serca half is the inverse of the T4 one) is still hard-wired to exactly two tiers. A level with no equal takes the one below it. Levels fetched in the old tier stay fetched, and so does the new tier's floor (`NORMAL_FLOOR`); levels ticked by hand stay Done. Whether a Serca character may go back is checked on the character being switched (`check_revert_ilevel_ok(profile)`), not the active one.

The market page's convert button (`convert_pooled` in `Utils/Convert.ts`) converts every `CONVERTS_FROM` material, special leaps included: it pools a roster's bound and tradable stock of the source, converts whole multiples of the ratio, and adds the result as roster-bound. Roster-bound source is spent first; the leftover below one ratio stays where it was. Disabled labels are read by value.

A Serca character's T4 stock is kept under the T4 labels (the scanner's export never converts) and counted as Serca where it is used: `merge_lower_tier` in the payload (`Wasm contract.md`), "+c from T4" on the costs distribution, and the auto-deduction. `compute_remaining_materials` spends in Rust's bands (`spend_band`): bound (Serca bound first, as the user sees it in game, then T4 bound at the ratio in whole conversions), then roster-bound (Serca roster-bound, then T4 roster-bound and tradable pooled, roster-bound taken before tradable), then Serca tradable; the T4 remainders are written back to the T4 cells. Free taps are spent the same way (`set_special_remaining`: the remaining count typed after special honing is of the merged total, Serca special leaps first). A disabled material is counted as unlimited bound stock, as everywhere (`parse_input`), so its cost comes out of that and its cells are never written; a disabled source is neither counted nor spent.

The costs distribution's "Show all mats (+ convertible)" (`show_all_rows`) also lists each `CONVERTS_FROM` source above the material it converts into, special leaps included: an owned input only (the selected band's column, the shared trade tick), no chance, average or graph, since Rust returns nothing for a label outside the tier. It is the calc page's only place to edit a Serca character's T4 stock.

## Progress and auto-deduction

The instructions' progress inputs (`NormalDetails.vue`, logic in `SuccessUtils.ts`) deduct what the taps since the last optimizer run cost, when `auto_deduct_costs` is on. Each keyed upgrade holds its own `used_materials` (session-only, not saved); the budgets written are `budget_snapshots[profile id]` minus the sum of them, so moving a slider back gives the materials back. `deduct` sets one upgrade's usage and rewrites the budgets.

- **The snapshot** is per profile id, taken at the first deduction. It is retaken whenever no upgrade has anything outstanding, since the budgets are then the truth (an edit, or another character's deduction of roster-bound stock, may have happened since). It is dropped (`clear_snapshot`) as the optimizer starts, on Confirm, and on a bound-budget edit while auto start is off: from then on the deductions are final.
- **Leaving the calculator settles** (`settle_deductions`, from `Calc.vue`'s unmount): like Confirm without starting the optimizer, every upgrade's `used_materials` and `taps_since_last_input` are cleared and the snapshot dropped. The budgets keep what was deducted and the slider already wrote the progress, so nothing is lost; the slider just starts again at 0. This is what keeps two characters of a roster, or a market-page or Choices edit, from being reverted by an older snapshot. `CharView` keys the calculator by profile id, so a character switch unmounts it too (and nothing of one character's page carries over to the next). The old page's unmount runs after the new one has started, so it cancels and settles only its own character, never the active one.
- **Succeed** (`finish_upgrade`, both the slider's and the free tap's) deducts the taps and the unlock, then folds that upgrade's cost into the snapshot, so the other upgrades' outstanding deductions still count from it.
- **The unlock** is charged unless the upgrade is already unlocked, Rust's rule: artisan or failed taps above 0. While the slider sits at 0 nothing is charged.
- The free-tap "remaining" input follows the budget; an empty one leaves the free taps alone.
- Advanced honing is never deducted. Its Chisel and Naber's Awl ticks are only sent while shown.

## Export / import

The Backup section of the character control panel writes `export_config` to a JSON file and reads it back through `import_config`, which runs the same migration chain as a load, so older backups still import. This is the only way state survives cleared site data or a different browser.

## Styling

Tailwind utilities, plus a small set of shared classes in `shared.css` and colour variables in `theme.css` (dark theme only; `index.css` sets `color-scheme: dark` so native controls match). Long-form text (the guide, the change logs) uses `.prose-card` from `shared.css`. A dismiss ✕ is `Common/CloseButton.vue` (square, `size` in px). Material tables use a shared grid with subgrid rows. Naming is PascalCase for files and folders, snake_case for functions and variables. TypeScript is non-strict.
