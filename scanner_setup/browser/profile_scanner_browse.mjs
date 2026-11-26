// A pnpm browse script: replays one debug capture through the scanner page and dumps its timings.
// Driven by scanner_setup/browser/profile_scanner.mjs. A replay scans every frame once the texts of
// the one before are back, so OCR waits are in the loop and the scan stages are what to compare.
import fs from "node:fs";

const FILE = process.env.REC;
const OUT = process.env.OUT;

export default async ({ page, hf }) => {
  await hf.open(page, "/chars/NA/Newchar/scanner");
  await page.getByLabel("Upload image or video").setInputFiles(FILE);

  // the page keeps the last 5000 of each, a long capture has more
  const kept = { scans: [], ocr: [], renders: [] };
  const drain = async () => {
    const part = await page.evaluate(() => ({
      scans: (window.__scan_timings ?? []).splice(0),
      ocr: (window.__ocr_timings ?? []).splice(0),
      renders: (window.__scan_renders ?? []).splice(0),
    }));
    for (const key in kept) kept[key].push(...part[key]);
  };
  const done = page.getByText("Replay finished, re-run on this build.");
  for (let i = 0; i < 1800; i++) {
    await page.waitForTimeout(2000);
    await drain();
    if (await done.isVisible()) break;
    if (i % 15 === 0) console.log(`  ${2 * i}s  ${kept.scans.length} scans`);
  }
  await drain();
  console.log(`  replay done, ${kept.scans.length} scans`);

  const data = await page.evaluate(() => ({ threads: navigator.hardwareConcurrency }));
  Object.assign(data, kept);
  data.recording = FILE;
  fs.writeFileSync(OUT, JSON.stringify(data));
  console.log(`  wrote ${OUT}`);
};
