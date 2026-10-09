// Prints the stage breakdown of one or more scanner profile dumps.
// node scripts/browser/scan_summary.mjs target/scan-profiles/*.json
import fs from "node:fs";

const q = (xs, p) => {
  if (!xs.length) return NaN;
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.floor(p * s.length))];
};
const mean = (xs) =>
  xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : NaN;
const f = (x) => (Number.isFinite(x) ? x.toFixed(1) : "-");
const rust_map = (r) => Object.fromEntries((r ?? []).map(([n, t, c]) => [n, [t, c]]));

function summarize(file) {
  const d = JSON.parse(fs.readFileSync(file, "utf8"));
  // the first scan is the whole-frame anchor search, and the records after the
  // video ended are its last frame repeating twice a second
  const raw = d.scans;
  let last_live = 0;
  for (let i = 1; i < raw.length; i++) {
    if (raw[i].frame_time - raw[i - 1].frame_time < 200_000) last_live = i;
  }
  const rows = raw.slice(1, last_live + 1).map((s) => ({ ...s, r: rust_map(s.rust) }));
  // a frame dropped by worth_scanning never reaches the anchors
  const skipped = rows.filter((s) => !s.r["anchors"]);
  const scanned = rows.filter((s) => s.r["anchors"]);
  const chest = scanned.filter((s) => s.r["tooltip/chest"]);
  const item = scanned.filter((s) => s.r["tooltip/layout"] && !s.r["tooltip/chest"]);
  const plain = scanned.filter((s) => !s.r["tooltip/layout"] && !s.r["tooltip/chest"]);

  console.log(`\n=== ${file.split(/[\\/]/).pop()} ===`);
  console.log(
    `${d.recording?.split(/[\\/]/).pop() ?? "?"}, ${d.threads} threads`,
  );
  console.log(
    `${d.draws} frames drawn, ${rows.length} scanned frames of the recording, ${(
      (100 * scanned.length) / rows.length
    ).toFixed(0)}% scanned (${skipped.length} skipped)`,
  );

  console.log("\nmain thread and worker, over scanned frames (ms)");
  console.log("  stage            mean   med   p99");
  for (const k of ["total", "read", "to_worker", "copy", "wasm_call", "to_main", "process_result"]) {
    const xs = scanned.map((s) => s[k]).filter(Number.isFinite);
    console.log(
      `  ${k.padEnd(16)}${f(mean(xs)).padStart(5)} ${f(q(xs, 0.5)).padStart(5)} ${f(q(xs, 0.99)).padStart(5)}`,
    );
  }
  const sk = skipped.map((s) => s.wasm_call).filter(Number.isFinite);
  console.log(`  a skipped frame's wasm call: mean ${f(mean(sk))}`);

  console.log("\nrust stages (mean over every scanned frame; med and p99 over the frames it ran)");
  console.log("  stage                     mean   med   p99  calls  frames it ran");
  for (const n of [...new Set(rows.flatMap((s) => Object.keys(s.r)))].sort()) {
    const hits = scanned.filter((s) => s.r[n]);
    const xs = hits.map((s) => s.r[n][0]);
    const per_all = hits.reduce((a, s) => a + s.r[n][0], 0) / scanned.length;
    const calls = hits.reduce((a, s) => a + s.r[n][1], 0) / Math.max(1, hits.length);
    console.log(
      `  ${n.padEnd(24)}${f(per_all).padStart(6)} ${f(q(xs, 0.5)).padStart(5)} ${f(q(xs, 0.99)).padStart(5)} ${calls.toFixed(1).padStart(6)}  ${String(hits.length).padStart(5)} (${((100 * hits.length) / scanned.length).toFixed(0)}%)`,
    );
  }

  console.log("\nwasm call by tooltip state (mean ms)");
  for (const [label, set] of [
    ["no tooltip", plain],
    ["item tooltip", item],
    ["chest tooltip", chest],
  ]) {
    console.log(`  ${label.padEnd(14)} ${f(mean(set.map((s) => s.wasm_call))).padStart(6)}  n=${set.length}`);
  }

  const ocr = d.ocr ?? [];
  const lines = ocr.reduce((a, b) => a + (b.jobs ?? 0), 0);
  const took = ocr.map((b) => b.took).filter(Number.isFinite);
  // a batch with no first read records 0
  const waited = ocr.map((b) => b.waited).filter((x) => Number.isFinite(x) && x > 0);
  console.log(`\nOCR: ${ocr.length} batches, ${lines} lines, ${f(took.reduce((a, b) => a + b, 0) / Math.max(1, lines))} ms a line`);
  console.log(
    `  a hover's first read came back in: med ${f(q(waited, 0.5))} p90 ${f(q(waited, 0.9))} max ${f(Math.max(...waited))} (n=${waited.length})`,
  );
  // `at` is the same clock as the scan records; older dumps have none, so the
  // timeline is then rebuilt from the batch durations over three workers
  let t = 0;
  const seen = ocr.map((b) => ({ ...b, t: b.at != null ? b.at / 1000 : (t += b.took / 3) / 1000 }));
  const base = seen.length ? seen[0].t : 0;
  const left = ocr.map((b) => b.left);
  const span = scanned.length && scanned[0].at != null
    ? (scanned[scanned.length - 1].at - scanned[0].at) / 1000
    : (rows[rows.length - 1].frame_time - rows[0].frame_time) / 1e6;
  console.log(
    `  demand ${(lines / span).toFixed(1)} lines a second against about ${(3000 / (took.reduce((a, b) => a + b, 0) / Math.max(1, lines))).toFixed(0)} three workers can read`,
  );
  console.log(
    `  queue after a batch: median ${q(left, 0.5)} p90 ${q(left, 0.9)} max ${Math.max(...left)} lines, empty ${((100 * left.filter((x) => x === 0).length) / Math.max(1, left.length)).toFixed(0)}% of the time`,
  );
  // a backlog is a stretch where the queue stays over 20 lines
  const spikes = [];
  let cur = null;
  for (const b of seen) {
    if (b.left > 20 && !cur) cur = { start: b.t, peak: b.left };
    else if (cur) {
      cur.peak = Math.max(cur.peak, b.left);
      if (b.left <= 20) {
        cur.end = b.t;
        spikes.push(cur);
        cur = null;
      }
    }
  }
  if (cur) spikes.push({ ...cur, end: seen[seen.length - 1].t });
  console.log(`  ${spikes.length} backlogs over 20 lines${spikes.length ? ":" : ""}`);
  for (const sp of spikes.filter((x) => x.end - x.start > 0.5)) {
    console.log(`    peak ${String(sp.peak).padStart(3)} lines, ${(sp.end - sp.start).toFixed(1)} s, from ${(sp.start - base).toFixed(0)} s`);
  }
  let i = seen.length - 1;
  while (i > 0 && left[i - 1] >= left[i]) i--;
  console.log(
    `  it ended with ${left[i]} lines queued, cleared in ${(seen[seen.length - 1].t - seen[i].t).toFixed(1)} s`,
  );
  const renders = d.renders ?? [];
  console.log(`table updates: ${renders.length}, mean ${f(mean(renders))} p99 ${f(q(renders, 0.99))}`);
}

const files = process.argv.slice(2);
if (!files.length) {
  console.error("usage: node scripts/browser/scan_summary.mjs <dump.json>...");
  process.exit(1);
}
files.forEach(summarize);
