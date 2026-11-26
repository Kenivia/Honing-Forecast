# Overview

Honing Forecast is a Lost Ark honing (gear upgrade) calculator. Given the upgrades a character wants, the materials they own and market prices, it finds the juice / book / free-tap plan that minimises **average gold spent**, and shows the cost distribution per material.

The core idea, which drives most of the complexity: owned materials are subtracted from the cost *per outcome, before averaging*. Leftover bound materials are worth nothing (or a lower price), so gold cost is a piecewise-linear function of total consumption and `E[f(X)] != f(E[X])`. See `README.md` for the motivating example.

These notes are high-level. They cover what is hard to work out from reading one file; they do not track constants, thresholds or UI details.

## Repo map

| Path | What it is |
| --- | --- |
| `crates/core` (`hf-core`) | All honing maths: domain model, average-gold evaluation, optimizer. |
| `crates/scanner` (`hf-scanner`) | Screen scanner image processing. In development. |
| `crates/market` (`hf-market`) | Native market scanner: drives the game's Market and Auction House and reads prices, gems and accessories into SQLite. |
| `crates/wasm` (`hf-wasm`) | Thin `wasm_bindgen` wrappers over core and scanner. The only crate the frontend loads. |
| `crates/arena` (`hf-arena`) | Native test harness for the core: Monte Carlo correctness, optimizer accuracy and speed, results in SQLite. |
| `frontend/` | Vue 3 + Pinia + Tailwind app. `@` aliases to this folder. |
| `cloudflare/` | A Worker that proxies and caches market prices and character lookups, and stores analytics uploads and the market scanner's passes in D1 (`schema.sql`). |
| `public/` | Static assets, changelogs, `WIP.md` (the author's roadmap), scanner config and OCR model. |
| `scanner_setup/` | Everything that builds or calibrates the scanner config: source icons (`templates/`), game-file extraction, brightness calibration, anchor setup, the browser profiling scripts. |
| `scripts/` | Changelog helper, the pre-deploy check, analytics pull and analysis (`analytics/`), a market price and population viewer (`market/`, `pnpm market`, `Forecasting/Market.md`), the Income page's mock save (`mock income/`) and game-data helpers (`game_files/`, `json_parser/`, `arena/`); everything scanner-specific lives in `scanner_setup/` instead. |
| `tests/` | Playwright e2e tests (`tests/e2e/`), node-side frontend tests (migrations, the real-save fixture, cross-tier relations, `pnpm test:node`), Vitest unit and component tests (`unit/`, `pnpm test:unit`), and test fixtures (real saves `V7 config`, `v8 config`, `V10 config`, `V10 config 2`, `V11 config`, `v12 config.json`; `test-results/`). |
| `arena_data/` | Arena scenarios and its gitignored results database, plus the Python report scripts that read it. |

Not source of truth: `dist/` (committed build output), `crates/wasm/pkg/` (wasm-pack output), and `ignored/` (untracked: `junkyard/`, `data_fetcher/`, `game_file_reader/`, old saddlepoint reference PDF, the overnight evaluator experiments, scanner recordings).

## Build and run

- `pnpm dev` builds the wasm package with wasm-pack, then starts Vite. Vite does not watch `crates/`, so **Rust changes need `pnpm run wasm` again**.
- `pnpm test:e2e` runs the browser tests; `pnpm browse` runs a one-off browser script. See `Browser harness/Overview.md`. `pnpm perf` benchmarks interaction latency on a production build (`Browser harness/Performance.md`).
- `pnpm check` runs `vue-tsc --noEmit` over the `.ts` layer and `.vue` templates. `pnpm test:node` runs the node-side suites in `tests/`, `pnpm test:unit` the Vitest unit and component tests in `tests/unit/`. All are fast; run them after a frontend change.
- `pnpm test:rust` runs `cargo test -p hf-core -p hf-arena` (the core's unit tests, plus a short Monte Carlo check and a seed-repeats check from the arena), then `cargo test -p hf-scanner --lib`.
- `pnpm test` runs every suite in turn: Rust, node, Vitest, then the browser tests in both browsers (those take several minutes and start `pnpm dev`, which rebuilds the wasm).
- `pnpm arena <command>` runs the core's test harness. See `Calculator core/Arena.md`.
- `pnpm build` chains `pnpm test` (every suite) before the wasm and Vite builds. `pnpm no-test-build` skips the tests.
- `pnpm updates` runs `rustup update`, `cargo update` and `pnpm update`: within the declared semver ranges only, never across a major version. Both sides have a release-age cooldown: Cargo `registry.global-min-publish-age = "10 days"` in `.cargo/config.toml` (needs Rust 1.100+; an older cargo warns and ignores it), pnpm `minimumReleaseAge: 6767` (minutes) in `pnpm-workspace.yaml`. `cargo upgrade` (cargo-edit) and `pnpm update --latest` cross majors and are deliberately left out.
- `pnpm deploy` only checks (`scripts/pre_deploy_check.ps1`) you are on a clean `main`, then force-pushes (`--force-with-lease`) `main` to the `Production` branch (assumed to be what the hosted site builds from). It does not run tests.
- `pnpm cloudflare-deploy` deploys the Worker separately; `pnpm cloudflare-dev` runs it remotely.
- `pnpm pull-analytics` downloads and deletes the analytics uploads, `pnpm analytics` pulls and then serves the plots (`Frontend/External services.md`).
- `pnpm market-scan` runs the market scanner (materials and gems) in a loop; see `Market scanner/Overview.md`.
- `pnpm change-log` scaffolds the next changelog file in `public/change-logs`.
- `pnpm browsers` installs the Playwright browsers; `pnpm scanner-suite`, `scanner-check`, `game-capture`, `scanner-profile` and `scanner-compare` are the scanner's tools (`Screen scanner/`). `pnpm remove-tmp` clears `.tmp/` (`Working in this repo.md`).

## Cargo features

- `hf-core` has one feature, `trace`: it fills in `Probe`, the instrumentation struct. Off in wasm, where it is empty; on in the arena.
- Nothing else in `hf-core` depends on a feature. Browser-only code is picked by `target_arch = "wasm32"` (wasm-only dependencies `web-sys`, `getrandom/wasm_js`), and timing uses `web_time::Instant`, which works on both.
- `my_dbg!` (`helpers.rs`) prints to the browser console in wasm and to stderr natively. It calls a function of `hf-core`, so its `cfg` is `hf-core`'s, not the calling crate's.
- The optimizer's progress goes through a hook (`optimizer/progress.rs`): `hf-wasm` sets one that posts to JS. With none set (native), the annealer skips preparing it.
- The wasm target is built with SIMD enabled (`.cargo/config.toml`).

## Game glossary

- **Tap**: one honing attempt. It costs materials whether or not it succeeds.
- **Normal honing**: +N to +N+1, with a success chance that rises after each failure.
- **Artisan / pity**: a meter that fills on failed taps; when full, the next tap is guaranteed.
- **Advanced honing**: a separate XP-based system in blocks of ten levels, with "grace" taps and its own consumables.
- **Juice, books, scrolls**: optional consumables that raise the success chance of a tap. Juices and books apply to normal honing, juices and scrolls to advanced. In code they share one id space ("juice ids").
- **Special honing / free tap**: an attempt paid for with special leapstones. Success skips the upgrade entirely.
- **Bound / roster-bound / tradable**: the three ownership bands of a material. What a leftover unit of each is worth is the user's choice, out of nothing / taxed sell price / buy price, and must not decrease across the bands. Main and Alt on the character card are presets (`MAIN_BANDS`, the default: nothing / nothing / taxed sell; `ALT_BANDS`: nothing / taxed sell / taxed sell); each band is fine-tuned next to "Avg value of leftover mats" under the cost distribution.
- **Tier**: 0 is T4, 1 is Serca. Each has its own material set and cost tables.
- **Express / event**: temporary modifiers to costs and chances.

## Index

Each note describes the code as it is now. Where something was tried and failed or ruled out, a `<note> tried.md` beside the note holds a "don't try X: why" warning. **Read a tried file only when trying something new** in its area. When changing something, update the main note to the new state; add to the tried file only a warning against an alternative that failed, never a changelog. `Calculator core/` keeps its tried files and other learnings (measurements, history, shelved work) in `Calculator core/Further learnings/`, the main folder holding only instructions and documentation.

- `Calculator core/Domain model.md`: the structs and index conventions.
- `Calculator core/Evaluation pipeline.md`: payload to result, method selection, invariants.
- `Calculator core/Optimizer.md`: what the annealer does and may and may not change.
- `Calculator core/State space.md`: what a normal honing plan is, and the exact best plan of one upgrade.
- `Calculator core/Arena.md`: the native test harness: variants, snapshots, scenarios, sets, what it measures.
- `Calculator core/Optimizer lab.md`: how optimizer investigations run: file layout, `pnpm lab`, tools, smoke tests and how results are judged.
- `Calculator core/Further learnings/`: the tried files of the notes above; `Optimizer lab results.md` (what the investigations found: candidates, what makes a result good, open questions); `Lab history.md` (snapshots in `runs.sqlite`, iterations, changes that split the record); `Marginal value.md` and `Maxroll override.md` (shelved or dead code); `Honing data in the game files.md`.
- `Working in this repo.md`: line endings, scratch space, what the type checker cannot see, and the test suites. Read before a large edit.
- `Frontend/State and persistence.md`: the store, data scopes, localStorage and migrations.
- `Frontend/Domain move.md`: handing each browser's localStorage from pages.dev to honing-forecast.com; staging and launch steps.
- `Frontend/Wasm contract.md`: payload building, worker protocol, what must match Rust.
- `Frontend/External services.md`: market prices, character import, analytics uploads, market scanner passes, changelogs.
- `Frontend/UI components.md`: shared components; `HoverTip`, the site's hover tooltip.
- `Screen scanner/Pipeline.md`: capture to recognised slots, and what is still stubbed.
- `Screen scanner/Tooltips.md`: reading the hover tooltip, chests among it, and tying it to a slot.
- `Screen scanner/Config and calibration.md`: the template config and brightness calibration.
- `Screen scanner/Icon matching bench.md`: the offline bench for how a slot is matched to an icon.
- `Screen scanner/Slot grid.md`: the scanner page's slot grid, slot statuses and manual edits.
- `Screen scanner/Game files.md`: reading items, chests and icons out of the installed game.
- `Screen scanner/Currency.md`: silver, gold and shards off the inventory, the currency window, the bar at the top of the screen and gold's tooltip.
- `Screen scanner/Live game.md`: driving the game on this PC, stills and captures of it (`pnpm game-capture`), and the local test inputs with known answers (`pnpm scanner-check`).
- `Screen scanner/Manifest.md`: the scan as materials per band and chests, merged with saved pages and other characters, the export, and the budget snapshots.
- `Screen scanner/Income.md`: the Income page: snapshots over time, their optimizer runs, the income estimate and manual income.
- `Screen scanner/Choices.md`: the Choices page: what taking each option of a choice (a raid's bonus chest) is worth, and adding it to the budgets.
- `Market scanner/Overview.md`: the native market scanner: running it, input, getting around the Market, reading numbers by the game's font and items by crops, what a pass reads, storage.
- `Forecasting/Market.md`: price and population data on file toward forecasting prices, the gap estimate, and observations so far.
- `Forecasting/HF analytics.md`: what the site's own analytics uploads offer for forecasting so far.
- `Forecasting/Community.md`: community income estimates per content (sites, their tables, how each values a run) for cross-checking.
- `Browser harness/Overview.md`: driving the site in Chromium and Firefox, for debugging and e2e tests. One note per site section alongside it:
  - `Browser harness/Calculator.md`: the calculator, market and roster pages.
  - `Browser harness/Screen scanner.md`: the scanner page, fed by an uploaded image or video.
  - `Browser harness/Performance.md`: `pnpm perf`, interaction and route-switch latency on a throttled production build.
