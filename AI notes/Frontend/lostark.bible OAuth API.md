# lostark.bible OAuth API

Verbatim copy of https://lostark.bible/help/oauth-api (saved by the user 2026-11-17; re-check the page if anything disagrees). Used, optionally, for the character import of the user's own characters; public scraping stays for everyone else (`External services.md`, "Character import"). Permission obtained from the lostark.bible devs. Our own conclusions are in the last section.

---

lostark.bible supports "Login with lostark.bible" for third-party applications, using a standard OAuth 2.0 Authorization Code flow with PKCE. This lets a user grant your app limited, revocable access to their identity and linked rosters without ever sharing their credentials.

Since this is a standard implementation, you can use an existing OAuth client library instead of implementing the flow by hand, e.g. [Arctic](https://arcticjs.dev/) via its generic OAuth2 provider.

## Registering an app

Apps are registered from [your account's Apps page](https://lostark.bible/me/apps). Your account needs developer access enabled first, contact an admin if you don't see the option to create an app.

You may register one app per account. Creating an app gives you a paired **production** and **development** client, each with its own `client_id` (and `client_secret`, for confidential clients) and its own set of redirect URIs. This lets you test against a dev client without touching production credentials or grants.

When creating an app you choose whether it's **confidential** (has a client secret, e.g. a server-side app) or **public** (no secret, e.g. a SPA or native/mobile app). Public clients must use PKCE; confidential clients may optionally use it too. Client secrets are shown exactly once at creation/regeneration time and stored hashed. If you lose one, regenerate it from the Apps page.

Redirect URIs must match exactly (no wildcard/prefix matching) against the ones registered for the client.

## Scopes

Requested scopes are space-separated in the `scope` parameter. Currently available scopes:

- `identify` (the user's lostark.bible user ID and Discord identity)
- `rosters` (the user's linked rosters and their characters)
- `logs` (public logs for characters in the user's linked rosters)
- `loadouts` (equipment, skills and other loadout data for characters in the user's linked rosters)

Characters a user has hidden from their roster (via their log/roster visibility settings) are not returned by the `rosters` scope and cannot be requested through the `logs` or `loadouts` scopes.

## 1. Redirect the user to authorize

Send the user's browser to `GET /oauth/authorize` with the following query parameters:

- `client_id`: required.
- `redirect_uri`: required, must exactly match a registered URI.
- `response_type`: required, must be `code`.
- `scope`: required, space-separated list of scopes.
- `state`: recommended, echoed back unchanged.
- `code_challenge` / `code_challenge_method`: PKCE, `S256` only. Required for public clients.

If the user isn't signed in, they're sent through Discord login first and land back on the consent screen with the original request intact. If they've already granted every requested scope to this app before, the consent screen is skipped and they're redirected straight back with a code.

On approval, the user is redirected to `redirect_uri` with `code` and `state` (if provided) as query parameters. On denial or error, they're redirected back with an `error` parameter (e.g. `access_denied`, `invalid_scope`, `unsupported_response_type`) instead. Problems with `client_id` or `redirect_uri` itself are shown as an error page rather than redirected, since it isn't safe to redirect to an unverified URI.

The authorization code is single-use and expires after 10 minutes.

## 2. Exchange the code for an access token

From your backend (or app, for public clients), `POST` the code to `/oauth/token` as `application/x-www-form-urlencoded`:

```
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&code=...&redirect_uri=...&client_id=...&client_secret=...&code_verifier=...
```

Confidential clients authenticate with `client_secret`, either in the body or via HTTP Basic auth (`Authorization: Basic base64(client_id:client_secret)`): Basic auth takes precedence if both are present. Public clients omit the secret and must supply `code_verifier` instead, matching the `code_challenge` sent to `/oauth/authorize`.

A successful response:

```json
{
  "access_token": "uwo_...",
  "token_type": "Bearer",
  "expires_in": 7776000,
  "scope": "identify rosters logs loadouts"
}
```

Access tokens are opaque bearer tokens prefixed `uwo_` and are valid for 90 days. There is currently no refresh token; once a token expires, send the user through `/oauth/authorize` again (it will auto-approve silently if the grant is still active).

Granting a scope is cumulative: re-authorizing with a new scope adds to the existing grant rather than replacing it.

## 3. Call the API

Send the access token as a bearer token on every request:

```
Authorization: Bearer uwo_...
```

Available endpoints:

- `GET /api/oauth/user`: requires `identify`. Returns the user's id, Discord ID, username, global name, and avatar.
- `GET /api/oauth/rosters`: requires `rosters`. Returns the user's linked rosters and their (non-hidden) characters.
- `GET /api/oauth/logs/{characterName}`: requires `logs`. Returns recent public encounter logs for a character in one of the user's linked rosters.
- `GET /api/oauth/loadouts/{characterName}`: requires `loadouts`. Returns the loadouts of a character in one of the user's linked rosters.

The logs endpoint accepts these query parameters:

- `region`: required. The character's region ID, either `CE` or `NA`.
- `page`: a one-based page number. Defaults to `1`.
- `bosses`: a boss name filter. Repeat the parameter to include multiple bosses.

```
GET /api/oauth/logs/Charactername?region=NA&page=1&bosses=BossName
Authorization: Bearer uwo_...
```

Character names and region IDs are matched case-insensitively. Characters hidden from the roster or without public logs return `404 character_not_found`.

The loadouts endpoint takes a required `region` parameter (`CE` or `NA`) and returns a `loadouts` array, one entry per loadout classification (`most_recent_raid`, `most_recent_chaos_dungeon`, `raid_merged`) that has been recorded for the character. IDs and gem, stat and addon types are numeric game IDs, while class, equipment slot and engraving grade are strings. Characters hidden from the roster, or without any recorded loadouts, return `404 character_not_found`.

```
GET /api/oauth/loadouts/Charactername?region=NA
Authorization: Bearer uwo_...
```

These endpoints are CORS-enabled for browser use. A failed request returns a standard OAuth error body, e.g.

```json
{ "error": "invalid_token", "error_description": "Access token expired" }
```

along with a `WWW-Authenticate` header. `401` means the token is missing, invalid, expired, or the app was disabled; `403` means the token doesn't carry the required scope.

## Revoking access

Your app can revoke a token it issued via `POST /oauth/revoke` (RFC 7009), same auth and body shape as the token endpoint, with a `token` parameter. This always responds `200`, even for an already-invalid token.

Users can revoke access to your app at any time from [their Apps page](https://lostark.bible/me/apps), which immediately deletes the grant and all of its tokens. If your account's developer access is revoked, all access tokens issued by your app's clients stop working immediately too.

---

## Observed responses (not from the page; probed 2026-11-17 with the dev client)
- Token exchange as a public client (PKCE, no secret) from Node worked; `scope` came back as requested. All API responses carry `cache-control: private, no-store`; no rate-limit headers.
- `user`: `{id (number), discordId, discordUsername, discordGlobalName, discordAvatar}`.
- `rosters`: `{rosters: [{region: "NA", world, characters: [{name, class, ilvl, lastUpdate}]}]}`. `class` is an id string (`bard`, `devil_hunter_female`, `yinyangshi`, `holyknight`), not a display name. `ilvl` is the current roster item level (matches the logs' latest `gearScore`, can run ahead of every loadout); `lastUpdate` in **seconds**.
- `loadouts/{name}`: `{loadouts: [...]}`, ~100 KB per character. Each entry: `classification`, `lastUpdated` (**milliseconds**), `itemLevel`, `classId`, `items`, plus gems, cards, elixirs, transcendences, skills, engravings, arkPassive, stats, battlePoint, etc.
  - Entries can be old (a 2025 `most_recent_chaos_dungeon` next to a 2026-11 `most_recent_raid`, the old one still on T3 gear), so pick the newest `lastUpdated`. `raid_merged` appeared only on some characters, same timestamp as `most_recent_raid`.
  - Honing gear: `items[]` with `slot` in `weapon, head, upper_body, lower_body, hand, shoulder` and `bracer` (vambrace, only present on Serca characters), `data.type: "equipment"`, `data.honing` (+N), `data.advancedHoning` (0–40, 10 per level). No per-item item level.
  - Item ids: T3 `1143xxxxx`, T4 `13451xxxx`/`13461xxxx`, Serca `13462xxxx`, vambrace seen `134401206`. Serca pieces seen with `advancedHoning: 40`.
  - Other slots: `neck, ear1, ear2, finger1, finger2` (`tier3_accessory`/`tier4_accessory`), `ability_stone`, `bracelet`, `trinity_core` (`orb`), `avatar_*` (`cosmetic`).
- `logs/{name}`: an **object keyed `"0"`…`"24"`** (not an array) of `{id, name, boss, difficulty, dps, bdps, ndps, rdps, rContribution, buffs, class (display name), spec, gearScore, combatPower, percentile, contributionPercentile, duration, timestamp (ms), isBus, isDead}`.

## How we use it (not from the page)
- Public client, PKCE, scopes `identify rosters loadouts`; done in the browser with `oauth4webapi` (`Utils/BibleAuth.ts`). Token, revoke and API endpoints all answer CORS `*`. Arctic, which the page suggests, is deprecated on npm.
- Client ids: dev `VITE_BIBLE_CLIENT_ID` in `.env.development` (redirect `http://localhost:5173/auth/callback`), production in `.env.production` (`https://honing-forecast.com/auth/callback`, `https://honing-forecast.pages.dev/auth/callback`). Ids are public; nothing secret exists for a public client.
- Redirect URI is `location.origin + "/auth/callback"`, a route placed before the catch-all old `/:characterName` redirect; `AuthCallback.vue` finishes the exchange and returns to the page that started it.
- Token stored with `expires_at = now + expires_in`; a 401 forgets it. Sign-out revokes it. The grant is per user + client, not per domain, so moving domain needs a new sign-in only (silent), not a token copy.
- UI: `SignInHint.vue` (a small muted line under the roster fetch button and on the character page's import card, hidden when signed in); header shows "lostark.bible: <name> · Sign out" when signed in.
- Signed in, characters refresh on launch and rosters sync at sign-in (`Utils/BibleSync.ts`; `External services.md`, "Character import").
- Only the user's linked rosters are reachable; the hourly analytics follow-ups stay on the Worker's public scraping.
- Planned next: backup / cross-device sync keyed by the `id` from `/api/oauth/user`, verified by the Worker.
