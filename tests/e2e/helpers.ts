// Shared by the e2e tests and `pnpm browse` scripts. See AI notes/Browser harness.
import { expect } from "@playwright/test";
import type { Locator, Page } from "@playwright/test";
import fs from "node:fs";

type GridKind = "normal" | "adv";
const MARKET_FIXTURE = new URL("./fixtures/market.json", import.meta.url);

// a Worker endpoint, deployed or under `pnpm cloudflare-dev`
export const worker_route = (path: string) =>
  new RegExp(String.raw`(workers\.dev|api\.honing-forecast\.com|localhost:8787)${path}`);

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
  await page.route(worker_route("/prices/"), (route) => {
    const region = new URL(route.request().url()).pathname.split("/").pop();
    return route.fulfill({ json: fixture[region] });
  });
}

// Firefox logs a failed run when a page is left mid-optimizer, so let any run finish first
async function settle(page: Page) {
  const busy = () =>
    page
      .evaluate(async () => {
        const url = performance
          .getEntriesByType("resource")
          .map((entry) => entry.name)
          .filter((name) => name.includes("Stores/RuntimeState.ts"))
          .pop();
        if (!url) return false;
        const runtime = (await import(url)).useRuntimeStore();
        return (
          runtime.optimizer.status === "busy" ||
          runtime.histogram.status === "busy" ||
          !!runtime.histogram.queued
        );
      })
      .catch(() => false);
  await expect.poll(busy, { timeout: 60_000 }).toBe(false);
}

export async function open(page: Page, path = "/") {
  await settle(page);
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

// NaN until the number is on the page
async function read_number(page: Page, pattern: RegExp) {
  const match = (await page.locator("body").textContent()).match(pattern);
  return match ? Number(match[1].replaceAll(",", "")) : NaN;
}

export const read_gold = (page: Page) =>
  read_number(page, /gold spent:\s*([\d,]+)/);

export const read_pending_ilevel = (page: Page) =>
  read_number(page, /Pending ilevel:\s*([\d.]+)/);

// Every material row in the costs card, and whether its graph drew anything. A row that
// is used but shows the empty message means its data did not reach the graph, which is
// what a row-versus-label mix-up looks like from the outside.
export async function material_graphs(page: Page) {
  return page
    .locator(".mats-row[role=group]:not(.instruction-row)")
    .evaluateAll((groups) =>
      groups.map((g) => ({
        label: g.getAttribute("aria-label"),
        drawn: !g.textContent.includes("never used"),
      })),
    );
}

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

// Runs `fn(store, arg)` in the page against the roster store (the app's own module instance).
export async function store_eval(page: Page, fn: Function, arg?: unknown) {
  return page.evaluate(
    async ([src, arg]) => {
      const url = performance
        .getEntriesByType("resource")
        .map((entry) => entry.name)
        .filter((name) => name.includes("Stores/RosterConfig.ts"))
        .pop();
      const { useRosterStore } = await import(url);
      return (0, eval)(`(${src})`)(useRosterStore(), arg);
    },
    [fn.toString(), arg] as [string, unknown],
  );
}

// an instruction row, by upgrade name: "Helmet +16" or "Helmet Advanced +0 - 10"
export function upgrade_row(page: Page, name: string) {
  return page.getByRole("group", { name, exact: true });
}

// the debounced save has to land before a reload
export async function reload(page: Page) {
  await page.waitForTimeout(1000);
  await settle(page);
  await page.reload();
  await expect(page.getByRole("link", { name: "Roster setup" })).toBeVisible();
}
