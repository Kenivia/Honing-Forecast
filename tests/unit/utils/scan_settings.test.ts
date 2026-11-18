import { beforeEach, describe, expect, it, vi } from "vitest";

// a fresh module reads localStorage again, as a new session would
const load = async () => {
  vi.resetModules();
  return import("@/Components/Character/InventoryScanner/ScanSettings");
};

describe("scanner settings", () => {
  beforeEach(() => localStorage.clear());

  it("start on auto and English, not overridden", async () => {
    const { scan_settings, settings_overridden } = await load();
    expect(scan_settings.forced_21_9.manual).toBe(false);
    expect(scan_settings.brightness.manual).toBe(false);
    expect(scan_settings.font).toBe("English");
    expect(settings_overridden(scan_settings)).toBe(false);
  });

  it("survive a new session once saved", async () => {
    const first = await load();
    first.scan_settings.forced_21_9 = { value: true, manual: true };
    first.scan_settings.brightness = { value: 62, manual: true };
    first.scan_settings.font = "Korean";
    first.save_settings(false);
    const { scan_settings, settings_overridden } = await load();
    expect(scan_settings.forced_21_9).toEqual({ value: true, manual: true });
    expect(scan_settings.brightness).toEqual({ value: 62, manual: true });
    expect(scan_settings.font).toBe("Korean");
    expect(settings_overridden(scan_settings)).toBe(true);
  });

  it("keep a custom font's bytes", async () => {
    const first = await load();
    const bytes = Uint8Array.from({ length: 70000 }, (_, i) => i % 256);
    first.custom_font.bytes = bytes;
    Object.assign(first.scan_settings, { font: "Custom", font_file: "a.ttf" });
    first.save_settings(true);
    const { scan_settings, custom_font } = await load();
    expect(scan_settings.font_file).toBe("a.ttf");
    expect(custom_font.bytes).toEqual(bytes);
  });

  it("fall back to English when a custom font was not kept", async () => {
    localStorage.setItem(
      "scanner_settings",
      JSON.stringify({ font: "Custom", font_file: "a.ttf" }),
    );
    const { scan_settings } = await load();
    expect(scan_settings.font).toBe("English");
    expect(scan_settings.font_file).toBeNull();
  });

  it("drop the stored font once back on a game font", async () => {
    const first = await load();
    first.custom_font.bytes = new Uint8Array([1, 2, 3]);
    first.scan_settings.font = "Custom";
    first.save_settings(true);
    first.scan_settings.font = "English";
    first.save_settings(false);
    expect(localStorage.getItem("scanner_font")).toBeNull();
  });
});
