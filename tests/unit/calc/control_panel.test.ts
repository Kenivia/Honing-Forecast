import { beforeEach, describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import "./helpers";
import { mount_app } from "../mount";
import ControlPanel from "@/Components/Character/ControlPanel.vue";
import LeftoverBands, {
  bands_editing,
} from "@/Components/Character/MaterialDist/LeftoverBands.vue";
import { ALT_BANDS, MAIN_BANDS } from "@/Utils/Constants";
import { useRosterStore } from "@/Stores/RosterConfig";

async function setup() {
  const { wrapper } = await mount_app(ControlPanel);
  return { wrapper, store: useRosterStore() };
}
const button = (wrapper, text: string) =>
  wrapper.findAll("button").find((b) => b.text() === text)!;
// the band dropdowns, opened with Customise
async function setup_bands() {
  const { wrapper } = await mount_app(LeftoverBands);
  await button(wrapper, "Customise").trigger("click");
  return { wrapper, store: useRosterStore() };
}

// value -> disabled, per dropdown
const options_of = (wrapper, band: string) =>
  Object.fromEntries(
    wrapper
      .find(`select[aria-label="${band} leftover value"]`)
      .findAll("option")
      .map((o) => [o.attributes("value"), o.element.disabled]),
  );

describe("Leftover band values", () => {
  // the editor's open state is shared module-wide
  beforeEach(() => (bands_editing.value = false));

  it("a summary until Customise, then dropdowns that disable values decreasing across bands", async () => {
    const { wrapper: closed } = await mount_app(LeftoverBands);
    expect(closed.findAll("select")).toHaveLength(0);
    expect(closed.text()).toContain("Treating as");
    expect(closed.text()).not.toContain("We consider");
    const { wrapper } = await setup_bands();
    const labels = wrapper
      .findAll("select")
      .map((s) => s.attributes("aria-label"));
    expect(labels).toEqual([
      "Bound leftover value",
      "Roster leftover value",
      "Tradable leftover value",
    ]);
    // default plan: nothing, nothing, taxed sell
    expect(options_of(wrapper, "Bound")).toEqual({
      Worthless: false,
      TaxedSell: true,
      Market: true,
    });
    expect(options_of(wrapper, "Roster")).toEqual({
      Worthless: false,
      TaxedSell: false,
      Market: true,
    });
    expect(options_of(wrapper, "Tradable")).toEqual({
      Worthless: false,
      TaxedSell: false,
      Market: false,
    });
  });

  it("a change writes the band and re-derives what the others allow", async () => {
    const { wrapper, store } = await setup_bands();
    await wrapper
      .find('select[aria-label="Tradable leftover value"]')
      .setValue("Market");
    await wrapper
      .find('select[aria-label="Roster leftover value"]')
      .setValue("Market");
    expect(store.active_profile.band_values).toEqual([
      "Worthless",
      "Market",
      "Market",
    ]);
    expect(options_of(wrapper, "Tradable")).toEqual({
      Worthless: true,
      TaxedSell: true,
      Market: false,
    });
    expect(options_of(wrapper, "Bound")).toEqual({
      Worthless: false,
      TaxedSell: false,
      Market: false,
    });
  });

  it("Main and Alt set the presets; anything else reads Custom", async () => {
    const { wrapper, store } = await setup_bands();
    expect(wrapper.text()).toContain("We consider all possible outcomes");
    await button(wrapper, "Alt").trigger("click");
    expect(store.active_profile.band_values).toEqual(ALT_BANDS);
    expect(wrapper.text()).not.toContain("Custom");
    await wrapper
      .find('select[aria-label="Tradable leftover value"]')
      .setValue("Market");
    expect(wrapper.text()).toContain("Custom");
    await button(wrapper, "Main").trigger("click");
    expect(store.active_profile.band_values).toEqual(MAIN_BANDS);
    expect(wrapper.text()).not.toContain("Custom");
  });

  it("an out-of-order loaded plan leaves a band with no allowed value", async () => {
    const { wrapper, store } = await setup_bands();
    store.active_profile.band_values = ["Market", "Worthless", "Worthless"];
    await wrapper.vm.$nextTick();
    expect(Object.values(options_of(wrapper, "Roster"))).toEqual([
      true,
      true,
      true,
    ]);
  });
});

describe("ControlPanel toggles", () => {
  it("binds the checkboxes to the store", async () => {
    const { wrapper, store } = await setup();
    const boxes = wrapper.findAll('input[type="checkbox"]');
    expect(boxes).toHaveLength(4);
    const lock = store.active_profile.lock_fetched_done;
    await boxes[2].setValue(!lock);
    expect(store.active_profile.lock_fetched_done).toBe(!lock);
    const show = store.roster_config.show_all_rows;
    await boxes[0].setValue(!show);
    expect(store.roster_config.show_all_rows).toBe(!show);
    const suggest = store.roster_config.show_suggestions;
    await boxes[3].setValue(!suggest);
    expect(store.roster_config.show_suggestions).toBe(!suggest);
  });

  it("resets the active character but keeps who it is", async () => {
    const { wrapper, store } = await setup();
    const { id, roster_id } = store.active_profile;
    store.active_profile.char_name = "Kept";
    store.active_profile.band_values = ["Market", "Market", "Market"];
    await button(wrapper, "Reset this char").trigger("click");
    // nothing changes until the popup is confirmed
    expect(store.active_profile.band_values).toEqual([
      "Market",
      "Market",
      "Market",
    ]);
    [...document.body.querySelectorAll("button")]
      .find((b) => b.textContent!.trim() === "Reset")!
      .click();
    expect(store.active_profile).toMatchObject({
      id,
      roster_id,
      char_name: "Kept",
    });
    expect(store.active_profile.band_values).toEqual([
      "Worthless",
      "Worthless",
      "TaxedSell",
    ]);
  });
});
