// Profiles the screen scanner end to end: builds the wasm, serves if needed, plays each
// recording through the scanner, then prints the stage breakdown.
// pnpm scanner-profile [--name=<run>] [--no-build] [--firefox] [--keep] [recording...]
// With no recording, every one in scripts/recordings. Dumps go to target/scan-profiles/<run>/
// ("last" without a name), so a before and an after can be kept side by side for scanner-compare.
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const BASE_URL = "http://localhost:5173";
const RECORDINGS = "scripts/recordings";
const VIDEO = /\.(mp4|mkv|webm|mov)$/i;

const args = process.argv.slice(2);
const flags = args.filter((a) => a.startsWith("--"));
const named = args.filter((a) => !a.startsWith("--"));
const files = named.length
  ? named
  : fs.readdirSync(RECORDINGS).filter((x) => VIDEO.test(x)).sort().map((x) => path.join(RECORDINGS, x));
const run_name = flags.find((x) => x.startsWith("--name="))?.slice(7) ?? "last";
const out_dir = path.join("target/scan-profiles", run_name);

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

// always, or a dev server that is already up would have the run measure an older build
if (!flags.includes("--no-build")) {
  await run("pnpm", ["run", "wasm"]);
}
let server = null;
if (!(await up())) {
  console.log("nothing on :5173 - starting vite");
  server = spawn("npx", ["vite", "--port", "5173"], { stdio: "ignore", shell: true, detached: false });
  for (let i = 0; i < 60 && !(await up()); i++) {
    await new Promise((r) => setTimeout(r, 1000));
  }
  if (!(await up())) {
    server.kill();
    throw new Error("vite did not come up");
  }
}

fs.mkdirSync(out_dir, { recursive: true });
const dumps = [];
try {
  for (const file of files) {
    const name = path.basename(file).replace(/\.[^.]+$/, "").replace(/\s+/g, "-");
    const out = path.join(out_dir, `${name}.json`);
    console.log(`\nprofiling ${file}`);
    await run(
      "node",
      [
        "scripts/browser/browse.mjs",
        "scripts/browser/profile_scanner_browse.mjs",
        ...(flags.includes("--firefox") ? ["--firefox"] : []),
      ],
      { REC: file, OUT: out },
    );
    dumps.push(out);
  }
} finally {
  if (server && !flags.includes("--keep")) server.kill();
}

await run("node", ["scripts/browser/scan_summary.mjs", ...dumps.map((x) => `"${x}"`)]);
