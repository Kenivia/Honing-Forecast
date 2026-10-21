import { afterEach, describe, expect, it, vi } from "vitest";
import {
  fetch_and_parse,
  market_to_uwuowo_region,
  parse_char,
  parse_roster,
} from "@/Components/Common/Uwuowo/UwuowoUtils";
import char_t4 from "./fixtures/char_t4.html?raw";
import char_mixed from "./fixtures/char_mixed.html?raw";
import char_serca from "./fixtures/char_serca.html?raw";
import not_found from "./fixtures/not_found.html?raw";
import missing_data from "./fixtures/missing_data.html?raw";
import roster from "./fixtures/roster.html?raw";

function serve(html: string, ok = true) {
  const fn = vi.fn((_url: string) =>
    Promise.resolve({ ok, text: () => Promise.resolve(html) }),
  );
  vi.stubGlobal("fetch", fn);
  return fn;
}
const req = (char_name = "Alpha", suffix = "") => ({
  region: "NA" as const,
  char_name,
  suffix,
});

afterEach(() => vi.unstubAllGlobals());

describe("lostark.bible scraping", () => {
  it("requests the Worker's character path", async () => {
    const fn = serve(roster);
    await fetch_and_parse(req("Alpha", "roster"), parse_roster);
    expect(String(fn.mock.calls[0][0])).toMatch(
      /\/character\/NA\/Alpha\/roster$/,
    );
  });

  it("parses a T4 character", async () => {
    serve(char_t4);
    const out = await fetch_and_parse(req(), parse_char);
    expect(out).toEqual({
      class_name: "Bard",
      achieved_ilevel: "1,712.50",
      pieces: Array(6).fill({ plus_n: 20, ilevel: 1710, tier: 0, adv: 25 }),
    });
  });

  it("ignores an equipped vambrace with no quality or item level", async () => {
    serve(
      char_t4.replace(
        "<span>Vambrace</span><span>Not Equipped</span>",
        "<div><span>Vambrace</span><span>+5</span><span>T4</span></div><div></div>",
      ),
    );
    const out: any = await fetch_and_parse(req(), parse_char);
    expect(out.pieces).toEqual(
      Array(6).fill({ plus_n: 20, ilevel: 1710, tier: 0, adv: 25 }),
    );
  });

  it("tells Serca pieces apart by a missing advanced cell and high ilevel", async () => {
    serve(char_mixed);
    const out: any = await fetch_and_parse(req(), parse_char);
    expect(out.class_name).toBe("Sorceress");
    expect(out.pieces[0]).toEqual({
      plus_n: 15,
      ilevel: 1760,
      tier: 1,
      adv: NaN,
    });
    expect(out.pieces[5]).toEqual({
      plus_n: 22,
      ilevel: 1735,
      tier: 0,
      adv: 30,
    });
    serve(char_serca);
    const serca: any = await fetch_and_parse(req(), parse_char);
    expect(serca.pieces.every((p) => p.tier === 1)).toBe(true);
  });

  it("a low-ilevel piece with no advanced cell is T4 at adv 0, and T0 reads as +25", async () => {
    serve(
      char_t4
        .replace("<span>+25</span>", "")
        .replace("<span>T4</span>", "<span>T0</span>"),
    );
    const out: any = await fetch_and_parse(req(), parse_char);
    expect(out.pieces[0]).toEqual({
      plus_n: 25,
      ilevel: 1710,
      tier: 0,
      adv: 0,
    });
  });

  it("reports not found, missing data, broken markup and HTTP errors as strings", async () => {
    serve(not_found);
    expect(await fetch_and_parse(req("Zed"), parse_char)).toBe(
      "Character: Zed not found (region: NA)",
    );
    serve(missing_data);
    expect(await fetch_and_parse(req(), parse_char)).toBe("Missing data");
    serve("<div>nothing</div>");
    expect(await fetch_and_parse(req(), parse_char)).toMatch(
      /^Parsing equipment failed/,
    );
    serve("<div>nothing</div>");
    expect(await fetch_and_parse(req(), parse_roster)).toMatch(
      /^Parsing roster failed/,
    );
    serve("Upstream error: 502", false);
    expect(await fetch_and_parse(req(), parse_char)).toBe(
      'Fetching failed with message Error: Fetch failed, "Upstream error: 502"',
    );
  });

  it("lists a roster's names", async () => {
    serve(roster);
    expect(await fetch_and_parse(req("Alpha", "roster"), parse_roster)).toEqual(
      ["Alpha", "Beta", "Gamma"],
    );
    // any member finds the same list
    serve(roster);
    expect(await fetch_and_parse(req("Gamma", "roster"), parse_roster)).toEqual(
      ["Alpha", "Beta", "Gamma"],
    );
  });

  it("maps market regions", () => {
    expect(market_to_uwuowo_region("nae")).toBe("NA");
    expect(market_to_uwuowo_region("euc")).toBe("CE");
    expect(market_to_uwuowo_region("Custom")).toBeNull();
  });
});
