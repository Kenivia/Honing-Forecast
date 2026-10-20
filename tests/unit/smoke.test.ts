import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import TierConvertButton from "@/Components/Common/TierConvertButton.vue";
import { useRosterStore } from "@/Stores/RosterConfig";
import { mount_app } from "./mount";

// Proves the setup: an SFC mounts, the store loads under jsdom.
describe("setup", () => {
  it("mounts an SFC and emits", async () => {
    const w = mount(TierConvertButton, { props: { labelText: "Go" } });
    expect(w.text()).toBe("Go");
    await w.trigger("click");
    expect(w.emitted("change-tier")).toHaveLength(1);
  });

  it("does not emit when ineligible", async () => {
    const w = mount(TierConvertButton, {
      props: { labelText: "Go", checkEligibility: () => false },
    });
    await w.trigger("click");
    expect(w.emitted("change-tier")).toBeUndefined();
  });

  it("store has a default profile", async () => {
    await mount_app({ render: () => null });
    expect(useRosterStore().all_profiles.length).toBeGreaterThan(0);
  });
});
