import { describe, expect, it, vi } from "vitest";
import "./helpers";
import { mount } from "@vue/test-utils";
import MaterialGraph from "@/Components/Character/MaterialDist/MaterialGraph.vue";
import { GRAPH_HEIGHT } from "@/Utils/Constants";

// jsdom lays nothing out, so the plot is its minimum width
const WIDTH = 180;
const DATA: [number, number][] = [
  [0, 0.2],
  [10, 0.6],
  [20, 1],
];

const graph = (props: Record<string, any>) =>
  mount(MaterialGraph, {
    props: { data: DATA, cumulative: true, materialLabel: "Red", ...props },
  });
const path_points = (d: string) =>
  [...d.matchAll(/[ML] ([\d.]+) ([\d.]+)/g)].map((m) => [+m[1], +m[2]]);
const scale_y = (y: number, max = 1) =>
  GRAPH_HEIGHT - (y / (max * 1.1)) * GRAPH_HEIGHT;

describe("MaterialGraph", () => {
  it("shows the empty message without data", () => {
    const w = graph({ data: null });
    expect(w.text()).toContain("This material is never used");
    expect(w.findAll("path")).toHaveLength(0);
    expect(graph({ data: [], empty_message: "None" }).text()).toContain("None");
  });

  it("draws the cumulative curve scaled to the width and the top value", () => {
    const d = graph({}).findAll("path")[1].attributes("d");
    const pts = path_points(d);
    expect(pts[0]).toEqual([0, GRAPH_HEIGHT]);
    expect(pts.slice(1).map(([x]) => x)).toEqual([0, WIDTH / 2, WIDTH]);
    expect(pts[3][1]).toBeCloseTo(scale_y(1));
    expect(pts[1][1]).toBeCloseTo(scale_y(0.2));
  });

  it("drops non-finite points", () => {
    const d = graph({
      data: [...DATA, [30, NaN]],
    })
      .findAll("path")[1]
      .attributes("d");
    expect(path_points(d)).toHaveLength(4);
  });

  it("draws steps when a cumulative value repeats", () => {
    const d = graph({
      data: [
        [0, 0],
        [10, 0.5],
        [20, 0.5],
        [30, 1],
      ],
    })
      .findAll("path")[1]
      .attributes("d");
    const xs = path_points(d).map(([x]) => x);
    expect(xs).toEqual([0, 0, 60, 60, 120, 120, 180]);
  });

  it("the hover snaps to the nearest point and passes its slope and cumulative value", async () => {
    const tip = vi.fn(() => "<b>tip</b>");
    const w = graph({ cumulative: false, tooltipTextFn: tip });
    await w.trigger("mousemove", { clientX: 80 });
    expect(tip).toHaveBeenLastCalledWith(
      10,
      expect.closeTo(0.04),
      0.6,
      "Red",
      "--text-main",
      false,
    );
    expect(w.find(".graph-tooltip").html()).toContain("<b>tip</b>");
    await w.trigger("mousemove", { clientX: 179 });
    expect(tip).toHaveBeenLastCalledWith(
      20,
      expect.closeTo(0.04),
      1,
      "Red",
      "--text-main",
      true,
    );
    await w.trigger("mouseleave");
    expect(w.find(".graph-tooltip").exists()).toBe(false);
  });

  it("an upside-down graph slopes the other way", async () => {
    const tip = vi.fn((..._args: any[]) => "");
    const w = graph({
      cumulative: false,
      upside_down: true,
      tooltipTextFn: tip,
      data: [
        [0, 0.8],
        [1, 0.3],
      ],
    });
    await w.trigger("mousemove", { clientX: 0 });
    expect(tip.mock.calls.at(-1)!.slice(0, 3)).toEqual([
      0,
      expect.closeTo(0.2),
      0.8,
    ]);
    await w.trigger("mousemove", { clientX: WIDTH });
    expect(tip.mock.calls.at(-1)![1]).toBeCloseTo(0.5);
  });

  it("the y override only applies to the cumulative graph", () => {
    const top = (props) =>
      path_points(graph(props).findAll("path")[1].attributes("d"))[3][1];
    expect(top({ maxYoverride: 2 })).toBeCloseTo(scale_y(1, 2));
    // non-cumulative, the top is the largest slope
    expect(top({ maxYoverride: 2, cumulative: false })).toBeCloseTo(
      scale_y(0.04, 0.04),
    );
  });

  it("places an annotation per value, its label kept inside the plot", () => {
    const w = graph({
      annotations: [0, 15],
      annotationColors: ["red", "blue"],
      annotationPositions: ["graph", "bottom"],
      annotationLabels: ["Average", "Bound"],
    });
    const circles = w.findAll("circle");
    expect(circles.map((c) => +c.attributes("cx")!)).toEqual([0, 135]);
    const texts = w.findAll("text");
    expect(texts.map((t) => t.text())).toEqual(["Average", "Bound"]);
    // "Average" is 35 wide, so its centre is pushed in
    expect(+texts[0].attributes("x")!).toBe(17.5);
    expect(+texts[1].attributes("y")!).toBe(GRAPH_HEIGHT);
    // a null one is skipped, the next keeps its own label
    const skipped = graph({
      annotations: [null, 15],
      annotationColors: ["red", "blue"],
      annotationLabels: ["Average", "Bound"],
    });
    expect(skipped.findAll("text").map((t) => t.text())).toEqual(["Bound"]);
    expect(skipped.findAll("circle").map((c) => +c.attributes("cx")!)).toEqual([
      135,
    ]);
    expect(
      graph({ data: null, annotations: [1] }).findAll("circle"),
    ).toHaveLength(0);
  });
});
