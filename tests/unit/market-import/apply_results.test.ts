import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { apply_results } from "@/Components/Common/Uwuowo/ApplyResults";
import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";
import { new_char_profile, CharProfile } from "@/Stores/CharacterProfile";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import type { UwuowoPiece } from "@/Components/Common/Uwuowo/UwuowoUtils";

vi.mock("@/Components/Character/CharWorkerUtils", () => ({
  grid_change_callback: vi.fn(),
}));

const { Done, Want, NotYet, FetchedDone } = UpgradeStatus;
const t4 = (plus_n: number, adv: number): UwuowoPiece => ({
  plus_n,
  ilevel: 1700,
  tier: 0,
  adv,
});
const serca = (plus_n: number): UwuowoPiece => ({
  plus_n,
  ilevel: 1760,
  tier: 1,
  adv: NaN,
});
// a normal-grid row: `done` FetchedDone, then Want up to `want`, then NotYet
const row = (done: number, want = done, kind = FetchedDone) =>
  Array.from({ length: 25 }, (_, i) =>
    i < done ? kind : i < want ? Want : NotYet,
  );

let profile: CharProfile;
beforeEach(() => {
  setActivePinia(createPinia());
  profile = new_char_profile();
  vi.mocked(grid_change_callback).mockClear();
});

describe("apply_results", () => {
  it("a sidereal weapon is set on the profile and fully ticked", () => {
    const pieces = Array(6).fill(t4(20, 20));
    pieces[5] = { ...t4(25, 40), sidereal_level: 9 };
    apply_results(profile, pieces, true);
    expect(profile.sidereal_level).toBe(9);
    expect(profile.normal_grid[5]).toEqual(row(25));
    expect(profile.adv_grid[5]).toEqual(Array(4).fill(FetchedDone));
    apply_results(profile, Array(6).fill(t4(20, 20)), true);
    expect(profile.sidereal_level).toBe(null);
  });

  it("keeps the fetched class, and a fetch without one leaves it", () => {
    apply_results(profile, [t4(20, 0)], true, "Bard");
    expect(profile.class_name).toBe("Bard");
    apply_results(profile, [t4(20, 0)], true);
    expect(profile.class_name).toBe("Bard");
  });

  it("marks a T4 character's done upgrades as fetched and keeps its wants", () => {
    profile.normal_grid[0] = row(10, 23, Done);
    apply_results(profile, Array(6).fill(t4(20, 20)), true);
    expect(profile.tier).toBe(0);
    expect(profile.normal_grid[0]).toEqual(row(20, 23));
    expect(profile.normal_grid[1]).toEqual(row(20));
    expect(profile.adv_grid[0]).toEqual([
      FetchedDone,
      FetchedDone,
      NotYet,
      NotYet,
    ]);
    expect(grid_change_callback).toHaveBeenCalledWith(true);
  });

  it("an adv level mid-block becomes a wanted upgrade with progress", () => {
    profile.adv_grid[1] = [NotYet, NotYet, NotYet, Want];
    apply_results(profile, [t4(20, 0), t4(20, 25)], false);
    expect(profile.adv_grid[1]).toEqual([FetchedDone, FetchedDone, Want, Want]);
    expect(profile.adv_grid[0]).toEqual([NotYet, NotYet, NotYet, NotYet]);
    const key = "1,2,false,0";
    expect(profile.keyed_upgrades[key].adv_progress[1]).toBe(0);
    expect(grid_change_callback).toHaveBeenCalledWith(false);
  });

  it("seeds adv progress as the in-block level, per piece", () => {
    apply_results(profile, [t4(20, 25), t4(20, 27)], true);
    expect(profile.keyed_upgrades["0,2,false,0"].adv_progress).toEqual([
      50,
      0,
      false,
      false,
    ]);
    expect(profile.keyed_upgrades["1,2,false,0"].adv_progress[0]).toBe(70);
  });

  it("switches to Serca and marks all advanced honing done", () => {
    apply_results(profile, Array(6).fill(serca(15)), true);
    expect(profile.tier).toBe(1);
    expect(profile.normal_grid[0]).toEqual(row(15));
    expect(
      profile.adv_grid.every((r) => r.every((s) => s === FetchedDone)),
    ).toBe(true);
  });

  it("all ready to transfer: Serca, T4 pieces converted into Serca levels", () => {
    const pieces = [serca(15), t4(22, 40), ...Array(3).fill(t4(20, 40))];
    pieces.push({ ...t4(25, 40), sidereal_level: 9 });
    apply_results(profile, pieces, true);
    expect(profile.tier).toBe(1);
    expect(profile.normal_grid[1]).toEqual(row(13)); // T4 +22 is Serca +13
    expect(profile.normal_grid[2]).toEqual(row(11));
  });

  it("one piece not ready: T4, Serca pieces count as fully honed T4", () => {
    const pieces = [serca(15), t4(22, 30), ...Array(4).fill(t4(20, 40))];
    apply_results(profile, pieces, true);
    expect(profile.tier).toBe(0);
    expect(profile.normal_grid[0]).toEqual(row(25));
    expect(profile.adv_grid[0]).toEqual(Array(4).fill(FetchedDone));
    expect(profile.normal_grid[1]).toEqual(row(22));
    expect(profile.adv_grid[1]).toEqual([
      FetchedDone,
      FetchedDone,
      FetchedDone,
      NotYet,
    ]);
    apply_results(profile, [...Array(5).fill(t4(20, 40)), t4(19, 40)], true);
    expect(profile.tier).toBe(0);
  });

  it("refresh keeps the tier unless every piece is Serca", () => {
    const mixed = [serca(15), ...Array(5).fill(t4(20, 40))];
    apply_results(profile, mixed, true, undefined, true);
    expect(profile.tier).toBe(0);
    expect(profile.normal_grid[0]).toEqual(row(23)); // Serca +15 takes +14, T4 +23
    profile.tier = 1;
    apply_results(
      profile,
      [serca(15), t4(22, 30), t4(10, 0)],
      true,
      undefined,
      true,
    );
    expect(profile.tier).toBe(1);
    expect(profile.normal_grid[1]).toEqual(row(13));
    expect(profile.normal_grid[2]).toEqual(row(11)); // below T4 +20 floors to Serca +11
    apply_results(profile, Array(6).fill(t4(20, 40)), true, undefined, true);
    expect(profile.tier).toBe(1);
    profile.tier = 0;
    apply_results(profile, Array(6).fill(serca(12)), true, undefined, true);
    expect(profile.tier).toBe(1);
  });

  it("the vambrace is applied on Serca, keeping its wants", () => {
    profile.normal_grid[6] = row(0, 15);
    apply_results(profile, [...Array(6).fill(serca(15)), serca(10)], true);
    expect(profile.tier).toBe(1);
    expect(profile.normal_grid[6]).toEqual(row(10, 15));
  });

  it("the vambrace neither picks the tier nor is applied on T4", () => {
    const before = row(0, 5);
    profile.normal_grid[6] = [...before];
    apply_results(profile, [...Array(6).fill(t4(20, 20)), serca(10)], true);
    expect(profile.tier).toBe(0);
    expect(profile.normal_grid[6]).toEqual(before);
    apply_results(
      profile,
      [...Array(5).fill(serca(15)), t4(20, 30), serca(10)],
      true,
    );
    expect(profile.tier).toBe(0);
    expect(profile.normal_grid[6]).toEqual(before);
  });

  it("a Serca profile importing T4-only gear switches back to T4", () => {
    profile.tier = 1;
    apply_results(profile, [t4(21, 10)], true);
    expect(profile.tier).toBe(0);
    expect(profile.normal_grid[0]).toEqual(row(21));
  });
});
