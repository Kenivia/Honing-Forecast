# External services: don't try

Traps and ruled-out alternatives. The current state is in `External services.md`.

## Worker

- Don't use the classic Cache API (`caches.default`): it does nothing on workers.dev, only on custom domains. Use Workers Cache with GET-only routes and `Cache-Control`.
- Don't cache in KV: the free plan allows 1k KV writes a day.
- Don't make the market a POST keyed by a hash of its body: Workers Cache keys GETs by path and query.
- Don't leave upstream fetches unlimited or the upload's raid lookups uncached: use the cached loopback route and the per-source per-minute limits in D1.

## Market fetch

- Don't pass the cached `[timestamp, response]` tuple to the parser on a failed fetch: prices fall back to the hard-coded table instead of the saved response.
- Don't start `market_fetch_failed` as `true` and require it false for a cache hit: the saved response never applies on load and every reload fetches.
- Don't write every bundle key on a fetch, fallbacks included: typed prices for unlisted items (Silver, Gold, Enhanced books, special leaps) are reset on each auto-fetch.
- Don't parse the response outside the try: a 200 that is not a list leaves `is_fetching` true for the session.
- Don't keep one global `is_fetching` / `market_fetch_failed`, nor a bypass argument around the in-flight guard: the status shows whichever region finished last. Keep them per region.

## Character import

- Don't seed advanced progress as `adv - floor(adv / 10)` (+25 gave 23, level 22 at 30% xp): use `(adv % 10) * 10`.
- Don't fetch with the custom region (`/character/null/...`), and don't write a result that lands after the name or region changed.
- Don't add a delay to the roster import: the 60-per-minute limiter already spaces requests.
