import { describe, expect, it } from "vitest";
import { click_cell, click_column, column_status } from "@/Utils/StatusGrid";
import { UpgradeStatus } from "@/Utils/KeyedUpgrades";

const S = { D: 0, W: 1, N: 2, F: 3 } as Record<string, UpgradeStatus>;
const row = (text: string) => [...text].map((c) => S[c]);
const text = (cells: UpgradeStatus[]) => cells.map((s) => "DWNF"[s]).join("");
const click = (cells: string, col: number, locked = true) => {
  const r = row(cells);
  click_cell(r, col, locked);
  return text(r);
};
// the vambrace row (index 6) is left out of a column
const grid = (...rows: string[]) => [...rows.map(row), row("NNNN")];

describe("click_cell", () => {
  it("NotYet wants it and below, turning done cells above to Want", () => {
    expect(click("FFNN", 3)).toBe("FFWW");
    expect(click("DNND", 1)).toBe("DWNW");
    // after a want below, a done cell below is wanted too
    expect(click("WDNN", 2)).toBe("WWWN");
  });

  it("Want makes it and below done; done clears it and above", () => {
    expect(click("FWWW", 2)).toBe("FDDW");
    expect(click("FDDW", 1)).toBe("FNNN");
  });

  it("keeps locked fetched cells, and only those", () => {
    expect(click("FFNN", 0)).toBe("FFNN");
    expect(click("FFNN", 0, false)).toBe("NNNN");
    expect(click("DFWW", 0)).toBe("NFNN");
  });
});

describe("columns", () => {
  it("a column shows its least advanced row", () => {
    const g = grid("FN", "FN", "FN", "FN", "FN", "FN");
    expect(column_status(g, 0)).toBe(UpgradeStatus.FetchedDone);
    expect(column_status(g, 1)).toBe(UpgradeStatus.NotYet);
    g[0][0] = UpgradeStatus.Done;
    expect(column_status(g, 0)).toBe(UpgradeStatus.Done);
    g[1][0] = UpgradeStatus.Want;
    expect(column_status(g, 0)).toBe(UpgradeStatus.Want);
  });

  it("a header click moves only the rows at the column's status", () => {
    const g = grid("DDN", "NNN", "WWN", "NNN", "NNN", "NNN");
    click_column(g, 1, true);
    expect(g.map(text).slice(0, 3)).toEqual(["DDN", "WWN", "WWN"]);
    expect(text(g[6])).toBe("NNNN");
    click_column(g, 1, true);
    expect(g.map(text).slice(0, 3)).toEqual(["DDN", "DDN", "DDN"]);
    click_column(g, 1, true);
    expect(g.map(text).slice(0, 3)).toEqual(["DNN", "DNN", "DNN"]);
  });

  it("a header click over locked fetched cells does nothing", () => {
    const g = grid("FW", "FW", "FW", "FW", "FW", "FW");
    click_column(g, 0, true);
    expect(g.slice(0, 6).map(text)).toEqual(Array(6).fill("FW"));
    click_column(g, 0, false);
    expect(g.slice(0, 6).map(text)).toEqual(Array(6).fill("NN"));
  });
});
