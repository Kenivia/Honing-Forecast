import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/Components/Character/CharWorkerUtils", () => ({
  grid_change_callback: vi.fn(),
  start_all_workers: vi.fn(),
  start_eval_hist: vi.fn(),
}));
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import ManualArtisanInput from "@/Components/Character/Instructions/Details/ManualArtisanInput.vue";
import AdvancedDetails from "@/Components/Character/Instructions/Details/AdvancedDetails.vue";
import {
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import { useRosterStore } from "@/Stores/RosterConfig";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { add_keyed, fake_adv, fake_upgrade, key_of } from "./fake";

beforeEach(() => vi.clearAllMocks());

// setValue fires both input and change
const set = (input, value) => input.setValue(value);
const button = (w, text: string) =>
  w.findAll("button").find((b) => b.text() === text);

describe("ManualArtisanInput", () => {
  function manual(props: any = {}) {
    const pinia = createPinia();
    setActivePinia(pinia);
    const upgrade = fake_upgrade();
    add_keyed([upgrade], { expanded: true });
    const w = mount(ManualArtisanInput, {
      props: {
        starting_artisan: "0.00",
        current_chance_percentage: "25.00",
        upgrade,
        optimizer_working: false,
        taps_since_last_input: 0,
        normal_dist_length: 6,
        using_slider: true,
        show_hints: true,
        perform_order: 1,
        show_hide: true,
        ...props,
      },
      global: { plugins: [pinia] },
    });
    const keyed = () =>
      useRosterStore().active_profile.keyed_upgrades[key_of(upgrade)];
    return { w, keyed };
  }

  it("typing updates the models and a change emits", async () => {
    const { w } = manual();
    const [artisan, chance] = w.findAll("input");
    await set(artisan, "12,5");
    expect(w.emitted("update:starting_artisan")[0]).toEqual(["12,5"]);
    expect(w.emitted("manual_artisan_change")).toHaveLength(1);
    await set(chance, "30");
    expect(w.emitted("update:current_chance_percentage")[0]).toEqual(["30"]);
    expect(w.emitted("manual_chance_change")).toHaveLength(1);
  });

  it("Confirm emits, and is disabled once every tap up to the pity is in", async () => {
    const { w } = manual();
    await button(w, "Confirm").trigger("click");
    expect(w.emitted("confirm")).toHaveLength(1);
    const full = manual({ taps_since_last_input: 5 }).w;
    expect(button(full, "Confirm").attributes("disabled")).toBeDefined();
  });

  it("reset reads Restore default off the slider, Reset with progress", async () => {
    const off = manual({ using_slider: false }).w;
    await button(off, "Restore default").trigger("click");
    expect(off.emitted("reset")).toHaveLength(1);
    const progress = manual({
      upgrade: fake_upgrade({ starting_num_taps: 1 }),
    }).w;
    expect(button(progress, "Reset this upgrade")).toBeDefined();
  });

  it("with nothing to reset, Hide collapses a later upgrade only", async () => {
    const { w, keyed } = manual();
    await button(w, "Hide").trigger("click");
    expect(keyed().expanded).toBe(false);
    expect(button(manual({ perform_order: 0 }).w, "Hide")).toBeUndefined();
    expect(button(manual({ show_hide: false }).w, "Hide")).toBeUndefined();
  });
});

describe("AdvancedDetails", () => {
  function adv(config: any = {}, upgrade_index = 1) {
    const pinia = createPinia();
    setActivePinia(pinia);
    const upgrade = fake_adv({ upgrade_index });
    Object.assign(upgrade.adv_config, config);
    add_keyed([upgrade]);
    const w = mount(AdvancedDetails, {
      props: { upgrade },
      global: { plugins: [pinia] },
    });
    const store = useRosterStore();
    const progress = () =>
      store.active_profile.keyed_upgrades[key_of(upgrade)].adv_progress;
    const nums = () => w.findAll("input[type=number]");
    return { w, store, upgrade, progress, nums };
  }
  const values = (inputs) =>
    inputs.map((i) => (i.element as HTMLInputElement).value);

  it("splits Rust's start_xp into level and bars", () => {
    const { nums } = adv({ start_xp: 23, start_balls: 4 });
    expect(values(nums())).toEqual(["12", "30", "4"]);
  });

  it("writes progress back, clamped to the block, xp floored to a bar", async () => {
    const { nums, progress } = adv();
    await set(nums()[0], 15);
    await set(nums()[1], 55);
    expect(progress()).toEqual([55, 0, false, false]);
    expect(values(nums())).toEqual(["15", "50", "0"]);
    await set(nums()[0], 25);
    await set(nums()[2], 9);
    expect(progress()).toEqual([95, 6, false, false]);
    expect(start_eval_hist).toHaveBeenCalled();
  });

  it("Chisel shows at 0 grace past the first bars, Naber's Awl at 6 grace from +20", async () => {
    const { w, nums } = adv();
    expect(w.text()).not.toContain("Chisel");
    await set(nums()[1], 60);
    expect(w.text()).toContain("Chisel");
    expect(adv({ start_balls: 6 }).w.text()).not.toContain("Naber");
    expect(adv({ start_balls: 6 }, 2).w.text()).toContain("Naber");
  });

  it("follows a new result's config", async () => {
    const { w, upgrade, nums } = adv();
    upgrade.adv_config.start_xp = 41;
    await w.setProps({ upgrade: { ...upgrade } });
    expect(values(nums()).slice(0, 2)).toEqual(["14", "10"]);
  });

  it("Succeed marks done, Confirm reruns", async () => {
    const { w, store } = adv();
    await button(w, "Succeed").trigger("click");
    expect(store.active_profile.adv_grid[0][1]).toBe(UpgradeStatus.Done);
    await button(w, "Confirm").trigger("click");
    expect(start_all_workers).toHaveBeenCalled();
  });

  it("a hidden Chisel tick is not sent", async () => {
    const { w, nums, progress } = adv();
    await set(nums()[1], 60);
    await w.find("input[type=checkbox]").setValue(true);
    await set(nums()[2], 3);
    expect(progress()[2]).toBe(false);
  });
});
