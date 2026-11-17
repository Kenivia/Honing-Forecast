# Performance benchmark

`pnpm perf` (`scripts/perf/bench.mjs`) measures how snappy the site feels: the latency of interactions and of route and character switches. It does not measure page load or optimizer speed.

## How it runs

- Builds the site into `.tmp/perf-dist` (never `dist/`) and serves it with `vite preview` on port 4317. The dev server is never measured: unbundled modules and Vue's dev mode distort everything. Rust changes still need `pnpm run wasm` first.
- Headless Playwright Chromium, 1600x1000, CPU throttled 4x through CDP after the app shell is up. Market prices are stubbed (`hf.stub_market`); every other non-localhost request is aborted, so analytics uploads never fire.
- Two states, each a fresh context per iteration: `fresh` (the default `Newchar`) and `real` (`tests/V10 config 2`, a production roster, seeded into localStorage and migrated on load).
- One scripted sequence per iteration (`scenario`): a Helmet cell and then its column header cycled NotYet → Want → Done → NotYet at the lowest free level, two edits of the costs card's first owned input, the route loop calc → market → roster → calc → scanner → choices → guide → calc twice (`#1` mounts cold, `#2` warm), and on the real save a character switch there and back twice.
- Results go to `ignored/perf/<label>.json`; flags: `--iters N` (8), `--label`, `--compare <json>`, `--ab <ref>|same`, `--only fresh|real`, `--no-build`, `--profile`, `--profile-runs N`, `--paint-only`, `--loaf <action prefix>`, `--trace`, `--headed`.
- `--ab <ref>` is the way to judge a change. It `git archive`s the ref's frontend (`AB_PATHS`) into `.tmp/perf-ab-src`, with the current wasm package copied in and `node_modules` junctioned for the build only, builds it into `.tmp/perf-dist-ab`, serves it on 4318 beside the working tree on 4317, and alternates the two per iteration (the order flips each time). The report's base column is the ref. `--ab same` serves the working tree's build on both ports: an A/A check of the noise. Rust changes are not compared: both sides use the current wasm.

## What the numbers mean

- **paint**: from the triggering event's `timeStamp` (`click`, or `change` for inputs) to just after the first frame in which the action's DOM predicate holds has painted (rAF polling, then a `setTimeout`). This is the user-visible latency, and unlike Event Timing it also covers async work such as a lazy route chunk. With no predicate it is the next paint after the event.
- **block**: summed `blockingDuration` of Long Animation Frames overlapping the 1 s after the event: trailing jank from worker results and charts.
- Before each action the page settles: no long frame for 500 ms, 1.5 s since the last click or change (so its debounced save has landed even when the save makes no long frame) and the optimizer progress at 100% (capped at 60 s), so actions do not overlap optimizer runs or saves.
- The report gives the median, p90 and CV of paint and the median block. `--compare` adds the baseline median and Δ, flagging `faster` / `SLOWER` only when the interquartile ranges do not overlap.

## Reliability

- Nothing heavy may run alongside it (arena, lab, a build); it shares the CPU.
- An `--ab same` run (8 iters, 2026-11-17) put every action within −6% to +12% and flagged 1 of 46: the floor for `--ab`. A full `--ab` run of both states takes about 22 minutes; `--only real` halves it. No flag is given under 4 iterations.
- Separate runs drift: two runs of one build differed by a uniform 5–14% on the fresh state (2026-11-17), though each run's own CV was mostly under 0.1. `--compare` between runs can only show changes well beyond that; use `--ab`.

## Diagnosis

- `--profile` adds one untimed iteration per state with the CDP sampling profiler around each action (from just before it to the end of the 1 s window). Each action's `.cpuprofile` and a `summary.txt` of self time by file and by function, mapped to the original sources through the build's source maps (`source-map-js`), go to `ignored/perf/<label>-<state>-profiles/`. Throttling inflates the sampled times; read them as shares. With `--ab` the ref is profiled too (`summary-base.txt`); `--profile-runs N` averages N runs per build; `--paint-only` stops the profiler at the paint instead of the end of the window, though the stop still lands late when the page is busy, so a window can include work after the paint.
- `--loaf <prefix>` prints, for each action whose name starts with it, the long frames between the event and the paint: their script entries (mapped through the source maps) and how the frame split into scripts, style and layout. A route change shows as one `onclick` entry; the router's work runs in its microtasks.
- `--trace` adds one traced iteration per state (`ignored/perf/<label>-<state>.trace.json`, screenshots on) for DevTools > Performance > Load profile; the build has source maps.
