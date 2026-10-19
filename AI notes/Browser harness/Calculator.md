# Driving the calculator

Read `Overview.md` first. This covers the character calc page plus the market and roster pages that feed it. `Frontend/State and persistence.md` explains what the controls change.

## Pages

| Path | What is there |
| --- | --- |
| `/<char>/calc` | Upgrade grids, character settings, costs distribution, optimizer panel, instructions. |
| `/market-mats` | Roster-bound and tradable materials, market prices, region. |
| `/roster-setup` | Characters and rosters. |
| `/<char>/guide`, `/change-logs` | Static content. |

A fresh profile has one character, `Newchar`, at ilevel 1640 with nothing selected and zero gold.

## How things are named

These accessible names are what MCP snapshots show and what the helpers use.

**Upgrade grids** are two groups, `Normal honing` and `Advanced honing`.

- Column headers are buttons named by level: `+15`. Normal and advanced share `+20`, so always scope to the group.
- Cells are buttons named `<Piece> +<level>: <status>`, for example `Helmet +15: Want`. Status is `NotYet`, `Want`, `Done` or `FetchedDone`.
- Clicking a `NotYet` cell sets it and every lower `NotYet` level of that piece to `Want`. Clicking `Want` marks it and everything below `Done`. Clicking `Done` clears it and everything above.
- A column header does the same to all six pieces.

**Material tables** have one group per row, named by the material (`Red`, `11-14 Armor`).

- Calc page: the row holds a `Bound owned` textbox and a `Bound owned enabled` checkbox, then three read-only values in order: chance, average amount, average gold. Read those by position within the row.
- Market page: rows sit inside a `T4 materials` or `Serca materials` group, with `Roster bound owned`, `Tradable owned` and `Market price` textboxes. `Shards`, `Gold`, `Silver` and the breaths appear in both tiers, so scope to the tier group.
- Inputs commit on change, not per keystroke: fill, then blur. They accept arithmetic (`100+50`).
- The `Chance column` select switches which ownership class the chance column describes.

**Results** have no labels and are read from text: `Pending ilevel:`, `Achieved ilevel:`, the gold line ending in `gold spent:`, and `Optimizer progress:`.

**Everything else** (control checkboxes, `Reset this char`, `Convert to T4.5 Serca`, `Express event`, `Auto start optimizer`, `Compare with simple strategies`) is reachable by its visible text.

## The optimizer

Any change to the grids or materials restarts it when `Auto start optimizer` is on. A first estimate of gold appears almost at once and is refined until progress reaches 100%, a few seconds for one column of upgrades.

- Progress reads 100% while idle, so "wait for 100%" passes instantly if checked right after a click. `hf.wait_for_optimizer` waits for it to leave 100% first.
- Read gold only after the optimizer finishes.
- With MCP, do not use `browser_wait_for` on the text `100.00%`: chance cells contain the same text. Wait a few seconds and snapshot again, or check with `browser_evaluate`.

## Helpers

`e2e/helpers.ts`, available as `hf` in `pnpm browse` scripts.

| Helper | Does |
| --- | --- |
| `open(page, path)` | Navigate and wait for the app shell. |
| `toggle_column(page, kind, level)` | Click a column header. `kind` is `"normal"` or `"adv"`. |
| `cell(page, kind, piece, level)` | Locator for one cell. |
| `cell_status(...)` | Its status string. |
| `wait_for_optimizer(page)` | Wait for a run to start and finish. |
| `read_gold(page)`, `read_pending_ilevel(page)` | Parse the result text. |
| `material_row(scope, label)` | Locator for a table row. |
| `set_material(scope, label, column, value)` | Fill an input and commit it. |
| `stub_market(page)` | Serve fixture prices. Call before `open`. |
| `watch_errors(page)` | Array that collects page and console errors. |

Add to this file when a new interaction is needed more than once.

## One-off debugging

With MCP: navigate to `http://localhost:5173/`, snapshot, click by ref. State persists, so `Reset this char` or a new session is needed to get back to a clean character. Prices are live.

With `pnpm browse`: write a short script using `hf`, print what you need, take a screenshot if layout matters. Add `--firefox` to compare browsers, `--headed` to let the user watch.

To inspect state directly, the whole config is one lz-string-compressed localStorage entry; reading the DOM is usually easier than decoding it.

## Rigorous testing

Add a spec under `e2e/`. Follow `smoke.spec.ts`:

- `watch_errors` and `stub_market` in `beforeEach`, and assert no errors in `afterEach`.
- Drive only through helpers and role-based locators.
- Assert ilevel and cell statuses exactly. Assert gold by relation (greater than zero, lower after adding materials) or a generous range.
- To test persistence, wait a second for the debounced save before reloading.

To refresh the price fixture, record the two Worker POST responses on a fresh load and write them to `e2e/fixtures/market.json` keyed by region slug (`nae`, `euc`).

## Not covered yet

The instructions section (per-upgrade progress, `Succeed`, `Confirm`), the simple-strategy overrides, tier conversion, character import and multi-roster setups have not been driven through the harness. They have no labels beyond visible text; add labels and helpers when first needed.
