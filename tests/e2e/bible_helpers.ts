// lostark.bible and Worker-scrape stubs plus store readers, for bible.spec.ts, bible_live.spec.ts, roster.spec.ts.
import type { Page } from "@playwright/test";
import fs from "node:fs";
import { worker_route } from "./helpers.ts";

const FIX = new URL("../unit/market-import/fixtures/", import.meta.url);
const read = (name: string) => fs.readFileSync(new URL(name, FIX), "utf8");
const PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
  "base64",
);
const CORS = {
  "access-control-allow-origin": "*",
  "access-control-allow-headers": "authorization, content-type",
};

export interface BibleChar {
  name: string;
  class?: string;
  ilvl?: number;
}
export interface BibleRosterStub {
  region: "NA" | "CE";
  world?: string;
  characters: BibleChar[];
}
export interface BibleReq {
  method: string;
  path: string; // pathname + search
  auth: string | null;
  body: string | null;
}

export class BibleStub {
  requests: BibleReq[] = [];
  rosters: BibleRosterStub[];
  loadouts: Record<string, any>;
  roster_status = 200;
  loadout_status = 200;
  authorize_error = false;
  constructor(rosters: BibleRosterStub[], loadouts: Record<string, any> = {}) {
    this.rosters = rosters;
    this.loadouts = loadouts;
  }
  // requests whose path starts with `prefix`
  hits(prefix: string) {
    return this.requests.filter(
      (r) => r.method !== "OPTIONS" && r.path.startsWith(prefix),
    );
  }
}

export const loadout_fixture = (kind: "serca" | "t4") =>
  JSON.parse(read(`loadouts_${kind}.json`));

// every lostark.bible request is answered here: an unstubbed one with a fake token would 401 on the real site and wipe the sign-in
export async function stub_bible(
  page: Page,
  rosters: BibleRosterStub[],
  loadouts: Record<string, any> = {},
) {
  const stub = new BibleStub(rosters, loadouts);
  await page.route(/cdn\.discordapp\.com/, (route) =>
    route.fulfill({ body: PNG, contentType: "image/png" }),
  );
  await page.route(/^https:\/\/lostark\.bible\//, async (route) => {
    const req = route.request();
    const url = new URL(req.url());
    const path = url.pathname + url.search;
    stub.requests.push({
      method: req.method(),
      path,
      auth: req.headers()["authorization"] ?? null,
      body: req.postData(),
    });
    if (req.method() === "OPTIONS")
      return route.fulfill({ status: 204, headers: CORS });
    const json = (data: any, status = 200) =>
      route.fulfill({ status, json: data, headers: CORS });
    if (url.pathname === "/oauth/authorize") {
      const back = new URL(url.searchParams.get("redirect_uri"));
      if (stub.authorize_error) back.searchParams.set("error", "access_denied");
      else back.searchParams.set("code", "c");
      back.searchParams.set("state", url.searchParams.get("state"));
      return route.fulfill({
        status: 302,
        headers: { location: back.toString() },
      });
    }
    if (url.pathname === "/oauth/token")
      return json({
        access_token: "uwo_t",
        token_type: "Bearer",
        expires_in: 7776000,
        scope: "identify rosters loadouts",
      });
    if (url.pathname === "/oauth/revoke")
      return route.fulfill({ status: 200, headers: CORS, body: "" });
    if (url.pathname === "/api/oauth/user")
      return json({
        id: 1,
        discordId: "123456789012345678",
        discordUsername: "tester",
        discordGlobalName: "Tester",
        discordAvatar: null,
      });
    if (url.pathname === "/api/oauth/rosters") {
      if (stub.roster_status !== 200)
        return json({ error: "invalid_token" }, stub.roster_status);
      return json({
        rosters: stub.rosters.map((r) => ({
          world: "Test",
          ...r,
          characters: r.characters.map((c) => ({
            class: "bard",
            ilvl: 1700,
            lastUpdate: 1,
            ...c,
          })),
        })),
      });
    }
    if (url.pathname.startsWith("/api/oauth/loadouts/")) {
      if (stub.loadout_status !== 200)
        return json({ error: "boom" }, stub.loadout_status);
      const name = decodeURIComponent(url.pathname.split("/").pop());
      return json(stub.loadouts[name] ?? loadout_fixture("serca"));
    }
    return route.fulfill({ status: 404, headers: CORS, body: "" });
  });
  return stub;
}

// a stored token, injected once per tab so a reload after sign-out stays signed out
export async function sign_in_as(page: Page, name = "Tester") {
  await page.addInitScript((n) => {
    if (sessionStorage.getItem("test_signed_in")) return;
    sessionStorage.setItem("test_signed_in", "1");
    localStorage.setItem(
      "bible_auth",
      JSON.stringify({
        token: "uwo_t",
        expires_at: Date.now() + 86_400_000,
        name: n,
      }),
    );
  }, name);
}

export const bible_auth = (page: Page) =>
  page.evaluate(() => JSON.parse(localStorage.getItem("bible_auth")));

export function char_html(kind: "t4" | "serca" = "t4", class_name?: string) {
  const html = read(`char_${kind}.html`);
  return class_name
    ? html.replace(/<p>(Bard|Berserker)<\/p>/, `<p>${class_name}</p>`)
    : html;
}
export const roster_html = (names: string[]) =>
  `<html><body><div><p>Roster</p></div><div>${names
    .map((n) => `<div><div><div>${n} 1,700 Bard</div></div></div>`)
    .join("")}</div></body></html>`;

export interface ScrapeStub {
  requests: string[]; // "<region>/<name>/<suffix>"
}
// Worker scrape: chars by name, rosters by any member's name; anything else is "not found"
export async function stub_character(
  page: Page,
  chars: Record<string, string>,
  rosters: Record<string, string[]> = {},
) {
  const stub: ScrapeStub = { requests: [] };
  await page.route(worker_route("/character/"), (route) => {
    const [, , region, name, suffix = ""] = new URL(
      route.request().url(),
    ).pathname
      .split("/")
      .map(decodeURIComponent);
    stub.requests.push(`${region}/${name}/${suffix}`);
    const body =
      suffix === "roster"
        ? rosters[name] && roster_html(rosters[name])
        : chars[name];
    return route.fulfill({
      status: 200,
      contentType: "text/html",
      body: body ?? read("not_found.html"),
    });
  });
  return stub;
}

// runs fn in the page against the roster store (reads the module the app already loaded)
export async function with_store<T, A = undefined>(
  page: Page,
  fn: (store: any, arg: A) => T,
  arg?: A,
): Promise<T> {
  const run = () =>
    page.evaluate(
      async ({ src, arg }) => {
        const find = () =>
          performance
            .getEntriesByType("resource")
            .map((entry) => entry.name)
            .filter((name) => name.includes("Stores/RosterConfig.ts"))
            .pop();
        for (let i = 0; i < 50 && !find(); i++)
          await new Promise((r) => setTimeout(r, 100));
        const { useRosterStore } = await import(find());
        return new Function("s", "a", `return (${src})(s, a)`)(
          useRosterStore(),
          arg,
        );
      },
      { src: fn.toString(), arg },
    );
  // the page may be mid-navigation (sign-in redirects)
  for (let i = 0; ; i++) {
    try {
      return (await run()) as T;
    } catch (e) {
      if (i >= 20) throw e;
      await page.waitForTimeout(200);
    }
  }
}

// character labels (the name, or Newchar... when unnamed) per roster, in roster order
export const roster_names = (page: Page) =>
  with_store(page, (s) =>
    s.roster_ids.map((id: string) =>
      s.roster_config.profiles
        .filter((p: any) => p.roster_id === id)
        .map((p: any) => s.char_label(p)),
    ),
  );
export const roster_regions = (page: Page) =>
  with_store(page, (s) =>
    s.roster_ids.map((id: string) => s.roster_config.all_regions[id]),
  );
export const profile = (page: Page, name: string) =>
  with_store(
    page,
    (s, n: string) =>
      JSON.parse(
        JSON.stringify(
          s.roster_config.profiles.find((p: any) => s.char_label(p) === n) ??
            null,
        ),
      ),
    name,
  );

// a failed request is logged by Chromium as a console error; drop those for a test that causes them on purpose
export function ignore_http(errors: string[], ...codes: number[]) {
  const re = new RegExp(`status of (${codes.join("|")})`);
  for (let i = errors.length - 1; i >= 0; i--)
    if (re.test(errors[i])) errors.splice(i, 1);
}

export interface SeedRoster {
  region?: "nae" | "euc" | "Custom";
  names: string[];
  fetched?: boolean; // mark as imported before (auto-refresh and linked rosters)
}
// replaces the config's characters with these rosters (first roster reuses the default character); "" is unnamed
export const seed = (page: Page, rosters: SeedRoster[]) =>
  with_store(
    page,
    (s, rosters: SeedRoster[]) => {
      const first = s.roster_config.profiles[0];
      rosters.forEach((r, i) => {
        const region = r.region ?? "nae";
        let p;
        if (i === 0) {
          p = first;
          s.roster_config.all_regions[p.roster_id] = region;
        } else p = s.add_roster(region);
        p.char_name = r.names[0];
        const ids = [p];
        for (const name of r.names.slice(1)) {
          const q = s.add_char(p.roster_id);
          q.char_name = name;
          ids.push(q);
        }
        if (r.fetched) ids.forEach((q) => (q.fetched_at = 1));
      });
    },
    rosters,
  );
