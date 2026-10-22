// Downloads every analytics upload from D1 into ignored/analytics/, then deletes what was
// downloaded. `node scripts/pull-analytics.mjs [--keep] [--local]`: --keep skips the delete,
// --local reads `wrangler dev`'s local database.
import { execFileSync } from "node:child_process";
import fs from "node:fs";

const PAGE = 100;
const keep = process.argv.includes("--keep");
const where = process.argv.includes("--local") ? "--local" : "--remote";

function d1(command) {
  const out = execFileSync(
    process.execPath,
    [
      "node_modules/wrangler/bin/wrangler.js",
      "d1",
      "execute",
      "hf-analytics",
      where,
      "--json",
      "--config",
      "cloudflare/wrangler.toml",
      "--command",
      command,
    ],
    { encoding: "utf8", maxBuffer: 1 << 30 },
  );
  return JSON.parse(out)[0].results;
}

// rows arriving during the pull are left for the next one
const cutoff = d1("SELECT MAX(id) AS max FROM uploads")[0].max;
if (cutoff === null) {
  console.log("nothing to pull");
  process.exit(0);
}

const rows = [];
for (let last = 0; ; ) {
  const page = d1(
    `SELECT * FROM uploads WHERE id > ${last} AND id <= ${cutoff} ORDER BY id LIMIT ${PAGE}`,
  );
  if (page.length === 0) break;
  for (const row of page) {
    rows.push({
      ...row,
      snapshots: JSON.parse(row.snapshots),
      raids: JSON.parse(row.raids),
    });
  }
  last = page.at(-1).id;
  console.log(`${rows.length} uploads`);
}

fs.mkdirSync("ignored/analytics", { recursive: true });
const file = `ignored/analytics/uploads-${new Date().toISOString().replace(/[:.]/g, "-")}.json`;
fs.writeFileSync(file, JSON.stringify(rows));
console.log(`written to ${file}`);

if (!keep) {
  d1(`DELETE FROM uploads WHERE id <= ${cutoff}`);
  console.log(`deleted up to id ${cutoff}`);
}
