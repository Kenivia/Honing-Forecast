import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/WasmInterface/PayloadBuilder", async (importOriginal) => {
  const actual: any = await importOriginal();
  return { ...actual, build_payload: vi.fn(actual.build_payload) };
});
import "./helpers";
import { fake_histogram } from "./helpers";
import { mount_app } from "../mount";
import MaterialDist from "@/Components/Character/MaterialDist/MaterialDist.vue";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { new_char_profile } from "@/Stores/CharacterProfile";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { NormalOverride } from "@/WasmInterface/Overrides";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import type { ManifestChest } from "@/Utils/Chest";

async function setup(tier = 0, before?: (store: any) => void) {
  const { wrapper } = await mount_app(MaterialDist);
  const store = useRosterStore();
  const runtime = useRuntimeStore();
  store.active_profile.tier = tier;
  before?.(store);
  runtime.histogram.result = fake_histogram(tier);
  await wrapper.vm.$nextTick();
  return { wrapper, store, runtime };
}
const cells = (wrapper, label: string) =>
  wrapper
    .findAllComponents(MaterialCell)
    .filter((c) => c.props("label") === label);
const owned_cell = (wrapper, label: string) =>
  cells(wrapper, label).find((c) => c.props("show_label"))!;
const plus = (wrapper, label: string) =>
  Object.fromEntries(
    owned_cell(wrapper, label)
      .props("plus")
      .map((l) => [l.text, l.amount]),
  );
const graph = (wrapper, label: string) =>
  wrapper
    .findAllComponents(MaterialGraph)
    .find((g) => g.props("materialLabel") === label)!;
const rows = (wrapper) =>
  wrapper
    .findAll('.mats-row[role="group"]')
    .map((r) => r.attributes("aria-label"));
async function pick_band(wrapper, band: number) {
  const select = wrapper.find('select[aria-label="Chance column"]');
  await select.setValue(select.findAll("option")[band].element.value);
}
const chest = (over: Partial<ManifestChest>): ManifestChest => ({
  key: "x",
  title: "Box",
  band: 0,
  count: 1,
  options: [],
  obtain_all: false,
  ...over,
});

describe("MaterialDist columns", () => {
  it("shows nothing but N/A before a histogram arrives", async () => {
    const { wrapper } = await mount_app(MaterialDist);
    expect(rows(wrapper)).toEqual([]);
    expect(wrapper.find(".metric-result").text()).toBe("N/A");
  });

  it("reads the chance, average and gold columns off the result", async () => {
    const { wrapper } = await setup();
    const [, chance, average, gold] = cells(wrapper, "Red");
    expect(chance.props("input_column").Red).toBe(0.1);
    expect(average.props("input_column").Red).toBe(100);
    // plan 0's negative gold is a spend; positive is nothing
    expect(gold.props("input_column").Red).toBe(500);
    expect(gold.props("input_column").Blue).toBe(0);
    expect(chance.text()).toContain("10");
  });

  it("the chance dropdown picks the band, its annotation and reruns the histogram", async () => {
    const { wrapper, store, runtime } = await setup();
    expect(graph(wrapper, "Red").props("annotationLabels")).toEqual([
      "Average",
      "Bound",
    ]);
    const hist = vi.spyOn(runtime.histogram, "throttled_start");
    await pick_band(wrapper, 2);
    expect(store.active_profile.chance_band).toBe(2);
    expect(cells(wrapper, "Red")[1].props("input_column").Red).toBe(0.9);
    expect(graph(wrapper, "Red").props("annotationLabels")).toEqual([
      "Average",
      "+Tradable",
    ]);
    expect(hist).toHaveBeenCalledOnce();
  });

  it("totals: gross spend, and the leftover credit only when a band is credited", async () => {
    const { wrapper, store, runtime } = await setup();
    runtime.histogram.result = fake_histogram(0, {
      metric_per_plan: [-1000, -700],
    });
    await wrapper.vm.$nextTick();
    const metrics = () =>
      wrapper.findAll(".metric-result").map((m) => m.text());
    expect(metrics()).toEqual(["1,000", "300"]);
    store.active_profile.band_values = ["Worthless", "Worthless", "Worthless"];
    await wrapper.vm.$nextTick();
    expect(metrics()).toEqual(["1,000"]);
  });

  it("hides juice rows unless wanted, used or show-all", async () => {
    const { wrapper, store, runtime } = await setup();
    expect(rows(wrapper)).toContain("Lava's Breath");
    expect(rows(wrapper)).not.toContain("11-14 Armor");
    store.active_profile.normal_grid[0][11] = UpgradeStatus.Want; // Helmet +12
    await wrapper.vm.$nextTick();
    expect(rows(wrapper)).toContain("11-14 Armor");
    expect(rows(wrapper)).not.toContain("11-14 Weapon");
    store.active_profile.normal_grid[0][11] = UpgradeStatus.NotYet;
    runtime.histogram.result = fake_histogram(0, {
      avg_used: { ...fake_histogram(0).avg_used, "Scroll 1 Weapon": 2 },
    });
    await wrapper.vm.$nextTick();
    expect(rows(wrapper)).toContain("Scroll 1 Weapon");
    expect(rows(wrapper)).not.toContain("11-14 Armor");
    store.roster_config.show_all_rows = true;
    await wrapper.vm.$nextTick();
    expect(rows(wrapper)).toHaveLength(25);
  });

  it("pads or cuts the free-tap chances to the normal upgrades", async () => {
    const { wrapper, runtime } = await setup();
    const normal = { is_normal_honing: true };
    runtime.histogram.result = fake_histogram(0, {
      state_bundle: {
        chest_states: new Map(),
        latest_special_probs: [0.5, 0.25],
        upgrade_arr: [normal, normal, { is_normal_honing: false }, normal],
      },
    });
    await wrapper.vm.$nextTick();
    expect(graph(wrapper, "Special").props("data")).toEqual([
      [0, 0.5],
      [1, 0.25],
      [2, 0],
    ]);
  });
});

describe("MaterialDist annotations", () => {
  it("draws no owned line for a material the user does not trade", async () => {
    const { wrapper, store } = await setup();
    // Shards is untraded by default
    expect(graph(wrapper, "Shards").props("annotations")).toEqual([0, null]);
    store.active_profile.bound_budgets.enabled.Shards = true;
    await wrapper.vm.$nextTick();
    expect(graph(wrapper, "Shards").props("annotations")).toEqual([0, 0]);
  });
});

describe("MaterialDist chest gains", () => {
  it("adds what the chests give, opened as the histogram's states say", async () => {
    const { wrapper, store, runtime } = await setup(0, (store) => {
      store.active_profile.chests.push(
        chest({ count: 3, options: [{ Red: 10 }, { Blue: 5 }] }),
      );
      store.roster_config.roster_chests[0] = [
        chest({
          key: "y",
          band: 2,
          count: 2,
          options: [{ Red: 7 }],
          obtain_all: true,
        }),
        chest({
          key: "z",
          band: 1,
          count: 2,
          options: [{ "Special Leap": 3 }],
        }),
      ];
    });
    const choice = store.processed_chests.find((c) => c.options.length > 1)!;
    // options are sorted: Blue first
    expect(choice.options).toEqual([{ Blue: 5 }, { Red: 10 }]);
    runtime.histogram.result = fake_histogram(0, {
      state_bundle: {
        ...fake_histogram(0).state_bundle,
        chest_states: new Map([[choice.key, [1, 2]]]),
      },
    });
    await wrapper.vm.$nextTick();
    expect(plus(wrapper, "Red")).toEqual({ "from chests": 20 });
    expect(plus(wrapper, "Blue")).toEqual({ "from chests": 5 });
    expect(graph(wrapper, "Red").props("annotations")).toEqual([100, 20]);
    // the special leap row sums every band
    const special = cells(wrapper, "Special Leap")[0];
    expect(special.props("plus")[0].amount).toBe(6);
    await pick_band(wrapper, 2);
    expect(plus(wrapper, "Red")).toEqual({
      "from chests": 14,
      "from roster-bound": 20,
    });
    expect(graph(wrapper, "Red").props("annotations")).toEqual([100, 34]);
  });

  it("puts a stack on its first option without a fitting state", async () => {
    const { wrapper } = await setup(0, (store) =>
      store.active_profile.chests.push(
        chest({ count: 3, options: [{ Red: 10 }, { Blue: 5 }] }),
      ),
    );
    expect(plus(wrapper, "Blue")).toEqual({ "from chests": 15 });
    expect(plus(wrapper, "Red")).toEqual({ "from chests": 0 });
  });
});

describe("MaterialDist +c from T4", () => {
  const stock = (store) => {
    store.active_profile.bound_budgets.values.Red = "12";
    store.active_profile.bound_budgets.values["Serca Red"] = "1";
    store.active_roster_mats_owned.values.Red = "3";
    store.active_tradable_mats_owned.values.Red = "4";
    store.active_profile.special_budget.values["Special Leap"] = "11";
  };

  it("shows per band what a Serca character's T4 adds", async () => {
    const { wrapper } = await setup(1, stock);
    expect(plus(wrapper, "Serca Red")).toEqual({
      "from chests": 0,
      "from T4": 2,
    });
    // a material with no T4 stock has no T4 line
    expect(plus(wrapper, "Serca Blue")).toEqual({ "from chests": 0 });
    expect(cells(wrapper, "Serca Special Leap")[0].props("plus")).toEqual([
      { amount: 0, text: "from chests" },
      { amount: 2, text: "from T4" },
    ]);
    await pick_band(wrapper, 1);
    // roster-bound and tradable T4 pool into roster bound
    expect(plus(wrapper, "Serca Red")).toEqual({
      "from chests": 0,
      "from T4": 1,
      "from char-bound": 3,
    });
    await pick_band(wrapper, 2);
    expect(plus(wrapper, "Serca Red")).toEqual({
      "from chests": 0,
      "from roster-bound": 4,
    });
    expect(graph(wrapper, "Serca Red").props("annotations")).toEqual([100, 4]);
  });

  it("adds nothing from a disabled T4 material, nor for a T4 character", async () => {
    const { wrapper, store } = await setup(1, stock);
    store.active_profile.bound_budgets.enabled.Red = false;
    await wrapper.vm.$nextTick();
    expect(plus(wrapper, "Serca Red")).toEqual({ "from chests": 0 });
    const t4 = await setup(0, stock);
    expect(plus(t4.wrapper, "Red")).toEqual({ "from chests": 0 });
  });
});

describe("MaterialDist chance band", () => {
  // the page survives a character switch
  it("follows the chance band of a character switched to", async () => {
    const { wrapper, store } = await setup();
    store.add_profile({ ...new_char_profile(), chance_band: 2 });
    store.switch_profile(1);
    await wrapper.vm.$nextTick();
    const select = wrapper.find('select[aria-label="Chance column"]');
    expect((select.element as HTMLSelectElement).selectedIndex).toBe(2);
    expect(graph(wrapper, "Red").props("annotationLabels")).toEqual([
      "Average",
      "+Tradable",
    ]);
  });

  it("the band picks the owned column; the tick stays the bound one", async () => {
    const { wrapper, store } = await setup();
    await pick_band(wrapper, 2);
    const owned = cells(wrapper, "Red")[0];
    expect(owned.props("aria_name")).toBe("Tradable owned");
    expect(owned.props("input_column")).toStrictEqual(
      store.active_tradable_mats_owned,
    );
    await owned.find('input[type="checkbox"]').setValue(false);
    expect(store.active_profile.bound_budgets.enabled.Red).toBe(false);
    expect(store.active_tradable_mats_owned.enabled.Red).toBe(true);
  });

  it("a chance band change keeps the active strategy override", async () => {
    const { wrapper, store } = await setup();
    store.active_profile.optimizer_override.normal.juice = NormalOverride.Full;
    vi.mocked(build_payload).mockClear();
    await pick_band(wrapper, 1);
    expect(build_payload).toHaveBeenCalledWith(
      store.active_profile.optimizer_override,
    );
  });
});
