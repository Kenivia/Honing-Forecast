import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useRosterStore } from "@/Stores/RosterConfig";
import {
  check_adv_all_done,
  check_all_plus_20,
  check_revert_ilevel_ok,
  clamp,
  clamp_percentage,
  clean_percentage_input,
  cssVar,
  debounce,
  format_char_name,
  get_icon_path,
  has_upgrades_in_range,
  locale_to_fixed,
  metric_to_text,
  toOrdinal,
} from "@/Utils/Helpers";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";
import { ADV_COLS, NUM_ADV_PIECES } from "@/Utils/Constants";

const profiles = (...names: string[]) =>
  names.map((char_name) => ({ char_name })) as any;
// sets every normal row (or one) done up to +done and wanted up to +want
const set_row = (
  grid: UpgradeStatus[][],
  row: number,
  done: number,
  want = done,
) =>
  grid[row].forEach(
    (_, col, r) =>
      (r[col] =
        col < done
          ? UpgradeStatus.Done
          : col < want
            ? UpgradeStatus.Want
            : UpgradeStatus.NotYet),
  );

describe("number formatting", () => {
  it("locale_to_fixed rounds or floors", () => {
    expect(locale_to_fixed(1.236, 2)).toBe("1.24");
    expect(locale_to_fixed(1.236, 2, true)).toBe("1.23");
    expect(locale_to_fixed(3.7)).toBe("4");
  });

  it("percentages fall back on garbage and clamp to 0..100", () => {
    expect(clean_percentage_input("abc", 12.5)).toBe("12.50");
    expect(clean_percentage_input("150%", 0)).toBe("100.00");
    expect(clean_percentage_input("-5", 0)).toBe("5.00");
    expect(clamp_percentage("33.333")).toBe("33.33");
    expect(clamp(0, -1, 10)).toBe(0);
  });

  // floor(0.29 * 100) alone is 28
  it("round down keeps a value typed with two decimals", () => {
    expect(clean_percentage_input("0.29", 0, true)).toBe("0.29");
  });

  it("metric_to_text shows a cost as positive gold", () => {
    expect(metric_to_text(-1234.5)).toBe("1,235");
    expect(metric_to_text(0)).toBe("0");
    expect(metric_to_text(null)).toBe("N/A");
    expect(metric_to_text(NaN)).toBe("N/A");
  });

  it("ordinals", () => {
    expect([1, 2, 3, 4, 11, 12, 13, 21, 22, 101].map(toOrdinal)).toEqual([
      "1st",
      "2nd",
      "3rd",
      "4th",
      "11th",
      "12th",
      "13th",
      "21st",
      "22nd",
      "101st",
    ]);
  });

  it("icon lookups and css vars fall back", () => {
    expect(get_icon_path("Red")).toBe("/Icons/Materials/Red.webp");
    expect(get_icon_path("nope")).toBe("");
    expect(cssVar("--surely-unset", "#fff")).toBe("#fff");
  });

  it("debounce runs once with the last arguments", () => {
    vi.useFakeTimers();
    const fn = vi.fn();
    const d = debounce(fn, 100);
    d(1);
    d(2);
    vi.advanceTimersByTime(99);
    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(fn).toHaveBeenCalledExactlyOnceWith(2);
    vi.useRealTimers();
  });
});

describe("format_char_name", () => {
  it("strips, lowercases the rest, capitalises the first", () => {
    expect(format_char_name("my Char-NAME!", 0, profiles())).toBe("Mycharname");
    expect(format_char_name("éCLAIR", 0, profiles())).toBe("Éclair");
    expect(format_char_name("a".repeat(20), 0, profiles())).toHaveLength(16);
  });

  it("dedups against the other profiles only", () => {
    const all = profiles("Bob", "Bob1", "Alice");
    expect(format_char_name("bob", 2, all)).toBe("Bob2");
    expect(format_char_name("bob", 0, all)).toBe("Bob");
    expect(format_char_name("!!", 0, profiles())).toBe("1");
  });
});

describe("grid checks on the active profile", () => {
  beforeEach(() => setActivePinia(createPinia()));
  const profile = () => useRosterStore().active_profile;

  it("has_upgrades_in_range splits weapon from armour", () => {
    const p = profile();
    expect(has_upgrades_in_range(p, 11, 14, true, false)).toBe(false);
    p.normal_grid[5][12] = UpgradeStatus.Want; // weapon +13
    expect(has_upgrades_in_range(p, 11, 14, true, false)).toBe(true);
    expect(has_upgrades_in_range(p, 11, 14, false, false)).toBe(false);
    expect(has_upgrades_in_range(p, 14, 14, true, false)).toBe(false);
    p.adv_grid[0][1] = UpgradeStatus.Want; // helmet block 2
    expect(has_upgrades_in_range(p, 2, 2, false, true)).toBe(true);
    // the vambrace row never counts
    p.normal_grid[6][12] = UpgradeStatus.Want;
    p.normal_grid[5][12] = UpgradeStatus.NotYet;
    expect(has_upgrades_in_range(p, 11, 14, false, false)).toBe(false);
  });

  it("check_adv_all_done and check_all_plus_20", () => {
    const p = profile();
    expect(check_adv_all_done(p)).toBe(false);
    expect(check_all_plus_20(p)).toBe(false);
    for (let row = 0; row < NUM_ADV_PIECES; row++) {
      for (let col = 0; col < ADV_COLS; col++)
        p.adv_grid[row][col] = UpgradeStatus.FetchedDone;
      set_row(p.normal_grid, row, 20);
    }
    expect(check_adv_all_done(p)).toBe(true);
    expect(check_all_plus_20(p)).toBe(true);
    p.normal_grid[3][19] = UpgradeStatus.Want;
    expect(check_all_plus_20(p)).toBe(false);
  });

  it("check_revert_ilevel_ok names the first level with no T4 equal", () => {
    const p = profile();
    expect(check_revert_ilevel_ok(p)).toBe(true); // T4 always
    p.tier = 1;
    for (let row = 0; row < NUM_ADV_PIECES; row++)
      set_row(p.normal_grid, row, 12, 14);
    expect(check_revert_ilevel_ok(p)).toBe(true);
    set_row(p.normal_grid, 2, 12, 15);
    expect(check_revert_ilevel_ok(p)).toBe(15);
    set_row(p.normal_grid, 2, 17, 18);
    expect(check_revert_ilevel_ok(p)).toBe(17);
  });
});
