# External services

The app is a static site with no backend of its own. Everything external goes through one Cloudflare Worker.

## The Worker

`cloudflare/worker.js`, deployed separately with `pnpm cloudflare-deploy`. Its URL reaches the frontend through `VITE_WORKER_URL`. It does two things, both as a caching proxy backed by a KV namespace:

- forwards market price requests to a third-party market data API;
- fetches character pages from lostark.bible, which the browser cannot call directly.

It holds no application logic. Parsing happens in the frontend.

## Market prices

`Utils/MarketDataFetcher.ts`.

- Prices are fetched per region on app load. A region set to custom is never fetched, and a per-region auto-fetch flag can turn fetching off.
- The raw response is kept in the store (and therefore in localStorage) and reused for a cooldown period.
- Item slugs from the API are mapped onto the material label order. A table of fallback prices covers anything missing.
- Shards are sold in bags of several sizes; the fetcher picks the cheapest per shard.
- Some Serca materials are priced from their T4 equivalents through a fixed conversion, and the market page keeps those rows in sync one way.

## Character import ("Uwuowo")

`Components/Common/Uwuowo`. The name is historical; the source is lostark.bible.

- The Worker returns the site's HTML and the frontend scrapes it by DOM position. This breaks whenever the site changes its markup, and should be assumed fragile.
- A successful import marks already-completed upgrades in the tickbox grids as fetched (optionally locked), may switch the character's tier, and seeds advanced-honing progress.
- Roster import fetches the roster list, then each missing character in turn, under a client-side rate limit.

## Changelogs

Markdown files in `public/change-logs`, one per version. Version names are discovered at build time; contents are fetched and rendered at runtime. `public/WIP.md` is rendered the same way as the unreleased entry and doubles as the author's roadmap. The stored last-seen version drives a "new" marker in the footer.
