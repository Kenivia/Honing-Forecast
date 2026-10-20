import { describe, expect, it } from "vitest";
import { artisan_number, artisan_string, ilevel } from "@/Utils/HoningUtil";
import { create_status_grid } from "@/Utils/StatusGrid";
import { UpgradeStatus, type Upgrade } from "@/Utils/KeyedUpgrades";
import {
  ADV_COLS,
  DEFAULT_ARTISAN_MULTIPLIER as M,
  NORMAL_COLS,
  NUM_ADV_PIECES,
  NUM_PIECES,
} from "@/Utils/Constants";

const profile = (tier: number) =>
  ({
    tier,
    normal_grid: create_status_grid(NUM_PIECES, NORMAL_COLS, tier, false),
    adv_grid: create_status_grid(NUM_ADV_PIECES, ADV_COLS, tier, true),
  }) as any;

describe("ilevel", () => {
  it("the default grids of each tier", () => {
    expect(ilevel(profile(0), "achieved")).toBe("1640.00");
    expect(ilevel(profile(1), "achieved")).toBe("1730.00");
  });

  it("pending counts wants, the vambrace never counts", () => {
    const p = profile(0);
    p.normal_grid[0][10] = UpgradeStatus.Want;
    p.adv_grid[5][0] = UpgradeStatus.Want;
    p.normal_grid[6][0] = UpgradeStatus.Done;
    expect(ilevel(p, "achieved")).toBe("1640.00");
    // +1 normal on one piece is 5/6, one adv block is 10/6
    expect(ilevel(p, "pending")).toBe((1640 + 5 / 6 + 10 / 6).toFixed(2));
  });

  it("is unknown while a piece has nothing done", () => {
    const p = profile(0);
    p.normal_grid[3].fill(UpgradeStatus.NotYet);
    expect(ilevel(p, "achieved")).toBe("?");
  });
});

// an upgrade with a flat 10% base chance and one juice of +10%
const upgrade = (state: number[][], extra: Partial<Upgrade> = {}) =>
  ({
    piece_type_usize: 0,
    upgrade_index: 3,
    base_chance: 0.1,
    extra_chance: 0,
    artisan_rate: 1,
    starting_artisan: 0,
    starting_num_taps: 0,
    state,
    ...extra,
  }) as Upgrade;
const juice_info = {
  all_juices: [[new Map([[3, { normal_chance: 0.1 }]])]],
};

describe("artisan", () => {
  it("adds each tap's chance, which rises by a tenth of base per failure", () => {
    const u = upgrade([[], [], [0]]);
    expect(artisan_number(u, 0, juice_info)).toBe(0);
    expect(artisan_number(u, 1, juice_info)).toBeCloseTo(M * 0.1);
    expect(artisan_number(u, 3, juice_info)).toBeCloseTo(
      M * (0.1 + 0.11 + (0.12 + 0.1)),
    );
  });

  it("starts from the given artisan and taps, caps the bonus at 10 taps", () => {
    const u = upgrade([[], []], {
      starting_artisan: 0.5,
      starting_num_taps: 9,
    });
    expect(artisan_number(u, 2, juice_info)).toBeCloseTo(
      0.5 + M * (0.19 + 0.2),
    );
  });

  it("stops at full artisan and the string caps at 100", () => {
    const u = upgrade(Array(40).fill([0]), { artisan_rate: 3 });
    expect(artisan_number(u, 40, juice_info)).toBeGreaterThanOrEqual(1);
    expect(artisan_number(u, 40, juice_info)).toBeLessThan(1 + M * 3);
    expect(artisan_string(u, 40, juice_info)).toBe("100.00");
  });

  // Rust reads a missing state as no juice
  it("taps beyond the state count as unjuiced", () => {
    expect(artisan_number(upgrade([]), 1, juice_info)).toBeCloseTo(M * 0.1);
  });
});
