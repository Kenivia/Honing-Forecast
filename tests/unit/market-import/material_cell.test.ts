import { describe, expect, it, vi } from "vitest";
import { reactive } from "vue";
import { mount } from "@vue/test-utils";
import MaterialCell from "@/Components/Common/MaterialCell.vue";
import { ALL_MATERIAL_LABELS } from "@/Utils/Constants";
import { create_input_column, InputType } from "@/Utils/InputColumn";

const column = () => create_input_column(InputType.Int, ALL_MATERIAL_LABELS);

describe("MaterialCell", () => {
  it("unticking a material unticks the materials it converts to or from", async () => {
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
    await w.find('input[type="checkbox"]').setValue(false);
    expect(col.enabled.Red).toBe(false);
    expect(col.enabled["Serca Red"]).toBe(false);
    expect(col.enabled.Blue).toBe(true);
    expect(callback).toHaveBeenCalledTimes(1);
  });

  it("the link works from the Serca side and skips labels the column lacks", async () => {
    const col = create_input_column(InputType.Int, ["Serca Fusion"]);
    const w = mount(MaterialCell, {
      props: {
        input_column: col,
        label: "Serca Fusion",
        show_label: true,
        setter: () => {},
      },
    });
    await w.find('input[type="checkbox"]').setValue(false);
    expect(col.enabled).toEqual({ "Serca Fusion": false });
  });

  it("hides the tick without a label or with hide_tick", () => {
    const props = { input_column: column(), label: "Red", setter: () => {} };
    expect(
      mount(MaterialCell, { props }).find('[type="checkbox"]').exists(),
    ).toBe(false);
    expect(
      mount(MaterialCell, {
        props: { ...props, show_label: true, hide_tick: true },
      })
        .find('[type="checkbox"]')
        .exists(),
    ).toBe(false);
  });

  it("evaluates arithmetic, clamps and hands the result to the setter", async () => {
    const col = column();
    col.upper_bound.Red = 1000;
    const setter = vi.fn();
    const w = mount(MaterialCell, {
      props: { input_column: col, label: "Red", setter },
    });
    const input = w.find('input[type="text"]');
    await input.setValue("2*(3+4)");
    expect(setter).toHaveBeenLastCalledWith("14");
    await input.setValue("5000");
    expect(setter).toHaveBeenLastCalledWith((1000).toLocaleString());
    expect((input.element as HTMLInputElement).value).toBe(
      (1000).toLocaleString(),
    );
  });

  it("a disabled cell is read-only", () => {
    const col = column();
    col.enabled.Red = false;
    const w = mount(MaterialCell, {
      props: { input_column: col, label: "Red", setter: () => {} },
    });
    expect(w.find('input[type="text"]').attributes("disabled")).toBeDefined();
  });

  it("follows the store when the source value changes", async () => {
    const col = reactive(column());
    const w = mount(MaterialCell, {
      props: { input_column: col, label: "Red", setter: () => {} },
    });
    col.values.Red = "55";
    await w.vm.$nextTick();
    expect((w.find("input").element as HTMLInputElement).value).toBe("55");
  });

  it("renders a result record as a formatted label", () => {
    const w = mount(MaterialCell, {
      props: {
        input_column: { Red: 1234.6, Blue: 0.456 },
        label: "Red",
        suffix: "Buy",
      },
    });
    expect(w.text()).toContain("1,235");
    expect(w.text()).toContain("Buy");
    const small = mount(MaterialCell, {
      props: { input_column: { Blue: 0.456 }, label: "Blue" },
    });
    expect(small.text()).toContain("0.46");
  });
});
