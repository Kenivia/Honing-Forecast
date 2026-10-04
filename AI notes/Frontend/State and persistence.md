# State and persistence

`docs/Frontend.md` has a diagram of how inputs flow through the store to the payload. This note covers the shape of the state and how it is saved.

## Pages

- **Roster setup**: manage characters and rosters, set each roster's region, import characters.
- **Market & mats**: roster-owned and tradable materials, market prices.
- **Character page**: one per character, with sub-pages chosen by the last path segment. The calculator sub-page is the main one: upgrade tickbox grids, material budgets and distribution graphs, optimizer controls, then the generated instructions.
- **Change logs**.

The character page does not use nested routes; one component switches on the path suffix.

## One store

There is a single Pinia store, `useRosterStore`, holding one `roster_config` object. The other files in `frontend/Stores` are plain modules: one defines a character profile, the other handles storage.

Data lives at four scopes inside that object:

| Scope | Examples |
| --- | --- |
| Per character | Upgrade grids, keyed upgrades, bound material budgets, free-tap budget, tier, optimizer settings and overrides, worker handles. |
| Per roster | Roster-owned materials, tradable materials, region. |
| Per region | Market prices, shard bag data, the last fetched market response. |
| Global | UI flags, the active character index, last seen version. |

The active character is set from the route, and most components read through `active_*` getters.

## Conventions

- **`InputColumn`** is the model behind every numeric input column: values are stored as formatted strings alongside keys and enable flags. Read numbers through its conversion helper, not by parsing the strings yourself. Inputs accept simple arithmetic.
- **Keyed upgrades** are the canonical list of upgrades for a character, keyed by piece, level, kind and tier. They are derived from the tickbox grids and carry per-upgrade progress and the last optimizer state.
- **Tier handling assumes exactly two tiers.** The Rust side would accept more; the frontend's tier-switching logic would not.
- Worker handles live inside the store but must never be persisted or deep-cloned. Take raw copies before posting store data to a worker.
- The scanner's frame source (the capture stream) is stored the same way at global scope. It is marked raw, created by the load-time validation, and stripped before saving.

## Persistence

- localStorage only, under one versioned key, as lz-string-compressed JSON.
- Any store mutation triggers a debounced write. A fixed list of transient fields (worker handles, caches, snapshots) is stripped first.
- On load, a chain of migrations upgrades older versions, each reading and removing its old key, followed by validation that pads and repairs the loaded data and rebuilds derived fields.

**Changing the saved shape** means bumping the key version and adding a migration, or making the validator tolerate the old shape. Reordering material rows counts as a shape change: saved columns are positional.

## Styling

Tailwind utilities, plus a small set of shared classes in `shared.css` and colour variables in `theme.css` (dark theme only). Material tables use a shared grid with subgrid rows. Naming is PascalCase for files and folders, snake_case for functions and variables. TypeScript is non-strict.
