# External services

The app is a static site with no backend of its own. Everything external goes through one Cloudflare Worker.

## The Worker

`cloudflare/worker.js`, deployed separately with `pnpm cloudflare-deploy`. Its URL reaches the frontend through `VITE_WORKER_URL`. It does two things, both as a caching proxy backed by a KV namespace:

- forwards market price requests to a third-party market data API;
- fetches character pages from lostark.bible, which the browser cannot call directly.

It holds no application logic for those two; parsing happens in the frontend. The one exception is the analytics route below.

## Analytics uploads

The budget snapshots (`Screen scanner/Manifest.md`) are uploaded so the author can study income across players. Opt-out: `analytics` in the roster config (global, saved, on by default), the "Share owned materials anonymously" checkbox, last in the scanner page's sidebar (`ScannerControlPanel.vue`), its tooltip saying what is sent.

- **Frontend** (`Utils/Analytics.ts`, `upload_snapshots`): production and dev builds (dev ones send version `"dev"`, so they can be told apart), only when `MODE` is `production` or `development`, so never under Vitest (`test`) or the node suites (esbuild defines `import.meta.env` as `{}`), nor under browser automation (`navigator.webdriver`, which keeps the e2e tests, on a dev server pointed at the real Worker, from uploading). `App.vue` calls it on load and whenever the number of snapshots or the flag changes. Per roster, the snapshots without `sent`, oldest first, up to 20 per request (`POST /snapshots`: roster id, region, `LATEST_VERSION`, the snapshots as stored, names included); each batch is marked `sent` once the Worker answers ok. A failure stops until the next trigger, so nothing is lost but a closed tab can send a batch twice: dedupe by roster and `at` when analysing. Existing snapshots are back-filled the same way. One upload at a time.
- **Worker** (`handleSnapshots`): rejects bodies over 1 MB, takes each character's latest name across the batch and **deletes every name** from the snapshots, fetches each character's logs from lostark.bible (`/character/<NA|CE>/<name>/logs/__data.json`, SvelteKit's data endpoint, decoded with `devalue`'s `unflatten`; at most 40 characters, the free plan allows 50 subrequests), and inserts one row into D1. A log is cut down to `{boss, difficulty, isBus, gearScore (floored), class, week}`, `week` being the start of its weekly reset (Thursday 10:00 UTC): the log id, name, exact time and dps numbers would let anyone find the character again on lostark.bible. Instead of logs a character gets a reason: "no name" (`Newchar…` defaults), "no region" (Custom), "not found", "private", "error …". The page only lists the last 25 logs, so overlapping uploads are what builds a history; names typed by hand that are not the in-game name fetch someone else's or nothing.
- **D1** `hf-analytics`, binding `DB`, one table `uploads` (`cloudflare/schema.sql`: id (`AUTOINCREMENT`, so never reused after a pull empties the table; unique across pull files), received, roster, region, version, snapshots and raids as JSON text). Free plan: 100k rows written a day, 500 MB per database.
- **Pulling**: `pnpm pull-analytics` (`scripts/pull-analytics.mjs`) takes the highest id, pages everything up to it into `ignored/analytics/uploads-<time>.json` (rows with `snapshots` and `raids` parsed), then deletes up to that id. `--keep` skips the delete, `--local` reads `wrangler dev`'s local database.
- **Testing**: the deployed Worker is whitelisted by lostark.bible (Workers are blocked by default), the logs endpoint included; verified 2026-10-22. `wrangler dev --local` runs the route against a local D1 (keyed by `database_id`; apply the schema with `d1 execute --local --file`), but lostark.bible answers 403 to it, the character proxy included, so raids come back as "error 403" there. Point a dev server at it with `VITE_WORKER_URL=http://localhost:8788 npx vite --port <p>`; a Playwright-driven page needs `navigator.webdriver` overridden in an init script to upload at all.

## Market prices

`Utils/MarketDataFetcher.ts`.

- Prices are fetched per region on app load. A region set to custom is never fetched, and one global `auto_fetch` flag can turn fetching off (the fetch button still forces one).
- The raw response is kept in the store (and therefore in localStorage) with its timestamp and reused for a cooldown period, across reloads too. A forced fetch only reuses a response under a second old. A failed fetch applies the saved response, however old.
- Item slugs from the API are mapped onto bundle keys. A fetch writes only the bundles the response lists, so anything the market does not sell (Silver, Gold, Enhanced books, special leaps, and the Serca Tremor books, whose slugs are in the map but which the API did not list yet on 2026-10-21) keeps whatever the user typed; fallback prices are only the column's defaults. A price typed for a listed item is a temporary override: the next fetch, or the cached response re-applied on load, replaces it, and the market page's sidebar says so. Custom is the region for prices of one's own.
- Shards are sold in bags of several sizes; after each fetch the cheapest per shard is selected.
- The fetch flags (`is_fetching`, `market_fetch_failed` in the runtime store) are per region, so regions fetch in parallel and the market page shows the selected region's status.
- Serca prices are not synced with T4: the market page only shows an effective price, the cheaper of buying and converting (`effective_unit_prices`).

## Character import ("Uwuowo")

`Components/Common/Uwuowo`. The name is historical; the source is lostark.bible.

- The Worker returns the site's HTML and the frontend scrapes it by DOM position. This breaks whenever the site changes its markup, and should be assumed fragile. Equipment is read from the first 12 cells of the equipment grid (armor alternating with accessories); since 2026-10-21 the grid has extra cells after them (Vambrace, Orb), which are ignored.
- A successful import marks already-completed upgrades in the tickbox grids as fetched (optionally locked), may switch the character's tier, and seeds advanced-honing progress.
- Advanced progress is seeded as `adv_progress[0] = (level % 10) * 10`, the unit AdvancedDetails and Rust use (10 per level within a block of 10, 100 when done).
- Roster import fetches the roster list, then each missing character in turn, under a client-side rate limit of 60 requests per rolling minute (`fetch_uwuowo`). A result that lands after the name or region changed is dropped, and a character fetch never runs without a region (custom).

## Changelogs

Markdown files in `public/change-logs`, one per version. Version names are discovered at build time; contents are fetched and rendered at runtime. `public/WIP.md` is rendered the same way as the unreleased entry and doubles as the author's roadmap. Visiting the change-log page sets the stored last-seen version to the latest, which drives a "new" marker in the footer, shown for any other version, patch bumps included (`version_equal` is plain equality). `load_change_log` (`Utils/Changelog.ts`) caches each rendered version for the session and returns null on a failed fetch, which the page shows as an error; `computedAsync` drops a fetch that lands after the version changed.
