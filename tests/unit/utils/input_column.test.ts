import { afterEach, describe, expect, it, vi } from "vitest";
import {
  column_labels,
  create_input_column,
  get_modified_cell,
  input_column_to_num,
  InputType,
  MAX_AMOUNT,
  parse_input,
  parse_locale_float,
  parse_locale_int,
  set_cell,
} from "@/Utils/InputColumn";

const column = (type = InputType.Int) =>
  create_input_column(type, ["A", "B"], {
    value: (l) => (l === "A" ? "1,000" : "7"),
    upper_bound: (l) => (l === "A" ? 5000 : MAX_AMOUNT),
  });
const event = (value: string) => ({ target: { value } }) as unknown as Event;

describe("create_input_column", () => {
  it("defaults and init", () => {
    const c = create_input_column(InputType.Int, ["X", "Y"], {
      enabled: (l) => l !== "Y",
    });
    expect(c.values).toEqual({ X: "0", Y: "0" });
    expect(c.upper_bound).toEqual({ X: MAX_AMOUNT, Y: MAX_AMOUNT });
    expect(c.enabled).toEqual({ X: true, Y: false });
    expect(column_labels(c)).toEqual(["X", "Y"]);
  });
});

describe("parsing (en-US)", () => {
  it("plain numbers drop grouping and junk", () => {
    expect(parse_locale_int("1,234,567")).toBe(1234567);
    expect(parse_locale_int(" 12 gold")).toBe(12);
    expect(parse_locale_int("1.9")).toBe(1);
    expect(parse_locale_float("1,234.5")).toBe(1234.5);
    expect(parse_locale_int("")).toBeNaN();
  });

  it("arithmetic with precedence and brackets", () => {
    expect(parse_locale_int("1,000+2*3")).toBe(1006);
    expect(parse_locale_int("(1+2)*3")).toBe(9);
    expect(parse_locale_int("10/4")).toBe(2);
    expect(parse_locale_float("10/4")).toBe(2.5);
    expect(parse_locale_int("2 * 3 - 1")).toBe(5);
    expect(parse_locale_float("1/0")).toBe(Infinity);
  });

  it("a leading minus is not a number", () => {
    expect(parse_locale_int("-5")).toBeNaN();
  });

  it("a trailing operator is not a number", () => {
    expect(parse_locale_int("5+")).toBeNaN();
    expect(parse_input(column(), "B", "5+")).toBe(0);
  });
});

describe("parse_input bounds", () => {
  it("clamps to the upper bound, garbage and infinity to 0", () => {
    const c = column();
    expect(parse_input(c, "A", "9999")).toBe(5000);
    expect(parse_input(c, "A", "abc")).toBe(0);
    expect(parse_input(c, "A", "1/0")).toBe(0);
  });

  it("a disabled label reads as unlimited unless pretended enabled", () => {
    const c = column();
    c.enabled.A = false;
    expect(parse_input(c, "A", "12")).toBe(MAX_AMOUNT);
    expect(parse_input(c, "A", "12", true)).toBe(12);
    expect(input_column_to_num(c)).toEqual({ A: MAX_AMOUNT, B: 7 });
    expect(input_column_to_num(c, true)).toEqual({ A: 1000, B: 7 });
  });

  it("a subtraction below zero clamps to 0", () => {
    expect(parse_input(column(), "B", "0-5")).toBe(0);
  });
});

describe("writing cells", () => {
  it("set_cell stores the parsed, bounded, locale-formatted value", () => {
    const c = column();
    set_cell(c, "A", "2,000*3");
    expect(c.values.A).toBe("5,000");
    set_cell(c, "B", "12+30");
    expect(c.values.B).toBe("42");
    c.enabled.B = false;
    set_cell(c, "B", "3");
    expect(c.values.B).toBe("3");
  });

  it("get_modified_cell ignores a disabled cell", () => {
    const c = column();
    expect(get_modified_cell(c, "B", event("2*4"))).toBe("8");
    c.enabled.B = false;
    expect(get_modified_cell(c, "B", event("2*4"))).toBe("7");
  });
});

describe("parsing (de-DE)", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.resetModules();
  });

  it("swaps group and decimal separators", async () => {
    const Real = Intl.NumberFormat;
    vi.spyOn(Intl, "NumberFormat").mockImplementation(function () {
      return new Real("de-DE");
    } as any);
    vi.resetModules();
    const m = await import("@/Utils/InputColumn");
    expect([m.LOCALE_GROUP, m.LOCALE_DECIMAL]).toEqual([".", ","]);
    expect(m.parse_locale_float("1.234,5")).toBe(1234.5);
    expect(m.parse_locale_int("1.000+2,5")).toBe(1002);
    expect(m.parse_locale_float("2,5*2")).toBe(5);
  });
});
