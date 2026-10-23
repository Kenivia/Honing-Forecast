# External services: history

## Market fetch cache and failures (Oct 2026)

Before:
- A failed fetch passed the `[timestamp, response]` tuple to the parser, so prices fell back to the hard-coded table instead of the saved response.
- `market_fetch_failed` started `true` and a cache hit required it false, so the saved response never applied on load: every reload fetched both regions.
- Every fetch wrote all bundle keys, fallbacks included, so typed prices for unlisted items (Silver, Gold, Enhanced books, special leaps) were reset on each auto-fetch.
- A 200 response that was not a list threw outside the try and left `is_fetching` true for the session.
- One global `is_fetching` / `market_fetch_failed`; App.vue bypassed the in-flight guard (`ignore_fetch_cooldown`) to fetch both regions, and the status showed whichever finished last.

After: per-region flags (the bypass argument is gone), only listed bundles are written, the cache applies on load and on failure, and parsing is inside the try. Covered by `tests/unit/market-import/market_fetcher.test.ts`.

## Character import (Oct 2026)

Before: imported advanced progress was `adv - floor(adv / 10)` (+25 gave 23, i.e. level 22 with 30% xp) instead of 50. The roster import had an un-awaited `setTimeout(…, 500)` that did nothing. A name change in the custom region still fetched `/character/null/…`. Results that landed after the name or region changed were written anyway.

After: `(adv % 10) * 10` in a fresh array, the no-op is gone (the 60/min limiter spaces requests), the custom region never fetches, and late results are dropped.

## Worker cache and limits (Oct 2026)

Before: market and character responses cached in KV for an hour (free plan: 1k KV writes a day); the market was a POST keyed by a hash of its body; upload raid lookups were never cached, and nothing limited upstream fetches. The classic Cache API (`caches.default`) was ruled out: it does nothing on workers.dev, only on custom domains.

After: Workers Cache, GET-only routes (market moved to `GET /market/<region>?items=`), raids through a cached loopback route, per-source per-minute limits in D1, upload bodies capped at 300 KB (was 1 MB).
