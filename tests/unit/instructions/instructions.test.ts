import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/Components/Character/CharWorkerUtils", () => ({
  grid_change_callback: vi.fn(),
  start_all_workers: vi.fn(),
  start_eval_hist: vi.fn(),
}));
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import Instructions from "@/Components/Character/Instructions/Instructions.vue";
import ActualInstructions from "@/Components/Character/Instructions/ActualInstructions.vue";
import {
  get_any_overwritten,
  get_optimizer_working,
  get_relevant_result,
} from "@/Components/Character/Instructions/InstructionUtils";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { AdvOverride, NormalOverride } from "@/WasmInterface/Overrides";
import { add_keyed, fake_adv, fake_upgrade, set_results } from "./fake";

function setup() {
  const pinia = createPinia();
  setActivePinia(pinia);
  return { pinia, store: useRosterStore(), runtime: useRuntimeStore() };
}

describe("InstructionUtils", () => {
  it("any override off the optimizer counts as overwritten", () => {
    const { store } = setup();
    const o = store.active_profile.optimizer_override;
    expect(get_any_overwritten()).toBe(false);
    o.advanced.scroll = AdvOverride.Grace;
    expect(get_any_overwritten()).toBe(true);
    o.advanced.scroll = AdvOverride.Optimizer;
    o.special.optimizer = false;
    expect(get_any_overwritten()).toBe(true);
  });

  it("reads the histogram's bundle when overwritten, the optimizer's otherwise", () => {
    const { runtime } = setup();
    set_results([fake_upgrade()]);
    expect(get_relevant_result(false)).toBe(runtime.optimizer.result);
    expect(get_relevant_result(true)).toBe(
      runtime.histogram.result.state_bundle,
    );
    expect(get_optimizer_working()).toBe(false);
    runtime.optimizer.status = "busy";
    expect(get_optimizer_working()).toBe(true);
  });
});

const columns = (w) =>
  w.findAll(".w-16").map((c) =>
    c
      .findAll(":scope > div")
      .slice(-2)
      .map((d) => d.text()),
  );

describe("ActualInstructions", () => {
  function mount_actual(upgrade) {
    const { pinia } = setup();
    set_results([upgrade]);
    return mount(ActualInstructions, {
      props: { upgrade },
      global: { plugins: [pinia] },
    });
  }

  it("groups equal taps into streaks and ends on the pity", () => {
    const w = mount_actual(fake_upgrade());
    expect(columns(w)).toEqual([
      ["x2 taps", "until 52.50% artisan"],
      ["x2 taps", "until 100.00% artisan"],
      ["Pity", "occurs on the 5th tap"],
    ]);
    // the first streak juiced, the book (id 2) not
    const first = w.findAll(".w-16")[0].findAll(".can-disable-icon-wrapper");
    expect(first.map((x) => x.classes("ticked"))).toEqual([true, false]);
  });

  it("an upgrade that always succeeds shows no pity", () => {
    const u = fake_upgrade({
      base_chance: 1,
      normal_dist: [0, 1],
      state: [[]],
    });
    expect(columns(mount_actual(u))).toEqual([
      ["x1 taps", "until 100.00% artisan"],
    ]);
  });

  it("advanced: grace and non-grace streaks from the strategy indices", () => {
    // juice index 15 = [all grace, 10 non-grace], scroll index 13 = [all grace, none]
    const both = fake_adv({ state: [[15], [13]] });
    expect(columns(mount_actual(both))).toEqual([
      ["All", "Grace"],
      ["Any 10", "non-Grace"],
    ]);
    // juice index 3 = first 3 grace only
    expect(columns(mount_actual(fake_adv({ state: [[3], [0]] })))).toEqual([
      ["First 3", "Grace"],
    ]);
    expect(columns(mount_actual(fake_adv({ state: [[0], [0]] })))).toEqual([
      ["Nothing", "on Grace"],
    ]);
  });
});

describe("Instructions", () => {
  // upgrade_arr is per piece, level ascending, as the payload sends it
  const helm11 = fake_upgrade({ piece_index: 0, upgrade_index: 10 });
  const helm12 = fake_upgrade({ piece_index: 0, upgrade_index: 11 });
  const shoulder11 = fake_upgrade({ piece_index: 1, upgrade_index: 10 });
  const adv2 = fake_adv({ piece_index: 0, upgrade_index: 2 });
  const adv1 = fake_adv({ piece_index: 1, upgrade_index: 1 });
  const arr = [helm11, helm12, shoulder11, adv2, adv1];

  function mount_instructions(is_normal: boolean, keyed = arr) {
    const s = setup();
    add_keyed(keyed);
    // Rust's cleaned order: shoulder, helm +12 valid; helm +11 (below +12) and the advanced ones after
    set_results(arr, [2, 1, 0, 3, 4], 2, [0.3, 0.1, 0, 0, 0]);
    const w = mount(Instructions, {
      props: { is_normal },
      global: { plugins: [s.pinia] },
    });
    return { ...s, w };
  }
  const rows = (w) => w.findAll(".mats-row").slice(1);

  it("renders nothing until both results exist", () => {
    const { pinia } = setup();
    const w = mount(Instructions, {
      props: { is_normal: true },
      global: { plugins: [pinia] },
    });
    expect(rows(w)).toHaveLength(0);
  });

  it("orders normal upgrades so lower levels of a free-tapped piece go first", () => {
    const { w } = mount_instructions(true);
    const r = rows(w);
    expect(r.map((x) => x.find("span").text())).toEqual([
      "Shoulder +11",
      "Helmet +11",
      "Helmet +12",
    ]);
    expect(r.map((x) => x.find(".text-4xl").text())).toEqual([
      "1st",
      "2nd",
      "3rd",
    ]);
    // free taps where in the valid prefix with a chance, not below
    expect(r.map((x) => x.text().includes("Free tap this"))).toEqual([
      true,
      false,
      true,
    ]);
  });

  it("advanced instructions are sorted by level", () => {
    const { w } = mount_instructions(false);
    expect(rows(w).map((x) => x.text().match(/\+\d+ - \d+/)[0])).toEqual([
      "+10 - 20",
      "+20 - 30",
    ]);
  });

  it("skips upgrades with no keyed entry", () => {
    const { w } = mount_instructions(
      true,
      arr.filter((u) => u !== helm12),
    );
    expect(rows(w).map((x) => x.text() !== "")).toEqual([true, true, false]);
  });

  it("overwritten: histogram's order, flagged, and no progress inputs", async () => {
    const { w, store, runtime } = mount_instructions(true);
    runtime.histogram.result.state_bundle.special_state = [0, 1, 2, 3, 4];
    runtime.histogram.result.state_bundle.special_invalid_index = 0;
    store.active_profile.optimizer_override.normal.juice = NormalOverride.Full;
    await w.vm.$nextTick();
    expect(w.text()).toContain("(Not optimized)");
    const r = rows(w);
    expect(r.map((x) => x.find("span").text())).toEqual([
      "Helmet +11",
      "Helmet +12",
      "Shoulder +11",
    ]);
    expect(
      r.every((x) => x.text().includes("FOR COMPARISON PURPOSE ONLY")),
    ).toBe(true);
    expect(w.find("input[type=range]").exists()).toBe(false);
  });

  it("shows progress while the optimizer works", async () => {
    const { w, runtime } = mount_instructions(true);
    runtime.optimizer.status = "busy";
    runtime.optimizer.est_progress_percentage = 42.5;
    await w.vm.$nextTick();
    expect(w.find(".card-title").text()).toContain("(42.50%)");
  });
});
