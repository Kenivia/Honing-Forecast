import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import { mount_app } from "../mount";
import ScannerControlPanel from "@/Components/Character/InventoryScanner/ScannerControlPanel.vue";
import { useRosterStore } from "@/Stores/RosterConfig";

describe("ScannerControlPanel", () => {
  it("the last checkbox opts out of sharing snapshots", async () => {
    const { wrapper } = await mount_app(ScannerControlPanel);
    const boxes = wrapper.findAll('input[type="checkbox"]');
    expect(boxes).toHaveLength(4);
    expect(wrapper.findAll("label").at(-1).text()).toBe(
      "Share owned material anonymously",
    );
    const store = useRosterStore();
    expect(store.roster_config.analytics).toBe(true);
    await boxes[3].setValue(false);
    expect(store.roster_config.analytics).toBe(false);
  });
});
