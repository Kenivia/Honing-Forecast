// Class emblems as single-colour SVGs, cropped to their content: the official site's class pages,
// and lostark.bible for the two it lacks (Guardianknight's page shows only the family emblem).
import { chromium } from "@playwright/test";
import { mkdirSync, writeFileSync } from "node:fs";

const OUT = "public/Icons/Classes";
const OFFICIAL = "https://www.playlostark.com/en-us/game/classes/";
const FAMILIES = [
  "warrior",
  "martial-artist",
  "gunner",
  "mage",
  "assassin",
  "specialist",
];
const FROM_BIBLE = ["guardianknight", "dimensionalist"];

const browser = await chromium.launch();
const page = await browser.newPage();
const icons = {};

for (const family of FAMILIES) {
  await page.goto(OFFICIAL + family);
  await page.waitForSelector("a svg");
  Object.assign(
    icons,
    await page.evaluate(() => {
      const out = {};
      for (const a of document.querySelectorAll("a")) {
        const m = a.textContent.trim().match(/^(.*?)(Melee|Range)/);
        const svg = a.querySelector("svg");
        if (!m || !svg) continue;
        const b = svg.getBBox();
        const box = [b.x, b.y, b.width, b.height].map((v) => +v.toFixed(2));
        const inner = svg.innerHTML.replace(/\s(class|id)="[^"]*"/g, "");
        out[m[1].toLowerCase().replace(/[^a-z]/g, "")] =
          `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${box.join(" ")}">${inner}</svg>\n`;
      }
      return out;
    }),
  );
}

await page.goto("https://lostark.bible/stats/generic");
await page.waitForSelector("svg");
const bible = await page.evaluate((keys) => {
  const out = {};
  for (const svg of document.querySelectorAll("svg")) {
    let row = svg.parentElement;
    while (row && !row.textContent.trim()) row = row.parentElement;
    const key = row?.textContent
      .trim()
      .split(/[\s\d]/)[0]
      .toLowerCase();
    if (!keys.includes(key) || out[key]) continue;
    const b = svg.getBBox();
    const box = [b.x, b.y, b.width, b.height].map((v) => +v.toFixed(2));
    const inner = svg.innerHTML.replace(/\s(class|id)="[^"]*"/g, "");
    out[key] =
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${box.join(" ")}">${inner}</svg>\n`;
  }
  return out;
}, FROM_BIBLE);
Object.assign(icons, bible);
await browser.close();

mkdirSync(OUT, { recursive: true });
for (const [key, svg] of Object.entries(icons))
  writeFileSync(`${OUT}/${key}.svg`, svg);
console.log(Object.keys(icons).sort().join(" "), Object.keys(icons).length);
