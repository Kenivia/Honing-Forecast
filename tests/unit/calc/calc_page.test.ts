import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/Components/Character/CharWorkerUtils", async (importOriginal) => {
  const actual: any = await importOriginal();
  return {
    ...actual,
    start_all_workers: vi.fn(),
    start_eval_hist: vi.fn(),
  };
});
import "./helpers";
import { mount_app } from "../mount";
import Calc from "@/Components/Character/Calc.vue";
import StatusInput from "@/Components/Character/StatusInput/StatusInput.vue";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import ChestEditor from "@/Components/Character/ChestEditor.vue";
import TierConvertButton from "@/Components/Common/TierConvertButton.vue";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { make_budget_snapshot } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";

const stubs = {
  StatusInput: true,
  MaterialDist: true,
  OptimizerControlPanel: true,
  Instructions: true,
};

async function calc() {
  vi.mocked(start_all_workers).mockClear();
  vi.mocked(start_eval_hist).mockClear();
  const { wrapper } = await mount_app(Calc, { global: { stubs } });
  return { wrapper, store: useRosterStore() };
}

describe("Calc page", () => {
  it("starts the workers on load and on a watched change", async () => {
    const { wrapper, store } = await calc();
    expect(start_all_workers).toHaveBeenCalledOnce();
    store.active_profile.auto_start_optimizer = false;
    await wrapper.vm.$nextTick();
    expect(start_all_workers).toHaveBeenCalledTimes(2);
    store.active_profile.band_values[2] = "Market";
    await wrapper.vm.$nextTick();
    expect(start_all_workers).toHaveBeenCalledTimes(3);
    store.active_profile.chests.push({
      key: "k",
      title: "Box",
      band: 0,
      count: 1,
      options: [{ Red: 1 }],
      obtain_all: true,
    });
    await wrapper.vm.$nextTick();
    expect(start_all_workers).toHaveBeenCalledTimes(4);
    expect(start_eval_hist).not.toHaveBeenCalled();
  });

  it("splits the processed chests by whether a choice is left", async () => {
    const { wrapper, store } = await calc();
    store.active_profile.chests.push(
      {
        key: "a",
        title: "Choice",
        band: 0,
        count: 1,
        options: [{ Red: 1 }, { Blue: 1 }],
        obtain_all: false,
      },
      {
        key: "b",
        title: "Trivial",
        band: 0,
        count: 1,
        // the second is no better than the first, so it goes
        options: [{ Red: 5 }, { Red: 2 }],
        obtain_all: false,
      },
    );
    await wrapper.vm.$nextTick();
    const panels = wrapper.findComponent(ChestPanels);
    expect(panels.props("selection").map((c) => c.title)).toEqual(["Choice"]);
    expect(panels.props("obtain_all").map((c) => c.title)).toEqual(["Trivial"]);
    expect(panels.props("states")).toBeUndefined();
  });

  it("Edit chests swaps the panels for the editor", async () => {
    const { wrapper } = await calc();
    expect(wrapper.findComponent(ChestEditor).exists()).toBe(false);
    await wrapper.find('input[type="checkbox"]').setValue(true);
    expect(wrapper.findComponent(ChestEditor).exists()).toBe(true);
    expect(wrapper.findAllComponents(ChestPanels)).toHaveLength(1);
    expect(wrapper.findComponent(ChestPanels).props("editing")).toBe(true);
  });

  it("on leaving cancels its own character's workers and settles its deductions", async () => {
    const { wrapper, store } = await calc();
    const own = store.active_profile;
    const runtime = useRuntimeStore();
    runtime.sync_profiles([own.id]);
    const bundles = runtime.for_char(own.id);
    const cancels = [
      vi.spyOn(bundles.optimizer, "cancel"),
      vi.spyOn(bundles.histogram, "cancel"),
    ];
    runtime.budget_snapshots[own.id] = make_budget_snapshot();
    // the next character is already active when this page unmounts
    store.add_char(own.roster_id);
    store.switch_profile(1);
    wrapper.unmount();
    for (const cancel of cancels) expect(cancel).toHaveBeenCalledOnce();
    expect(runtime.budget_snapshots[own.id]).toBeUndefined();
  });
});

async function status_input() {
  const { wrapper } = await mount_app(StatusInput, {
    global: { stubs: { Uwuowo: true } },
  });
  return { wrapper, store: useRosterStore() };
}

describe("StatusInput", () => {
  it("a fresh T4 character warns what converting to Serca would set", async () => {
    const { wrapper } = await status_input();
    expect(wrapper.text()).toContain("Normal Honing (T4)");
    expect(wrapper.text()).toContain("Advanced Honing");
    const button = wrapper.findComponent(TierConvertButton);
    expect(button.props("labelText")).toBe("Convert to T4.5 Serca");
    expect(button.props("warning")).toBe(true);
    expect(button.props("tooltipText")).toBe(
      "Warning! All Adv honing will be set to +40, \nAll Gear will be set to +11 (Serca)",
    );
  });

  it("a T4 character at +20 with advanced done is eligible", async () => {
    const { wrapper, store } = await status_input();
    const profile = store.active_profile;
    for (let row = 0; row < 6; row++) {
      profile.normal_grid[row].fill(UpgradeStatus.Done, 0, 20);
      profile.adv_grid[row].fill(UpgradeStatus.Done);
    }
    await wrapper.vm.$nextTick();
    const button = wrapper.findComponent(TierConvertButton);
    expect(button.props("tooltipText")).toBe(
      "Eligible for conversion to T4.5 Serca gear",
    );
    expect(button.props("warning")).toBe(false);
  });

  it("converting switches the page to Serca", async () => {
    const { wrapper, store } = await status_input();
    await wrapper.findComponent(TierConvertButton).trigger("click");
    expect(store.active_profile.tier).toBe(1);
    expect(wrapper.text()).toContain("Normal Honing (Serca)");
    expect(wrapper.text()).not.toContain("Advanced Honing");
    expect(wrapper.findComponent(TierConvertButton).props("labelText")).toBe(
      "Revert back to T4",
    );
    expect(
      store.active_profile.adv_grid
        .flat()
        .every((s) => s === UpgradeStatus.Done),
    ).toBe(true);
  });
});
