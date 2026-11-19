# Domain move

Moving the site from `honing-forecast.pages.dev` to `honing-forecast.com`. localStorage is per origin, so the old site has to hand each browser's data over itself; a server-side redirect (Bulk Redirects, `_redirects`) would strand it. Same scheme as loa-sim's move off `loa-sim.vercel.app`.

## Switch

`VITE_MIGRATE_FROM` / `VITE_MIGRATE_TO` (origins, no trailing slash), filled into `index.html` and `migrate/index.html` by Vite's `%VITE_…%` HTML replacement. Empty in `.env.development` and `.env.production`, which turns everything off; process env overrides the files (Pages environment variables, Playwright's `webServer.env`). One Pages project serves both domains with the same build, so the sender checks `location.origin === FROM` exactly; branch previews (`<branch>.honing-forecast.pages.dev`) never send.

## Protocol

Keys: old site `hf_moved` (done; later visits only redirect), `hf_move_tries`; new site `hf_imported`, `hf_moved_in` (the parked save), `hf_moved_notice` (`moved`: the info banner, shown on one page load ever, after which the app sets `seen`, which the receiver never overwrites, so old links don't bring it back; `failed`: until dismissed, which also sets `seen`), sessionStorage `hf_import_buffer`.

- **Sender** (inline script at the top of `index.html`'s head, before any bundle): on the old origin sets `window.hf_leaving` (`main.ts` then skips mounting) and either redirects to `TO/migrate/#hfm=arrive&to=<path>` (nothing stored, 3 tries used, any error), or, once moved (`hf_moved`, also set when there is nothing to send), straight to `TO<path>`, skipping the receiver (one fewer page load, ~100 ms), or sends: every localStorage key as JSON, `deflate-raw` (`CompressionStream`), base64url, in 500k-character chunks, `TO/migrate/#n&of&len&to&d=<chunk>`. The hash never reaches a server. `?stay` skips it all, so the old site stays usable (the failed banner links there for a manual backup).
- **Receiver** (`migrate/index.html`, a second Vite entry in `vite.config.ts`, inline script, no bundle): `arrive` sets the notice and goes to `to`. Data is accepted only with `document.referrer` on `FROM`, else ignored (a hand-made link must not write into storage). Chunks collect in sessionStorage, each answered with `FROM/#hfm=part&part=N+1`; after the last: decompress, write non-save keys only where missing, park the save keys (`HF_(CONFIG|UI_STATE)_V<n>_COMPRESSED`) in `hf_moved_in`, set `hf_imported`, answer `FROM/#hfm=done`, which marks the old site moved and arrives. Any failure sets notice `failed` and opens the app; the old site retries on its next visits up to 3.
- **App** (`RosterConfig` `init`): if `hf_moved_in` exists and this site's save is fresh (`fresh_profile_id`: one untouched `Newchar`, shared with lostark.bible sync; plus no snapshots), the moved save replaces every save key (`use_moved_save`) and loads. Otherwise it stays parked and the conflict banner offers Download (`moved_backup`, an importable backup), Use it instead (swaps, re-inits the store so `pagehide` doesn't write this site's save back, reloads) and Discard. Visiting the new site at all writes a default save on `pagehide`, which is why fresh counts as empty rather than "no key".
- Banners in `TopBanners.vue` (`Banner.vue`, see `UI components.md`): conflict, failed, then the info "has moved" notice.

## Monitoring

Every outcome is reported to the Worker, whatever the analytics opt-out, in every real mode including `staging` (`Utils/MoveReport.ts`). The hand-off pages never send anything, so reporting costs the move nothing: `migrate/index.html` parks `hf_move_report` and the loaded app (`App.vue`, `flush_move_report`) POSTs it to `/moves` once per outcome per browser (`hf_move_reported` lists what was sent).

- Outcomes: `moved` / `conflict` (the receiver's `imported`, settled by `RosterConfig` `init`), `empty`, `gave_up`, `bad_part`, `error` (from the sender via the `why` / `err` hash on `arrive`), `failed` (receiver, with the error), and the conflict banner's `chose_download` / `chose_old` / `chose_discard`. A hand-made link reports nothing.
- Worker `handleMove`: one row in D1 `moves` (received, site (the arrival host, so staging rows stand apart), version (`LATEST_VERSION`, or the mode outside production), outcome, error, chunks, bytes); no names, no save data. Before deploying the Worker, create the table remotely: `wrangler d1 execute hf-analytics --remote --file schema.sql` (from `cloudflare/`; `IF NOT EXISTS`, so safe to rerun).
- Reading: `wrangler d1 execute hf-analytics --remote --command "select site, outcome, count(*) from moves group by 1, 2"`. Not part of `pull-analytics`.
- Blind spot: browsers that never reach the new site's app (domain down, a redirect bug). Compare pages.dev visits in Cloudflare Web Analytics against the rows.

## Testing

- `tests/e2e/domain_move.spec.ts` (stubs `/moves` and checks each case's reported outcomes): a second dev server on 5174 (`playwright.config.ts`) with `localhost:5174` as the old site and `127.0.0.1:5174` as the new; storage is seeded on a routed blank `/seed` page so the app's `pagehide` write doesn't interfere. Uses the wasm the first server's `pnpm dev` built.
- `pnpm test:e2e:staging` (`playwright.staging.config.ts`): the same spec against the deployed staging pair, no local server; deploy `pnpm staging-build` first, since it tests whatever is deployed. The deployed sites inject Cloudflare Web Analytics, which the spec stubs (its beacon logs a CORS error).
- Staging: branch `migration2` (`migration2.honing-forecast.pages.dev`) as the old site and `test.honing-forecast.com` (custom domain CNAMEd to that branch alias, proxied) as the new. Pages serves the committed `dist/` (no build on Cloudflare), so the domains are baked in locally: `pnpm staging-build` (`--mode staging`, reading the committed `.env.staging`: production Worker and lostark.bible client plus both move vars) builds `dist` for the `migration2` branch; rebuild normally before the next real deploy. Mode `staging` also keeps analytics uploads off (`Analytics.ts` uploads only in `production` / `development`). Setting the vars in a shell instead is easy to lose (one staging push went out with them empty).
- lostark.bible redirect URIs: production client has `honing-forecast.com`, `honing-forecast.pages.dev` and `test.honing-forecast.com`; dev client has `localhost:5173`, `127.0.0.1:5173`, `test.honing-forecast.com`.

## Launch (not done yet)

Connect `honing-forecast.com` (+ `www`) to the Pages project, wait for Active, then deploy with both vars set in `.env.production`, built locally like every deploy. Also: README links, a canonical link, the Worker's `Referer` (`cloudflare/worker.js`; tell lostark.bible first). The Worker stays on workers.dev. Keep the pages.dev deployment indefinitely; a later 301 would strand whoever hasn't visited since.
