// lostark.bible sign-in, sync, refresh; everything lostark.bible and the Worker answers is stubbed.
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import * as hf from "./helpers.ts";
import * as bh from "./bible_helpers.ts";

let errors: string[];
let bible: bh.BibleStub;
test.beforeEach(async ({ page }) => {
  errors = hf.watch_errors(page);
  await hf.stub_market(page);
  await bh.stub_character(page, {});
});
test.afterEach(() => expect(errors).toEqual([]));

const na = (...names: string[]): bh.BibleRosterStub => ({
  region: "NA",
  characters: names.map((name) => ({ name })),
});
const ce = (...names: string[]): bh.BibleRosterStub => ({
  region: "CE",
  characters: names.map((name) => ({ name })),
});
const sign_in_button = (page: Page) =>
  page.locator("header").getByRole("button", { name: /^Sign in/ });
const sign_out_button = (page: Page) =>
  page.locator("header").getByRole("button", { name: "Sign out" });
const sync_button = (page: Page) =>
  page.getByRole("button", { name: "Sync linked roster" });

// signed in on the roster page with these local rosters
async function setup(
  page: Page,
  local: bh.SeedRoster[],
  rosters: bh.BibleRosterStub[],
  loadouts = {},
) {
  bible = await bh.stub_bible(page, rosters, loadouts);
  await bh.sign_in_as(page);
  await hf.open(page, "/roster-setup");
  await bh.seed(page, local);
  await persist(page);
}
async function sync(page: Page) {
  await sync_button(page).click();
  await expect(sync_button(page)).toBeEnabled();
  // Firefox logs worker errors when a page with a running job is left
  await page.waitForTimeout(1500);
}
const persist = (page: Page) => page.waitForTimeout(1000);

test("sign-in flow: authorize, callback, back to the page, sync", async ({
  page,
}) => {
  bible = await bh.stub_bible(page, [na("Alpha", "Beta")]);
  await hf.open(page, "/roster-setup");
  await bh.seed(page, [{ names: ["Mine"] }]);
  await page.goto("/chars/NA/Mine/calc");
  await persist(page);

  await sign_in_button(page).click();
  await expect(sign_out_button(page)).toBeVisible();
  await expect(page).toHaveURL(/\/chars\/NA\/Mine\/calc$/);
  await expect(
    page.locator("header").getByRole("img", { name: "Tester" }),
  ).toBeVisible();

  const auth = new URL(`https://x${bible.hits("/oauth/authorize")[0].path}`);
  const q = auth.searchParams;
  expect(q.get("client_id")).toBeTruthy();
  expect(q.get("redirect_uri")).toBe(
    new URL(page.url()).origin + "/auth/callback",
  );
  expect(q.get("response_type")).toBe("code");
  expect(q.get("code_challenge_method")).toBe("S256");
  expect(q.get("code_challenge")).toBeTruthy();
  expect(q.get("scope")).toBe("identify rosters loadouts");
  expect(q.get("state")).toBeTruthy();
  expect(bible.hits("/oauth/token")[0].body).toContain("code_verifier=");

  const stored = await bh.bible_auth(page);
  expect(stored.token).toBe("uwo_t");
  expect(stored.name).toBe("Tester");
  const days = (stored.expires_at - Date.now()) / 86_400_000;
  expect(days).toBeGreaterThan(89);
  expect(days).toBeLessThan(91);

  // the sync ran: Mine unmatched and Beta/Alpha both on lostark.bible -> new roster
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Mine"], ["Alpha", "Beta"]]);
});

test("callback failures show the message and a way back", async ({ page }) => {
  bible = await bh.stub_bible(page, []);
  await hf.open(page);
  await page.goto("/auth/callback");
  await expect(
    page.getByText("Signing in with lostark.bible failed"),
  ).toBeVisible();
  await page.getByRole("link", { name: "Back to Honing Forecast" }).click();
  await expect(page).toHaveURL(/\/chars\/NA\/Newchar\/calc$/);
  // Firefox logs a failed run when goto leaves a running optimizer
  await expect(page.locator("body")).toContainText(
    /Optimizer progress:\s*100\.00%/,
  );

  // a state that does not match the pending one
  await page.evaluate(() =>
    sessionStorage.setItem(
      "bible_auth_pending",
      JSON.stringify({ verifier: "v".repeat(50), state: "right", back: "/" }),
    ),
  );
  await page.goto("/auth/callback?code=c&state=wrong");
  await expect(
    page.getByText("Signing in with lostark.bible failed"),
  ).toBeVisible();
  expect(await bh.bible_auth(page)).toBeNull();
  expect(bible.hits("/oauth/token")).toHaveLength(0);

  // denied on lostark.bible
  bible.authorize_error = true;
  await page.getByRole("link", { name: "Back to Honing Forecast" }).click();
  await sign_in_button(page).click();
  await expect(
    page.getByText("Signing in with lostark.bible failed"),
  ).toBeVisible();
  expect(await bh.bible_auth(page)).toBeNull();
});

test("fresh install: the untouched Newchar is replaced", async ({ page }) => {
  bible = await bh.stub_bible(page, [na("Alpha", "Beta")]);
  await hf.open(page, "/chars/NA/Newchar/calc");
  await sign_in_button(page).click();
  await expect.poll(() => bh.roster_names(page)).toEqual([["Alpha", "Beta"]]);
  await expect(page).toHaveURL(/\/chars\/NA\/(Alpha|Beta)\/calc$/);
  await expect(
    page.getByRole("group", { name: "Normal honing" }),
  ).toBeVisible();
  await persist(page);
  await page.reload();
  await expect(
    page.getByRole("group", { name: "Normal honing" }),
  ).toBeVisible();
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Beta"]]);
});

test("a touched Newchar survives the first sync", async ({ page }) => {
  await setup(page, [{ names: [""] }], [na("Alpha", "Beta")]);
  await hf.open(page, "/chars/NA/Newchar/calc");
  await hf.toggle_column(page, "normal", 15);
  await hf.set_material(page, "Red", "Bound owned", 777);
  await page.goto("/market-mats");
  await hf.set_material(page, "Red", "Roster bound owned", 5000);
  await persist(page);
  await page.goto("/roster-setup");
  await sync(page);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Newchar"], ["Alpha", "Beta"]]);
  await page.goto("/chars/NA/Newchar/calc");
  expect(await hf.cell_status(page, "normal", "Helmet", 15)).toBe("Want");
  const kept = await bh.with_store(page, (s) => {
    const p = s.roster_config.profiles[0];
    return [
      p.bound_budgets.values.Red,
      s.roster_config.roster_mats_owned[p.roster_id].values.Red,
    ];
  });
  expect(kept.map((v) => v.replace(/\D/g, ""))).toEqual(["777", "5000"]);
});

test("a chest on Newchar also keeps it", async ({ page }) => {
  await setup(page, [{ names: [""] }], [na("Alpha")]);
  await bh.with_store(page, (s) =>
    s.roster_config.profiles[0].chests.push({ key: "k" }),
  );
  await sync(page);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Newchar"], ["Alpha"]]);
});

test("sync matches by normalised name and renames to the exact one", async ({
  page,
}) => {
  await setup(page, [{ names: ["Elsa"] }], [na("Ëlsa")]);
  await sync(page);
  await expect.poll(() => bh.roster_names(page)).toEqual([["Ëlsa"]]);
  await page.goto(`/chars/NA/${encodeURIComponent("Ëlsa")}/calc`);
  await expect(
    page.getByRole("group", { name: "Normal honing" }),
  ).toBeVisible();
  await hf.wait_for_optimizer(page);
  await persist(page);
  await page.reload();
  await expect(
    page.getByRole("group", { name: "Normal honing" }),
  ).toBeVisible();
  expect(await bh.roster_names(page)).toEqual([["Ëlsa"]]);
});

test("unmatched characters open the pairing popup; Confirm renames and adds the rest", async ({
  page,
}) => {
  await setup(
    page,
    [{ names: ["Solo"] }, { names: ["Alpha", "Typo"] }],
    [na("Alpha", "Beta", "Gamma")],
  );
  await sync_button(page).click();
  const dialog = page.getByRole("dialog", { name: "Match your characters" });
  await expect(dialog).toContainText("Roster 2");
  await dialog
    .getByRole("combobox", { name: "Match Typo" })
    .selectOption("Beta");
  await dialog.getByRole("button", { name: "Confirm" }).click();
  await expect(dialog).toHaveCount(0);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Solo"], ["Alpha", "Beta", "Gamma"]]);
});

test("Skip and closing the popup add nothing", async ({ page }) => {
  await setup(page, [{ names: ["Alpha", "Typo"] }], [na("Alpha", "Beta")]);
  const dialog = page.getByRole("dialog", { name: "Match your characters" });
  await sync_button(page).click();
  await dialog.getByRole("button", { name: "Skip" }).click();
  await expect(sync_button(page)).toBeEnabled();
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Typo"]]);

  await sync_button(page).click();
  await expect(dialog).toBeVisible();
  await page.mouse.click(3, 3);
  await expect(dialog).toHaveCount(0);
  await expect(sync_button(page)).toBeEnabled();
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Typo"]]);
});

// a save broken by the old sync: the accented character was renamed to the plain name plus a suffix
test("accented and plain names: the suffixed leftover is paired back", async ({
  page,
}) => {
  await setup(
    page,
    [{ names: ["Inngrimschblade1", "Inngrimschblade"], fetched: true }],
    [na("Ínngrimschblade", "Inngrimschblade")],
    { Ínngrimschblade: bh.loadout_fixture("t4") },
  );
  await sync_button(page).click();
  const dialog = page.getByRole("dialog", { name: "Match your characters" });
  await dialog
    .getByRole("combobox", { name: "Match Inngrimschblade1" })
    .selectOption("Ínngrimschblade");
  await dialog.getByRole("button", { name: "Confirm" }).click();
  await expect(dialog).toHaveCount(0);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Ínngrimschblade", "Inngrimschblade"]]);
  await expect
    .poll(() =>
      bible
        .hits("/api/oauth/loadouts/")
        .map((r) => decodeURIComponent(r.path.split("/").pop().split("?")[0]))
        .sort(),
    )
    .toEqual(["Inngrimschblade", "Ínngrimschblade"]);
});

test("a second sync changes nothing, a new bible character adds one row", async ({
  page,
}) => {
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha", "Beta")]);
  await sync(page);
  await expect.poll(() => bh.roster_names(page)).toEqual([["Alpha", "Beta"]]);
  await sync(page);
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Beta"]]);
  bible.rosters = [na("Alpha", "Beta", "Gamma")];
  await sync(page);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Alpha", "Beta", "Gamma"]]);
  await sync(page);
  expect(await bh.roster_names(page)).toEqual([["Alpha", "Beta", "Gamma"]]);
});

test("NA and CE rosters: the missing one becomes a new EU roster", async ({
  page,
}) => {
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha"), ce("Cara", "Dan")]);
  await sync(page);
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Alpha"], ["Cara", "Dan"]]);
  expect(await bh.roster_regions(page)).toEqual(["nae", "euc"]);
  await expect
    .poll(() =>
      bible
        .hits("/api/oauth/loadouts/Cara")
        .map((r) => new URL(`https://x${r.path}`).searchParams.get("region")),
    )
    .toEqual(["CE"]);
  const na_hit = bible.hits("/api/oauth/loadouts/Alpha")[0];
  expect(na_hit.path).toContain("region=NA");
  expect(na_hit.auth).toBe("Bearer uwo_t");
});

test("a Custom region roster is left alone", async ({ page }) => {
  await setup(page, [{ region: "Custom", names: ["Elsa"] }], [na("Ëlsa")]);
  await sync(page);
  await expect.poll(() => bh.roster_names(page)).toEqual([["Elsa"], ["Ëlsa"]]);
  expect(await bh.roster_regions(page)).toEqual(["Custom", "nae"]);
});

test("linked characters are locked and signing out unlocks them", async ({
  page,
}) => {
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha")]);
  await sync(page);
  const roster = page.getByRole("group", { name: "Roster 1" });
  await expect(
    roster.getByText("Linked roster on lostark.bible"),
  ).toBeVisible();
  await expect(
    roster.getByRole("textbox", { name: "Char name" }),
  ).toHaveAttribute("readonly", "");
  await expect(roster.getByRole("combobox", { name: "Region" })).toBeDisabled();

  await page.goto("/chars/NA/Alpha/calc");
  await expect(
    page.getByRole("textbox", { name: "Char name" }),
  ).toHaveAttribute("readonly", "");
  await expect(page.getByRole("combobox", { name: "Region" })).toBeDisabled();

  await sign_out_button(page).click();
  await expect(
    page.getByRole("textbox", { name: "Char name" }),
  ).not.toHaveAttribute("readonly", "");
  await expect(page.getByRole("combobox", { name: "Region" })).toBeEnabled();
  await page.goto("/roster-setup");
  await expect(page.getByText("Linked roster on lostark.bible")).toHaveCount(0);
});

test("refresh button fetches the loadout and fills the grids", async ({
  page,
}) => {
  await setup(page, [{ names: ["Alpha"], fetched: true }], [na("Alpha")]);
  await page.goto("/chars/NA/Alpha/calc");
  const refresh = page.getByRole("button", {
    name: "Refresh from lostark.bible",
  });
  await expect(refresh).toBeVisible();
  // auto-refresh on launch already ran once
  await expect
    .poll(() => bible.hits("/api/oauth/loadouts/Alpha").length)
    .toBe(1);
  const before = await bh.profile(page, "Alpha");
  await page.waitForTimeout(50);
  await refresh.click();
  await expect
    .poll(() => bible.hits("/api/oauth/loadouts/Alpha").length)
    .toBe(2);
  expect(bible.hits("/api/oauth/loadouts/Alpha")[1].auth).toBe("Bearer uwo_t");
  await expect
    .poll(async () => (await bh.profile(page, "Alpha")).fetched_at)
    .toBeGreaterThan(before.fetched_at);
  await expect(page.getByText(/Last fetched/)).toBeVisible();
  const profile = await bh.profile(page, "Alpha");
  // the Serca fixture's pieces are all Serca
  expect(profile.tier).toBe(1);
  await expect(
    hf
      .grid(page, "normal")
      .getByRole("button", { name: /: FetchedDone$/ })
      .first(),
  ).toBeVisible();
  await expect(
    hf
      .grid(page, "normal")
      .getByRole("button", { name: /^Helmet .*: FetchedDone$/ })
      .first(),
  ).toBeVisible();
  expect(profile.class_name).toBe("Bard");
});

test("a failing loadouts call falls back to the scrape", async ({ page }) => {
  await bh.stub_character(page, { Alpha: bh.char_html("t4", "Berserker") });
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha")]);
  bible.loadout_status = 500;
  await page.goto("/chars/NA/Alpha/calc");
  await page
    .getByRole("button", { name: "Refresh from lostark.bible" })
    .click();
  await expect
    .poll(async () => (await bh.profile(page, "Alpha")).class_name)
    .toBe("Berserker");
  expect((await bh.profile(page, "Alpha")).tier).toBe(0);
  expect(bible.hits("/api/oauth/loadouts/Alpha").length).toBeGreaterThan(0);
  bh.ignore_http(errors, 500);
});

test("class ids map to the names and icons the site shows", async ({
  page,
}) => {
  test.setTimeout(60_000);
  const classes: Record<string, string> = {
    Zero: "berserker_female",
    One: "dragon_knight",
    Two: "yinyangshi",
    Three: "devil_hunter_female",
    Four: "warlord",
  };
  const wanted: Record<string, string> = {
    Zero: "Slayer",
    One: "Guardianknight",
    Two: "Artist",
    Three: "Gunslinger",
    Four: "Gunlancer",
  };
  bible = await bh.stub_bible(page, [
    {
      region: "NA",
      characters: Object.entries(classes).map(([name, c]) => ({
        name,
        class: c,
      })),
    },
  ]);
  await bh.sign_in_as(page);
  await hf.open(page, "/roster-setup");
  await bh.seed(page, [{ names: ["Zero"] }]);
  await sync(page);
  await expect
    .poll(async () => (await bh.roster_names(page))[0].length)
    .toBe(5);
  for (const [name, class_name] of Object.entries(wanted)) {
    await expect
      .poll(async () => (await bh.profile(page, name))?.class_name)
      .toBe(class_name);
    await hf.open_char(page, name);
    await expect(page.getByTitle(class_name).first()).toBeVisible();
    await page.getByRole("link", { name: "Roster setup" }).click();
  }
});

test("a 401 signs out without a reload; an expired token never calls the API", async ({
  page,
}) => {
  bible = await bh.stub_bible(page, [na("Alpha")]);
  await bh.sign_in_as(page);
  bible.roster_status = 401;
  await hf.open(page, "/roster-setup");
  await expect(sign_in_button(page)).toBeVisible();
  expect(await bh.bible_auth(page)).toBeNull();
  bh.ignore_http(errors, 401);

  const fresh = await page.context().newPage();
  await fresh.addInitScript(() =>
    localStorage.setItem(
      "bible_auth",
      JSON.stringify({
        token: "uwo_t",
        expires_at: Date.now() - 1000,
        name: "T",
      }),
    ),
  );
  const other = await bh.stub_bible(fresh, [na("Alpha")]);
  await hf.stub_market(fresh);
  await hf.open(fresh, "/roster-setup");
  await expect(sign_in_button(fresh)).toBeVisible();
  expect(other.hits("/api/oauth")).toHaveLength(0);
});

test("sign out revokes the token and stays signed out after a reload", async ({
  page,
}) => {
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha")]);
  await sign_out_button(page).click();
  await expect(sign_in_button(page)).toBeVisible();
  await expect.poll(() => bible.hits("/oauth/revoke").length).toBe(1);
  expect(bible.hits("/oauth/revoke")[0].body).toContain("token=uwo_t");
  expect(await bh.bible_auth(page)).toBeNull();
  await persist(page);
  await page.reload();
  await expect(sign_in_button(page)).toBeVisible();
  expect(await bh.bible_auth(page)).toBeNull();
});

test("auto-refresh runs once per launch, not per route", async ({ page }) => {
  bible = await bh.stub_bible(page, [na("Alpha")]);
  await bh.sign_in_as(page);
  await hf.open(page, "/roster-setup");
  await bh.seed(page, [{ names: ["Alpha"], fetched: true }]);
  await persist(page);
  bible.requests.length = 0;
  await page.reload();
  await expect.poll(() => bible.hits("/api/oauth/loadouts/").length).toBe(1);
  expect(bible.hits("/api/oauth/rosters")).toHaveLength(1);
  for (const link of ["Market & Mats", "Roster setup", "Info"])
    await page.getByRole("link", { name: link }).first().click();
  await page.getByRole("link", { name: "Characters", exact: true }).click();
  await page.waitForTimeout(500);
  expect(bible.hits("/api/oauth/loadouts/")).toHaveLength(1);
  expect(bible.hits("/api/oauth/rosters")).toHaveLength(1);
});

test("the sign-in hint shows only when signed out", async ({ page }) => {
  bible = await bh.stub_bible(page, [na("Alpha")]);
  const hint = page.getByRole("button", {
    name: "Sign in with lostark.bible to fetch automatically",
  });
  await hf.open(page, "/roster-setup");
  await expect(hint).toBeVisible();

  const signed = await page.context().newPage();
  await bh.stub_bible(signed, [na("Alpha")]);
  await hf.stub_market(signed);
  await bh.sign_in_as(signed);
  await hf.open(signed, "/roster-setup");
  await expect(sign_out_button(signed)).toBeVisible();
  await expect(
    signed.getByRole("button", {
      name: "Sign in with lostark.bible to fetch automatically",
    }),
  ).toHaveCount(0);
});

test("renaming to a linked name refreshes without a popup and locks the input", async ({
  page,
}) => {
  await setup(page, [{ names: ["Zed", "Alpha"] }], [na("Alpha", "Beta")]);
  const input = page.getByRole("textbox", { name: "Char name" }).first();
  await input.fill("Beta");
  await input.blur();
  await expect(input).toHaveAttribute("readonly", "");
  await expect
    .poll(() => bible.hits("/api/oauth/loadouts/Beta").length)
    .toBe(1);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect
    .poll(async () => (await bh.profile(page, "Beta")).fetched_at)
    .toBeGreaterThan(0);
});

test("Fetch remaining uses the linked roster, not the scrape", async ({
  page,
}) => {
  const scrape = await bh.stub_character(page, {});
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha", "Beta", "Gamma")]);
  await page
    .getByRole("button", { name: "Fetch other characters in the roster" })
    .click();
  await page.getByRole("button", { name: "Confirm all" }).click();
  await expect
    .poll(() => bh.roster_names(page))
    .toEqual([["Alpha", "Beta", "Gamma"]]);
  expect(scrape.requests).toEqual([]);
  expect(bible.hits("/api/oauth/loadouts/Beta")).toHaveLength(1);
  expect(bible.hits("/api/oauth/loadouts/Gamma")).toHaveLength(1);
});

// the same name in NA and CE: both keep the exact name, the region in the route tells them apart
test("the same name in NA and CE keeps its name and opens at its own url", async ({
  page,
}) => {
  bible = await bh.stub_bible(page, [na("Alpha", "Bob"), ce("Bob", "Cara")]);
  await hf.open(page, "/chars/NA/Newchar/calc");
  await sign_in_button(page).click();
  await expect.poll(async () => (await bh.roster_names(page)).length).toBe(2);
  await expect(sync_button(page).or(sign_out_button(page))).toBeVisible();
  expect(await bh.roster_names(page)).toEqual([
    ["Alpha", "Bob"],
    ["Bob", "Cara"],
  ]);
  await page.waitForTimeout(1500);

  await page.getByRole("link", { name: "Roster setup" }).click();
  await hf.open_char(page.getByRole("group", { name: "Roster 2" }), "Bob");
  await expect(page).toHaveURL(/\/chars\/EU\/Bob\/calc$/);
  expect(await bh.with_store(page, (s) => s.active_region)).toBe("euc");
  // both are linked, so both are locked
  await expect(page.getByLabel("Char name").first()).toHaveAttribute(
    "readonly",
    "",
  );
  await hf.reload(page);
  expect(await bh.with_store(page, (s) => s.active_region)).toBe("euc");
  await page.getByRole("link", { name: "Roster setup" }).click();
  await hf.open_char(page.getByRole("group", { name: "Roster 1" }), "Bob");
  await expect(page).toHaveURL(/\/chars\/NA\/Bob\/calc$/);
  expect(await bh.with_store(page, (s) => s.active_region)).toBe("nae");
});

test("sync twice with duplicate names adds no rows", async ({ page }) => {
  await setup(page, [{ names: ["Alpha"] }], [na("Alpha", "Bob"), ce("Bob")]);
  await sync(page);
  await expect
    .poll(async () => (await bh.roster_names(page)).flat().length)
    .toBe(3);
  await sync(page);
  expect((await bh.roster_names(page)).flat()).toHaveLength(3);
});
