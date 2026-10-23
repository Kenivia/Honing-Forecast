import { afterEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import FetchRosterButton from "@/Components/Common/Uwuowo/FetchRosterButton.vue";
import FetchCharButton from "@/Components/Common/Uwuowo/FetchCharButton.vue";
import Uwuowo from "@/Components/Common/Uwuowo/Uwuowo.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { new_char_profile } from "@/Stores/CharacterProfile";
import char_t4 from "./fixtures/char_t4.html?raw";
import char_mixed from "./fixtures/char_mixed.html?raw";
import char_serca from "./fixtures/char_serca.html?raw";
import roster from "./fixtures/roster.html?raw";

// serves pages by the last path segments of the Worker URL
function serve(pages: Record<string, string | null>) {
  const fn = vi.fn((url: string) => {
    const key = Object.keys(pages).find((k) => String(url).endsWith(k));
    const html = key === undefined ? null : pages[key];
    return Promise.resolve({
      ok: html !== null,
      text: () => Promise.resolve(html ?? "Upstream error: 502"),
    });
  });
  vi.stubGlobal("fetch", fn);
  return fn;
}

// pinia with profiles named `names`, roster ids from `rosters`
function pinia_with(names: string[], rosters: number[] = []) {
  const pinia = createPinia();
  setActivePinia(pinia);
  const cfg = useRosterStore().roster_config;
  cfg.profiles = names.map((char_name, i) => ({
    ...new_char_profile(),
    char_name,
    roster_id: String(rosters[i] ?? 0),
  }));
  return pinia;
}

const opts = (pinia: any, props: any) => ({
  props,
  global: { plugins: [pinia], stubs: { teleport: true } },
});
const button = (w: any, text: string) =>
  w.findAll("button").find((b: any) => b.text() === text);

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("FetchRosterButton", () => {
  it("fetches only the missing characters, one at a time, and confirms all", async () => {
    const fn = serve({
      "/NA/Alpha/roster": roster,
      "/NA/Beta/": char_t4,
      "/NA/Gamma/": char_serca,
    });
    const apply = vi.fn();
    const w = mount(
      FetchRosterButton,
      opts(pinia_with(["Alpha"]), {
        region: "NA",
        any_char_name: "Alpha",
        apply,
      }),
    );
    await button(w, "Fetch remaining characters").trigger("click");
    await flushPromises();
    expect(
      fn.mock.calls.map((c) => String(c[0]).split("/character")[1]),
    ).toEqual(["/NA/Alpha/roster", "/NA/Beta/", "/NA/Gamma/"]);
    expect(w.text()).toContain("Are these your children?");
    expect(w.text()).toContain("Class: Bard");
    expect(w.text()).toContain("Class: Berserker");

    await button(w, "Confirm all").trigger("click");
    expect(apply.mock.calls.map((c) => [c[1], c[2], c[0].length])).toEqual([
      [false, "Beta", 7],
      [false, "Gamma", 7],
    ]);
    expect(w.text()).not.toContain("Are these your children?");
  });

  it("confirming one by one closes the popup on the last", async () => {
    serve({
      "/NA/Alpha/roster": roster,
      "/NA/Beta/": char_t4,
      "/NA/Gamma/": null,
    });
    const apply = vi.fn();
    const w = mount(
      FetchRosterButton,
      opts(pinia_with(["Alpha"]), {
        region: "NA",
        any_char_name: "Alpha",
        apply,
      }),
    );
    await button(w, "Fetch remaining characters").trigger("click");
    await flushPromises();
    expect(w.text()).toContain("Gamma failed: Fetching failed");
    // Gamma failed, so Beta is not the last panel and the popup stays open
    await button(w, "Confirm").trigger("click");
    expect(apply).toHaveBeenCalledWith(expect.any(Array), false, "Beta");
    expect(w.text()).not.toContain("Class: Bard");
    expect(w.text()).toContain("Gamma failed");
  });

  it("says when the whole roster is already here, or spread over rosters", async () => {
    serve({ "/NA/Alpha/roster": roster });
    const w = mount(
      FetchRosterButton,
      opts(pinia_with(["Alpha", "Beta", "Gamma"]), {
        region: "NA",
        any_char_name: "Alpha",
        apply: vi.fn(),
      }),
    );
    await button(w, "Fetch remaining characters").trigger("click");
    await flushPromises();
    expect(w.text()).toContain("are already here.");

    const w2 = mount(
      FetchRosterButton,
      opts(pinia_with(["Alpha", "Beta", "Gamma"], [0, 0, 5]), {
        region: "NA",
        any_char_name: "Alpha",
        apply: vi.fn(),
      }),
    );
    await button(w2, "Fetch remaining characters").trigger("click");
    await flushPromises();
    expect(w2.text()).toContain("somewhere in Honing Forecast.");
    expect(w2.text()).toContain("(Roster 2) Gamma");
  });
});

describe("FetchCharButton", () => {
  it("shows the parsed character and applies it on confirm", async () => {
    serve({ "/NA/Beta/": char_t4 });
    const apply = vi.fn();
    const w = mount(
      FetchCharButton,
      opts(pinia_with(["Beta"]), { region: "NA", char_name: "Beta", apply }),
    );
    await w.find("button").trigger("click");
    await flushPromises();
    expect(w.text()).toContain("This you?");
    expect(w.text()).toContain("Achieved ilevel: 1,712.50");
    expect(w.text()).not.toContain("(T4)");
    await button(w, "Confirm").trigger("click");
    expect(apply).toHaveBeenCalledWith(expect.any(Array), false, "Beta");
    expect(w.text()).not.toContain("This you?");
  });

  it("offers T4 or Serca for mixed gear", async () => {
    serve({ "/NA/Beta/": char_mixed });
    const apply = vi.fn();
    const w = mount(
      FetchCharButton,
      opts(pinia_with(["Beta"]), { region: "NA", char_name: "Beta", apply }),
    );
    await w.find("button").trigger("click");
    await flushPromises();
    expect(w.text()).toContain("both Serca gear and T4 gear");
    await button(w, "Confirm (T4)").trigger("click");
    expect(apply).toHaveBeenLastCalledWith(expect.any(Array), true, "Beta");
  });

  it("shows the error and a link on failure; disabled without a region", async () => {
    serve({});
    const w = mount(
      FetchCharButton,
      opts(pinia_with(["Beta"]), {
        region: "NA",
        char_name: "Beta",
        apply: vi.fn(),
      }),
    );
    await w.find("button").trigger("click");
    await flushPromises();
    expect(w.text()).toContain("Fetching failed with message");
    expect(
      w.find('a[href="https://lostark.bible/character/NA/Beta"]').exists(),
    ).toBe(true);
    expect(button(w, "Confirm")).toBeUndefined();

    const off = mount(
      FetchCharButton,
      opts(pinia_with(["Beta"]), {
        region: null,
        char_name: "Beta",
        apply: vi.fn(),
      }),
    );
    expect(off.find("button").attributes("disabled")).toBeDefined();
  });
});

describe("late results", () => {
  it("a result that lands after the name changed is dropped", async () => {
    let resolve: (v: any) => void;
    vi.stubGlobal(
      "fetch",
      vi.fn(() => new Promise((r) => (resolve = r))),
    );
    const w = mount(
      FetchCharButton,
      opts(pinia_with(["Beta"]), {
        region: "NA",
        char_name: "Beta",
        apply: vi.fn(),
      }),
    );
    await w.find("button").trigger("click");
    await w.setProps({ char_name: "Other" });
    resolve({ ok: true, text: () => Promise.resolve(char_t4) });
    await flushPromises();
    expect(w.text()).not.toContain("This you?");
  });

  it("the roster import stops when the roster's name changes", async () => {
    let release: () => void;
    const gate = new Promise<void>((r) => (release = r));
    const fn = vi.fn(async (url: string) => {
      if (String(url).endsWith("/Beta/")) await gate;
      const html = String(url).endsWith("roster") ? roster : char_t4;
      return { ok: true, text: () => Promise.resolve(html) };
    });
    vi.stubGlobal("fetch", fn);
    const w = mount(
      FetchRosterButton,
      opts(pinia_with(["Alpha"]), {
        region: "NA",
        any_char_name: "Alpha",
        apply: vi.fn(),
      }),
    );
    await button(w, "Fetch remaining characters").trigger("click");
    await flushPromises();
    await w.setProps({ any_char_name: "Zed" });
    release();
    await flushPromises();
    expect(fn).toHaveBeenCalledTimes(2); // Gamma never fetched
    expect(w.text()).not.toContain("Class: Bard");
  });
});

describe("Uwuowo", () => {
  it("a name change is formatted, passed up and triggers a fetch", async () => {
    const fn = serve({ "/NA/Beta/": char_t4 });
    const name_change = vi.fn();
    const w = mount(
      Uwuowo,
      opts(pinia_with(["Beta"]), {
        name: "Beta",
        profile_index: 0,
        name_change,
        region: "nae",
        hide_region: true,
      }),
    );
    await w.find("input").setValue("  new name!");
    expect(name_change).toHaveBeenCalledWith("Newname");
    await flushPromises();
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("does not fetch on a name change in the custom region", async () => {
    const fn = serve({});
    const w = mount(
      Uwuowo,
      opts(pinia_with(["Beta"]), {
        name: "Beta",
        profile_index: 0,
        name_change: vi.fn(),
        region: "Custom",
        hide_region: true,
      }),
    );
    await w.find("input").setValue("Other");
    await flushPromises();
    expect(fn).not.toHaveBeenCalled();
  });
});
