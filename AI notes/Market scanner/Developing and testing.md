# Developing and testing the market scanner

How to explore a screen, teach the scanner a new one, check the readers, and try a change without breaking the machines that run it. Driving the game in general is in `Lost Ark client.md`.

## Exploring a screen

- Use `lostark-market` (or `market_ctl "<script>"` from a shell): `screenshot` to look, `run` to click and read. The language (`ctl.rs`) can read: `ocr x0 x1 y0 y1` (PP-OCRv5), `num x0 x1 y0 y1` (the font reader), and `looks <name> [dy]` (a reference crop's distance, which says whether a step's check would pass on this screen).
- **A new reference crop**: once the screen shows the right thing, `ref <name> x y w h` saves `refs/<name>@x_y.png` into the profile (the crate's `refs/` in dev). It is used at once, with no rebuild. Check it with `looks` on this screen and on a few others it must not match (the kept screens in `ignored/market/debug/` and `maintenance/`). Pick pixels that do not move with what is behind a translucent window, or give the crop alpha so those are not compared (as `in_stronghold` does).
- Name blocks, gem crops and item art work the same way (`names/`, `gems/`, `items/`); the item art is generated (`market_items.py`).

## When a screen changes

A unit that fails every try with the same screenshot means the step's check no longer matches:

1. Find the step in the code that drives that screen (`market.rs`, `auction.rs`, `exchange.rs`, `region.rs`, `ui.rs`).
2. Look at it with `screenshot`, take the crop again with `ref`, and move any click whose button moved.
3. Run the unit: `market_scan --once --quick --only <section> [--regions NAE]`.
4. If it reads numbers or text, run with `--strips` and check them (below).

## Checking the readers

- `--strips` keeps every crop read, with what was read and its item or search, in `ignored/market/strips/<time>/` (`index.jsonl`, about 1,900 crops a pass; `strips.rs`).
- `python crates/market/tools/verify_strips.py [folder]` (onnxruntime, numpy, pillow) reads them again with PP-OCRv5 in Python and draws every disagreement into `disagree_<n>.png`. For numbers it is an independent second reader; for text it checks the Rust port.
- `glyph_eval` scores the font reader on `ignored/market/dataset/labels.json` (102 labelled numbers over every number style; all read). `glyph_eval <strips folders>` reads their number crops again and prints every read that changed, which is how to check a reader change against real passes. `--debug` prints the closest digits per glyph.
- `pp_eval <strips folder>` prints the Rust text reads of a folder's text crops.
- `input_test`: which ways of sending input the game takes (run it if input ever stops working).

## Building and running beside the machines

- Never build a bin while it runs (Windows locks the exe): `CARGO_TARGET_DIR=<scratch>` for a build next to a running scan. Relinking `market_mcp` needs the Claude Code sessions holding it closed or the server disabled.
- `market_scan` and `pnpm market-agent` (dev layout) on this PC write `ignored/market/`. The portable layout is tested in `ignored/market/agent_test/` (exe, `data/`, `profile/`) as machine `home`.
- **The relay locally**: `pnpm relay-dev` (port 8788, secrets in `cloudflare/relay/.dev.vars`), with `ignored/market/relay.json` and `relay_admin.json` pointing at it. Discord needs a public URL (a tunnel), so test the bot against the deployed relay.
- **Shipping**: the user deploys the Worker and relay. `pnpm relay-publish` builds, signs and publishes `market_agent`; every agent takes it at its next round (`Remote.md`, "Self-update"). Test a change on `home` first: a bad build reaches every machine.
- **A crop for one machine only** (its characters, a watermark) goes into that machine's `profile/refs/` with `run "ref ..."` through its MCP server, which also uploads it.
