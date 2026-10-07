# Overview

Honing Forecast is a Lost Ark honing (gear upgrade) calculator. Given the upgrades a character wants, the materials they own and market prices, it finds the juice / book / free-tap plan that minimises **average gold spent**, and shows the cost distribution per material.

The core idea, which drives most of the complexity: owned materials are subtracted from the cost *per outcome, before averaging*. Leftover bound materials are worth nothing (or a lower price), so gold cost is a piecewise-linear function of total consumption and `E[f(X)] != f(E[X])`. See `README.md` for the motivating example.

These notes are high-level. They cover what is hard to work out from reading one file; they do not track constants, thresholds or UI details.

## Repo map

| Path | What it is |
| --- | --- |
| `crates/core` (`hf-core`) | All honing maths: domain model, average-gold evaluation, optimizer. |
| `crates/scanner` (`hf-scanner`) | Screen scanner image processing. In development. |
| `crates/wasm` (`hf-wasm`) | Thin `wasm_bindgen` wrappers over core and scanner. The only crate the frontend loads. |
| `crates/arena` (`hf-arena`) | Native binary for optimizer benchmarking. Defunct and commented out of the workspace, along with the `verification` module it drives. |
| `frontend/` | Vue 3 + Pinia + Tailwind app. `@` aliases to this folder. |
| `cloudflare/` | A Worker that proxies and caches market prices and character lookups. |
| `public/` | Static assets, changelogs, `WIP.md` (the author's roadmap), scanner config and OCR model. |
| `templates/` | Source icons and the script that builds the icon half of the scanner config. |
| `scripts/` | Changelog helper, constants export, scanner brightness calibration, optimizer test tooling. |
| `e2e/` | Playwright tests, shared browser helpers and the market price fixture. |
| `tests/` | Node-side frontend tests: migrations, the real-save fixture, cross-tier relations. `pnpm test:frontend`. |
| `test_cases/` | Payloads and cached results for the test harness. |
| `docs/` | Author-written docs: the saddlepoint paper, evaluation flow, optimizer, frontend flow, constants. |

Not source of truth: `dist/` (committed build output), `crates/wasm/pkg/` (wasm-pack output), `crates/core/src/optimizer/Old/` (dead archive), and the untracked `junkyard/` and `data_fetcher/`.

## Build and run

- `pnpm dev` builds the wasm package with wasm-pack, then starts Vite. Vite does not watch `crates/`, so **Rust changes need `pnpm run wasm` again**.
- `pnpm e2e` runs the browser tests; `pnpm browse` runs a one-off browser script. See `Browser harness/Overview.md`.
- `pnpm check` runs `vue-tsc --noEmit` over the `.ts` layer and `.vue` templates. `pnpm test:frontend` runs the node-side suites in `tests/`. Both are fast; run them after a frontend change.
- `pnpm test` runs `cargo test -p hf-core`. It works again now that `verification` is commented out; `materials.rs` holds the valuation unit tests.
- `pnpm build` chains `pnpm test` before the wasm and Vite builds and an unused-dependency check that needs nightly. `pnpm no-test-build` skips the test.
- `pnpm deploy` only checks you are on a clean `main`, then force-pushes `main` to the `Production` branch (assumed to be what the hosted site builds from). It does not run tests.
- `pnpm cloudflare-deploy` deploys the Worker separately.
- `pnpm change-log` scaffolds the next changelog file in `public/change-logs`.

## Cargo features

- `hf-core` has no default features. An optimizer version feature (`v35`, aliased as `active_version`) must be enabled or `optimizer::solve` does not exist.
- `wasm` gates the histogram, the progress-message bridge and browser logging and timing.
- `run_tests` gates the verification module and pulls in native-only dependencies.
- `my_dbg!` is an exported macro whose feature checks resolve in the *calling* crate. That is why `hf-wasm` and `hf-scanner` declare dummy `wasm` / `run_tests` features.
- The wasm target is built with SIMD enabled (`.cargo/config.toml`).

## Game glossary

- **Tap**: one honing attempt. It costs materials whether or not it succeeds.
- **Normal honing**: +N to +N+1, with a success chance that rises after each failure.
- **Artisan / pity**: a meter that fills on failed taps; when full, the next tap is guaranteed.
- **Advanced honing**: a separate XP-based system in blocks of ten levels, with "grace" taps and its own consumables.
- **Juice, books, scrolls**: optional consumables that raise the success chance of a tap. Juices and books apply to normal honing, juices and scrolls to advanced. In code they share one id space ("juice ids").
- **Special honing / free tap**: an attempt paid for with special leapstones. Success skips the upgrade entirely.
- **Bound / roster-bound / tradable**: the three ownership bands of a material. What a leftover unit of each is worth is the user's choice, out of nothing / taxed sell price / buy price, and must not decrease across the bands.
- **Tier**: 0 is T4, 1 is Serca. Each has its own material set and cost tables.
- **Express / event**: temporary modifiers to costs and chances.

## Index

- `Calculator core/Domain model.md`: the structs and index conventions.
- `Calculator core/Evaluation pipeline.md`: payload to result, method selection, invariants.
- `Calculator core/Optimizer.md`: what the annealer may and may not change; test harness status.
- `Working in this repo.md`: line endings, scratch space, what the type checker cannot see, and the test suites. Read before a large edit.
- `Frontend/State and persistence.md`: the store, data scopes, localStorage and migrations.
- `Frontend/Wasm contract.md`: payload building, worker protocol, what must match Rust.
- `Frontend/External services.md`: market prices, character import, changelogs.
- `Screen scanner/Pipeline.md`: capture to recognised slots, and what is still stubbed.
- `Screen scanner/Config and calibration.md`: the template config and brightness calibration.
- `Screen scanner/Slot grid.md`: the scanner page's slot grid, slot statuses and manual edits.
- `Screen scanner/Manifest.md`: the scan as materials per band and select-one chests, with overrides and T4 conversion.
- `Browser harness/Overview.md`: driving the site in Chromium and Firefox, for debugging and e2e tests. One note per site section alongside it.

Read alongside `docs/`: `Saddlepoint Approximation.pdf` for the maths, `Average Evaluation.md` and `Frontend.md` for flow diagrams, `Constants.md` for updating game data.
