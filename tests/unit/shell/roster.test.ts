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
  default: props_of("Uwuowo", ["profile", "hide_region", "region", "apply"]),
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
const labelled = (w, prefix: string) =>
  w
    .findAll("button")
    .filter((b) => b.attributes("aria-label")?.startsWith(prefix));
const drag_to = (w, roster: number, order: number[]) => {
  const grid = w.findAllComponents({ name: "VueDraggable" })[roster];
  const list = grid.props("modelValue");
  grid.vm.$emit(
    "update:modelValue",
    order.map((i) => list[i]),
  );
};
const row_names = (w) =>
  w
    .findAllComponents({ name: "Uwuowo" })
    .map((u) => u.props("profile").char_name);

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
    expect(cards.map((c) => c.attributes("aria-label"))).toEqual([
      "Roster 1",
      "Roster 2",
    ]);
    expect(
      cards.map((c) => c.find(".card-title").attributes("placeholder")),
    ).toEqual(["Roster 1", "Roster 2"]);
    expect(row_names(wrapper)).toEqual(["Alice", "Bob", "Carol"]);
  });

  it("the card's name box renames the roster, blank unnames it", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    const box = () => wrapper.findAll(".card-shell")[1].find(".card-title");
    await box().setValue("Alts");
    expect(store.roster_config.roster_names).toEqual({ 1: "Alts" });
    expect(wrapper.findAll(".card-shell")[1].attributes("aria-label")).toBe(
      "Alts",
    );
    await box().setValue(" ");
    expect(store.roster_config.roster_names).toEqual({});
    await nextTick();
    expect((box().element as HTMLInputElement).value).toBe("");
  });

  it("roster arrows move the roster up and down, disabled at either end", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
      ["Bob", 0],
    ]);
    const cards = () => wrapper.findAll(".card-shell");
    expect(cards()[0].find(".arrow-btn-up").attributes()).toHaveProperty(
      "disabled",
    );
    expect(cards()[1].find(".arrow-btn-down").attributes()).toHaveProperty(
      "disabled",
    );
    await cards()[0].find(".arrow-btn-down").trigger("click");
    expect(store.roster_ids).toEqual(["1", "0"]);
    expect(names()).toEqual(["Carol", "Alice", "Bob"]);
    await cards()[1].find(".arrow-btn-up").trigger("click");
    expect(store.roster_ids).toEqual(["0", "1"]);
  });

  it("add new character joins that roster with a fresh id and name", async () => {
    const { wrapper, store } = await mount_roster([
      ["Newchar", 0],
      ["Carol", 1],
    ]);
    await button(
      wrapper.findAll(".card-shell")[1],
      "Add new character",
    )[0].trigger("click");
    const added = store.all_profiles.at(-1);
    expect(added.char_name).toBe("");
    expect(store.char_label(added)).toBe("Newchar1");
    expect(added.roster_id).toBe("1");
    expect(new Set(store.all_profiles.map((p) => p.id)).size).toBe(3);
  });

  it("add new roster gets a fresh id with its own columns and NA", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 3],
    ]);
    await button(wrapper, "Add new roster")[0].trigger("click");
    const [alice, carol, added] = store.roster_ids;
    expect([alice, carol, added.length]).toEqual(["0", "3", 36]);
    const config = store.roster_config;
    expect(config.all_regions[added]).toBe("nae");
    expect(config.roster_mats_owned[added]).toBeDefined();
    expect(config.tradable_mats_owned[added]).toBeDefined();
    expect(config.roster_mats_owned[added]).not.toBe(
      config.roster_mats_owned[0],
    );
  });

  it("make a copy deep-copies under a new id and name", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    store.all_profiles[1].tier = 1;
    await labelled(wrapper, "Make a copy")[1].trigger("click");
    const [, carol, copy] = store.all_profiles;
    expect(copy.char_name).toBe("");
    expect(copy.roster_id).toBe("1");
    expect(copy.tier).toBe(1);
    expect(copy.id).not.toBe(carol.id);
    copy.normal_grid[0][0] = !carol.normal_grid[0][0] as any;
    expect(copy.normal_grid).not.toBe(carol.normal_grid);
    expect(copy.normal_grid[0][0]).not.toEqual(carol.normal_grid[0][0]);
  });

  it("delete is hidden with a single character", async () => {
    const { wrapper } = await mount_roster([["Alice", 0]]);
    expect(labelled(wrapper, "Delete")).toHaveLength(0);
  });

  it("deleting the last character of a roster drops its columns and region", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
    ]);
    await labelled(wrapper, "Delete")[1].trigger("click");
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
    await labelled(wrapper, "Delete")[1].trigger("click");
    expect(store.active_profile.char_name).toBe("Alice");
  });

  it("deleting an earlier character keeps the active one", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Bob", 0],
      ["Carol", 0],
    ]);
    store.switch_profile(2);
    await labelled(wrapper, "Delete")[0].trigger("click");
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
    await labelled(wrapper, "Delete")[1].trigger("click");
    expect(config.roster_chests[1]).toBeUndefined();
    expect(config.roster_income[1]).toBeUndefined();
  });

  it("a drop reorders within a roster across interleaved profiles", async () => {
    const { wrapper } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
      ["Bob", 0],
      ["Dave", 0],
    ]);
    drag_to(wrapper, 0, [2, 0, 1]);
    expect(names()).toEqual(["Dave", "Carol", "Alice", "Bob"]);
  });

  it("reordering keeps the active character", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Bob", 0],
    ]);
    drag_to(wrapper, 0, [1, 0]);
    expect(names()).toEqual(["Bob", "Alice"]);
    expect(store.active_profile.char_name).toBe("Alice");
  });

  it("delete asks first only when materials were typed", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Bob", 0],
      ["Carol", 0],
    ]);
    store.all_profiles[0].tier = 1;
    await labelled(wrapper, "Delete")[0].trigger("click");
    expect(names()).toEqual(["Bob", "Carol"]);
    store.all_profiles[0].chests.push({} as any);
    await labelled(wrapper, "Delete")[0].trigger("click");
    expect(names()).toEqual(["Bob", "Carol"]);
    const confirm = [...document.body.querySelectorAll("button")].find(
      (b) => b.textContent.trim() === "Delete",
    );
    confirm.click();
    await nextTick();
    expect(names()).toEqual(["Carol"]);
  });

  it("imports go to the right profile", async () => {
    const { wrapper, store } = await mount_roster([
      ["Alice", 0],
      ["Carol", 1],
      ["Bob", 0],
    ]);
    const uwu = wrapper.findAllComponents({ name: "Uwuowo" });
    expect(uwu[1].props("profile")).toStrictEqual(store.all_profiles[2]);
    uwu[1].props("apply")(["result"], "Bob", "Bard");
    expect(vi.mocked(apply_results).mock.calls[0]).toEqual([
      store.all_profiles[2],
      ["result"],
      true,
      "Bard",
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
    fetchers[1].props("apply")(["r"], "Dave", "Bard");
    const dave = store.all_profiles.at(-1);
    expect([dave.char_name, dave.roster_id]).toEqual(["Dave", "1"]);
    expect(vi.mocked(apply_results)).toHaveBeenCalledWith(
      dave,
      ["r"],
      true,
      "Bard",
    );
  });

  it("a roster import looks the roster up by its first valid name", async () => {
    const { wrapper } = await mount_roster([
      ["", 0],
      ["Bob1", 0],
      ["Carol", 0],
      ["", 1],
    ]);
    const fetchers = wrapper.findAllComponents({ name: "FetchRosterButton" });
    expect(fetchers.map((f) => f.props("any_char_name"))).toEqual([
      "Carol",
      "",
    ]);
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
