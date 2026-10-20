# State and persistence

The shape of the frontend state and how it is saved.

## Pages

- **Roster setup**: manage characters and rosters, set each roster's region, import characters.
- **Market & mats**: roster-owned and tradable materials, market prices.
- **Character page**: one per character, with sub-pages as child routes. The calculator sub-page is the main one: upgrade tickbox grids, material budgets and distribution graphs, the chest panels (the character's chests and its roster's, `active_chests`, as `process_chests` leaves them for the character's tier; "Edit chests" swaps in the stored ones, editable) and a provisional panel of how those with a choice are opened (`ChestState.vue`, read off the histogram's `state_bundle.chest_states`, so it follows the optimizer's interim states like the graphs do; each stack as its icon, then per option how many chests and what one chest gives), optimizer controls, then the generated instructions. The other sub-pages are the guide, the scanner, Income (the roster's budget snapshots and income per week, `Screen scanner/Income.md`) and Choices (what a raid's bonus chest is worth, `Screen scanner/Choices.md`).
- **Change logs**.

The character page uses nested routes: `CharView` owns the sidebar and renders the sub-page through `<RouterView/>`.

## Two stores

The split is by lifetime, and it is what keeps the save file honest.

- **`useRosterStore`** (`RosterConfig.ts`) holds one `roster_config` object. Everything in it is persisted; there is no opt-out list.
- **`useRuntimeStore`** (`RuntimeState.ts`) holds everything session-only: the per-character worker bundles, the scanner's cropper bundle and frame source, `budget_snapshot`, `adv_cache`, and the in-flight fetch / details-update flags. None of it is ever written.

Data in `roster_config` lives at four scopes:

| Scope | Examples |
| --- | --- |
| Per character | Upgrade grids, keyed upgrades, bound material budgets, free-tap budget, chests, tier, band values, optimizer settings. |
| Per roster | Roster-owned materials, tradable materials, roster chests, region, the scanner's saved pages and the budget snapshots (both lists in the config, filtered by roster), manual roster-bound and tradable income. |
| Per region | Market prices per bundle, the chosen bundle per material, the last fetched market response. |
| Global | UI flags, the active character index, last seen version. |

The active character is set from the route, and most components read through `active_*` getters.

Every profile has an `id` (`crypto.randomUUID()`), and everything internal identifies a character by it, never by `char_name`. The name is only for display, the route, lostark.bible requests, and matching imported in-game characters to profiles. The route `/:characterName` is resolved to a profile in the router guard and `CharView`, and from there the active index and ids take over. `new_char_profile` makes a fresh id and duplicating a character replaces the copied one; `reset_active_profile` keeps it.

Runtime worker bundles are keyed by profile id. `App.vue` watches the list of ids and calls `sync_profiles`, which is the only place entries are created or dropped, so adds and deletes flow through one point and a rename keeps its workers.

## Conventions

- **Materials** are described once, by label, in `MATERIAL_TABLE` in `Constants.ts`: which tiers it exists in, graph colour, bundle sizes, fallback prices and icon. Everything else is derived from it, including `ALL_LABELS`, `TIER_MATERIALS`, `SHARED_LABELS`, `ALL_BUNDLE_KEYS`, `FALLBACK_PRICES` and `IconMap`. Never add a parallel array indexed by material row; look an attribute up by label.
- **The order of `MATERIAL_TABLE` is a contract.** Filtering it by tier produces that tier's cost-array row order, which Rust indexes by, so entries are interleaved (`Red`, `Serca Red`, `Blue`, `Serca Blue`, ...) such that each tier's projection matches its Rust constants. Inserting a material in the wrong place silently misaligns the payload.
- **Two cross-tier relations, and no others.** A label listed under several `tiers` is one material with one stored value, so nothing is ever synced between tiers and `SHARED_LABELS` is derived rather than hand-written. `CONVERTS_FROM` says a material can be made from a lower tier's at a fixed rate, which drives the effective price, the convert button and how a Serca character counts its T4 stock. A material and the one it converts to or from share their trade tick (`linked_labels`): `MaterialCell` sets both, and a load disables both when the save disabled either. Adding a tier means adding table rows, not new branches.
- **Prices are per bundle; everything else is per unit.** A material has a list of `bundle_sizes` (shards sell in 1000 / 2000 / 3000 bags, everything else has one size). `mats_prices` is keyed by `bundle_key(label, size)`, `selected_bundles` records which bundle the user buys, and `unit_prices` / `effective_unit_prices` produce the per-unit numbers the payload and UI use. A market fetch prices every bundle and selects the cheapest per unit.
- **`InputColumn`** is the runtime model behind every numeric input column: `values`, `upper_bound` and `enabled`, all keyed by label (or by bundle key, for prices). A column carries no row order; callers iterate the labels they want to show. `input_column_to_num` returns a label-keyed record. Inputs accept simple arithmetic. Only the values and the disabled labels are saved; everything else is rebuilt from `Constants.ts`.
- **Nothing is indexed by material row.** The payload carries `material_labels` and a label-keyed `materials` map, and every per-material result comes back label-keyed, so `ALL_LABELS[tier]` is used for display order and to declare the row order to Rust, and for nothing else.
- **Keyed upgrades** are the canonical list of upgrades for a character, keyed by piece, level, kind and tier. They are derived from the tickbox grids and carry per-upgrade progress and the last optimizer state.
- **Tier handling assumes exactly two tiers.** The Rust side would accept more; the frontend's tier-switching logic would not.
- Worker bundles use `shallowReactive`, not refs, so their fields read as plain values whether or not they sit in a store, and large wasm results are never deep-wrapped.

## Persistence

`SavedConfig.ts` defines the saved shape and converts to and from it; `ConfigStorage.ts` owns localStorage and the migrations.

- One key, `HF_CONFIG_V8_COMPRESSED`, lz-string-compressed JSON, with the version **also inside** the payload. The key carries the version so a save is never read by code that predates it; the payload version drives the migration chain.
- One column per scope, not one per tier, so a material shared between tiers is stored once.
- Two rules govern the shape: nothing derivable from `Constants.ts` is stored, and material values are keyed by **label**, never by row index. Reordering, inserting or removing a material row is therefore not a save-shape change.
- `to_saved` / `from_saved` are explicit. A field that is not listed is not saved, so transient state cannot leak in by being added to the store.
- Numbers are saved as numbers. Saving locale-formatted strings corrupted values when the browser locale changed between sessions.
- Any store mutation triggers a debounced write.
- On load, keys are tried newest-first; the first one that decompresses wins, older keys are deleted, then `MIGRATIONS[v]` runs in sequence up to `STORAGE_VERSION`.

**Changing the saved shape** means bumping `STORAGE_VERSION`, setting `STORAGE_KEY` to the new version's key, moving the old key to the top of `LEGACY_KEYS`, and adding one entry to `MIGRATIONS`. Adding a material no longer needs a migration at all: an unknown label in the save is dropped, a missing one falls back to the template value from `Constants.ts`.

A profile's `id` is saved and survives load, save, load unchanged. It also arrived during V8: a save without ids (or with an id an earlier profile already took) gets fresh ones in `profile_from_saved`, so there is no migration step for it.

A profile's `chests` are saved as the scanner exported them (title, band, count, options by label, `obtain_all`), with both tiers in them; what the calculator uses is derived on the page. They arrived during V8, which is not released: V7 had none, so the migration writes an empty list and a save without the field loads as empty. Only char-bound chests are a profile's now; the rest are the roster's (`roster_chests`, keyed by roster id). `roster_chests`, `scan_pages` and `snapshots` also arrived during V8 and load empty from a save without them, V7 included (the migration goes through `from_saved`); there is no V9.

`roster_income` and a profile's `income_manual` / `bound_income` / `choice_levels` arrived during V8 too and load empty or off. `scan_pages` and `snapshots` only grow with exports and are rewritten with the rest of the config on every change. If that gets slow, they are the part to move to a key of their own.

**Do not trust a pre-V8 column's own `keys` array.** Nothing before V8 read it, so it drifted out of order in real saves while `data` stayed positional against `ALL_LABELS`. `migrate_7_to_8` maps by position for that reason.

`migrate_7_to_8` **drops** V7's `optimizer_treatment_plan` rather than mapping it: two of its four values behaved identically, one was unreachable, and it is not worth carrying, so `band_values` is left out and `profile_from_saved` supplies the default. `histogram_treatment_plan` is a different setting — which ownership line the chance column reads — and becomes `chance_band`. `leftover_price` is dropped too; it was always zero.

`migrate_7_to_8` also collapses V7's per-tier columns into one. Where both tiers held a value for the same material, the character's **active tier wins** for its own budgets, and tier 0 wins for the roster-wide columns V7 kept in sync by a watcher. V7's separate `shard_infos` structure folds into Shards' three bundle prices plus `selected_bundles`.

V3 and V4 saves are no longer migrated; their keys are deleted on load.

## Cross-tier behaviour

`change_tier` converts nothing: budgets stay as they are, and a Serca character's T4 is converted when the payload is built. The upgrade-grid remap is still hard-wired to exactly two tiers.

The market page's convert button pools a roster's bound and tradable stock of a source material and converts it at the ratio, with the result roster-bound.

A Serca character's T4 stock is kept under the T4 labels (the scanner's export never converts) and counted as Serca where it is used: `merge_lower_tier` in the payload (`Wasm contract.md`), "+c from T4" on the costs distribution, and the auto-deduction. `compute_remaininig_materials` spends in Rust's bands (`spend_band`): bound (Serca bound first, as the user sees it in game, then T4 bound at the ratio in whole conversions), then roster-bound (Serca roster-bound, then T4 roster-bound and tradable pooled, roster-bound taken before tradable), then Serca tradable; the T4 remainders are written back to the T4 cells. Free taps are spent the same way (`set_special_remaining`: the remaining count typed after special honing is of the merged total, Serca special leaps first). A disabled material is neither counted nor spent.

## Export / import

The Backup section of the character control panel writes `export_config` to a JSON file and reads it back through `import_config`, which runs the same migration chain as a load, so older backups still import. This is the only way state survives cleared site data or a different browser.

## Styling

Tailwind utilities, plus a small set of shared classes in `shared.css` and colour variables in `theme.css` (dark theme only). Material tables use a shared grid with subgrid rows. Naming is PascalCase for files and folders, snake_case for functions and variables. TypeScript is non-strict.
