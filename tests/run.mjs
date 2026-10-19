// Bundles each *.test.ts with esbuild and runs it under node with a browser shim.
// Frontend tests only; `pnpm test` runs the Rust side.
import { build } from "esbuild";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "..");
const out_dir = path.join(root, ".tmp", "tests");
fs.rmSync(out_dir, { recursive: true, force: true });
fs.mkdirSync(out_dir, { recursive: true });

const only = process.argv[2];
const tests = fs
  .readdirSync(here)
  .filter((f) => f.endsWith(".test.ts"))
  .filter((f) => !only || f.includes(only))
  .sort();

let failed = 0;
for (const test of tests) {
  const outfile = path.join(out_dir, test.replace(/\.ts$/, ".mjs"));
  await build({
    entryPoints: [path.join(here, test)],
    outfile,
    bundle: true,
    platform: "node",
    format: "esm",
    alias: { "@": path.join(root, "frontend") },
    define: { "import.meta.env": "{}" },
    logLevel: "error",
  });
  console.log(`\n######## ${test} ########`);
  // on Windows --import needs a file:// URL, not a bare absolute path
  const res = spawnSync(
    process.execPath,
    ["--import", pathToFileURL(path.join(here, "shim.mjs")).href, outfile],
    { stdio: "inherit", cwd: root },
  );
  if (res.status !== 0) {
    failed += 1;
  }
}

console.log(
  failed === 0
    ? `\n${tests.length} suite(s) passed`
    : `\n${failed} of ${tests.length} suite(s) FAILED`,
);
process.exit(failed === 0 ? 0 : 1);
