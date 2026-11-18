// Screenshots for the Guide page (frontend/Components/Info/Guide.vue) into public/Guide/.
// pnpm guide-shots [shot names...]   (needs pnpm dev running; no names = all)
import { chromium } from "@playwright/test";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import * as hf from "../../tests/e2e/helpers.ts";

const BASE_URL = process.env.BASE_URL ?? "http://localhost:5173";
const OUT = fileURLToPath(new URL("../../public/Guide/", import.meta.url));
const CAPTURE = fileURLToPath(
  new URL(
    "../../ignored/recordings/honing-forecast-2026-10-20T11-43-25-861Z.hfcap",
    import.meta.url,
  ),
);

const card = (page, title) =>
  page
    .locator(".card-shell")
    .filter({ has: page.locator(".card-title", { hasText: title }) })
    .first();

async function fill(locator, value) {
  await locator.fill(String(value));
  await locator.blur();
}

// +19 and +20 wanted on helmet, shoulder and chest, the rest done to +20, some special leaps and bound mats
async function example_char(page, mats = true) {
  await hf.open(page);
  for (const piece of ["Helmet", "Shoulder", "Chest"]) {
    await hf.cell(page, "normal", piece, 18).click();
    await hf.cell(page, "normal", piece, 18).click();
    await hf.cell(page, "normal", piece, 20).click();
  }
  for (const piece of ["Pants", "Glove", "Weapon"]) {
    await hf.cell(page, "normal", piece, 20).click();
    await hf.cell(page, "normal", piece, 20).click();
  }
  await fill(page.getByRole("textbox", { name: "Special leaps owned" }), 300);
  if (mats) {
    for (const [label, amount] of [
      ["Blue", 60000],
      ["Leaps", 1200],
      ["Fusion", 600],
      ["Gold", 80000],
      ["Silver", 5000000],
    ])
      await hf.set_material(page, label, "Bound owned", amount);
  }
  await hf.wait_for_optimizer(page);
}

const instruction_row = (page, name) =>
  card(page, "Normal Honing Instructions")
    .locator(".mats-row")
    .filter({ hasText: new RegExp(name.replace("+", "\\s*\\+")) })
    .first();

const SHOTS = {
  async signin(page) {
    await hf.open(page);
    const button = page.getByTitle("Sign in with lostark.bible");
    await button.evaluate(
      (el) =>
        (el.style.cssText +=
          "outline: 2px solid var(--gold); outline-offset: -2px"),
    );
    const b = await button.boundingBox();
    await page.screenshot({
      path: OUT + "signin.png",
      clip: { x: b.x - 200, y: 0, width: b.width + 200, height: 60 },
    });
  },

  async "char-card"(page) {
    await hf.open(page);
    await page
      .getByRole("button", { name: "Fetch from lostark.bible" })
      .waitFor();
    await page
      .getByText("Char name:")
      .locator(
        "xpath=ancestor::div[contains(@class,'card-shell') or contains(@class,'border')][1]",
      )
      .screenshot({ path: OUT + "char-card.png" });
  },

  async ticks(page) {
    await example_char(page, false);
    await page
      .getByRole("group", { name: "Normal honing" })
      .locator("xpath=ancestor::div[contains(@class,'card-shell')][1]")
      .screenshot({ path: OUT + "ticks.png" });
  },

  async scanner(page) {
    if (!fs.existsSync(CAPTURE))
      return console.log("skipped scanner: no capture");
    await hf.open(page, "/chars/Newchar/scanner");
    await page.getByLabel("Upload image or video").setInputFiles(CAPTURE);
    await page.getByText("Replay finished").waitFor({ timeout: 300_000 });
    await page.waitForTimeout(1000);
    await page.mouse.move(0, 0);
    await page
      .getByRole("group", { name: "Roster storage" })
      .first()
      .locator("xpath=ancestor::div[contains(@class,'card-shell')][1]")
      .screenshot({ path: OUT + "scanner.png" });
  },

  async "cost-dist"(page) {
    await example_char(page);
    await card(page, "Costs distribution").screenshot({
      path: OUT + "cost-dist.png",
    });
  },

  async market(page) {
    await hf.open(page, "/market-mats");
    // header and the main mats, down to Lava's Breath
    const table = page.getByRole("group", { name: "T4 materials" });
    const top = await table.boundingBox();
    const last = await hf.material_row(table, "Lava's Breath").boundingBox();
    await page.screenshot({
      path: OUT + "market.png",
      clip: { ...top, height: last.y + last.height - top.y },
    });
  },

  async instructions(page) {
    await example_char(page);
    await card(page, "Normal Honing Instructions").screenshot({
      path: OUT + "instructions.png",
    });
  },

  async freetap(page) {
    await example_char(page);
    // the first free tap, so its order matches the instructions shot
    const row = instruction_row(page, "Free tap this until");
    await row.getByRole("button", { name: "Show input anyway" }).click();
    await fill(row.getByRole("spinbutton"), 120);
    await row.screenshot({ path: OUT + "freetap.png" });
  },

  async nonfreetap(page) {
    await example_char(page);
    const row = instruction_row(page, "Do not use special");
    await row.getByRole("slider").fill("12");
    await row.screenshot({ path: OUT + "nonfreetap.png" });
  },
};

const names = process.argv.slice(2);
const todo = names.length ? names : Object.keys(SHOTS);
const unknown = todo.filter((n) => !SHOTS[n]);
if (unknown.length) {
  console.error(`unknown shots: ${unknown}. Known: ${Object.keys(SHOTS)}`);
  process.exit(1);
}
if (!(await fetch(BASE_URL).catch(() => null))) {
  console.error(`nothing on ${BASE_URL} - start it with pnpm dev`);
  process.exit(1);
}

const browser = await chromium.launch();
let failed = false;
for (const name of todo) {
  // fresh profile per shot
  const page = await browser.newPage({
    baseURL: BASE_URL,
    viewport: { width: 1600, height: 1000 },
  });
  await hf.stub_market(page);
  // the sticky header would cover element shots
  await page.addInitScript(() =>
    addEventListener("DOMContentLoaded", () => {
      const style = document.createElement("style");
      style.textContent = "header, .sticky { position: static !important; }";
      document.head.append(style);
    }),
  );
  try {
    await SHOTS[name](page);
    console.log("shot", name);
  } catch (e) {
    failed = true;
    console.error(name, e);
  }
  await page.close();
}
await browser.close();
process.exit(failed ? 1 : 0);
