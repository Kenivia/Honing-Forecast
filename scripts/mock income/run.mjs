// Bundles make.ts and runs it under node with the tests' browser shim:
// node "scripts/mock income/run.mjs" writes tests/income-fixture.json from the real save.
import { build } from "esbuild";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../..");
const outfile = path.join(root, ".tmp", "mock-income.mjs");
await build({
  entryPoints: [path.join(here, "make.ts")],
  outfile,
  bundle: true,
  platform: "node",
  format: "esm",
  alias: { "@": path.join(root, "frontend") },
  define: { "import.meta.env": "{}" },
  logLevel: "error",
});
const res = spawnSync(
  process.execPath,
  [
    "--import",
    pathToFileURL(path.join(root, "tests", "shim.mjs")).href,
    outfile,
    path.join(root, "tests", "honing-forecast-2026-10-17.json"),
    path.join(root, "tests", "income-fixture.json"),
  ],
  { stdio: "inherit", cwd: root },
);
process.exit(res.status);
