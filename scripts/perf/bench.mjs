// Interaction latency benchmark on a production build. See AI notes/Browser harness/Performance.md.
// pnpm perf [--iters N] [--label name] [--compare file.json] [--ab <git ref>|same] [--only fresh|real]
//           [--no-build] [--profile] [--trace] [--headed]
import { chromium } from "@playwright/test";
import { build, preview } from "vite";
import { SourceMapConsumer } from "source-map-js";
import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import * as hf from "../../tests/e2e/helpers.ts";

const ROOT = path.resolve(import.meta.dirname, "../..");
const OUT_DIR = path.join(ROOT, ".tmp/perf-dist");
const AB_SRC = path.join(ROOT, ".tmp/perf-ab-src");
const AB_DIR = path.join(ROOT, ".tmp/perf-dist-ab");
const RESULTS = path.join(ROOT, "ignored/perf");
const THROTTLE = 4;
const WINDOW_MS = 1000; // blocking time is summed over this long after each action
const QUIET_MS = 500; // settled once no long frame for this long
const REAL_SAVE = { key: "HF_CONFIG_V10_COMPRESSED", file: "tests/V10 config 2" };
// what a build needs from a git ref; the wasm package is not in git, so the current one is used
const AB_PATHS = ["frontend", "public", "index.html", "vite.config.ts", "tsconfig.json", "package.json", "scanner_setup/templates/items.json", "scanner_setup/templates/chests.json"];

const args = process.argv.slice(2);
const opt = (name, fallback) => {
  const i = args.indexOf(`--${name}`);
  return i < 0 ? fallback : args[i + 1];
};
const flag = (name) => args.includes(`--${name}`);
const ITERS = Number(opt("iters", 8));
const LABEL = opt("label", new Date().toISOString().slice(0, 16).replace(/[:T]/g, "-"));
const STATES = opt("only", "fresh,real").split(",");
const AB = opt("ab");

// observers installed before the app loads
function init_perf() {
  window.__perf = { loaf: [], last: {} };
  new PerformanceObserver((list) => {
    for (const e of list.getEntries())
      window.__perf.loaf.push({ start: e.startTime, end: e.startTime + e.duration, blocking: e.blockingDuration });
  }).observe({ type: "long-animation-frame", buffered: true });
  for (const kind of ["click", "change"])
    window.addEventListener(kind, (e) => (window.__perf.last[kind] = e.timeStamp), true);
}

// resolves after the frame that first shows the triggering event's effect has painted
function wait_paint([kind, pred_src]) {
  const before = window.__perf.last[kind];
  const pred = pred_src ? new Function(`return (${pred_src})`) : null;
  const begin = performance.now();
  return new Promise((resolve) => {
    const frame = () => {
      const t0 = window.__perf.last[kind];
      if (t0 === undefined || t0 === before || (pred && !pred())) {
        if (performance.now() - begin > 20_000) return resolve(null);
        return requestAnimationFrame(frame);
      }
      setTimeout(() => resolve({ t0, t1: performance.now() }));
    };
    requestAnimationFrame(frame);
  });
}

// no long frame for QUIET_MS and the optimizer idle, capped
function settle(quiet) {
  const begin = performance.now();
  const optimizer_idle = () => {
    const label = [...document.querySelectorAll("span")].find((s) => s.textContent === "Optimizer progress:");
    return !label || label.nextElementSibling.textContent.trim().startsWith("100");
  };
  return new Promise((resolve) => {
    const poll = () => {
      const last = window.__perf.loaf.at(-1)?.end ?? 0;
      const now = performance.now();
      if ((now - last >= quiet && optimizer_idle()) || now - begin > 60_000) return resolve();
      setTimeout(poll, 100);
    };
    poll();
  });
}

const blocking = (page, t0) =>
  page.evaluate(
    ([t0, w]) => window.__perf.loaf.filter((e) => e.end > t0 && e.start < t0 + w).reduce((s, e) => s + e.blocking, 0),
    [t0, WINDOW_MS],
  );

// page markers, evaluated in the page
const on_page = (p, marker, gone = '[aria-label="Normal honing"]') =>
  `decodeURIComponent(location.pathname).endsWith(${JSON.stringify(p)}) && document.querySelector(${JSON.stringify(marker)}) && !document.querySelector(${JSON.stringify(gone)})`;
const CALC = (name) =>
  `decodeURIComponent(location.pathname) === ${JSON.stringify(`/${name}/calc`)} && document.querySelector('[aria-label="Normal honing"]') && document.querySelector('input[aria-label="Char name"]')?.value === ${JSON.stringify(name)}`;
const PAGES = {
  market: on_page("/market-mats", '[aria-label="T4 materials"]'),
  roster: on_page("/roster-setup", 'input[aria-label="Char name"]'),
  scanner: on_page("/scanner", ".card-shell"),
  choices: on_page("/choices", '[aria-label="Choices"]'),
  guide: on_page("/guide", ".prose-card"),
};

async function measure(ctx, name, kind, pred, act) {
  const { page, cdp, results } = ctx;
  await page.evaluate(settle, QUIET_MS);
  if (ctx.profiles) await cdp.send("Profiler.start");
  const done = page.evaluate(wait_paint, [kind, pred]);
  await act();
  const r = await done;
  if (!r) throw new Error(`${name}: never painted`);
  await page.waitForTimeout(WINDOW_MS);
  if (ctx.profiles) ctx.profiles[name] = (await cdp.send("Profiler.stop")).profile;
  (results[name] ??= { paint: [], blocking: [] }).paint.push(r.t1 - r.t0);
  results[name].blocking.push(await blocking(page, r.t0));
}

const header = (page, text) => page.locator("nav").getByRole("link", { name: text, exact: true });
const side = (page, text) => page.locator("a.side-bar-link", { hasText: text });

async function scenario(ctx, state) {
  const { page } = ctx;
  const chars = await page
    .locator("nav a.header-button")
    .evaluateAll((as) => as.map((a) => a.textContent.trim()).filter((t) => !["Roster setup", "Market & Mats"].includes(t)));
  const [a, b] = chars;

  // calc: a single cell and a column header, each cycled NotYet -> Want -> Done -> NotYet
  const level = await page.evaluate(() => {
    const cells = [...document.querySelectorAll('[aria-label="Normal honing"] button[aria-label^="Helmet +"]')];
    const free = cells.map((c) => c.getAttribute("aria-label")).filter((l) => l.endsWith(": NotYet"));
    return Number(free[0].match(/\+(\d+):/)[1]);
  });
  const sel = `[aria-label="Normal honing"] button[aria-label^="Helmet +${level}:"]`;
  const status = `document.querySelector(${JSON.stringify(sel)}).getAttribute("aria-label")`;
  for (const [what, act] of [
    ["cell", () => hf.cell(page, "normal", "Helmet", level).click()],
    ["column", () => hf.toggle_column(page, "normal", level)],
  ])
    for (const step of ["want", "done", "clear"]) {
      const before = await page.evaluate(status);
      await measure(ctx, `calc ${what} ${step}`, "click", `${status} !== ${JSON.stringify(before)}`, act);
    }

  // the first owned input of the costs card, whatever tier and band the character shows
  const input = page.locator(".mats-row[role=group]").first().getByRole("textbox").first();
  for (const value of [12345, 54321])
    await measure(ctx, "calc material input", "change", null, async () => {
      await input.fill(String(value));
      await input.blur();
    });

  // routes, twice: first visit mounts cold, second is warm
  for (const visit of [1, 2]) {
    const nav = (name, pred, act) => measure(ctx, `${name} #${visit}`, "click", pred, act);
    await nav("calc→market", PAGES.market, () => header(page, "Market & Mats").click());
    await nav("market→roster", PAGES.roster, () => header(page, "Roster setup").click());
    await nav("roster→calc", CALC(a), () => header(page, a).click());
    await nav("calc→scanner", PAGES.scanner, () => side(page, "Scanner").click());
    await nav("scanner→choices", PAGES.choices, () => side(page, "Choices").click());
    await nav("choices→guide", PAGES.guide, () => side(page, "Guide").click());
    await nav("guide→calc", CALC(a), () => side(page, "Calc").click());
  }

  if (b) {
    for (const visit of [1, 2]) {
      await measure(ctx, `char switch #${visit}`, "click", CALC(b), () => header(page, b).click());
      await measure(ctx, `char switch back #${visit}`, "click", CALC(a), () => header(page, a).click());
    }
  } else if (state === "real") throw new Error("real save has one character");
}

async function run_once(browser, port, state, results, { trace_path, profiles } = {}) {
  const context = await browser.newContext({ baseURL: `http://localhost:${port}`, viewport: { width: 1600, height: 1000 } });
  const page = await context.newPage();
  // nothing leaves the machine: market is stubbed, analytics and the rest aborted
  await page.route(/^https?:\/\/(?!localhost)/, (route) => route.abort());
  await hf.stub_market(page);
  await page.addInitScript(init_perf);
  if (state === "real") {
    const raw = fs.readFileSync(path.join(ROOT, REAL_SAVE.file), "utf8").replace(/\r?\n$/, "");
    await page.addInitScript(([k, v]) => localStorage.setItem(k, v), [REAL_SAVE.key, raw]);
  }
  await hf.open(page);
  await page.waitForSelector('[aria-label="Normal honing"]');
  const cdp = await context.newCDPSession(page);
  await cdp.send("Emulation.setCPUThrottlingRate", { rate: THROTTLE });
  if (profiles) {
    await cdp.send("Profiler.enable");
    await cdp.send("Profiler.setSamplingInterval", { interval: 200 });
  }
  if (trace_path) await browser.startTracing(page, { path: trace_path, screenshots: true });
  await scenario({ page, cdp, results, profiles }, state);
  if (trace_path) await browser.stopTracing();
  await context.close();
}

// self time per original function, through the build's source maps
const consumers = {};
function where(frame) {
  const file = frame.url.match(/\/assets\/([^/?]+\.js)/)?.[1];
  const map = file && path.join(OUT_DIR, "assets", `${file}.map`);
  if (map && fs.existsSync(map)) {
    consumers[map] ??= new SourceMapConsumer(JSON.parse(fs.readFileSync(map, "utf8")));
    const pos = consumers[map].originalPositionFor({ line: frame.lineNumber + 1, column: frame.columnNumber });
    if (pos.source) {
      const src = pos.source.replace(/^.*node_modules\/(?:\.pnpm\/[^/]+\/node_modules\/)?/, "").replace(/^(\.\.\/)+/, "");
      return { fn: frame.functionName || pos.name || "(anon)", src, at: `${src}:${pos.line}` };
    }
  }
  const src = frame.url ? path.basename(frame.url) : "(native)";
  return { fn: frame.functionName || "(anon)", src, at: frame.url ? `${src}:${frame.lineNumber + 1}` : src };
}
function summarise(profile, top = 10) {
  const nodes = new Map(profile.nodes.map((n) => [n.id, n]));
  const self = new Map();
  profile.samples.forEach((id, i) => self.set(id, (self.get(id) ?? 0) + (profile.timeDeltas[i + 1] ?? 0) / 1000));
  const by_fn = new Map();
  const by_src = new Map();
  let total = 0;
  for (const [id, t] of self) {
    const f = nodes.get(id).callFrame;
    if (f.functionName === "(idle)" || f.functionName === "(root)") continue;
    const w = where(f);
    by_fn.set(`${w.fn} ${w.at}`, (by_fn.get(`${w.fn} ${w.at}`) ?? 0) + t);
    by_src.set(w.src, (by_src.get(w.src) ?? 0) + t);
    total += t;
  }
  const list = (m) => [...m].sort((x, y) => y[1] - x[1]).slice(0, top).map(([k, t]) => `    ${t.toFixed(1).padStart(7)} ms  ${k}`);
  return [`  sampled ${total.toFixed(0)} ms (main thread; throttling inflates it, read as shares)`, "  by file:", ...list(by_src), "  by function:", ...list(by_fn)].join("\n");
}

// stats
const sorted = (xs) => [...xs].sort((x, y) => x - y);
const quantile = (xs, q) => {
  const s = sorted(xs);
  const i = (s.length - 1) * q;
  return s[Math.floor(i)] + (s[Math.ceil(i)] - s[Math.floor(i)]) * (i % 1);
};
const cv = (xs) => {
  const m = xs.reduce((s, x) => s + x, 0) / xs.length;
  return Math.sqrt(xs.reduce((s, x) => s + (x - m) ** 2, 0) / xs.length) / m;
};
const ms = (x) => x.toFixed(0).padStart(6);

function report(data, base, base_name) {
  for (const [state, actions] of Object.entries(data.states)) {
    if (!Object.keys(actions).length) continue;
    console.log(`\n${state} (${data.meta.iters} iters, ${THROTTLE}x CPU) — ms${base ? `, base = ${base_name}` : ""}`);
    const head = `${"action".padEnd(26)} ${"paint".padStart(6)} ${"p90".padStart(6)} ${"cv".padStart(5)} ${"block".padStart(6)}`;
    console.log(base ? `${head} ${"base".padStart(6)} ${"Δ".padStart(6)}` : head);
    for (const [name, r] of Object.entries(actions)) {
      let line = `${name.padEnd(26)} ${ms(quantile(r.paint, 0.5))} ${ms(quantile(r.paint, 0.9))} ${cv(r.paint).toFixed(2).padStart(5)} ${ms(quantile(r.blocking, 0.5))}`;
      const b = base?.[state]?.[name];
      if (b) {
        const [lo, hi, blo, bhi] = [quantile(r.paint, 0.25), quantile(r.paint, 0.75), quantile(b.paint, 0.25), quantile(b.paint, 0.75)];
        const delta = quantile(r.paint, 0.5) / quantile(b.paint, 0.5) - 1;
        // flagged only when the interquartile ranges do not overlap
        const tag = r.paint.length < 4 || b.paint.length < 4 ? "" : hi < blo ? " faster" : lo > bhi ? " SLOWER" : "";
        line += ` ${ms(quantile(b.paint, 0.5))} ${(delta * 100).toFixed(0).padStart(5)}%${tag}`;
      }
      console.log(line);
    }
  }
}

// a git ref's frontend, built against the current node_modules and wasm package
async function build_ref(ref) {
  fs.rmSync(AB_SRC, { recursive: true, force: true });
  fs.mkdirSync(AB_SRC, { recursive: true });
  execSync(`git archive ${ref} ${AB_PATHS.map((p) => `"${p}"`).join(" ")} | tar -x -C "${AB_SRC}"`, { cwd: ROOT });
  fs.cpSync(path.join(ROOT, "crates/wasm/pkg"), path.join(AB_SRC, "crates/wasm/pkg"), { recursive: true });
  const modules = path.join(AB_SRC, "node_modules");
  fs.symlinkSync(path.join(ROOT, "node_modules"), modules, "junction");
  try {
    await build({ root: AB_SRC, logLevel: "warn", build: { outDir: AB_DIR, emptyOutDir: true } });
  } finally {
    fs.rmdirSync(modules); // removes the junction only
  }
}

if (!flag("no-build")) {
  console.log("building into .tmp/perf-dist (needs pnpm run wasm after Rust changes)");
  await build({ root: ROOT, logLevel: "warn", build: { outDir: OUT_DIR, emptyOutDir: true } });
  if (AB && AB !== "same") {
    console.log(`building ${AB} into .tmp/perf-dist-ab`);
    await build_ref(AB);
  }
}
const serve = (dir, port) => preview({ root: ROOT, logLevel: "warn", build: { outDir: dir }, preview: { port, strictPort: true } });
const servers = [await serve(OUT_DIR, 4317)];
if (AB) servers.push(await serve(AB === "same" ? OUT_DIR : AB_DIR, 4318));
const browser = await chromium.launch({ headless: !flag("headed") });
fs.mkdirSync(RESULTS, { recursive: true });

const data = {
  meta: {
    label: LABEL,
    date: new Date().toISOString(),
    commit: execSync("git describe --always --dirty", { cwd: ROOT }).toString().trim(),
    ab: AB,
    chromium: browser.version(),
    iters: ITERS,
    throttle: THROTTLE,
    window_ms: WINDOW_MS,
  },
  states: {},
  ...(AB ? { ab_states: {} } : {}),
};
try {
  for (const state of STATES) {
    const results = (data.states[state] = {});
    const ab_results = AB && (data.ab_states[state] = {});
    for (let i = 0; i < ITERS; i++) {
      process.stdout.write(`\r${state}: iteration ${i + 1}/${ITERS}`);
      // alternate which build goes first, so drift over the run hits both alike
      const order = AB ? (i % 2 ? [[4318, ab_results], [4317, results]] : [[4317, results], [4318, ab_results]]) : [[4317, results]];
      for (const [port, r] of order) await run_once(browser, port, state, r);
    }
    process.stdout.write("\n");
    if (flag("trace")) {
      const trace_path = path.join(RESULTS, `${LABEL}-${state}.trace.json`);
      await run_once(browser, 4317, state, {}, { trace_path });
      console.log(`trace: ${trace_path} (DevTools > Performance > Load profile)`);
    }
    if (flag("profile")) {
      const profiles = {};
      await run_once(browser, 4317, state, {}, { profiles });
      const dir = path.join(RESULTS, `${LABEL}-${state}-profiles`);
      fs.mkdirSync(dir, { recursive: true });
      const text = Object.entries(profiles).map(([name, p], i) => {
        fs.writeFileSync(path.join(dir, `${String(i).padStart(2, "0")} ${name.replace(/[→#]/g, "-")}.cpuprofile`), JSON.stringify(p));
        return `${name}\n${summarise(p)}`;
      });
      fs.writeFileSync(path.join(dir, "summary.txt"), text.join("\n\n"));
      console.log(`profiles: ${dir} (summary.txt; .cpuprofile files load in DevTools > Performance)`);
    }
  }
} finally {
  await browser.close();
  for (const s of servers) await new Promise((r) => s.httpServer.close(r));
}

const out = path.join(RESULTS, `${LABEL}.json`);
fs.writeFileSync(out, JSON.stringify(data));
const compare = opt("compare");
if (AB) report(data, data.ab_states, AB);
else report(data, compare && JSON.parse(fs.readFileSync(path.resolve(compare), "utf8")).states, compare);
console.log(`\nsaved ${out}`);
