import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises } from "@vue/test-utils";
import { nextTick } from "vue";
import RosterView from "@/Components/RosterView.vue";
import { apply_results } from "@/Components/Common/Uwuowo/ApplyResults";
import { start_fetch } from "@/Utils/MarketDataFetcher";
import { useRosterStore } from "@/Stores/RosterConfig";
import { mount_app } from "../mount";
import { names, set_profiles, stub_match_media } from "./helpers";

const { props_of } = vi.hoisted(() => ({
  props_of: (name: string, props: string[]) => ({
    name,
    props,
    render: () => null,
  }),
}));
vi.mock("@/Components/Common/Uwuowo/Uwuowo.vue", () => ({
  default: props_of("Uwuowo", [
    "profile_index",
    "name",
    "name_change",
    "hide_region",
    "region",
    "apply",
  ]),
}));
vi.mock("@/Components/Common/Uwuowo/FetchRosterButton.vue", () => ({
  default: props_of("FetchRosterButton", ["region", "any_char_name", "apply"]),
}));
vi.mock("@/Components/Common/Uwuowo/ApplyResults", () => ({
  apply_results: vi.fn(),
}));
vi.mock("@/Utils/MarketDataFetcher", async (orig) => ({
  ...(await orig<any>()),
  start_fetch: vi.fn(),
}));

async function mount_roster(entries: [string, number][]) {
  const out = await mount_app(RosterView);
  const store = set_profiles(entries);
  await nextTick();
  return { ...out, store };
}

const button = (w, text: string) =>
  w.findAll("button").filter((b) => b.text() === text);
const rows = (w) => w.findAll(".char-row");
const row_names = (w) =>
  w.findAllComponents({ name: "Uwuowo" }).map((u) => u.props("name"));

beforeEach(() => {
  stub_match_media(false);
  vi.mocked(apply_results).mockClear();
});

describe("RosterView", () => {
  it("one card per roster, characters in profile order", async () => {
    const { wrapper } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
      ["Bob", 0],
    ]);
    const cards = wrapper.findAll(".card-shell");
    expect(cards.map((c) => c.find(".card-title").text())).toEqual([
      "Roster 1",
      "Roster 2",
    ]);
    expect(row_names(wrapper)).toEqual(["Alice", "Bob", "Carol"]);
  });

  it("add new character joins that roster with a fresh id and name", async () => {
    const { wrapper, store } = await mount_roster([
      ["Newchar", 0],
      ["Carol", 1],
    ]);
    await wrapper
      .findAll(".card-shell")[1]
      .find("button.w-full")
      .trigger("click");
    const added = store.all_profiles.at(-1);
    expect(added.char_name).toBe("Newchar1");
    expect(added.roster_id).toBe(1);
    expect(new Set(store.all_profiles.map((p) => p.id)).size).toBe(3);
  });

  it("add new roster takes the next id with its own columns and NA", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 3],
    ]);
    await button(wrapper, "Add new roster")[0].trigger("click");
    expect(store.roster_ids).toEqual([0, 3, 4]);
    const config = store.roster_config;
    expect(config.all_regions[4]).toBe("nae");
    expect(config.roster_mats_owned[4]).toBeDefined();
    expect(config.tradable_mats_owned[4]).toBeDefined();
    expect(config.roster_mats_owned[4]).not.toBe(config.roster_mats_owned[0]);
  });

  it("make a copy deep-copies under a new id and name", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    store.all_profiles[1].tier = 1;
    await button(wrapper, "Make a copy")[1].trigger("click");
    const [, carol, copy] = store.all_profiles;
    expect(copy.char_name).toBe("Newchar");
    expect(copy.roster_id).toBe(1);
    expect(copy.tier).toBe(1);
    expect(copy.id).not.toBe(carol.id);
    copy.normal_grid[0][0] = !carol.normal_grid[0][0] as any;
    expect(copy.normal_grid).not.toBe(carol.normal_grid);
    expect(copy.normal_grid[0][0]).not.toEqual(carol.normal_grid[0][0]);
  });

  it("delete is hidden with a single character", async () => {
    const { wrapper } = await mount_roster([["Alice", 0]]);
    expect(button(wrapper, "Delete")).toHaveLength(0);
  });

  it("deleting the last character of a roster drops its columns and region", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    await button(wrapper, "Delete")[1].trigger("click");
    expect(names()).toEqual(["Alice"]);
    const config = store.roster_config;
    expect(config.roster_mats_owned[1]).toBeUndefined();
    expect(config.tradable_mats_owned[1]).toBeUndefined();
    expect(config.all_regions[1]).toBeUndefined();
  });

  it("deleting the active character moves to the one before it", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Bob", 0],
      ["Carol", 0],
    ]);
    store.switch_profile(1);
    await button(wrapper, "Delete")[1].trigger("click");
    expect(store.active_profile.char_name).toBe("Alice");
  });

  it("deleting an earlier character keeps the active one", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Bob", 0],
      ["Carol", 0],
    ]);
    store.switch_profile(2);
    await button(wrapper, "Delete")[0].trigger("click");
    expect(store.active_profile.char_name).toBe("Carol");
  });

  it("deleting a roster drops its chests and income", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    const config = store.roster_config;
    config.roster_chests[1] = [];
    config.roster_income[1] = [];
    await button(wrapper, "Delete")[1].trigger("click");
    expect(config.roster_chests[1]).toBeUndefined();
    expect(config.roster_income[1]).toBeUndefined();
  });

  it("arrows swap within a roster across interleaved profiles", async () => {
    const { wrapper } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
      ["Bob", 0],
    ]);
    const first = rows(wrapper)[0];
    expect(first.find(".arrow-btn-up").exists()).toBe(false);
    expect(rows(wrapper)[1].find(".arrow-btn-down").exists()).toBe(false);
    await first.find(".arrow-btn-down").trigger("click");
    expect(names()).toEqual(["Bob", "Carol", "Alice"]);
    await rows(wrapper)[1].find(".arrow-btn-up").trigger("click");
    expect(names()).toEqual(["Alice", "Carol", "Bob"]);
  });

  it("reordering keeps the active character", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Bob", 0],
    ]);
    await rows(wrapper)[0].find(".arrow-btn-down").trigger("click");
    expect(store.active_profile.char_name).toBe("Alice");
  });

  it("renames and imports go to the right profile", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
      ["Bob", 0],
    ]);
    const uwu = wrapper.findAllComponents({ name: "Uwuowo" });
    uwu[1].props("name_change")("Robert");
    expect(names()).toEqual(["Alice", "Carol", "Robert"]);
    uwu[1].props("apply")(["result"], true);
    expect(vi.mocked(apply_results).mock.calls[0]).toEqual([
      store.all_profiles[2],
      ["result"],
      true,
      true,
    ]);
  });

  it("a roster import adds a named character to that roster", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    const fetchers = wrapper.findAllComponents({ name: "FetchRosterButton" });
    expect(fetchers.map((f) => f.props("any_char_name"))).toEqual([
      "Alice",
      "Carol",
    ]);
    fetchers[1].props("apply")(["r"], false, "Dave");
    const dave = store.all_profiles.at(-1);
    expect([dave.char_name, dave.roster_id]).toEqual(["Dave", 1]);
    expect(vi.mocked(apply_results)).toHaveBeenCalledWith(
      dave,
      ["r"],
      false,
      true,
    );
  });

  it("a region change sets the roster's region and fetches it", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    await wrapper.findAll("select.selector")[1].setValue("euc");
    await flushPromises();
    expect(store.roster_config.all_regions).toMatchObject({
      0: "nae",
      1: "euc",
    });
    expect(start_fetch).toHaveBeenCalledWith("euc");
  });

  it("custom regions get no roster import, EU gets CE", async () => {
    const { wrapper } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    const config = useRosterStore().roster_config;
    config.all_regions[0] = "Custom";
    config.all_regions[1] = "euc";
    await nextTick();
    const fetchers = wrapper.findAllComponents({ name: "FetchRosterButton" });
    expect(fetchers.map((f) => f.props("region"))).toEqual(["CE"]);
  });
});
