import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";

vi.mock("@/Components/Character/CharWorkerUtils", () => ({
  grid_change_callback: vi.fn(),
}));

import { grid_change_callback } from "@/Components/Character/CharWorkerUtils";
import {
  apply_done_want,
  change_tier,
  convert_apply_done_want,
} from "@/Components/Character/StatusInput/StatusInputUtil";
import { useRosterStore } from "@/Stores/RosterConfig";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { NORMAL_COLS, NUM_ADV_PIECES } from "@/Utils/Constants";

const { Done, Want, NotYet, FetchedDone } = UpgradeStatus;
// a row as [done, fetched done, wanted] counts from the left
const shape = (row: UpgradeStatus[]) => {
  const done = row.filter((x) => x === Done || x === FetchedDone).length;
  const want = row.filter((x) => x === Want).length;
  return [done, done + want];
};
const row_of = (fetched: number, done: number, want: number) =>
  Array.from({ length: NORMAL_COLS }, (_, col) =>
    col < fetched
      ? FetchedDone
      : col < done
        ? Done
        : col < want
          ? Want
          : NotYet,
  );

describe("convert_apply_done_want", () => {
  it("maps T4 +20..+25 onto Serca +11..+18", () => {
    const row = row_of(10, 22, 24);
    convert_apply_done_want(0, 1, 22, 24, row);
    expect(shape(row)).toEqual([13, 16]);
  });

  it("maps back, and raises anything below the floor to it", () => {
    const row = row_of(11, 13, 16);
    convert_apply_done_want(1, 0, 13, 16, row);
    expect(shape(row)).toEqual([22, 24]);
    const low = row_of(10, 15, 15);
    convert_apply_done_want(0, 1, 15, 15, low);
    expect(shape(low)).toEqual([11, 11]);
  });

  it("fetched marks every done cell", () => {
    const row = row_of(0, 0, 0);
    convert_apply_done_want(0, 1, 23, 0, row, true);
    expect(row.slice(0, 14).every((x) => x === FetchedDone)).toBe(true);
    expect(row[14]).toBe(NotYet);
  });

  it("a want with no equal falls back to the level below it", () => {
    const row = row_of(11, 12, 17);
    convert_apply_done_want(1, 0, 12, 17, row);
    expect(shape(row)).toEqual([21, 24]);
  });

  it("levels ticked by hand stay Done", () => {
    const row = row_of(10, 22, 22);
    convert_apply_done_want(0, 1, 22, 22, row);
    expect(row[12]).toBe(Done);
  });
});

it("apply_done_want fills done, then want, then not yet", () => {
  const row = row_of(0, 0, 0);
  apply_done_want(3, 5, row);
  expect(row.slice(0, 6)).toEqual([Done, Done, Done, Want, Want, NotYet]);
  apply_done_want(2, 0, row, true);
  expect(row.slice(0, 3)).toEqual([FetchedDone, FetchedDone, NotYet]);
});

describe("change_tier", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(grid_change_callback).mockClear();
  });
  const profile = () => useRosterStore().active_profile;

  it("T4 to Serca and back remaps every armour and weapon row", () => {
    const p = profile();
    for (let r = 0; r < NUM_ADV_PIECES; r++)
      p.normal_grid[r].splice(0, NORMAL_COLS, ...row_of(10, 22, 24));
    p.normal_grid[6].splice(0, NORMAL_COLS, ...row_of(0, 3, 5));
    change_tier(p);
    expect(p.tier).toBe(1);
    expect(p.normal_grid.slice(0, 6).map(shape)).toEqual(
      Array(6).fill([13, 16]),
    );
    expect(p.adv_grid.flat().every((x) => x === Done)).toBe(true);
    // the vambrace row is left alone
    expect(shape(p.normal_grid[6])).toEqual([3, 5]);
    expect(grid_change_callback).toHaveBeenCalledOnce();
    change_tier(p);
    expect(p.tier).toBe(0);
    expect(p.normal_grid.slice(0, 6).map(shape)).toEqual(
      Array(6).fill([22, 24]),
    );
  });

  it("refuses to revert a Serca level with no T4 equal", () => {
    const p = profile();
    p.tier = 1;
    p.normal_grid[2].splice(0, NORMAL_COLS, ...row_of(11, 15, 15));
    const before = p.normal_grid.map((r) => [...r]);
    change_tier(p);
    expect(p.tier).toBe(1);
    expect(p.normal_grid).toEqual(before);
    expect(grid_change_callback).not.toHaveBeenCalled();
  });

  it("checks its target, not the active profile", () => {
    const active = profile();
    active.tier = 1;
    for (let r = 0; r < NUM_ADV_PIECES; r++)
      active.normal_grid[r].splice(0, NORMAL_COLS, ...row_of(11, 13, 13));
    const other = JSON.parse(JSON.stringify(active));
    active.normal_grid[0].splice(0, NORMAL_COLS, ...row_of(11, 15, 15));
    change_tier(other);
    expect(other.tier).toBe(0);
    expect(active.tier).toBe(1);
  });

  it("fetched flips the tier and only fills the Serca adv grid", () => {
    const p = profile();
    const before = p.normal_grid.map((r) => [...r]);
    change_tier(p, true);
    expect(p.tier).toBe(1);
    expect(p.adv_grid.flat().every((x) => x === FetchedDone)).toBe(true);
    expect(p.normal_grid).toEqual(before);
    expect(grid_change_callback).not.toHaveBeenCalled();
  });
});
