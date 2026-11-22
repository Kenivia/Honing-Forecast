# Remote scanning

Running the scanner on other PCs (a friend's, with its own Steam account and characters) and controlling it from here and from Discord. Built 2026-11-22. Deployed relay: `https://hf-market-relay.kenivia-fan.workers.dev` (secrets set). Tested on this PC as machine `home`, portable layout in `ignored/market/agent_test/` (exe + `data/` + `profile/`), logged into the new Steam account: launch, setup crops, a quick round, pause, partial upload, reconnect.

## Pieces

- **`market_agent`** (`crates/market/src/bin/market_agent.rs`): one console exe. Reads `relay.json` (`{"url", "machine", "token"}`) from the data folder, keeps a WebSocket to the relay (reconnects every 5 to 60 s), and only once connected asks "Start scanning? [Y/n]". From then on the console and the relay are the same controls: Enter / `y` or `/start` starts when idle; F10 or `/pause` stops the pass (`game::STOP`), which uploads what it read, and it is idle again. While scanning it runs rounds of every region with no gap. A pass that panics is reported with a screenshot and leaves it idle. It launches the game through Steam (`steam://rungameid/1599340`) when the game is not running at the start of a round, and waits up to 30 min (Steam may update the game first) for the 1920x1080 window to show character or server select (`Game::try_find`; the splash comes first); Steam itself is started and logged in by hand. Flags: `--dev` (the repo's folders instead of the portable layout, `Overview.md`), `--setup`, `--quick`, `--strips`, `--only`, `--regions`. Default sections: materials, gems, exchange.
- **The relay** (`cloudflare/relay/`, Worker `hf-market-relay`, separate from the site's Worker so its deploys cannot break the site; same account, so the free-plan limits are shared anyway). One Durable Object (`Hub`, SQLite-backed, WebSocket hibernation) holds every agent's socket by machine name, forwards commands and keeps each machine's last status; R2 bucket `hf-market-relay` keeps files. Passes still go to the site's Worker (`POST /scans`), not here.
- **`market_mcp --remote <machine>`**: `screenshot` and `run` go to that machine (run only while it is idle), plus `control` (status, start, pause, launch, log, setup, sync) and `files` (list or download keys to `ignored/market/relay_files/`). Reads `relay_admin.json` (`{"url", "token"}`) from the data folder.

## Outages and updates

- **Offline**: the relay thread sends "ping" every 30 s with its status; the Durable Object answers "pong" without waking (`setWebSocketAutoResponse`). Nothing received for 75 s, or a failed connect, sets `report::OFFLINE` (a dead connection does not error by itself).
- **Failing fast**: a unit of work that fails while `OFFLINE`, or while the game is not running, panics at once instead of being tried again (four tries of every unit would keep a broken round going about half an hour).
- **Recovery** (`scan` in market_agent): a round that panics or reads nothing is reported (with a screenshot when one can be taken), then the game is killed (`taskkill /F`), the agent waits until the relay is reachable, 30 s more, and the next round relaunches the game. Three such rounds in a row and it reports and goes idle. The same covers a game update: the client gets kicked or fails, is killed, and Steam updates it on launch.
- **Tested 2026-11-22** (Wi-Fi off for 3 min with `netsh wlan disconnect`, which needs no admin, during NAE materials): the game closed its own window about 45 s after the network went; the round ended ("could not bring the game to the front"), the game was killed, the relay came back 1 min after the network (reconnects back off to 60 s), the game was launched 30 s later, reached server select in about 3 min, and the round went on and read normally. A unit that failed just before the relay noticed was retried as usual.
- **Self-update**: `pnpm relay-publish` builds `market_agent` and uploads it (`cloudflare/relay/publish.mjs`, `PUT /update`: R2 `update/market_agent.exe` with its SHA-256 as metadata). The agent compares its own SHA-256 (`version` in its status, 8 hex digits) at start, at the start of every round, and on the `update` command when idle; on a difference it downloads (`GET /update/exe`, either token), checks the hash, swaps itself with `self_replace`, and starts itself again with the same arguments in the same console (`--resume` when it was scanning) and exits. Tested: idle and mid-scan, two publishes in a row skip straight to the newest, no restart loop. An exe with bytes appended runs the same and hashes differently, which tests an update without a code change.

## Relay routes

- `GET /agent/<machine>`: the agent's WebSocket, `Authorization: Bearer <MACHINE_TOKENS[machine]>`.
- `PUT /files/<machine>/<path>`: the machine's token or the admin token; the path is percent-encoded (gem crop names have spaces). Keys: `<machine>/shots/` (error screenshots), `<machine>/steps/` (setup mode), `<machine>/profile/{refs,names,gems}/` (`sync`, and after every `run` script with a `ref`).
- `GET /update` (`{sha256}`), `GET /update/exe`: any machine's token or the admin token; `PUT /update`: admin.
- Admin (`ADMIN_TOKEN`): `POST /cmd/<machine>` `{op, ...}` (waits up to 120 s for the reply), `GET /machines`, `GET /files?prefix=`, `GET /files/<key>`.
- `POST /discord`: Discord interactions, checked with the app's Ed25519 public key; only the users in `DISCORD_USERS` may run them.

Agent messages: `status` every 30 s and on each state change (`state` idle/scanning/stopping, `game`, `last_pass`, `log` and `log_at` = the last `ui::log` line, `setup`); `event` (`kind` info/error/step/pass, `text`, `file`); `reply` to a command id. Commands: `status`, `start`, `pause`, `launch`, `log` (`lines`), `setup` (`on`), `sync`, `update`, `shot` (`x y w h scale`), `run` (`script`).

## Discord

- App "1574142665015042318" (ids in `wrangler.toml` `[vars]`); the bot token is the secret `DISCORD_BOT_TOKEN`. Posts go to channel `DISCORD_CHANNEL_ID` through the bot (no webhook).
- Posted: info (started, paused) and error events (with their screenshot attached from R2); a machine disconnected for over 2 min (once, then "back" when it returns); a scanning machine whose log has not moved for 20 min. Not posted: passes (about 150 a day per machine; `/status` shows the last) and setup steps.
- `/start`, `/pause`, `/status`, each with an optional `machine` (default: the only one connected; `/status` without it lists all). Registered on the channel's server by `node cloudflare/relay/register.mjs` (guild commands, instant; run again when they change). Discord wants an answer in 3 s, so these give the agent 2.5 s.
- The Interactions Endpoint URL must be public: `https://<relay>/discord` once deployed. A local relay needs a tunnel (`cloudflared tunnel --url http://localhost:8788`), not installed here yet.

## Running and deploying

- Local: `pnpm relay-dev` (port 8788, secrets from `cloudflare/relay/.dev.vars`, gitignored: `DISCORD_BOT_TOKEN`, `ADMIN_TOKEN`, `MACHINE_TOKENS` = `{"home": ...}`); this PC is machine `home` with `ignored/market/relay.json` and `relay_admin.json` pointing at `http://127.0.0.1:8788`.
- Deploy (the user does it): `wrangler r2 bucket create hf-market-relay` (R2 must be enabled on the account), `wrangler secret put` for the three secrets with `--config cloudflare/relay/wrangler.toml`, `pnpm relay-deploy`, then the Interactions Endpoint URL, then point the configs at the deployed URL.
- A machine of its own: `market_agent.exe` with `data/relay.json` and `data/upload.json` beside it (portable layout); its characters' crops (`char_nae`, `char_euc`, `server_*`) and anything else that differs go in `profile/refs/` beside it, taken remotely with `run "ref ..."`, which also uploads them.

## First run on a new account (2026-11-22)

- Launching through Steam: the process and a 1152x720 splash come at once; the game window stays hidden for about 4 min on the first launch. A first-run "Game Entry Narration Assistance" screen (OFF picked; Continue at 960, 682) comes before server select.
- The new account (characters Ialsolovemarket on NAE Thaemine, Ilovemarketprice on EUC Arcturus, both slot 1 of page 1) matched every crop of this PC (server, tabs, character select) except the characters; `char_nae` and `char_euc` were taken remotely with `ref char_<region> 236 896 216 48` into `agent_test/profile/refs/`. F9 opened the shop on both.
- A character's first Stronghold visit opens a "Guide" window; Esc closes it.
- `in_stronghold` (the Exit and Teleport icons, translucent) failed on the new EUC character: it stands elsewhere in its Stronghold, over bright sand, and the full crop read 39. It is now masked (below): Stronghold screens read 1.3 to 7.0 (with the Guide window too), every other kept screen 33.9 or more.

## Open

- A real game update has not happened yet: the recovery path should cover it, unchecked. A login queue after maintenance would outlast the switch's 90 s wait for character select.
- D1 storage: at no gap each machine stores about 9 MB of passes a day in `hf-analytics` (free plan: 500 MB a database); to be addressed later.
