import { describe, expect, it, vi } from "vitest";
vi.mock("@/../crates/wasm/pkg/hf_wasm.js", () => ({ default: vi.fn() }));
import { mount } from "@vue/test-utils";
import UpgradeTip from "@/Components/Character/Instructions/UpgradeTip.vue";
import { fake_upgrade, fresh_stores, set_results } from "./fake";

function tip(upgrade) {
  fresh_stores();
  set_results([upgrade]);
  return mount(UpgradeTip, { props: { upgrade } }).text();
}

describe("UpgradeTip events", () => {
  it("says so when no event applies", () => {
    expect(tip(fake_upgrade())).toContain("No events affecting this upgrade");
  });

  it("shows extra chance, artisan rate and cost multipliers", () => {
    const text = tip(
      fake_upgrade({
        tap: { event: 5000, artisan_percent: 150 },
        cost_multipliers: [0.8, 1, 1, 1, 1, 1, 1],
      }),
    );
    expect(text).toContain("+5% extra chance");
    expect(text).toContain("×1.5 artisan rate");
    expect(text).toContain("×0.8");
    expect(text).not.toContain("No events");
  });
});
