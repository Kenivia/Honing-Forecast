# Browser harness

How an agent drives the site in a real browser. Playwright is the only tool; it covers Chromium and Firefox (a patched build, not stock Firefox).

Per-section instructions:

- `Calculator.md`: the calculator, market and roster pages.
- `Screen scanner.md`: the scanner and scanner setup pages, fed by an uploaded image or video.

## Setup

1. `pnpm install`
2. `pnpm browsers` installs every browser build needed. Run it again after upgrading `@playwright/test` or `@playwright/mcp`; the two pin different Firefox builds.
3. `pnpm dev` and leave it running (port 5173). It builds wasm first, so the first start takes about a minute.

**Rust changes need `pnpm run wasm`** before the browser sees them. Vite does not watch `crates/`. Frontend changes hot-reload.

## Three ways to drive the site

| Way | Use it for | Command |
| --- | --- | --- |
| Playwright MCP | One-off debugging when the `pw-chrome` / `pw-firefox` tools are loaded. | tools named `browser_*` |
| `pnpm browse` | One-off debugging without MCP tools (subagents, non-interactive runs), or anything needing a loop or timing. | `pnpm browse <script.mjs> [--firefox] [--headed] [--stub]`; `BASE_URL` in the environment points it at a dev server on another port |
| `pnpm e2e` | Rigorous, repeatable checks in both browsers. | `pnpm e2e`, `pnpm e2e --project=firefox`, `pnpm e2e -g "name"` |

### Playwright MCP

Configured in `.mcp.json`, run from the pinned `@playwright/mcp` dev dependency. `pw-chrome` uses the installed Google Chrome, `pw-firefox` the patched Firefox. Servers load at session start, so a new `.mcp.json` needs a reload and the user's approval.

- The browser is headed and keeps its state for the whole session.
- `browser_snapshot` returns the accessibility tree with a `ref` per element; pass that ref to `browser_click` or `browser_type`. Take a fresh snapshot after anything that re-renders.
- Snapshots of the calculator are large (the instructions list). Prefer `browser_find` or `browser_evaluate` when one value is needed.
- `browser_console_messages` and `browser_network_requests` are the debugging tools.
- Snapshots are written to `.playwright-mcp/` (gitignored).

### pnpm browse

Runs one script file against the running dev server with a fresh browser profile, and prints uncaught page errors and console errors at the end. Put the script in the scratchpad, not the repo.

```js
export default async ({ page, hf }) => {
  await hf.open(page);
  await hf.toggle_column(page, "normal", 15);
  await hf.wait_for_optimizer(page);
  console.log(await hf.read_gold(page));
  await page.screenshot({ path: "out.png" });
};
```

`page` is a Playwright page with the base URL set. `hf` is `e2e/helpers.ts`. `--stub` serves the saved market prices instead of live ones.

### pnpm e2e

Tests live in `e2e/*.spec.ts` and run in both browsers. The config starts `pnpm dev` itself, or reuses one already running. A failure leaves a trace in `test-results/`; open it with `pnpm exec playwright show-trace <trace.zip>`. `pnpm e2e:ui` opens the interactive runner for the user.

Every test should start with `hf.watch_errors` and `hf.stub_market`, as `smoke.spec.ts` does.

## One-off versus rigorous

| | One-off debugging | Rigorous testing |
| --- | --- | --- |
| Market prices | Live, from the Worker | Stubbed from `e2e/fixtures/market.json` |
| State | MCP: persists for the session. `pnpm browse`: fresh each run | Fresh profile per test |
| Waiting | Fixed waits and re-snapshots are fine | Helpers and auto-waiting assertions only |
| Numbers | Read and eyeball | Assert relations and ranges, never exact gold |

## Things that will trip you up

- **Never wait for network idle.** It does not resolve on this site. Wait for an element or text.
- **Every launch is a fresh profile** with the default single character `Newchar`, except within one MCP session. `/` redirects to `/Newchar/calc`.
- **Saving is debounced.** Wait about a second after the last change before reloading, or the change is lost.
- **Gold is not reproducible.** The optimizer is randomised, so the same inputs give slightly different gold between runs and browsers, even with stubbed prices.
- **Live prices hit the author's Worker** on every fresh load. Use the stub for anything run repeatedly.
- **Screen capture does not work headless.** The scanner pages no longer ask for it on mount; upload an image or video instead, see `Screen scanner.md`.
- **Elements are found by accessible name.** The frontend has few labels; the ones the helpers rely on are listed in the section notes. If something has no name, add an `aria-label` in the component instead of selecting by position or CSS class.
- **Helpers import with the `.ts` extension** and use only erasable TypeScript, because `pnpm browse` loads them through Node directly.
