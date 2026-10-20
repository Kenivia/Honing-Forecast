# External services

The app is a static site with no backend of its own. Everything external goes through one Cloudflare Worker.

## The Worker

`cloudflare/worker.js`, deployed separately with `pnpm cloudflare-deploy`. Its URL reaches the frontend through `VITE_WORKER_URL`. It does two things, both as a caching proxy backed by a KV namespace:

- forwards market price requests to a third-party market data API;
- fetches character pages from lostark.bible, which the browser cannot call directly.

It holds no application logic. Parsing happens in the frontend.

## Market prices

`Utils/MarketDataFetcher.ts`.

- Prices are fetched per region on app load. A region set to custom is never fetched, and one global `auto_fetch` flag can turn fetching off (the fetch button still forces one).
- The raw response is kept in the store (and therefore in localStorage) with its timestamp and reused for a cooldown period, across reloads too. A forced fetch only reuses a response under a second old. A failed fetch applies the saved response, however old.
- Item slugs from the API are mapped onto bundle keys. A fetch writes only the bundles the response lists, so anything the market does not sell (Silver, Gold, Enhanced books, special leaps) keeps whatever the user typed; fallback prices are only the column's defaults.
- Shards are sold in bags of several sizes; after each fetch the cheapest per shard is selected.
- The fetch flags (`is_fetching`, `market_fetch_failed` in the runtime store) are per region, so regions fetch in parallel and the market page shows the selected region's status.
- Serca prices are not synced with T4: the market page only shows an effective price, the cheaper of buying and converting (`effective_unit_prices`).

## Character import ("Uwuowo")

`Components/Common/Uwuowo`. The name is historical; the source is lostark.bible.

- The Worker returns the site's HTML and the frontend scrapes it by DOM position. This breaks whenever the site changes its markup, and should be assumed fragile.
- A successful import marks already-completed upgrades in the tickbox grids as fetched (optionally locked), may switch the character's tier, and seeds advanced-honing progress.
- Advanced progress is seeded as `adv_progress[0] = (level % 10) * 10`, the unit AdvancedDetails and Rust use (10 per level within a block of 10, 100 when done).
- Roster import fetches the roster list, then each missing character in turn, under a client-side rate limit of 60 requests per rolling minute (`fetch_uwuowo`). A result that lands after the name or region changed is dropped, and a character fetch never runs without a region (custom).

## Changelogs

Markdown files in `public/change-logs`, one per version. Version names are discovered at build time; contents are fetched and rendered at runtime. `public/WIP.md` is rendered the same way as the unreleased entry and doubles as the author's roadmap. Visiting the change-log page sets the stored last-seen version to the latest, which drives a "new" marker in the footer, shown for any other version, patch bumps included (`version_equal` is plain equality). `load_change_log` (`Utils/Changelog.ts`) caches each rendered version for the session and returns null on a failed fetch, which the page shows as an error; `computedAsync` drops a fetch that lands after the version changed.
