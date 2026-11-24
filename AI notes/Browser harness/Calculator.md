# Driving the calculator

Read `Overview.md` first. This covers the character calc page plus the market and roster pages that feed it. `Frontend/State and persistence.md` explains what the controls change.

## Pages

| Path | What is there |
| --- | --- |
| `/chars/<region>/<char>/calc` | Upgrade grids, character settings, costs distribution, optimizer panel, instructions. |
| `/market-mats` | Roster-bound and tradable materials, market prices, region. |
| `/roster-setup` | Characters and rosters. |
| `/info/guide`, `/info/change-logs` | Static content. |

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
- The `Chance column` select (the first column's header) switches which ownership class the owned column shows and the chance column describes. The owned textbox is `Bound owned`, `Roster bound owned` or `Tradable owned` to match; its tickbox is always `<name> enabled` of the bound column.

**Results** have no labels and are read from text: `Pending ilevel:`, `Achieved ilevel:`, the gold line ending in `gold spent:`, and `Optimizer progress:`.

**Everything else** (control checkboxes, `Reset this char`, `Convert to T4.5 Serca`, `Auto start optimizer`, `Compare with simple strategies`) is reachable by its visible text. Named controls elsewhere: `Char name` (calc page and each roster row), `Region`, `Active roster`, `Auto fetch`, `Move <name> up` / `down` on the roster page, and `<band> leftover value` next to "Avg value of leftover mats" (shown after clicking `Customise`). `Main` / `Alt` (`RoleToggle.vue`, on the character card and as "Treating as" on that line) set all three at once.

**Added for the e2e specs:**

- Instruction rows are groups named by upgrade (`Helmet +16`, `Helmet Advanced +10 - 20`, class `instruction-row`). Only the first shows its details; others need "Show input anyway". Lists collapse to 10 rows: click `Show all N upgrades` after every reload. Detail inputs: `Taps since last run`, `Taps slider`, `Special leaps remaining`, `Current artisan energy`, `Current base chance`, `Current upgrade`, `Current xp`, `Grace progress`, `Chisel`, `Naber's Awl`.
- Main / Alt carry `aria-pressed`; the two copies sit in groups `Treat as` (character card) and `Treating as` (leftover line). `getByText("Custom")` also hits the Region option; scope it to those groups.
- Override radio columns are groups `Juice Usage`, `Book Usage`, `Adv Juice Usage`, `Adv Scroll Usage`, `Special Usage`.
- Chance column options read `Bound mats`, `Roster mats`, `Tradable`. `Special leaps owned` / `Lower tier special leaps owned` have no row group; find them by textbox name.
- Roster page: cards are groups `Roster N`; row buttons `Go to character <name>`, `Make a copy of <name>`, `Delete <name>`. Header `Sign in` / `Sign out`; the sync pairing popup is the dialog `Match your characters` with selects `Match <local name>`.

A second tab of the site shows a warning banner ("open in another tab"); a test with two pages in one context sees it.

## The optimizer

Any change to the grids or materials restarts it when `Auto start optimizer` is on. A first estimate of gold appears almost at once and is refined until progress reaches 100%, a few seconds for one column of upgrades.

- Progress reads 100% while idle, so "wait for 100%" passes instantly if checked right after a click. `hf.wait_for_optimizer` waits for it to leave 100% first.
- Read gold only after the optimizer finishes.
- Wait for it before the next click too: its results re-lay out the page, and under full e2e load a click can land after the target moved (lost clicks in `calc.spec.ts` after forbid / tier revert, Firefox, 2026-11-24).
- With MCP, do not use `browser_wait_for` on the text `100.00%`: chance cells contain the same text. Wait a few seconds and snapshot again, or check with `browser_evaluate`.

## Helpers

`tests/e2e/helpers.ts`, available as `hf` in `pnpm browse` scripts.

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
| `store_eval(page, fn, arg)` | Runs `fn(store, arg)` on the roster store in the page; `fn` is serialised, so self-contained. |
| `upgrade_row(page, name)` | An instruction row by its group name. |
| `reload(page)` | Waits out the debounced save, reloads, waits for the shell. |

`tests/e2e/bible_helpers.ts` has the lostark.bible side: `stub_bible` (answers every lostark.bible request, records them; status knobs for 401/500), `sign_in_as` (injects `bible_auth` once per tab, so a reload after sign-out stays out), `stub_character` (the Worker scrape from `tests/unit/market-import/fixtures`), `with_store`, `seed`, `roster_names`. **Route lostark.bible in every signed-in test**: an unstubbed call with a fake token 401s on the real site and wipes the sign-in.

Add to this file when a new interaction is needed more than once.

## One-off debugging

With MCP: navigate to `http://localhost:5173/`, snapshot, click by ref. State persists, so `Reset this char` or a new session is needed to get back to a clean character. Prices are live.

With `pnpm browse`: write a short script using `hf`, print what you need, take a screenshot if layout matters. Add `--firefox` to compare browsers, `--headed` to let the user watch.

To inspect state directly, the whole config is one lz-string-compressed localStorage entry; reading the DOM is usually easier than decoding it.

## Rigorous testing

Add a spec under `tests/e2e/`. Follow `smoke.spec.ts`:

- `watch_errors` and `stub_market` in `beforeEach`, and assert no errors in `afterEach`.
- Drive only through helpers and role-based locators.
- Assert ilevel and cell statuses exactly. Assert gold by relation (greater than zero, lower after adding materials) or a generous range.
- To test persistence, wait a second for the debounced save before reloading.

The price fixture is `tests/e2e/fixtures/market.json`, keyed by region slug (`nae`, `euc`), in the market API's response shape. `stub_market` fulfils `GET /prices/<region>` with that region's list; if the route or the response shape changes, update both. Worker stubs route through `worker_route(path)` in `helpers.ts`, which matches the deployed Worker (`api.honing-forecast.com`, and the old `workers.dev`) and `pnpm cloudflare-dev` (`localhost:8787`), so a local `VITE_WORKER_URL` override doesn't bypass them.

## Specs

Each test checks the effect (store, page, gold relation) and that it survives a reload, not just that a control fired: unit tests with hand-built data missed a broken Forbid button (stored vs merged chest keys).

- `calc.spec.ts`: forbid on a merged chest stack (injected chests with equal options merge, titles joined `A / B`; forbidden ones list under their own titles), chest editor, advice split, tier conversion and revert, Succeed / slider deduction with auto-deduct on and off, manual artisan / chance, free taps, advanced details, owned tickbox, chance column across two characters, leftover bands, show all mats, Reset this char, optimizer panel (auto start checked by `runtime.optimizer.run_counter`, not the progress text, which flickers while the histogram redraws), long lists, Advanced and vambrace headers.
- `roster.spec.ts`: add / rename / reorder / copy / delete, regions (a name in two regions, an old region-less link opened cold), accented names, scrape Fetch and Fetch remaining, per-roster mats on the market page.
- `bible.spec.ts` (mocked): sign-in flow and callback errors, sync (fresh vs touched `Newchar`, accent matching, pairing popup, second sync, NA + CE, Custom skipped, the same name in NA and CE), locks, refresh and its scrape fallback, 401 / expiry, sign-out, auto-refresh once per launch.
- `bible_live.spec.ts`: against the real API; skipped unless `.env.local` has `BIBLE_TOKEN` (sign in on localhost, then `JSON.parse(localStorage.bible_auth).token`). Revoke is stubbed so it never kills the token.
- `domain_move.spec.ts`: the pages.dev to .com hand-off. Left out of `pnpm test:e2e` (`testIgnore`) since the move is done; it runs only through `pnpm test:e2e:staging`. See `Frontend/Domain move.md`.

Quirks: Firefox needs 120 s test timeouts. It errors the workers of a page left mid-run (no message); `WorkerBundle.ts` stays quiet once `beforeunload`/`pagehide` fired, and `hf.open` / `hf.reload` wait for idle runs anyway (`settle`). Click, then wait for its effect before the next click in a list that re-lays out (Allow / Forbid). Chromium logs a stubbed 4xx/5xx as a console error (`ignore_http` in bible_helpers). Typing a char name while signed out opens the scrape popup; close it on the overlay. More than ~3 workers make Firefox page loads time out.

Not covered: tap counts (only in hover tips), the "here" link in the Treat-as hint.
