import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
vi.mock("@/WasmInterface/PayloadBuilder", async (importOriginal) => {
  const actual: any = await importOriginal();
  return { ...actual, build_payload: vi.fn(actual.build_payload) };
});
import "./helpers";
import { mount_app } from "../mount";
import OptimizerControlPanel from "@/Components/Character/OptimizerControlPanel/OptimizerControlPanel.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import { build_payload } from "@/WasmInterface/PayloadBuilder";
import { AdvOverride, NormalOverride } from "@/WasmInterface/Overrides";

async function setup() {
  const { wrapper } = await mount_app(OptimizerControlPanel);
  vi.mocked(build_payload).mockClear();
  return {
    wrapper,
    profile: useRosterStore().active_profile,
    runtime: useRuntimeStore(),
  };
}
const button = (wrapper, text: string) =>
  wrapper.findAll("button").find((b) => b.text() === text)!;
const radio = (wrapper, column: number, text: string) =>
  wrapper
    .findAll(".mats-row")[1]
    .element.children[column].querySelectorAll("label")
    .values()
    .find((l: Element) => l.textContent!.trim() === text)!
    .querySelector("input") as HTMLInputElement;

describe("OptimizerControlPanel", () => {
  it("shows progress and the auto-start warning", async () => {
    const { wrapper, profile, runtime } = await setup();
    runtime.optimizer.est_progress_percentage = 37.5;
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).toContain("37.50%");
    expect(wrapper.find(".progress-fill").attributes("style")).toContain(
      "width: 37.5%",
    );
    expect(wrapper.text()).not.toContain("Optimizer will not re-run");
    await wrapper.find('input[type="checkbox"]').setValue(false);
    expect(profile.auto_start_optimizer).toBe(false);
    expect(wrapper.text()).toContain("Optimizer will not re-run");
  });

  it("the compare button shows the overrides and resets them each toggle", async () => {
    const { wrapper, profile } = await setup();
    expect(wrapper.text()).not.toContain("Juice Usage");
    profile.optimizer_override.normal.juice = NormalOverride.Full;
    profile.optimizer_override.special.optimizer = false;
    profile.optimizer_override.special.weapon_first = false;
    await button(wrapper, "Compare with simple strategies").trigger("click");
    expect(wrapper.text()).toContain("Juice Usage");
    expect(profile.optimizer_override.normal.juice).toBe(
      NormalOverride.Optimizer,
    );
    expect(profile.optimizer_override.special).toEqual({
      optimizer: true,
      weapon_first: true,
      highest_first: true,
    });
    expect(build_payload).toHaveBeenCalledWith(profile.optimizer_override);
    await button(wrapper, "Compare with simple strategies").trigger("click");
    expect(wrapper.text()).not.toContain("Juice Usage");
  });

  it("radio clicks set the override and rerun the histogram with it", async () => {
    const { wrapper, profile, runtime } = await setup();
    const hist = vi.spyOn(runtime.histogram, "throttled_start");
    await button(wrapper, "Compare with simple strategies").trigger("click");
    vi.mocked(build_payload).mockClear();
    radio(wrapper, 0, "No juice").click();
    radio(wrapper, 1, "Full Book").click();
    radio(wrapper, 3, "Juice on Grace").click();
    radio(wrapper, 4, "No Scroll").click();
    await wrapper.vm.$nextTick();
    expect(profile.optimizer_override.normal).toEqual({
      juice: NormalOverride.Empty,
      book: NormalOverride.Full,
    });
    expect(profile.optimizer_override.advanced).toEqual({
      juice: AdvOverride.Grace,
      scroll: AdvOverride.Empty,
    });
    expect(build_payload).toHaveBeenCalledTimes(4);
    expect(vi.mocked(build_payload).mock.calls.at(-1)![0]).toBe(
      profile.optimizer_override,
    );
    expect(hist).toHaveBeenCalled();
    expect(radio(wrapper, 0, "No juice").checked).toBe(true);
    expect(radio(wrapper, 0, "Original").checked).toBe(false);
  });

  it("special order radios only show once Original is unticked", async () => {
    const { wrapper, profile } = await setup();
    await button(wrapper, "Compare with simple strategies").trigger("click");
    const special = () =>
      wrapper
        .findAll(".mats-row")[1]
        .element.children[2].querySelectorAll("label");
    expect(special()).toHaveLength(1);
    await wrapper.findAll('input[type="checkbox"]')[2].setValue(false);
    expect(profile.optimizer_override.special.optimizer).toBe(false);
    expect(special()).toHaveLength(5);
    radio(wrapper, 2, "Armor First").click();
    radio(wrapper, 2, "Lowest first").click();
    await wrapper.vm.$nextTick();
    expect(profile.optimizer_override.special).toEqual({
      optimizer: false,
      weapon_first: false,
      highest_first: false,
    });
    expect(radio(wrapper, 2, "Weapon First").checked).toBe(false);
  });

  it("is greyed out while the optimizer runs", async () => {
    const { wrapper, runtime } = await setup();
    runtime.optimizer.status = "busy";
    await wrapper.vm.$nextTick();
    expect(
      button(wrapper, "Compare with simple strategies").attributes("style"),
    ).toContain("pointer-events: none");
    expect(wrapper.find(".outer-grid").attributes("style")).toContain(
      "opacity: 0.5",
    );
  });
});
