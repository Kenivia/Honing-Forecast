import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import "./helpers";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import ChestEditor from "@/Components/Character/ChestEditor.vue";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import type { Chest, ManifestChest } from "@/Utils/Chest";
import { ALL_LABELS, SPECIAL_LEAP_LABELS } from "@/Utils/Constants";

const own: ManifestChest = {
  key: "0|own",
  title: "Own box",
  band: 0,
  count: 3,
  options: [{ Red: 10 }, { Blue: 20 }],
  obtain_all: false,
};
const shared: ManifestChest = {
  key: "1|shared",
  title: "Shared box",
  band: 1,
  count: 2,
  options: [{ "Serca Red": 4, Gold: 100 }],
  obtain_all: true,
};

function editor(tier = 0) {
  setActivePinia(createPinia());
  const store = useRosterStore();
  store.active_profile.tier = tier;
  store.active_profile.chests.push(structuredClone(own));
  store.roster_config.roster_chests[store.active_profile.roster_id] = [structuredClone(shared)];
  const wrapper = mount(ChestEditor);
  return { wrapper, store };
}
const button = (wrapper, text: string) =>
  wrapper.findAll("button").find((b) => b.text() === text)!;
const show_obtain_all = (wrapper) =>
  wrapper
    .findAll("label")
    .find((l) => l.text().startsWith("Show obtain-all"))!
    .find("input")
    .setValue(true);
const panel = (wrapper, title: string) =>
  wrapper.find(`[role="group"][aria-label="${title}"]`);

describe("ChestEditor", () => {
  it("lists the stored chests unprocessed, split by obtain_all", async () => {
    const { wrapper } = editor(0);
    expect(panel(wrapper, "Obtain-all chests").exists()).toBe(false);
    await show_obtain_all(wrapper);
    // a Serca option shows to a T4 character too, and gold is kept
    expect(panel(wrapper, "Obtain-all chests").text()).toContain("Shared box");
    expect(
      panel(wrapper, "Obtain-all chests")
        .findAll("img")
        .map((i) => i.attributes("alt")),
    ).toEqual(["Serca Red", "Gold"]);
    expect(panel(wrapper, "Selection chests").text()).toContain("Own box");
    expect(
      wrapper.find<HTMLInputElement>('input[aria-label="Own box count"]')
        .element.disabled,
    ).toBe(false);
  });

  it("edits the count in whichever list holds the chest", async () => {
    const { wrapper, store } = editor();
    await show_obtain_all(wrapper);
    await wrapper.find('input[aria-label="Own box count"]').setValue("7");
    await wrapper.find('input[aria-label="Shared box count"]').setValue("-4");
    expect(store.active_profile.chests[0].count).toBe(7);
    expect(store.roster_config.roster_chests[store.active_profile.roster_id][0].count).toBe(0);
    // not a number: left as it was
    await wrapper.find('input[aria-label="Own box count"]').setValue("abc");
    expect(store.active_profile.chests[0].count).toBe(7);
  });

  it("removes from whichever list holds the chest", async () => {
    const { wrapper, store } = editor();
    await show_obtain_all(wrapper);
    await wrapper
      .find('button[aria-label="Delete Shared box"]')
      .trigger("click");
    expect(store.roster_config.roster_chests[store.active_profile.roster_id]).toEqual([]);
    expect(store.active_profile.chests).toHaveLength(1);
    await wrapper.find('button[aria-label="Delete Own box"]').trigger("click");
    expect(store.active_profile.chests).toEqual([]);
    // a key in neither list removes nothing
    store.roster_config.roster_chests[store.active_profile.roster_id] = [structuredClone(shared)];
    wrapper.findComponent(ChestPanels).vm.$emit("remove", "nope");
    wrapper.findComponent(ChestPanels).vm.$emit("count", "nope", {
      target: { value: "5" },
    });
    expect(store.roster_config.roster_chests[store.active_profile.roster_id]).toEqual([shared]);
  });

  it("offers the tier's materials and its special leap", async () => {
    const { wrapper } = editor(1);
    const offered = wrapper
      .find('select[aria-label="Option 1 material"]')
      .findAll("option")
      .slice(1)
      .map((o) => o.text());
    expect(offered).toEqual([...ALL_LABELS[1], SPECIAL_LEAP_LABELS[1]]);
  });

  it("adds a chest only once an option has a material and an amount", async () => {
    const { wrapper, store } = editor();
    store.roster_config.roster_chests = {};
    const add = () => button(wrapper, "Add chest");
    expect(add().element.disabled).toBe(true);
    await wrapper
      .find('select[aria-label="Option 1 material"]')
      .setValue("Red");
    expect(add().element.disabled).toBe(true);
    await wrapper.find('input[aria-label="Option 1 amount"]').setValue("5");
    await button(wrapper, "Add option").trigger("click");
    await wrapper
      .find('select[aria-label="Option 2 material"]')
      .setValue("Blue");
    await button(wrapper, "Add option").trigger("click");
    // a third, left blank, is dropped
    await wrapper
      .find('select[aria-label="Option 2 material"]')
      .setValue("Blue");
    await wrapper.find('input[aria-label="Option 2 amount"]').setValue("8");
    await wrapper.find('input[aria-label="New chest count"]').setValue("2.7");
    expect(add().element.disabled).toBe(false);
    await add().trigger("click");
    // band defaults to roster bound, whose list did not exist yet
    const added = store.roster_config.roster_chests[store.active_profile.roster_id][0];
    expect(added).toMatchObject({
      title: "Added chest",
      band: 1,
      count: 2,
      options: [{ Red: 5 }, { Blue: 8 }],
      obtain_all: false,
    });
    expect(added.key).toMatch(/^added\|[0-9a-f-]{36}$/);
    // the form starts over
    expect(wrapper.findAll('select[aria-label^="Option"]')).toHaveLength(1);
    expect(add().element.disabled).toBe(true);
  });

  it("a char-bound chest goes to the character", async () => {
    const { wrapper, store } = editor();
    await wrapper.find('select[aria-label="New chest ownership"]').setValue(0);
    await wrapper
      .find('input[aria-label="New chest name"]')
      .setValue("  Mine ");
    await wrapper.find('input[aria-label="New chest count"]').setValue("0");
    await wrapper
      .find('select[aria-label="Option 1 material"]')
      .setValue("Leaps");
    await wrapper.find('input[aria-label="Option 1 amount"]').setValue("3");
    await button(wrapper, "Add chest").trigger("click");
    expect(store.active_profile.chests.at(-1)).toMatchObject({
      title: "Mine",
      band: 0,
      count: 1,
      options: [{ Leaps: 3 }],
    });
    expect(store.roster_config.roster_chests[store.active_profile.roster_id]).toHaveLength(1);
  });
});

// the selection panel, split by the states
function state_panel(chests: Chest[], states?: [string, number[]][]) {
  return mount(ChestPanels, {
    props: {
      selection: chests,
      obtain_all: [],
      states: states ? new Map(states) : new Map(),
    },
  }).find('[role="group"][aria-label="Selection chests"]');
}
const choice: Chest = {
  key: "k",
  title: "Pick one",
  band: 1,
  count: 1500,
  options: [{ Red: 10.25 }, { Blue: 20, Leaps: 2 }],
  icons: [],
};

describe("ChestPanels opening split", () => {
  it("shows the options joined by or when no state is known", () => {
    const text = state_panel([choice]).text().replace(/\s+/g, "");
    expect(text).toContain("PickoneRosterbound:x10.3orx20x2");
    expect(text).not.toContain("as");
  });

  it("shows how many of each stack go to each option", () => {
    const text = state_panel([choice], [["k", [1000, 500]]])
      .text()
      .replace(/\s+/g, "");
    expect(text).toContain("PickoneRosterbound:1,000asx10.3,500asx20x2");
  });

  // the same rule as chest_gains, the "+n from chests" beside it
  it("shows no split for a state that does not fit", () => {
    for (const state of [
      [1, 2, 3],
      [6, 6],
    ]) {
      const text = state_panel([choice], [["k", state]]).text();
      expect(text).not.toContain(" as");
      expect(text).toContain("or");
    }
  });
});
