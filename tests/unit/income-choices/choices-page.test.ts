import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount, type VueWrapper } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import { PRESET_CHOICES } from "@/Utils/Choices";
import Choices from "@/Components/Character/Choices/Choices.vue";
import type { ChoiceJob } from "@/Components/Character/Choices/ChoiceJobs";
import { queue_jobs, stop_jobs } from "@/WasmInterface/OptimizerJobs";
import { fake_out, fake_state, want_upgrade } from "./helpers";

// The Choices page: what is queued, the worth shown, raids and difficulties, adding to budgets.
vi.mock("@/WasmInterface/OptimizerJobs", async (original) => ({
  ...(await original<object>()),
  queue_jobs: vi.fn(),
  stop_jobs: vi.fn(),
}));

let store: ReturnType<typeof useRosterStore>;
let wrapper: VueWrapper;
// bound gold differs per test so the session's results never carry over
let gold = 100000;

beforeEach(() => {
  vi.mocked(queue_jobs).mockClear();
  setActivePinia(createPinia());
  store = useRosterStore();
  set_cell(store.active_profile.bound_budgets, "Gold", String(gold++));
});
afterEach(() => wrapper?.unmount());

const open = () => (wrapper = mount(Choices));
const queued = (): ChoiceJob[] => vi.mocked(queue_jobs).mock.lastCall[0];
const headers = () => wrapper.findAll('[role="rowheader"]');
const worth_cells = () => wrapper.findAll('[aria-label$=" worth"]');
const add_buttons = () =>
  wrapper.findAll("button").filter((b) => b.text() === "Add to budget");
const shown_difficulty = (group: string) =>
  wrapper
    .findAll(`[aria-label="${group} difficulty"] button`)
    .find((b) => b.attributes("aria-pressed") === "true")
    .text();

describe("Choices page", () => {
  it("without upgrades runs nothing and says so", () => {
    open();
    expect(queued()).toEqual([]);
    expect(wrapper.find('[role="alert"]').exists()).toBe(true);
    expect(worth_cells().every((c) => c.text() === "N/A")).toBe(true);
  });

  it("runs the base first, then every option shown, and shows the worth", async () => {
    want_upgrade(store.active_profile);
    open();
    expect(queued()).toHaveLength(1);
    expect(worth_cells().every((c) => c.text() === "…")).toBe(true);

    queued()[0].store(fake_out(0, 1000), fake_state());
    await wrapper.vm.$nextTick();
    const options = queued();
    expect(options).toHaveLength(worth_cells().length);
    expect(new Set(options.map((job) => job.hash)).size).toBe(options.length);

    options[0].store(fake_out(0, 1500.4), fake_state());
    options[1].store(fake_out(0, 400), fake_state());
    await wrapper.vm.$nextTick();
    expect(worth_cells()[0].text()).toBe("+500");
    expect(worth_cells()[1].text()).toBe("−600");
    // the next queue leaves out what is done
    expect(queued()).toHaveLength(options.length - 2);
  });

  it("a worth under one gold shows as 0, neither green nor red", async () => {
    want_upgrade(store.active_profile);
    open();
    queued()[0].store(fake_out(0, 1000), fake_state());
    await wrapper.vm.$nextTick();
    queued()[0].store(fake_out(0, 999.6), fake_state());
    await wrapper.vm.$nextTick();
    expect(worth_cells()[0].text()).toBe("0");
    expect(worth_cells()[0].attributes("style")).toContain("--text-main");
  });

  it("shows three raids, one more per press", async () => {
    open();
    expect(headers()).toHaveLength(3);
    expect(headers()[0].text()).toContain("Horizon Cathedral");
    const more = wrapper
      .findAll("button")
      .find((b) => b.text().startsWith("Show one more raid"));
    expect(more.text()).toContain("(7 hidden)");
    await more.trigger("click");
    expect(headers()).toHaveLength(4);
  });

  it("a difficulty picked by hand is saved and lists its gates", async () => {
    open();
    const group = "Horizon Cathedral";
    const button = wrapper
      .findAll(`[aria-label="${group} difficulty"] button`)
      .find((b) => b.text() === "Level 3");
    await button.trigger("click");
    expect(store.active_profile.choice_levels[group]).toBe("Level 3");
    expect(shown_difficulty(group)).toBe("Level 3");
    const titles = worth_cells()
      .map((c) => c.attributes("aria-label"))
      .filter((label) => label.startsWith(group));
    expect(titles).toEqual([`${group} Gate 1 worth`, `${group} Gate 2 worth`]);
  });

  it("Add to budget takes the gold and adds what it gives", async () => {
    open();
    const profile = store.active_profile;
    const group = "Horizon Cathedral";
    const choice = PRESET_CHOICES.find(
      (x) =>
        x.group === group &&
        x.difficulty === shown_difficulty(group) &&
        x.title === "Gate 1",
    );
    const before = input_column_to_num(profile.bound_budgets, true);
    await add_buttons()[0].trigger("click");
    const after = input_column_to_num(profile.bound_budgets, true);
    for (const [label, amount] of Object.entries(choice.options[1]))
      expect(after[label] - (before[label] ?? 0)).toBe(amount);
  });

  it("Add to budget is disabled when no band can pay", () => {
    set_cell(store.active_profile.bound_budgets, "Gold", "0");
    open();
    expect(
      add_buttons().every((b) => b.attributes("disabled") !== undefined),
    ).toBe(true);
  });

  it("gold in the looser bands pays too", () => {
    set_cell(store.active_profile.bound_budgets, "Gold", "0");
    set_cell(store.active_tradable_mats_owned, "Gold", "1000000");
    open();
    expect(add_buttons()[0].attributes("disabled")).toBeUndefined();
  });

  it("stops the runs on leaving", () => {
    open().unmount();
    expect(stop_jobs).toHaveBeenCalled();
    wrapper = undefined;
  });
});

describe("Paradise tab", () => {
  const tab = (name: string) =>
    wrapper.findAll('[role="tab"]').find((b) => b.text() === name);
  const button = (group: string, name: string) =>
    wrapper
      .findAll(`[aria-label="${group}"] button`)
      .find((b) => b.text() === name);
  // the 1750 Destiny base screen, every option of which gives something
  const open_paradise = async () => {
    open();
    await tab("Paradise").trigger("click");
    await button("Paradise item level", "1750").trigger("click");
  };

  it("queues only its own runs, base first, and sorts by worth", async () => {
    want_upgrade(store.active_profile);
    await open_paradise();
    expect(queued()).toHaveLength(1);
    queued()[0].store(fake_out(0, 1000), fake_state());
    await wrapper.vm.$nextTick();
    const options = queued();
    expect(options).toHaveLength(worth_cells().length);
    // a T4 character keeps only the shards, juice and gold of Serca rewards, so most read the same
    const labels = worth_cells().map((c) => c.attributes("aria-label"));
    const job = (name: string) =>
      options[labels.findIndex((x) => x.startsWith(name))];
    job("Ancient Bracelet Chest").store(fake_out(0, 1100), fake_state());
    job("5,500 Gold Bars").store(fake_out(0, 1300), fake_state());
    await wrapper.vm.$nextTick();
    const cells = worth_cells();
    expect(cells[0].attributes("aria-label")).toMatch(/^5,500 Gold Bars/);
    expect(cells[0].text()).toBe("+300");
    expect(cells[1].text()).toBe("+100");
    expect(cells.at(-1).attributes("aria-label")).toMatch(/^Lava's Breath/);
    expect(cells.at(-1).text()).toBe("…");
  });

  it("Wealth runs the Wealth options instead", async () => {
    want_upgrade(store.active_profile);
    await open_paradise();
    queued()[0].store(fake_out(0, 1000), fake_state());
    await wrapper.vm.$nextTick();
    const plain = queued().map((job) => job.hash);
    await wrapper.find('input[type="checkbox"]').setValue(true);
    const wealth = queued().map((job) => job.hash);
    expect(wealth).toHaveLength(plain.length);
    expect(wealth.filter((x) => plain.includes(x))).toEqual([]);
  });

  it("mode, item level and level are saved per character", async () => {
    await open_paradise();
    await button("Paradise mode", "Netherworld").trigger("click");
    await button("Paradise level", "Level 3").trigger("click");
    expect(store.active_profile.choice_levels).toMatchObject({
      "Paradise mode": "Netherworld",
      "Paradise level": "Level 3",
      "Paradise: Destiny": "1750",
    });
  });
});
