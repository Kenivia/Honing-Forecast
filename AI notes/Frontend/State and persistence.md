# State and persistence

`docs/Frontend.md` has a diagram of how inputs flow through the store to the payload. This note covers the shape of the state and how it is saved.

## Pages

- **Roster setup**: manage characters and rosters, set each roster's region, import characters.
- **Market & mats**: roster-owned and tradable materials, market prices.
- **Character page**: one per character, with sub-pages chosen by the last path segment. The calculator sub-page is the main one: upgrade tickbox grids, material budgets and distribution graphs, optimizer controls, then the generated instructions.
- **Change logs**.

The character page does not use nested routes; one component switches on the path suffix.

## Two stores

The split is by lifetime, and it is what keeps the save file honest.

- **`useRosterStore`** (`RosterConfig.ts`) holds one `roster_config` object. Everything in it is persisted; there is no opt-out list.
- **`useRuntimeStore`** (`RuntimeState.ts`) holds everything session-only: the per-character worker bundles, the scanner's cropper bundle and frame source, `budget_snapshot`, `adv_cache`, and the in-flight fetch / details-update flags. None of it is ever written.

Data in `roster_config` lives at four scopes:

| Scope | Examples |
| --- | --- |
| Per character | Upgrade grids, keyed upgrades, bound material budgets, free-tap budget, tier, optimizer settings. |
| Per roster | Roster-owned materials, tradable materials, region. |
| Per region | Market prices, shard bag data, the last fetched market response. |
| Global | UI flags, the active character index, last seen version. |

The active character is set from the route, and most components read through `active_*` getters.

Runtime worker bundles are keyed by `char_name`. `App.vue` watches the list of names and calls `sync_profiles`, which is the only place entries are created or dropped, so adds, renames and deletes all flow through one point.

## Conventions

- **`InputColumn`** is the runtime model behind every numeric input column: values are formatted strings, alongside keys, bounds and enable flags. Read numbers through its conversion helper. Inputs accept simple arithmetic. Only the values and the disabled labels are saved; everything else is rebuilt from `Constants.ts`.
- **Keyed upgrades** are the canonical list of upgrades for a character, keyed by piece, level, kind and tier. They are derived from the tickbox grids and carry per-upgrade progress and the last optimizer state.
- **Tier handling assumes exactly two tiers.** The Rust side would accept more; the frontend's tier-switching logic would not.
- Worker bundles use `shallowReactive`, not refs, so their fields read as plain values whether or not they sit in a store, and large wasm results are never deep-wrapped.

## Persistence

`SavedConfig.ts` defines the saved shape and converts to and from it; `ConfigStorage.ts` owns localStorage and the migrations.

- One key, `HF_CONFIG`, lz-string-compressed JSON, with the version **inside** the payload.
- Two rules govern the shape: nothing derivable from `Constants.ts` is stored, and material values are keyed by **label**, never by row index. Reordering, inserting or removing a material row is therefore not a save-shape change.
- `to_saved` / `from_saved` are explicit. A field that is not listed is not saved, so transient state cannot leak in by being added to the store.
- Numbers are saved as numbers. Saving locale-formatted strings corrupted values when the browser locale changed between sessions.
- Any store mutation triggers a debounced write.
- On load, the newest save wins; failing that, the pre-V8 keys are tried newest-first, then `MIGRATIONS[v]` runs in sequence up to `STORAGE_VERSION`.

**Changing the saved shape** means bumping `STORAGE_VERSION` and adding one entry to `MIGRATIONS`. Adding a material no longer needs a migration at all: an unknown label in the save is dropped, a missing one falls back to the template value from `Constants.ts`.

**Do not trust a pre-V8 column's own `keys` array.** Nothing before V8 read it, so it drifted out of order in real saves while `data` stayed positional against `ALL_LABELS`. `migrate_7_to_8` maps by position for that reason.

V3 and V4 saves are no longer migrated; their keys are deleted on load.

## Export / import

The Backup section of the character control panel writes `export_config` to a JSON file and reads it back through `import_config`, which runs the same migration chain as a load, so older backups still import. This is the only way state survives cleared site data or a different browser.

## Styling

Tailwind utilities, plus a small set of shared classes in `shared.css` and colour variables in `theme.css` (dark theme only). Material tables use a shared grid with subgrid rows. Naming is PascalCase for files and folders, snake_case for functions and variables. TypeScript is non-strict.
