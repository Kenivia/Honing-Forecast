// A pnpm browse script: plays one upload into the scanner and dumps its timings.
// Driven by scripts/browser/profile_scanner.mjs; see AI notes/Browser harness.
import fs from "node:fs";

const FILE = process.env.REC;
const DEBUG_ON = process.env.DEBUG_ON === "1";
const OUT = process.env.OUT;

export default async ({ page, hf }) => {
  await page.addInitScript(() => {
    // headless Chromium skips frames of a video playing at full speed
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      if (String(this.src).startsWith("blob:")) this.playbackRate = 0.5;
      return play.apply(this, arguments);
    };
    window.__draws = 0;
    const draw = CanvasRenderingContext2D.prototype.drawImage;
    CanvasRenderingContext2D.prototype.drawImage = function () {
      window.__draws++;
      return draw.apply(this, arguments);
    };
  });

  await hf.open(page, "/Newchar/scanner");
  // the debug UI is switched off for now, so there is nothing to hide
  const hide = page.getByRole("button", { name: "Hide debug info" });
  if (!DEBUG_ON && (await hide.count())) await hide.click();
  await page.getByLabel("Upload image or video").setInputFiles(FILE);
  await page.getByText("● Live").waitFor({ timeout: 180_000 });

  // once the video ends the upload only repeats its last frame twice a second
  // the page keeps the last 5000 of each, a long recording has more
  const kept = { scans: [], ocr: [], renders: [] };
  const drain = async () => {
    const part = await page.evaluate(() => ({
      scans: window.__scan_timings.splice(0),
      ocr: window.__ocr_timings.splice(0),
      renders: window.__scan_renders.splice(0),
    }));
    for (const key in kept) kept[key].push(...part[key]);
  };
  let draws = 0;
  for (let i = 0; i < 1200; i++) {
    await page.waitForTimeout(2000);
    await drain();
    const now = await page.evaluate(() => window.__draws);
    const rate = (now - draws) / 2;
    draws = now;
    if (i > 2 && rate < 5) break;
    if (i % 5 === 0) console.log(`  ${2 * i}s  ${draws} frames  ${rate}/s`);
  }
  console.log(`  video done, ${draws} frames drawn; waiting for OCR`);

  for (let i = 0; i < 180; i++) {
    await drain();
    if (!kept.ocr.length || !kept.ocr[kept.ocr.length - 1].left) break;
    await page.waitForTimeout(1000);
  }

  const data = await page.evaluate(() => ({
    draws: window.__draws,
    threads: navigator.hardwareConcurrency,
  }));
  Object.assign(data, kept);
  data.recording = FILE;
  data.debugging = DEBUG_ON;
  fs.writeFileSync(OUT, JSON.stringify(data));
  console.log(`  wrote ${OUT}`);
};
