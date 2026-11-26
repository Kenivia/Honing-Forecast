# Remote scanning

Running the scanner on other PCs (a friend's, with its own Steam account and characters) and controlling it from here and from Discord. Relay: `https://relay.honing-forecast.com`; the old `hf-market-relay.kenivia-fan.workers.dev` stays on (`workers_dev = true`) until nothing uses it.

## Pieces

- **`market_agent`** (`crates/market/src/bin/market_agent.rs`; its header and `--help` list the behaviour and switches): one console exe. It reads `relay.json` (`{"url", "machine", "token"}`) from the data folder and keeps a WebSocket to the relay, reconnecting every 5 to 60 s. Only once connected does it ask "Start scanning? [Y/n]". From then on the console and the relay are the same controls: Enter or `/start` starts, F10 or `/pause` stops the pass (which uploads what it read) and goes idle.
  - **Rounds**: a round of every region at once, then one at every multiple of the gap after the last ended (UTC; default 120 min; `--gap`, else the last set remotely in `data/gap`). After a round that read something it closes the game and waits. 0 runs rounds back to back with the game kept open. A changed gap counts at once, also mid-wait. A round that failed recovers at once, never waits.
  - **Awake**: while scanning or waiting it holds `SetThreadExecutionState(SYSTEM_REQUIRED | DISPLAY_REQUIRED)` (`input::awake`); the screen must stay on (`Lost Ark client.md`, "Life cycle"). The power saving is the game being closed.
  - **Launching**: when the game is not running at a round's start, it launches it through Steam and waits up to 30 min for the 1920x1080 window to show character or server select. Steam is started and logged in by hand. There is no autostart and the console stays visible.
- **The relay** (`cloudflare/relay/`, Worker `hf-market-relay`, separate from the site's Worker so its deploys cannot break the site; same account and free-plan limits). One Durable Object (`Hub`, SQLite, WebSocket hibernation) holds every agent's socket by machine name, forwards commands and keeps each machine's last status. The R2 bucket `hf-market-relay` keeps files. Passes still go to the site's Worker (`POST /scans`).
- **`market_mcp --remote <machine>`**: `screenshot` and `run` go to that machine (`run` only while it is idle), plus `control` (status, start, pause, launch, log, setup, sync, update), `exec` and `files` (list or download keys to `ignored/market/relay_files/`). It reads `relay_admin.json` (`{"url", "token"}`).

## Outages and recovery

- **Offline**: the agent pings every 30 s with its status, and the Durable Object answers without waking (`setWebSocketAutoResponse`). Nothing received for 75 s, or a failed connect, sets `report::OFFLINE`; a dead connection does not error by itself.
- **Failing fast**: a unit that fails while `OFFLINE`, or while the game is not running, panics at once instead of being retried. Four tries of every unit would keep a broken round going for half an hour.
- **Recovery** (`scan` in `market_agent`): a round that panics or reads nothing is reported (with a screenshot when one can be taken). Then the game is killed, the agent waits until the relay is reachable plus 30 s, and the next round relaunches it. Three such rounds in a row and it reports and goes idle. A game update takes the same path: the client is kicked or fails, is killed, and Steam updates it on launch. With the network lost, the game closes its own window within about a minute and the same path brings it back.
- **Maintenance**: the crop `region_closed` is checked in the launch wait and after every failed unit (`ui::check_closed`). It panics with `Closed`: no strike, an info event with the screen, and a rest to the next slot (with no gap: 4 h, then every 2 h while still closed). Both regions go down together, so the round is skipped; passes already read stay uploaded. Kept screens are in `ignored/market/maintenance/`.

## Security

- **Self-update**: `pnpm relay-publish` (`cloudflare/relay/publish.mjs`) builds `market_agent` with `MARKET_BUILD` set to unix seconds (0 for a plain cargo build), signs `hf-market-agent:<build>:<sha256>` with the Ed25519 key `ignored/market/update_signing_key.pem` and uploads it. The agent has the public key built in (`crates/market/update_key.hex`) and checks at start, at every round and on `update` when idle. It takes a build only with a higher number and a valid signature, checks the hash, swaps itself (`self_replace`) and restarts in the same console with the same arguments (`--resume` when it was scanning). A bad signature is reported and refused, and rollback is impossible.
- **Signed commands** (`sign.rs`): `status`, `start`, `pause` and `gap` are taken as sent (`gap` only stores 0..=1440). Every other op must come as `{op, signed, sig}`, where `signed` holds `n` (unix ms) and `sig` signs `hf-market-cmd:<machine>:<signed>` with the update key. It is refused when signed for another machine, when `n` is not above the last one taken (`data/cmd_counter`) or is over 10 min old, or when the signature is bad (which is also reported). `market_mcp` signs with the key file. So the admin token alone can only start, pause and read status.
- **`exec`** (signed) runs a PowerShell script on its own thread, in any state, and replies `exit <code>` with every stream merged (the last 200 KB), wrapped so that stderr is not CLIXML. A bare `exit` in the script loses the buffered output; `throw` keeps it. The relay gives up after 120 s and the process keeps running. The agent logs only that a command ran, never the script, since the last log line goes to Discord and a script may hold tokens.
- **Files beside the exe**: Windows looks beside the exe first for programs started by bare name and for DLL imports. So `cmd`, `powershell` and `taskkill` run by full System32 path, and `crates/market/build.rs` links the bins with `/DEPENDENTLOADFLAG:0x800` (DLL imports from System32 only). Keep any new `Command::new` on a full path.
- **The signing key** exists only on this PC; back it up. If it is lost, no agent takes an update or a signed command again, and a new key means a new exe sent by hand. If it leaks, rotate it the same way. The relay, R2 and the admin token cannot push code or run anything; at worst they block updates and start or pause scans.

## Relay routes and messages

- `GET /agent/<machine>`: the agent's WebSocket, `Authorization: Bearer <MACHINE_TOKENS[machine]>`.
- `PUT /files/<machine>/<path>` (percent-encoded): the machine's token or admin. Keys go under `files/<machine>/shots/` (error screenshots), `steps/` (setup mode) and `profile/{refs,names,gems}/` (`sync`, and after any `run` with a `ref`). Each is up to 20 MB (413), and each machine up to 1 GiB a UTC day (429 and one Discord post). An event's `file` is attached to Discord only from the sender's own folder.
- `GET /update` and `/update/exe`: any machine token or admin. `PUT /update` (headers `x-build`, `x-signature`): admin.
- Admin: `POST /cmd/<machine>` `{op, ...}` (waits up to 120 s), `GET /machines`, `GET /files?prefix=`, `GET /files/<key>`.
- `POST /discord`: interactions, checked with the app's public key, from `DISCORD_USERS` only.
- Unauthenticated requests are refused before the Durable Object or R2, but still count toward the account's 100k Worker requests a day; only the zone's WAF rule stops them (`Frontend/External services.md`, "Per-IP limits").
- **Agent messages**:
  - `status`, every 30 s and on each change: `state` (idle, scanning, waiting or stopping), `game`, `last_pass`/`last_pass_at`, `log`/`log_at` (the last `ui::log` line), `setup`, `gap`, `next` (unix seconds of the next round), `version`.
  - `event`: `kind` info, error, step or pass, with `text` and `file`.
  - `reply`, taken only from the machine the command went to.
- **Commands**: `status`, `start`, `pause` and `gap` are unsigned. `launch`, `log`, `setup`, `sync`, `update`, `shot`, `run` and `exec` are signed.

## Cost

Only R2 bills: Workers and Durable Objects on the free plan fail past their limits instead. Nothing reaches R2 without a token, and the bucket has no public access. Storage is bounded by the size and daily caps above and by lifecycle rules deleting `files/` after 7 days (`files-7d`; `old-home-7d` for keys from before the prefix). Setup mode turns itself off after a round or a pause. The user is to set a budget alert, since spend limits do not cover R2.

## Discord

- The app's ids are in `wrangler.toml` `[vars]`, the bot token in the secret `DISCORD_BOT_TOKEN`. Posts go to `DISCORD_CHANNEL_ID` through the bot, with `allowed_mentions` empty.
- **Posted**: info and error events (with their screenshot); a machine disconnected for over 2 min (once, then "back"); a scanning machine whose log has not moved for 20 min, counted from the later of its last log line and its last move into scanning (`scan_at`). Ages show the largest whole unit. **Not posted**: passes (about 150 a day per machine; `/status` shows the last one) and setup steps.
- `/start`, `/pause`, `/gap minutes`, `/status`, each with an optional `machine` (default: the only one connected). They are registered as guild commands by `node cloudflare/relay/register.mjs`; run it again when they change. Discord wants an answer within 3 s, so the agent gets 2.5. The relay forwards only start, pause and gap from Discord, so a compromised Discord account can do nothing more.
- The Interactions Endpoint URL is `https://<relay>/discord`.

## Deploying

The user does it: `wrangler r2 bucket create hf-market-relay`, `wrangler secret put` for the three secrets (`--config cloudflare/relay/wrangler.toml`), `pnpm relay-deploy`, the Interactions Endpoint URL, then the configs pointed at the URL. Local testing is in `Developing and testing.md`.

## Machines

- `home` is this PC, running the portable layout in `ignored/market/agent_test/`, logged into the scanner's Steam account (characters Ialsolovemarket on NAE Thaemine, Ilovemarketprice on EUC Arcturus, slot 1 of page 1).
- `borry-laptop` is the friend's: `C:\Users\omgwt\Downloads\borry-laptop\` (a subfolder, so downloads do not land beside the exe). It has a 2560x1440 monitor with the game set to Full Screen 1920x1080, unactivated Windows (its own `buy_button` crop, watermark included; delete it if Windows is activated), panel brightness set to 0 (`WmiSetBrightness`), and a UTC+8 clock. It was packaged as `ignored/market/borry-laptop.zip` from `friend_package/`.
- **Adding a machine**: `market_agent.exe` with `data/relay.json` and `data/upload.json` beside it, and a token in `MACHINE_TOKENS`. Machine names go into URLs unencoded, so no spaces.
  - On a new account the crops of this PC (server, tabs, character select) have matched every time except the characters. Take `char_nae` and `char_euc` remotely with `run "ref char_<region> 236 896 216 48"` (which uploads them), and anything else that differs the same way.
  - Set F9 to the shop on each character.
  - Expect the first-launch narration screen and a Guide window on each character's first Stronghold visit.
  - The agent reads `relay.json` only at start (`upload.json` on every upload). Change them with a signed `exec`, then restart it (a `relay-publish` does).

## Open

- A real game update has not happened yet: the recovery path should cover it, unchecked. A login queue after maintenance would outlast the switch's wait for character select. The end of a maintenance has not been seen.
- D1 storage: at no gap each machine stores about 9 MB of passes a day in `hf-analytics` (free plan: 500 MB a database). Deferred by the user.
