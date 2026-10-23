import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { default_roster_config } from "@/Stores/RosterConfig";
import { upload_snapshots } from "@/Utils/Analytics";
import { snapshot } from "../stores/fixtures";

const fetch = vi.fn();
beforeEach(() => {
  vi.stubEnv("MODE", "production");
  vi.stubEnv("DEV", false);
  vi.stubGlobal("fetch", fetch);
  fetch.mockReset().mockResolvedValue({ ok: true });
});
afterEach(() => {
  vi.unstubAllEnvs();
  vi.unstubAllGlobals();
});

function config_with(count: number) {
  const config = default_roster_config();
  const { roster_id, id } = config.profiles[0];
  config.snapshots = Array.from({ length: count }, (_, i) => ({
    ...snapshot(roster_id, id),
    at: i,
  }));
  return config;
}
const sent = (call: number) => JSON.parse(fetch.mock.calls[call][1].body);

describe("upload_snapshots", () => {
  it("sends what was not sent, in batches, and marks it", async () => {
    const config = config_with(25);
    config.snapshots[0].sent = true;
    await upload_snapshots(config);
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(sent(0).snapshots.map((s) => s.at)).toEqual(
      [...Array(20).keys()].map((i) => i + 1),
    );
    expect(sent(1).snapshots).toHaveLength(4);
    expect(sent(0)).toMatchObject({
      roster: String(config.profiles[0].roster_id),
      region: "nae",
    });
    expect(config.snapshots.every((s) => s.sent)).toBe(true);
    await upload_snapshots(config);
    expect(fetch).toHaveBeenCalledTimes(2);
  });

  it("keeps bodies under 300 KB and skips a snapshot that never fits", async () => {
    const config = config_with(4);
    const pad = (i: number, size: number) =>
      ((config.snapshots[i] as any).pad = "x".repeat(size));
    pad(0, 400_000);
    pad(1, 250_000);
    pad(2, 90_000);
    await upload_snapshots(config);
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(sent(0).snapshots.map((s) => s.at)).toEqual([1]);
    expect(sent(1).snapshots.map((s) => s.at)).toEqual([2, 3]);
    for (const call of fetch.mock.calls)
      expect(call[1].body.length).toBeLessThanOrEqual(300_000);
    expect(config.snapshots.every((s) => s.sent)).toBe(true);
  });

  it("marks nothing when the upload fails, and stops", async () => {
    const config = config_with(25);
    fetch.mockResolvedValue({ ok: false });
    await upload_snapshots(config);
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(config.snapshots.some((s) => s.sent)).toBe(false);
    fetch.mockRejectedValue(new Error("offline"));
    await upload_snapshots(config);
    expect(config.snapshots.some((s) => s.sent)).toBe(false);
  });

  it("a dev build's uploads are tagged dev", async () => {
    vi.stubEnv("MODE", "development");
    vi.stubEnv("DEV", true);
    await upload_snapshots(config_with(1));
    expect(sent(0).version).toBe("dev");
  });

  it("sends nothing when opted out, outside a real build or under automation", async () => {
    const config = config_with(1);
    config.analytics = false;
    await upload_snapshots(config);
    config.analytics = true;
    vi.stubEnv("MODE", "test");
    await upload_snapshots(config);
    // what the node suites see
    vi.stubEnv("MODE", undefined);
    await upload_snapshots(config);
    vi.stubEnv("MODE", "development");
    vi.stubGlobal("navigator", { webdriver: true });
    await upload_snapshots(config);
    expect(fetch).not.toHaveBeenCalled();
  });
});
