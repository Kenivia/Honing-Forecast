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
  char_name_valid,
  debounce,
  default_char_labels,
  clean_char_name,
  name_problem,
  route_labels,
  upload_name,
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

describe("clean_char_name", () => {
  it("keeps what was typed, trimmed, lowercases A-Z after the first, capitalises the first", () => {
    expect(clean_char_name("  my Char-NAME!  ")).toBe("My char-name!");
    expect(clean_char_name("éCLAIR")).toBe("Éclair");
    // decomposed accents compose
    expect(clean_char_name("éclair")).toBe("Éclair");
    expect(clean_char_name("bob_2")).toBe("Bob_2");
    expect(clean_char_name("a/b?c%d#e")).toBe("A/b?c%d#e");
    expect(clean_char_name("a".repeat(20))).toHaveLength(20);
    expect(clean_char_name("   ")).toBe("");
  });
});

describe("route_labels", () => {
  const regions = { na: "nae", na2: "nae", eu: "euc" };
  const label = (p: any) => p.char_name || "Newchar";
  const route = (...ps: [string, string, string][]) =>
    route_labels(
      ps.map(([id, char_name, roster_id]) => ({
        id,
        char_name,
        roster_id,
      })) as any,
      regions,
      label,
    );

  it("a repeat in a region takes -2, -3, ...; the first keeps the plain name", () => {
    expect(
      route(["a", "Bob", "na"], ["b", "Bob", "na2"], ["c", "Bob", "na"]),
    ).toEqual({ a: "Bob", b: "Bob-2", c: "Bob-3" });
  });

  it("the same name in two regions is plain in both", () => {
    expect(route(["a", "Bob", "na"], ["b", "Bob", "eu"])).toEqual({
      a: "Bob",
      b: "Bob",
    });
  });

  it("a typed name that looks like a suffix keeps its own key; the repeat skips it", () => {
    expect(
      route(["a", "Bob", "na"], ["b", "Bob", "na"], ["c", "Bob-2", "na"]),
    ).toEqual({ a: "Bob", b: "Bob-3", c: "Bob-2" });
  });
});

describe("char names in the game and on the site", () => {
  it("valid: 2 to 16 latin letters, accented or not", () => {
    for (const ok of ["Ab", "Éclair", "Ñandú", "a".repeat(16), "éclair"])
      expect(char_name_valid(ok), ok).toBe(true);
    for (const bad of ["", "A", "a".repeat(17), "Bob1", "Bo_b", "Bo b", "Бob"])
      expect(char_name_valid(bad), bad).toBe(false);
  });

  it("empty asks for a name, invalid says so, uploads only carry valid names", () => {
    expect(name_problem("")).toBe("Enter a name");
    expect(name_problem("Bob1")).toBe("Cannot contain numbers");
    expect(name_problem("Bob_1")).toBe("Cannot contain numbers");
    expect(name_problem("Bob_")).toBe('Cannot contain "_"');
    expect(name_problem("Bob 2!")).toBe("Cannot contain numbers");
    expect(name_problem("Bo b!")).toBe("Cannot contain space");
    expect(name_problem("Bo-b/")).toBe('Cannot contain "-"');
    expect(name_problem("Bo/b")).toBe('Cannot contain "/"');
    expect(name_problem("Bobα")).toBe("Latin letters only");
    expect(name_problem("B")).toBe("Too short (min 2)");
    expect(name_problem("B".repeat(17))).toBe("Too long (max 16)");
    expect(name_problem("Bob")).toBeNull();
    expect([upload_name("Bob"), upload_name("Bob1"), upload_name("")]).toEqual([
      "Bob",
      "",
      "",
    ]);
  });

  it("unnamed characters are Newchar, Newchar1, ... per region, skipping names in use", () => {
    const all = [
      { id: "a", char_name: "", roster_id: "na" },
      { id: "b", char_name: "Newchar", roster_id: "na" },
      { id: "c", char_name: "", roster_id: "na" },
      { id: "d", char_name: "", roster_id: "eu" },
      { id: "e", char_name: "Bob", roster_id: "eu" },
    ] as any;
    expect(default_char_labels(all, { na: "nae", eu: "euc" })).toEqual({
      a: "Newchar1",
      c: "Newchar2",
      d: "Newchar",
    });
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
