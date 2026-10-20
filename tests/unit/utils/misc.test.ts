import { describe, expect, it } from "vitest";
import { ALL_VERSIONS, LATEST_VERSION, version_equal } from "@/Utils/Changelog";

describe("changelog versions", () => {
  it("newest first, numerically", () => {
    expect(ALL_VERSIONS.length).toBeGreaterThan(0);
    const nums = ALL_VERSIONS.map((v) => v.slice(1).split(".").map(Number));
    for (let i = 1; i < nums.length; i++) {
      const [a, b] = [nums[i - 1], nums[i]];
      expect(a[0] - b[0] || a[1] - b[1] || a[2] - b[2]).toBeGreaterThan(0);
    }
    expect(LATEST_VERSION).toBe(ALL_VERSIONS[0]);
    expect(ALL_VERSIONS.every((v) => /^v\d+\.\d+\.\d+$/.test(v))).toBe(true);
  });

  it("version_equal is plain equality, patch included", () => {
    expect(version_equal("v1.2.8", "v1.2.8")).toBe(true);
    expect(version_equal("1.2.8", "v1.2.8")).toBe(true);
    expect(version_equal("v1.2.7", "v1.2.8")).toBe(false);
    expect(version_equal("", "v1.2.8")).toBe(false);
    expect(version_equal("v1.0.0", "v1.2.5")).toBe(false);
  });
});
