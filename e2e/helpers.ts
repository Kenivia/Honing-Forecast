// Shared by the e2e tests and `pnpm browse` scripts. See AI notes/Browser harness.
import { expect } from "@playwright/test";
import type { Locator, Page } from "@playwright/test";
import fs from "node:fs";

type GridKind = "normal" | "adv";
const MARKET_FIXTURE = new URL("./fixtures/market.json", import.meta.url);

// collects uncaught errors and console errors
export function watch_errors(page: Page): string[] {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
  return errors;
}

// serves saved market prices instead of hitting the Worker; call before open
export async function stub_market(page: Page) {
  const fixture = JSON.parse(fs.readFileSync(MARKET_FIXTURE, "utf8"));
  await page.route(/workers\.dev/, (route) => {
    const request = route.request();
    if (request.method() !== "POST") return route.continue();
    return route.fulfill({ json: fixture[request.postDataJSON().region_slug] });
  });
}

export async function open(page: Page, path = "/") {
  await page.goto(path);
  await expect(page.getByRole("link", { name: "Roster setup" })).toBeVisible();
}

export function grid(page: Page, kind: GridKind): Locator {
  return page.getByRole("group", {
    name: kind === "normal" ? "Normal honing" : "Advanced honing",
  });
}

export function cell(page: Page, kind: GridKind, piece: string, level: number) {
  return grid(page, kind).getByRole("button", {
    name: new RegExp(`^${piece} \\+${level}:`),
  });
}

// NotYet | Want | Done | FetchedDone
export async function cell_status(
  page: Page,
  kind: GridKind,
  piece: string,
  level: number,
) {
  const name = await cell(page, kind, piece, level).getAttribute("aria-label");
  return name.split(": ")[1];
}

export async function toggle_column(page: Page, kind: GridKind, level: number) {
  await grid(page, kind)
    .getByRole("button", { name: `+${level}`, exact: true })
    .click();
}

// progress sits at 100% when idle, so wait for it to leave 100% first
export async function wait_for_optimizer(page: Page, timeout = 60_000) {
  const done = /Optimizer progress:\s*100\.00%/;
  const body = page.locator("body");
  await expect(body)
    .not.toContainText(done, { timeout: 2000 })
    .catch(() => {});
  await expect(body).toContainText(done, { timeout });
}

async function read_number(page: Page, pattern: RegExp) {
  const text = await page.locator("body").textContent();
  return Number(text.match(pattern)[1].replaceAll(",", ""));
}

export const read_gold = (page: Page) =>
  read_number(page, /gold spent:\s*([\d,]+)/);

export const read_pending_ilevel = (page: Page) =>
  read_number(page, /Pending ilevel:\s*([\d.]+)/);

// a row of a material table, on the calc or market page
export function material_row(scope: Page | Locator, label: string) {
  return scope.getByRole("group", { name: label, exact: true });
}

// column is "Bound owned" on calc; "Roster bound owned", "Tradable owned" or "Market price" on market
export async function set_material(
  scope: Page | Locator,
  label: string,
  column: string,
  value: string | number,
) {
  const input = material_row(scope, label).getByRole("textbox", {
    name: column,
  });
  await input.fill(String(value));
  await input.blur();
}
