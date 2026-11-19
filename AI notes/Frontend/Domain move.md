# Domain move

Moving the site from `honing-forecast.pages.dev` to `honing-forecast.com`. localStorage is per origin, so the old site has to hand each browser's data over itself; a server-side redirect (Bulk Redirects, `_redirects`) would strand it. Same scheme as loa-sim's move off `loa-sim.vercel.app`.

## Switch

`VITE_MIGRATE_FROM` / `VITE_MIGRATE_TO` (origins, no trailing slash), filled into `index.html` and `migrate/index.html` by Vite's `%VITE_…%` HTML replacement. Empty in `.env.development` and `.env.production`, which turns everything off; process env overrides the files (Pages environment variables, Playwright's `webServer.env`). One Pages project serves both domains with the same build, so the sender checks `location.origin === FROM` exactly; branch previews (`<branch>.honing-forecast.pages.dev`) never send.

## Protocol

Keys: old site `hf_moved` (done; later visits only redirect), `hf_move_tries`; new site `hf_imported`, `hf_moved_in` (the parked save), `hf_moved_notice` (`moved` / `failed`, until dismissed), sessionStorage `hf_import_buffer`.

- **Sender** (inline script at the top of `index.html`'s head, before any bundle): on the old origin sets `window.hf_leaving` (`main.ts` then skips mounting) and either redirects to `TO/migrate/#hfm=arrive&to=<path>` (already moved, nothing stored, 3 tries used, any error) or sends: every localStorage key as JSON, `deflate-raw` (`CompressionStream`), base64url, in 500k-character chunks, `TO/migrate/#n&of&len&to&d=<chunk>`. The hash never reaches a server. `?stay` skips it all, so the old site stays usable (the failed banner links there for a manual backup).
- **Receiver** (`migrate/index.html`, a second Vite entry in `vite.config.ts`, inline script, no bundle): `arrive` sets the notice and goes to `to`. Data is accepted only with `document.referrer` on `FROM`, else ignored (a hand-made link must not write into storage). Chunks collect in sessionStorage, each answered with `FROM/#hfm=part&part=N+1`; after the last: decompress, write non-save keys only where missing, park the save keys (`HF_(CONFIG|UI_STATE)_V<n>_COMPRESSED`) in `hf_moved_in`, set `hf_imported`, answer `FROM/#hfm=done`, which marks the old site moved and arrives. Any failure sets notice `failed` and opens the app; the old site retries on its next visits up to 3.
- **App** (`RosterConfig` `init`): if `hf_moved_in` exists and this site's save is fresh (`fresh_profile_id`: one untouched `Newchar`, shared with lostark.bible sync; plus no snapshots), the moved save replaces every save key (`use_moved_save`) and loads. Otherwise it stays parked and the conflict banner offers Download (`moved_backup`, an importable backup), Use it instead (swaps, re-inits the store so `pagehide` doesn't write this site's save back, reloads) and Discard. Visiting the new site at all writes a default save on `pagehide`, which is why fresh counts as empty rather than "no key".
- Banners in `TopBanners.vue` (`Banner.vue`, see `UI components.md`): conflict, failed, then the info "has moved" notice.

## Testing

- `tests/e2e/domain_move.spec.ts`: a second dev server on 5174 (`playwright.config.ts`) with `localhost:5174` as the old site and `127.0.0.1:5174` as the new; storage is seeded on a routed blank `/seed` page so the app's `pagehide` write doesn't interfere. Uses the wasm the first server's `pnpm dev` built.
- Staging: branch `migration2` (`migration2.honing-forecast.pages.dev`) as the old site and `test.honing-forecast.com` (custom domain CNAMEd to that branch alias, proxied) as the new. `VITE_MIGRATE_FROM` / `VITE_MIGRATE_TO` are set as Pages dashboard variables for all environments (no preview-only option); harmless on production since the sender matches the exact origin. Never set `VITE_BIBLE_CLIENT_ID` there: dashboard vars override `.env.production`, and the dev client lacks the pages.dev redirect.
- lostark.bible redirect URIs: production client has `honing-forecast.com`, `honing-forecast.pages.dev` and `test.honing-forecast.com`; dev client has `localhost:5173`, `127.0.0.1:5173`, `test.honing-forecast.com`.

## Launch (not done yet)

Connect `honing-forecast.com` (+ `www`) to the Pages project, wait for Active, then deploy with both vars set in `.env.production` and the dashboard variables deleted (they would override it) or set to the real origins. Also: README links, a canonical link, the Worker's `Referer` (`cloudflare/worker.js`; tell lostark.bible first). The Worker stays on workers.dev. Keep the pages.dev deployment indefinitely; a later 301 would strand whoever hasn't visited since.
