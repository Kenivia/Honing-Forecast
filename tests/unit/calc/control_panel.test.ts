import { afterEach, describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import "./helpers";
import { flush } from "./helpers";
import { mount_app } from "../mount";
import ControlPanel from "@/Components/Character/ControlPanel.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { export_config } from "@/Stores/ConfigStorage";

const read_blob = (blob: Blob) =>
  new Promise<string>((resolve) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.readAsText(blob);
  });

async function setup() {
  const { wrapper } = await mount_app(ControlPanel);
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

describe("ControlPanel band values", () => {
  it("lists tradable first and disables values that would decrease across bands", async () => {
    const { wrapper } = await setup();
    const labels = wrapper
      .findAll("select")
      .map((s) => s.attributes("aria-label"));
    expect(labels).toEqual([
      "Tradable leftover value",
      "Roster leftover value",
      "Bound leftover value",
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
    const { wrapper, store } = await setup();
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

  it("an out-of-order loaded plan leaves a band with no allowed value", async () => {
    const { wrapper, store } = await setup();
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
    expect(boxes).toHaveLength(3);
    const lock = store.active_profile.lock_fetched_done;
    await boxes[2].setValue(!lock);
    expect(store.active_profile.lock_fetched_done).toBe(!lock);
    const show = store.roster_config.show_all_rows;
    await boxes[0].setValue(!show);
    expect(store.roster_config.show_all_rows).toBe(!show);
  });

  it("resets the active character but keeps who it is", async () => {
    const { wrapper, store } = await setup();
    const { id, roster_id } = store.active_profile;
    store.active_profile.char_name = "Kept";
    store.active_profile.band_values = ["Market", "Market", "Market"];
    await wrapper
      .findAll("button")
      .find((b) => b.text() === "Reset this char")!
      .trigger("click");
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

describe("ControlPanel backup", () => {
  afterEach(() => vi.restoreAllMocks());

  it("exports the whole config to a dated json file", async () => {
    const { wrapper, store } = await setup();
    let blob: Blob | undefined;
    URL.createObjectURL = vi.fn((b: Blob) => ((blob = b), "blob:x"));
    URL.revokeObjectURL = vi.fn();
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(function (this: HTMLAnchorElement) {
        expect(this.download).toMatch(
          /^honing-forecast-\d{4}-\d{2}-\d{2}\.json$/,
        );
        expect(this.href).toBe("blob:x");
      });
    await wrapper
      .findAll("button")
      .find((b) => b.text() === "Export to file")!
      .trigger("click");
    expect(click).toHaveBeenCalledOnce();
    expect(blob!.type).toBe("application/json");
    expect(await read_blob(blob!)).toBe(export_config(store.roster_config));
    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:x");
  });

  async function pick(wrapper, text: string) {
    const input = wrapper.find('input[type="file"]');
    Object.defineProperty(input.element, "files", {
      // jsdom's File has no text()
      value: [{ text: async () => text }],
      configurable: true,
    });
    await input.trigger("change");
    await flush();
    await wrapper.vm.$nextTick();
  }

  it("imports a backup into the store", async () => {
    const { wrapper, store } = await setup();
    store.active_profile.char_name = "Saved";
    const text = export_config(store.roster_config);
    store.active_profile.char_name = "Changed";
    await pick(wrapper, text);
    expect(store.active_profile.char_name).toBe("Saved");
    expect(wrapper.text()).not.toContain("Import failed");
  });

  it("shows why an import failed and leaves the config alone", async () => {
    const { wrapper, store } = await setup();
    const before = store.roster_config;
    await pick(wrapper, JSON.stringify({ version: 999 }));
    expect(wrapper.text()).toContain(
      "Import failed: backup is from a newer version (999)",
    );
    expect(store.roster_config).toBe(before);
    await pick(wrapper, "not json");
    expect(wrapper.text()).toContain("Import failed:");
    // a good import clears the message
    await pick(wrapper, export_config(store.roster_config));
    expect(wrapper.text()).not.toContain("Import failed");
  });
});
