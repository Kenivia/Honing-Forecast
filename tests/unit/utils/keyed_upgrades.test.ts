import { describe, expect, it } from "vitest";
import {
  get_upgrade_map,
  grids_to_keyed,
  to_upgrade_key,
  UpgradeStatus,
  type KeyedUpgrades,
  type Upgrade,
} from "@/Utils/KeyedUpgrades";
import {
  create_status_grid,
  get_valid_status_grid,
  is_enum,
} from "@/Utils/StatusGrid";
import {
  ADV_COLS,
  NORMAL_COLS,
  NUM_ADV_PIECES,
  NUM_PIECES,
} from "@/Utils/Constants";

const { Done, Want, NotYet, FetchedDone } = UpgradeStatus;
const grids = (tier = 0) => ({
  normal: create_status_grid(NUM_PIECES, NORMAL_COLS, tier, false),
  adv: create_status_grid(NUM_ADV_PIECES, ADV_COLS, tier, true),
});

describe("create_status_grid", () => {
  it("each tier's floor is fetched done, the vambrace row never", () => {
    const t4 = grids(0);
    const serca = grids(1);
    expect(t4.normal[0].indexOf(NotYet)).toBe(10);
    expect(serca.normal[5].indexOf(NotYet)).toBe(11);
    expect(t4.normal[6].every((x) => x === NotYet)).toBe(true);
    expect(serca.normal[6].every((x) => x === NotYet)).toBe(true);
    expect(t4.adv.flat().every((x) => x === NotYet)).toBe(true);
    expect(serca.adv.flat().every((x) => x === FetchedDone)).toBe(true);
    expect([t4.normal.length, t4.normal[0].length]).toEqual([7, 25]);
    expect([t4.adv.length, t4.adv[0].length]).toEqual([6, 4]);
  });

  it("rows are not shared", () => {
    const { normal } = grids();
    normal[0][20] = Want;
    expect(normal[1][20]).toBe(NotYet);
  });
});

describe("get_valid_status_grid", () => {
  const example = grids().adv;
  it("keeps a valid grid as is", () => {
    const g = grids().adv;
    g[2][3] = Want;
    expect(get_valid_status_grid(g, example)).toBe(g);
  });

  it("replaces a wrong shape or value with a copy of the example", () => {
    for (const bad of [
      undefined,
      example.slice(1),
      example.map((r) => r.slice(1)),
      example.map((r, i) => (i ? r : [...r.slice(1), 4])),
      example.map((r, i) => (i ? r : [...r.slice(1), "1"])),
    ]) {
      const out = get_valid_status_grid(bad as any, example);
      expect(out).toEqual(example);
      expect(out).not.toBe(example);
      expect(out[0]).not.toBe(example[0]);
    }
  });

  it("is_enum takes the numeric values only", () => {
    expect([0, 1, 2, 3].every((x) => is_enum(UpgradeStatus, x))).toBe(true);
    expect(is_enum(UpgradeStatus, 4)).toBe(false);
    expect(is_enum(UpgradeStatus, "Done")).toBe(false);
  });
});

describe("grids_to_keyed", () => {
  it("one upgrade per wanted cell, keyed by piece, index, kind and tier", () => {
    const { normal, adv } = grids();
    normal[5][10] = Want;
    normal[5][11] = Want;
    normal[0][12] = Done;
    adv[2][0] = Want;
    const keyed = grids_to_keyed(normal, adv, {}, 0);
    expect(Object.keys(keyed).sort()).toEqual(
      ["2,0,false,0", "5,10,true,0", "5,11,true,0"].sort(),
    );
    expect(keyed["5,10,true,0"]).toEqual({
      piece_index: 5,
      upgrade_index: 10,
      is_normal_honing: true,
      starting_artisan: 0,
      starting_num_taps: 0,
      state: null,
      unlocked: false,
      adv_progress: null,
      expanded: false,
      taps_since_last_input: 0,
      used_materials: null,
      double_balls: true,
    });
    expect(keyed["2,0,false,0"].adv_progress).toEqual([0, 0, false, false]);
    expect(to_upgrade_key(2, 0, false, 0)).toBe("2,0,false,0");
  });

  it("keeps a valid existing upgrade, drops what is no longer wanted", () => {
    const { normal, adv } = grids();
    normal[1][15] = Want;
    const first = grids_to_keyed(normal, adv, {}, 0);
    const kept = first["1,15,true,0"];
    kept.starting_artisan = 0.42;
    kept.expanded = true;
    normal[1][16] = Want;
    const second = grids_to_keyed(normal, adv, first, 0);
    expect(second["1,15,true,0"]).toBe(kept);
    normal[1][15] = Done;
    expect(Object.keys(grids_to_keyed(normal, adv, second, 0))).toEqual([
      "1,16,true,0",
    ]);
  });

  it("another tier's progress is not carried over", () => {
    const { normal, adv } = grids();
    normal[1][15] = Want;
    const t4 = grids_to_keyed(normal, adv, {}, 0);
    t4["1,15,true,0"].starting_artisan = 0.5;
    const serca = grids_to_keyed(normal, adv, t4, 1);
    expect(Object.keys(serca)).toEqual(["1,15,true,1"]);
    expect(serca["1,15,true,1"].starting_artisan).toBe(0);
  });

  it("replaces a malformed or non-boolean double_balls entry with a fresh one", () => {
    const { normal, adv } = grids();
    normal[0][12] = Want;
    normal[0][13] = Want;
    normal[0][14] = Want;
    const good = {
      piece_index: 0,
      upgrade_index: 14,
      is_normal_honing: true,
      unlocked: true,
      expanded: true,
      taps_since_last_input: 2,
    };
    const old = {
      "0,12,true,0": [0, 12, true, 3, null, true, false, null],
      "0,13,true,0": { ...good, upgrade_index: 13, double_balls: "yes" },
      "0,14,true,0": good,
    } as unknown as KeyedUpgrades;
    const keyed = grids_to_keyed(normal, adv, old, 0);
    for (const key of ["0,12,true,0", "0,13,true,0"] as const) {
      expect(keyed[key]).toMatchObject({ unlocked: false, state: null });
    }
    expect(keyed["0,14,true,0"]).toBe(good);
  });

  // apply_results writes adv_progress[0] in place
  it("fresh advanced upgrades do not share adv_progress", () => {
    const { normal, adv } = grids();
    adv[0][1] = Want;
    adv[3][2] = Want;
    const keyed = grids_to_keyed(normal, adv, {}, 0);
    keyed["0,1,false,0"].adv_progress[0] = 40;
    expect(keyed["3,2,false,0"].adv_progress[0]).toBe(0);
  });
});

it("get_upgrade_map keys Rust's upgrades like keyed upgrades", () => {
  const up = { piece_index: 5, upgrade_index: 20, is_normal_honing: true };
  const map = get_upgrade_map([up as Upgrade], 1);
  expect(map.get("5,20,true,1")).toBe(up);
  expect(get_upgrade_map(null, 0).size).toBe(0);
});
