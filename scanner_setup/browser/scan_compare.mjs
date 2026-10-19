// Two scanner profile runs side by side: mean ms per scanned frame of every stage, per recording.
// pnpm scanner-compare <run a> <run b>     (names under target/scan-profiles, as --name gave them)
// Only compare runs made back to back: the same build differs by a fifth an hour apart.
import fs from "node:fs";
import path from "node:path";

const DIR = "target/scan-profiles";
const MAIN = ["wasm_call", "copy", "to_main", "process_result"];

// stage -> mean over the scanned frames of the recording, as scan_summary counts them
function means(file) {
  const raw = JSON.parse(fs.readFileSync(file, "utf8")).scans;
  let last_live = 0;
  for (let i = 1; i < raw.length; i++) {
    if (raw[i].frame_time - raw[i - 1].frame_time < 200_000) last_live = i;
  }
  const rows = raw.slice(1, last_live + 1);
  const scanned = rows.filter((s) => s.rust.some((x) => x[0] === "anchors"));
  const total = {};
  const add = (name, value) => (total[name] = (total[name] ?? 0) + value);
  for (const scan of scanned) {
    for (const name of MAIN) add(name, scan[name] ?? 0);
    add("loop without read", scan.total - scan.read);
    for (const [name, ms] of scan.rust) add(`  ${name}`, ms);
  }
  for (const name in total) total[name] /= scanned.length;
  return { total, scanned: scanned.length, taken: rows.length };
}

const [a, b] = process.argv.slice(2);
if (!b) {
  console.error("usage: pnpm scanner-compare <run a> <run b>");
  process.exit(1);
}
for (const file of fs.readdirSync(path.join(DIR, a)).filter((x) => x.endsWith(".json")).sort()) {
  if (!fs.existsSync(path.join(DIR, b, file))) continue;
  const [one, other] = [a, b].map((run) => means(path.join(DIR, run, file)));
  console.log(`\n=== ${file}: ${one.scanned} / ${other.scanned} scanned of ${one.taken} / ${other.taken} taken ===`);
  console.log(`  ${"stage".padEnd(24)}${a.padStart(10)}${b.padStart(10)}    change`);
  const names = [...new Set([...Object.keys(one.total), ...Object.keys(other.total)])];
  const main = names.filter((x) => !x.startsWith("  "));
  for (const name of [...main, ...names.filter((x) => x.startsWith("  ")).sort()]) {
    const [x, y] = [one.total[name] ?? 0, other.total[name] ?? 0];
    const change = `${y - x >= 0 ? "+" : ""}${(y - x).toFixed(2)}`;
    console.log(`  ${name.padEnd(24)}${x.toFixed(2).padStart(10)}${y.toFixed(2).padStart(10)}${change.padStart(10)}`);
  }
}
