import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import UpgradeOrder from "@/Components/Info/UpgradeOrder.vue";

// Fails on purpose until the page the upgrade order tooltip links to is written.
describe("learn pages", () => {
  it("the upgrade order page is written", () => {
    expect(mount(UpgradeOrder).text()).not.toContain("Coming soon");
  });
});
