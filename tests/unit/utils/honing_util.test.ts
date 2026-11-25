import { describe, expect, it } from "vitest";
import { artisan_number, artisan_string, ilevel } from "@/Utils/HoningUtil";
import { create_status_grid } from "@/Utils/StatusGrid";
import { UpgradeStatus, type Upgrade } from "@/Utils/KeyedUpgrades";
import {
  ADV_COLS,
  NORMAL_COLS,
  NUM_ADV_PIECES,
  NUM_PIECES,
} from "@/Utils/Constants";

const profile = (tier: number) =>
  ({
    tier,
    sidereal_level: null,
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

  it("a sidereal weapon counts at its own item level", () => {
    // Kowiz on lostark.bible: Serca +25 armour and a +10 sidereal weapon, 1805
    const p = profile(1);
    for (const row of p.normal_grid) row.fill(UpgradeStatus.FetchedDone);
    p.sidereal_level = 10;
    expect(ilevel(p, "achieved")).toBe("1805.00");
    // a T4 one at +9 replaces the weapon's honing and advanced honing
    const q = profile(0);
    q.sidereal_level = 9;
    q.normal_grid[5].fill(UpgradeStatus.NotYet);
    expect(ilevel(q, "achieved")).toBe(
      ((1640 * 6 - 1640 + 1745) / 6).toFixed(2),
    );
  });

  it("is unknown while a piece has nothing done", () => {
    const p = profile(0);
    p.normal_grid[3].fill(UpgradeStatus.NotYet);
    expect(ilevel(p, "achieved")).toBe("?");
  });
});

// the artisan trail as Rust sends it
const upgrade = (tap_artisan: number[]) => ({ tap_artisan }) as Upgrade;

describe("artisan", () => {
  it("reads the trail Rust worked out, held at its end", () => {
    const u = upgrade([0, 0.1, 0.25]);
    expect(artisan_number(u, 0)).toBe(0);
    expect(artisan_number(u, 2)).toBe(0.25);
    expect(artisan_number(u, 9)).toBe(0.25);
  });

  it("the string caps at 100", () => {
    expect(artisan_string(upgrade([0, 0.6, 1.2]), 2)).toBe("100.00");
  });
});
