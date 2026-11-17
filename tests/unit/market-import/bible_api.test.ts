import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  fetch_char,
  fetch_roster,
  loadouts_to_result,
} from "@/Components/Common/Uwuowo/UwuowoUtils";
import {
  bible_auth,
  handle_callback,
  sign_out,
  signed_in,
} from "@/Utils/BibleAuth";
import serca from "./fixtures/loadouts_serca.json";
import t4 from "./fixtures/loadouts_t4.json";
import char_t4 from "./fixtures/char_t4.html?raw";

const ROSTERS = {
  rosters: [
    {
      region: "NA",
      world: "Thaemine",
      characters: [
        {
          name: "Alpha",
          class: "devil_hunter_female",
          ilvl: 1781.6666,
          lastUpdate: 1,
        },
        { name: "Beta", class: "bard", ilvl: 1730, lastUpdate: 1 },
      ],
    },
  ],
};
const json = (body: any, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });

// routes by URL; anything else is the Worker's scraped page
function serve(routes: Record<string, () => Response>) {
  const fn = vi.fn((url: string, _init?: any) => {
    const hit = Object.keys(routes).find((k) => String(url).includes(k));
    return Promise.resolve(hit ? routes[hit]() : new Response(char_t4));
  });
  vi.stubGlobal("fetch", fn);
  return fn;
}
const urls = (fn: any) => fn.mock.calls.map((c: any) => String(c[0]));
const sign_in_as = () => {
  bible_auth.value = {
    token: "uwo_x",
    expires_at: Date.now() + 1e9,
    name: "Me",
  };
};

beforeEach(() => {
  vi.stubEnv("VITE_BIBLE_CLIENT_ID", "test_client");
  serve({});
  sign_out();
});
afterEach(() => {
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
});

describe("loadouts to import result", () => {
  it("reads the newest loadout of a Serca character, vambrace included", () => {
    const out = loadouts_to_result(
      serca.loadouts,
      ROSTERS.rosters[0].characters[0],
    );
    expect(out.pieces.map((p) => p.plus_n)).toEqual([
      21, 21, 21, 21, 21, 23, 13,
    ]);
    expect(
      out.pieces.every((p) => p.tier === 1 && isNaN(p.adv) && isNaN(p.ilevel)),
    ).toBe(true);
    expect(out.class_name).toBe("Devil Hunter Female");
    expect(out.achieved_ilevel).toBe("1,781.67");
  });

  it("reads T4 pieces with advanced honing and no vambrace", () => {
    const out = loadouts_to_result(
      t4.loadouts,
      ROSTERS.rosters[0].characters[1],
    );
    expect(out.pieces.slice(0, 6)).toEqual(
      Array(6).fill({ plus_n: 20, ilevel: NaN, tier: 0, adv: 40 }),
    );
    expect(out.pieces[6].plus_n).toBe(0);
  });
});

describe("API or scraping", () => {
  it("scrapes when signed out", async () => {
    const fn = serve({});
    expect(signed_in.value).toBe(false);
    await fetch_char("NA", "Alpha");
    expect(urls(fn)).toEqual([
      expect.stringMatching(/\/character\/NA\/Alpha\/$/),
    ]);
  });

  it("uses the API for a linked character, matching names case-insensitively", async () => {
    sign_in_as();
    const fn = serve({
      "/api/oauth/rosters": () => json(ROSTERS),
      "/api/oauth/loadouts/": () => json(serca),
    });
    const out: any = await fetch_char("NA", "alpha");
    expect(out.pieces[6].plus_n).toBe(13);
    expect(urls(fn)[1]).toMatch(/\/api\/oauth\/loadouts\/Alpha\?region=NA$/);
    expect(fn.mock.calls[1][1].headers.Authorization).toBe("Bearer uwo_x");
    expect(await fetch_roster("NA", "Beta")).toEqual(["Alpha", "Beta"]);
  });

  it("scrapes characters outside the linked rosters, and on API errors", async () => {
    sign_in_as();
    let fn = serve({ "/api/oauth/rosters": () => json(ROSTERS) });
    await fetch_char("NA", "Stranger");
    await fetch_char("CE", "Alpha");
    expect(urls(fn).filter((u) => u.includes("/character/"))).toHaveLength(2);

    fn = serve({
      "/api/oauth/rosters": () => json(ROSTERS),
      "/api/oauth/loadouts/": () => json({}, 500),
    });
    expect(await fetch_char("NA", "Alpha")).toMatchObject({
      class_name: "Bard",
    });
    expect(urls(fn).at(-1)).toMatch(/\/character\/NA\/Alpha\/$/);
  });

  it("forgets the token on a 401", async () => {
    sign_in_as();
    serve({
      "/api/oauth/rosters": () => json({ error: "invalid_token" }, 401),
    });
    await fetch_char("NA", "Alpha");
    expect(bible_auth.value).toBeNull();
  });
});

describe("sign-in callback", () => {
  const pending = (state: string) =>
    sessionStorage.setItem(
      "bible_auth_pending",
      JSON.stringify({
        verifier: "v".repeat(43),
        state,
        back: "/roster-setup",
      }),
    );

  it("exchanges the code and saves the token with its expiry", async () => {
    pending("s1");
    history.replaceState(null, "", "/auth/callback?code=c1&state=s1");
    const fn = serve({
      "/oauth/token": () =>
        json({
          access_token: "uwo_new",
          token_type: "Bearer",
          expires_in: 7776000,
          scope: "identify rosters loadouts",
        }),
      "/api/oauth/user": () =>
        json({ id: 1, discordUsername: "me", discordGlobalName: "Me" }),
    });
    expect(await handle_callback()).toBe("/roster-setup");
    expect(bible_auth.value).toMatchObject({ token: "uwo_new", name: "Me" });
    expect(bible_auth.value.expires_at - Date.now()).toBeGreaterThan(
      7775000_000,
    );
    expect(String(fn.mock.calls[0][1].body)).toContain("code_verifier=");
  });

  it("rejects a callback whose state does not match", async () => {
    pending("s1");
    history.replaceState(null, "", "/auth/callback?code=c1&state=other");
    await expect(handle_callback()).rejects.toThrow();
    expect(bible_auth.value).toBeNull();
  });
});
