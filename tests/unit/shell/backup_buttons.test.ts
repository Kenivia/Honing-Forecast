import { afterEach, describe, expect, it, vi } from "vitest";
import { mount_app } from "../mount";
import BackupButtons from "@/Components/Common/BackupButtons.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { export_config } from "@/Stores/ConfigStorage";

const read_blob = (blob: Blob) =>
  new Promise<string>((resolve) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.readAsText(blob);
  });

async function setup() {
  const { wrapper } = await mount_app(BackupButtons);
  return { wrapper, store: useRosterStore() };
}

async function pick(wrapper, text: string) {
  const input = wrapper.find('input[type="file"]');
  Object.defineProperty(input.element, "files", {
    // jsdom's File has no text()
    value: [{ text: async () => text }],
    configurable: true,
  });
  await input.trigger("change");
  await new Promise((r) => setTimeout(r));
  await wrapper.vm.$nextTick();
}

describe("BackupButtons", () => {
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
