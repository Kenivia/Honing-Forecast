import { describe, expect, it } from "vitest";
import {
  annotations_shown,
  owned_lines,
  special_graph_data,
  t4_gains,
} from "@/Components/Character/MaterialDist/DistLines";

describe("DistLines", () => {
  it("t4_gains: bound into bound, roster and tradable pooled into roster", () => {
    const gains = t4_gains(
      1,
      [
        { Red: 12, "Serca Red": 1 },
        { Red: 3, "Serca Red": 0 },
        { Red: 4, "Serca Red": 0 },
      ],
      {},
      { "Special Leap": 11, "Serca Special Leap": 0 },
    );
    expect(gains["Serca Red"]).toEqual([2, 1, 0]);
    expect(gains["Serca Special Leap"]).toEqual([2]);
    // a T4 character adds nothing
    const t4 = { Red: 12 };
    expect(t4_gains(0, [t4, t4, t4], {}, {})["Red"]).toEqual([0, 0, 0]);
    // nor a disabled source
    expect(
      t4_gains(1, [{ Red: 12, "Serca Red": 0 }, {}, {}], { Red: false }, {})[
        "Serca Red"
      ][0],
    ).toBe(0);
  });

  it("annotations_shown: the average and the chance band's line", () => {
    expect(annotations_shown(0)).toEqual([true, true, false, false]);
    expect(annotations_shown(2)).toEqual([true, false, false, true]);
  });

  it("owned_lines: cumulative per band, none from an untraded band up", () => {
    expect(owned_lines(5, [1, 2, 3], [10, 0, 1], [true, true, true])).toEqual([
      5, 11, 13, 17,
    ]);
    expect(owned_lines(5, [1, 2, 3], [0, 0, 0], [true, false, true])).toEqual([
      5,
      1,
      null,
      null,
    ]);
  });

  it("special_graph_data: point k+1 is k pieces tapped, 0 to the normal upgrade count", () => {
    const normal = { is_normal_honing: true };
    const adv = { is_normal_honing: false };
    const bundle = {
      latest_special_probs: [0.5, 0.25, 0.1],
      upgrade_arr: [normal, adv, normal],
    };
    expect(special_graph_data(bundle, true)).toEqual([
      [1, 1],
      [2, 0.5],
      [3, 0.25],
    ]);
    expect(special_graph_data(bundle, false)).toEqual([
      [1, 0.5],
      [2, 0.25],
      [3, 0.1],
    ]);
    expect(
      special_graph_data(
        { latest_special_probs: [], upgrade_arr: [adv] },
        true,
      ),
    ).toEqual([]);
  });
});
