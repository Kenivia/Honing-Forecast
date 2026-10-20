import { describe, expect, it } from "vitest";
import { convert_pooled, CONVERT_SUMMARY } from "@/Utils/Convert";
import { create_default_owned_input_column } from "@/Stores/RosterConfig";
import { input_column_to_num } from "@/Utils/InputColumn";

function columns(roster: Record<string, string>, tradable = {}) {
  const r = create_default_owned_input_column();
  const t = create_default_owned_input_column();
  Object.assign(r.values, roster);
  Object.assign(t.values, tradable);
  return [r, t] as const;
}

describe("convert_pooled", () => {
  it("pools both bands, spends roster first and keeps the leftover", () => {
    const [r, t] = columns(
      { Red: "1,000", "Serca Red": "7", Blue: "3", Shards: "999" },
      { Red: "502", "Serca Red": "3", Blue: "9" },
    );
    convert_pooled(r, t);
    const rn = input_column_to_num(r);
    const tn = input_column_to_num(t);
    expect([rn["Serca Red"], rn.Red, tn.Red, tn["Serca Red"]]).toEqual([
      307, 0, 2, 3,
    ]);
    // 12 Blue: 10 used, roster's 3 first, 2 tradable left
    expect([rn["Serca Blue"], rn.Blue, tn.Blue]).toEqual([2, 0, 2]);
    expect(rn.Shards).toBe(999);
  });

  it("converts Fusion and Special Leap too", () => {
    const [r, t] = columns({ Fusion: "10", "Special Leap": "11" });
    convert_pooled(r, t);
    const rn = input_column_to_num(r);
    expect([rn["Serca Fusion"], rn.Fusion]).toEqual([2, 0]);
    expect([rn["Serca Special Leap"], rn["Special Leap"]]).toEqual([2, 1]);
    expect(CONVERT_SUMMARY).toContain("5 Fusion to 1 Serca Fusion");
    expect(CONVERT_SUMMARY).toContain("5 Special Leap to 1 Serca Special Leap");
  });

  it("reads disabled labels by value, not as unlimited", () => {
    const [r, t] = columns({ Red: "10" }, { Red: "5" });
    t.enabled.Red = false;
    convert_pooled(r, t);
    expect(input_column_to_num(r, true)["Serca Red"]).toBe(3);
  });
});
