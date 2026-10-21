import { beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { nextTick, reactive } from "vue";
import ChestPanels from "@/Components/Common/ChestPanels.vue";
import ChestIcon from "@/Components/Common/ChestIcon.vue";
import Popup from "@/Components/Common/Popup.vue";
import QuestionMark from "@/Components/Common/QuestionMark.vue";
import RegionSelector from "@/Components/Common/RegionSelector.vue";
import LabeledPieceIcon from "@/Components/Common/LabeledPieceIcon.vue";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { useRuntimeStore } from "@/Stores/RuntimeState";
import type { Chest } from "@/Utils/Chest";
import { create_input_column, InputType } from "@/Utils/InputColumn";

beforeEach(() => setActivePinia(createPinia()));

const chest = (key: string, extra: Partial<Chest> = {}): Chest => ({
  key,
  title: `${key} title`,
  band: 1,
  count: 3,
  options: [{ Red: 10 }, { Blue: 2.25, Gold: 1000 }],
  ...extra,
});

describe("ChestPanels", () => {
  const panels = (props = {}) =>
    mount(ChestPanels, {
      props: { selection: [chest("a")], obtain_all: [], ...props },
      slots: { default: "<i class='foot'>foot</i>" },
    });

  it("two panels, empty one says None, slot under selection", async () => {
    const w = panels({ obtain_all_title: "Opened" });
    // obtain-all is hidden until asked for
    expect(w.findAll("[role=group]")).toHaveLength(1);
    await w.find("label input").setValue(true);
    const groups = w.findAll("[role=group]");
    expect(groups.map((g) => g.attributes("aria-label"))).toEqual([
      "Selection chests",
      "Opened",
    ]);
    expect(groups[1].text()).toContain("None.");
    expect(groups[0].find(".foot").exists()).toBe(true);
    expect(groups[1].find(".foot").exists()).toBe(false);
  });

  it("collapses from the header", async () => {
    const w = panels();
    await w.find("button[aria-expanded]").trigger("click");
    // kept in the layout so the card keeps its width
    expect(w.find("[inert]").find("[role=group]").exists()).toBe(true);
    expect(w.emitted("update:open")).toEqual([[false]]);
  });

  it("a chest row: count, band, options joined by or", () => {
    const row = panels().find("[role=group] .border-b");
    expect((row.find("input").element as HTMLInputElement).value).toBe("3");
    expect(row.text()).toContain("Roster bound");
    expect(row.text()).toMatch(/:\s*x10\s*or\s*x2\.3\s*x1,000/);
  });

  it("read-only unless editing", () => {
    const w = panels();
    expect(w.find("[role=group] input").attributes("disabled")).toBeDefined();
    expect(w.find("[role=group] button").exists()).toBe(false);
  });

  it("editing emits count and remove by key", async () => {
    const w = panels({ editing: true });
    await w.find("[role=group] input").setValue("5");
    await w.find("[role=group] button").trigger("click");
    expect(w.emitted("count")[0][0]).toBe("a");
    expect(w.emitted("remove")).toEqual([["a"]]);
  });
});

describe("ChestIcon", () => {
  const icon = (icons?: string[]) =>
    mount(ChestIcon, { props: { chest: chest("a", { icons }) } });

  it("draws each icon with the title on hover", () => {
    const imgs = icon(["all_quest_01_252", "all_quest_01_253"]).findAll("img");
    expect(imgs).toHaveLength(2);
    expect(imgs[0].attributes("src")).toContain("all_quest_01_252");
    expect(imgs[0].attributes("alt")).toBe("a title");
  });

  it("falls back to the title without art or with an unknown icon", () => {
    expect(icon().text()).toBe("a title");
    const w = icon(["all_quest_01_252", "nope"]);
    expect(w.find("img").exists()).toBe(false);
    expect(w.text()).toBe("a title");
  });
});

describe("Popup", () => {
  it("teleports to body and closes on overlay click only", async () => {
    const w = mount(Popup, {
      props: { show_popup: true },
      slots: { default: "<p class='inside'>hi</p>" },
      attachTo: document.body,
    });
    const overlay = document.body.querySelector(
      ".popup-overlay",
    ) as HTMLElement;
    (document.body.querySelector(".inside") as HTMLElement).click();
    expect(w.emitted("close_popup")).toBeUndefined();
    overlay.click();
    expect(w.emitted("close_popup")).toHaveLength(1);
    await w.setProps({ show_popup: false });
    expect(document.body.querySelector(".popup-overlay")).toBeNull();
    w.unmount();
  });
});

describe("QuestionMark", () => {
  it("renders one question mark with the passed class", () => {
    for (const position of ["none", "left", "bottom"] as const) {
      const w = mount(QuestionMark, {
        props: { text: "help", position },
        attrs: { class: "ml-2" },
      });
      expect(w.findAll(".question-mark")).toHaveLength(1);
      expect(w.classes()).toContain("ml-2");
    }
  });
});

describe("RegionSelector", () => {
  it("shows the region, calls back on change, locks while fetching", async () => {
    const region_change = vi.fn();
    const w = mount(RegionSelector, {
      props: { region: "euc", region_change },
    });
    const select = w.find("select");
    expect((select.element as HTMLSelectElement).value).toBe("euc");
    await select.setValue("Custom");
    expect(region_change).toHaveBeenCalledTimes(1);
    useRuntimeStore().is_fetching.euc = true;
    await nextTick();
    expect(select.attributes("disabled")).toBeDefined();
  });
});

describe("LabeledPieceIcon", () => {
  it("names the piece and draws its icon", () => {
    const w = mount(LabeledPieceIcon, { props: { piece: "Helmet" } });
    expect(w.text()).toBe("Helmet");
    expect(w.find("img").attributes("alt")).toBe("Helmet");
  });
});

describe("MaterialCell", () => {
  const column = () =>
    reactive(create_input_column(InputType.Int, ["Red", "Serca Red", "Blue"]));

  it("result mode formats numbers and percentages", () => {
    const text = (props) =>
      mount(MaterialCell, { props: { label: "Red", ...props } })
        .find(".material-cell-result")
        .text();
    expect(text({ input_column: { Red: 1234.6 } })).toBe("1,235");
    expect(text({ input_column: { Red: 0.456 } })).toBe("0.46");
    expect(text({ input_column: { Red: 0.1234 }, is_percentage: true })).toBe(
      "12.34%",
    );
  });

  it("input change parses, sets and calls back", async () => {
    const col = column();
    const setter = vi.fn((v: string) => (col.values.Red = v));
    const callback = vi.fn();
    const w = mount(MaterialCell, {
      props: { input_column: col, label: "Red", setter, callback },
    });
    const input = w.find("input[type=text]");
    await input.setValue("1000+234");
    expect(setter).toHaveBeenCalledWith("1,234");
    expect(callback).toHaveBeenCalled();
    expect((input.element as HTMLInputElement).value).toBe("1,234");
  });

  it("follows outside changes to the value", async () => {
    const col = column();
    const w = mount(MaterialCell, {
      props: { input_column: col, label: "Blue", setter: () => {} },
    });
    col.values.Blue = "77";
    await nextTick();
    expect((w.find("input[type=text]").element as HTMLInputElement).value).toBe(
      "77",
    );
  });

  it("the tick toggles the material and its tier-linked one", async () => {
    const col = column();
    const callback = vi.fn();
    const w = mount(MaterialCell, {
      props: {
        input_column: col,
        label: "Red",
        show_label: true,
        setter: () => {},
        callback,
      },
    });
    await w.find("input[type=checkbox]").setValue(false);
    expect(col.enabled).toEqual({ Red: false, "Serca Red": false, Blue: true });
    expect(callback).toHaveBeenCalled();
    expect(w.find("input[type=text]").attributes("disabled")).toBeDefined();
  });

  it("no tick when hidden or unlabeled", () => {
    const tick = (props) =>
      mount(MaterialCell, {
        props: {
          input_column: column(),
          label: "Red",
          setter: () => {},
          ...props,
        },
      })
        .find("input[type=checkbox]")
        .exists();
    expect(tick({ show_label: true })).toBe(true);
    expect(tick({ show_label: true, hide_tick: true })).toBe(false);
    expect(tick({})).toBe(false);
  });

  it("plus lines hide non-positive amounts but keep their space", () => {
    const w = mount(MaterialCell, {
      props: {
        input_column: { Red: 1 },
        label: "Red",
        plus: [
          { amount: 12.9, text: "from T4" },
          { amount: 0.5, text: "chests" },
        ],
      },
    });
    const lines = w.findAll(".annotation.whitespace-nowrap");
    expect(lines.map((l) => l.text())).toEqual(["+12 from T4", "+0 chests"]);
    expect(lines[1].classes()).toContain("invisible");
  });
});
