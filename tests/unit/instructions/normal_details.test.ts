import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/Components/Character/CharWorkerUtils", () => ({
  grid_change_callback: vi.fn(),
  start_all_workers: vi.fn(),
  start_eval_hist: vi.fn(),
}));
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import NormalDetails from "@/Components/Character/Instructions/Details/NormalDetails/NormalDetails.vue";
import NormalDetailsWrapper from "@/Components/Character/Instructions/Details/NormalDetails/NormalDetailsWrapper.vue";
import {
  grid_change_callback,
  start_all_workers,
  start_eval_hist,
} from "@/Components/Character/CharWorkerUtils";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { input_column_to_num, set_cell } from "@/Utils/InputColumn";
import { SPECIAL_LEAP_LABELS } from "@/Utils/Constants";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { deduct } from "@/Components/Character/Instructions/Details/NormalDetails/SuccessUtils";
import { add_keyed, fake_upgrade, key_of, set_results } from "./fake";

beforeEach(() => vi.clearAllMocks());

function setup(upgrade = fake_upgrade(), props: any = {}) {
  const pinia = createPinia();
  setActivePinia(pinia);
  const store = useRosterStore();
  const runtime = useRuntimeStore();
  set_cell(store.active_profile.bound_budgets, "Red", "1000");
  add_keyed([upgrade]);
  set_results([upgrade]);
  const w = mount(NormalDetails, {
    props: { upgrade, perform_order: 0, is_free_tap: false, ...props },
    global: { plugins: [pinia] },
  });
  const keyed = () => store.active_profile.keyed_upgrades[key_of(upgrade)];
  const red = () => input_column_to_num(store.active_profile.bound_budgets).Red;
  return { w, store, runtime, keyed, red, upgrade };
}
// setValue fires both input and change
const slide = async (w, taps: number) => {
  await w.find("input[type=range]").setValue(taps);
  await flushPromises();
};
const artisan_input = (w) => w.find("input.w-13");
const chance_input = (w) => w.find("input.w-10");
const button = (w, text: string) =>
  w.findAll("button").find((b) => b.text() === text);

describe("NormalDetails, slider", () => {
  it("records artisan, taps and deducts the costs", async () => {
    const { w, runtime, keyed, red } = setup();
    await slide(w, 2);
    expect(keyed()).toMatchObject({
      starting_artisan: 0.525,
      starting_num_taps: 2,
      taps_since_last_input: 2,
      expanded: true,
    });
    expect(keyed().used_materials.Red).toBe(20);
    expect(red()).toBe(980);
    expect(artisan_input(w).element.value).toBe("52.50");
    expect(chance_input(w).element.value).toBe("30.00");
    expect(start_eval_hist).toHaveBeenCalled();
    expect(runtime.is_details_update).toBe(false);
  });

  it("moving back to 0 restores the budgets", async () => {
    const { w, red, keyed } = setup();
    await slide(w, 3);
    expect(red()).toBe(970);
    await slide(w, 0);
    expect(red()).toBe(1000);
    expect(keyed().used_materials.Shards).toBe(0);
  });

  it("with auto start off, a budget edit resets the taps but the slider's own does not", async () => {
    const { w, store, keyed } = setup();
    store.active_profile.auto_start_optimizer = false;
    await slide(w, 2);
    expect(keyed().taps_since_last_input).toBe(2);
    set_cell(store.active_profile.bound_budgets, "Blue", "5");
    await flushPromises();
    expect(keyed().taps_since_last_input).toBe(0);
    expect(keyed().used_materials).toBeNull();
  });

  it("an optimizer start resets the slider", async () => {
    const { w, runtime, keyed } = setup();
    await slide(w, 2);
    runtime.optimizer.status = "busy";
    runtime.optimizer.run_counter += 1;
    await flushPromises();
    expect(keyed().taps_since_last_input).toBe(0);
    expect(runtime.budget_snapshots).toEqual({});
    expect(
      (w.find("input[type=range]").element as HTMLInputElement).value,
    ).toBe("0");
  });
});

describe("NormalDetails, manual input", () => {
  it("typed artisan is clamped, drops the slider and its deduction", async () => {
    const { w, keyed, red } = setup();
    await slide(w, 2);
    await artisan_input(w).setValue("150");
    await flushPromises();
    expect(artisan_input(w).element.value).toBe("100.00");
    expect(keyed().starting_artisan).toBe(1);
    expect(keyed().used_materials).toBeNull();
    expect(red()).toBe(1000);
    expect(w.text()).toContain("N/A");
    expect(w.text()).toContain("Restore default");
  });

  it("typed chance becomes the failed tap count, capped at double the base", async () => {
    const { w, keyed } = setup();
    await chance_input(w).setValue("40");
    expect(keyed().starting_num_taps).toBe(6);
    await chance_input(w).setValue("99");
    expect(chance_input(w).element.value).toBe("50.00");
    expect(keyed().starting_num_taps).toBe(10);
  });

  it("typed chance keeps the extra chance", async () => {
    const { w, keyed } = setup(
      fake_upgrade({ base_chance: 0.1, extra_chance: 0.05 }),
    );
    expect(chance_input(w).element.value).toBe("15.00");
    await chance_input(w).setValue("18");
    expect(chance_input(w).element.value).toBe("18.00");
    expect(keyed().starting_num_taps).toBe(3);
  });
});

describe("NormalDetails, buttons", () => {
  it("Succeed charges unlock and taps and marks done", async () => {
    const u = fake_upgrade({ unlock_costs: [7, 0, 0, 0, 0, 0, 0] });
    const { w, store, keyed, red } = setup(u);
    await slide(w, 2);
    await button(w, "Succeed").trigger("click");
    expect(red()).toBe(1000 - 7 - 20);
    expect(keyed().used_materials).toBeNull();
    expect(store.active_profile.normal_grid[0][10]).toBe(UpgradeStatus.Done);
    expect(grid_change_callback).toHaveBeenCalled();
  });

  it("Confirm reruns everything and resets the slider", async () => {
    const { w, keyed } = setup();
    await slide(w, 2);
    await button(w, "Confirm").trigger("click");
    expect(start_all_workers).toHaveBeenCalled();
    expect(keyed().taps_since_last_input).toBe(0);
  });

  it("Reset wipes progress and the state, and reruns the optimizer", async () => {
    const u = fake_upgrade({ starting_artisan: 0.3, starting_num_taps: 2 });
    const { w, keyed } = setup(u);
    keyed().starting_artisan = 0.3;
    await button(w, "Reset this upgrade").trigger("click");
    expect(keyed()).toMatchObject({
      starting_artisan: 0,
      starting_num_taps: 0,
      state: [],
    });
    expect(start_all_workers).toHaveBeenCalled();
  });

  it("a fresh upgrade has no reset; the top one has no hide either", async () => {
    const { w } = setup();
    expect(w.text()).not.toContain("Reset this upgrade");
    expect(button(w, "Hide")).toBeUndefined();
  });
});

describe("NormalDetails, free tap", () => {
  const free = (index_in_special_state = 1) => {
    const u = fake_upgrade({
      unlock_costs: [7, 0, 0, 0, 0, 0, 0],
      this_special_chance: 0.3,
    });
    const pinia = createPinia();
    setActivePinia(pinia);
    const store = useRosterStore();
    set_cell(store.active_profile.special_budget, SPECIAL_LEAP_LABELS[0], "10");
    set_cell(store.active_profile.bound_budgets, "Red", "1000");
    add_keyed([u]);
    set_results([u]);
    const w = mount(NormalDetails, {
      props: {
        upgrade: u,
        perform_order: 1,
        is_free_tap: true,
        index_in_special_state,
      },
      global: { plugins: [pinia] },
    });
    const leaps = () =>
      input_column_to_num(store.active_profile.special_budget)[
        SPECIAL_LEAP_LABELS[0]
      ];
    return { w, store, runtime: useRuntimeStore(), leaps, u };
  };

  it("shows the joint chance and defaults the remaining to the total", () => {
    const { w } = free();
    expect(w.text()).toContain(
      "30.00% chance to succeed this upgrade AND the above",
    );
    expect((w.find("input").element as HTMLInputElement).value).toBe("10");
    expect(free(0).w.text()).toContain("30.00% chance to succeed)");
  });

  it("Succeed spends the leaps used, charges the unlock and marks done", async () => {
    const { w, store, leaps } = free();
    await w.find("input").setValue(3);
    await button(w, "Succeed").trigger("click");
    expect(leaps()).toBe(3);
    expect(input_column_to_num(store.active_profile.bound_budgets).Red).toBe(
      993,
    );
    expect(store.active_profile.normal_grid[0][10]).toBe(UpgradeStatus.Done);
  });

  it("all failed spends every leap and reruns", async () => {
    const { w, leaps } = free();
    await button(w, "All free taps failed").trigger("click");
    expect(leaps()).toBe(0);
    expect(start_all_workers).toHaveBeenCalled();
  });

  it("Hide collapses, and the artisan input toggles", async () => {
    const { w, store, u } = free();
    await button(w, "Hide").trigger("click");
    expect(store.active_profile.keyed_upgrades[key_of(u)].expanded).toBe(false);
    expect(w.text()).not.toContain("Current artisan energy");
    await button(w, "Show artisan input").trigger("click");
    expect(w.text()).toContain("Current artisan energy");
  });

  it("after Succeed, a later deduction keeps the unlock deducted", async () => {
    const { w, store } = free();
    await button(w, "Succeed").trigger("click");
    const other = fake_upgrade({ piece_index: 1 });
    add_keyed([other]);
    const keyed = store.active_profile.keyed_upgrades[key_of(other)];
    deduct(keyed, { Red: 10 });
    deduct(keyed, { Red: 0 });
    expect(input_column_to_num(store.active_profile.bound_budgets).Red).toBe(
      993,
    );
  });

  it("the remaining input follows a budget change", async () => {
    const { w, store } = free();
    set_cell(store.active_profile.special_budget, SPECIAL_LEAP_LABELS[0], "20");
    await flushPromises();
    expect((w.find("input").element as HTMLInputElement).value).toBe("20");
  });

  it("an empty remaining input leaves the free taps alone", async () => {
    const { w, leaps } = free();
    await w.find("input").setValue("");
    await button(w, "Succeed").trigger("click");
    expect(leaps()).toBe(10);
  });
});

describe("NormalDetailsWrapper", () => {
  function wrap(perform_order: number, expanded = false) {
    const u = fake_upgrade();
    const pinia = createPinia();
    setActivePinia(pinia);
    add_keyed([u], { expanded });
    set_results([u]);
    const w = mount(NormalDetailsWrapper, {
      props: {
        upgrade: u,
        perform_order,
        free_tap_this_upgrade: false,
        index_in_special_state: 0,
      },
      global: { plugins: [pinia] },
    });
    return {
      w,
      keyed: () => useRosterStore().active_profile.keyed_upgrades[key_of(u)],
    };
  }

  it("the first upgrade is always open", () => {
    expect(wrap(0).w.find("input[type=range]").exists()).toBe(true);
  });

  it("later ones ask to do the above first until shown", async () => {
    const { w, keyed } = wrap(2);
    expect(w.text()).toContain("You should do the upgrades above first");
    expect(wrap(1).w.text()).toContain("the upgrade above first");
    await w.find("button").trigger("click");
    expect(keyed().expanded).toBe(true);
    expect(w.find("input[type=range]").exists()).toBe(true);
  });
});
