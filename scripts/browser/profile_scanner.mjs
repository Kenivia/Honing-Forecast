// Profiles the screen scanner end to end: builds and serves if needed, plays each
// recording through the scanner, then prints the stage breakdown.
// pnpm scanner-profile [--firefox] [--keep] [recording...]
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const BASE_URL = "http://localhost:5173";
const OUT_DIR = "target/scan-profiles";
const DEFAULT_RECORDINGS = [
  "scripts/brightness/1080p raw/2026-10-07 17-28-55.mp4",
  "scripts/brightness/1080p raw/2026-10-07 22-55-45.mp4",
];

const args = process.argv.slice(2);
const flags = new Set(args.filter((a) => a.startsWith("--")));
const recordings = args.filter((a) => !a.startsWith("--"));
const files = recordings.length ? recordings : DEFAULT_RECORDINGS;

const run = (cmd, cmd_args, env = {}) =>
  new Promise((resolve, reject) => {
    const child = spawn(cmd, cmd_args, {
      stdio: "inherit",
      shell: true,
      env: { ...process.env, ...env },
    });
    child.on("exit", (code) => (code ? reject(new Error(`${cmd} exited ${code}`)) : resolve()));
  });

const up = () => fetch(BASE_URL).then(() => true).catch(() => false);

let server = null;
if (!(await up())) {
  console.log("nothing on :5173 - building wasm");
  await run("pnpm", ["run", "wasm"]);
  console.log("starting vite");
  server = spawn("npx", ["vite", "--port", "5173"], { stdio: "ignore", shell: true, detached: false });
  for (let i = 0; i < 60 && !(await up()); i++) {
    await new Promise((r) => setTimeout(r, 1000));
  }
  if (!(await up())) {
    server.kill();
    throw new Error("vite did not come up");
  }
}

fs.mkdirSync(OUT_DIR, { recursive: true });
const dumps = [];
try {
  for (const file of files) {
    const name = path.basename(file).replace(/\.[^.]+$/, "").replace(/\s+/g, "-");
    const out = path.join(OUT_DIR, `${name}.json`);
    console.log(`\nprofiling ${file}`);
    await run(
      "node",
      [
        "scripts/browser/browse.mjs",
        "scripts/browser/profile_scanner_browse.mjs",
        ...(flags.has("--firefox") ? ["--firefox"] : []),
      ],
      { REC: file, OUT: out },
    );
    dumps.push(out);
  }
} finally {
  if (server && !flags.has("--keep")) server.kill();
}

await run("node", ["scripts/browser/scan_summary.mjs", ...dumps]);
