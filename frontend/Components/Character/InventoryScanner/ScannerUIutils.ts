import { ref } from "vue";

// uploads and debug captures, on by default on a dev build
export const show_debug = ref(import.meta.env.DEV);

// what a slot's border, or a currency, is coloured by its status
export const STATUS_COLORS = {
  Pending: "var(--text-muted)",
  Good: "var(--achieved)",
  NeedHover: "var(--series-fusion)",
  NeedTradability: "var(--series-fusion)",
  Error: "var(--warning)",
  Irrelevant: "var(--border-very-muted)",
};

export const STATUS_NAMES = {
  Pending: "Reading",
  Good: "Good",
  NeedHover: "Needs hover",
  NeedTradability: "Needs hover",
  Error: "Error",
  Irrelevant: "Irrelevant",
};

const BORDER_WIDTHS = {
  Pending: "1px",
  Good: "1px",
  NeedHover: "3px",
  NeedTradability: "3px",
  Error: "3px",
  Irrelevant: "1px",
};

// a slot's or currency's outline, dashed while it waits on a hover or was set by hand
export function status_border(status: string, edited = false) {
  return {
    borderColor: edited ? "var(--average)" : STATUS_COLORS[status],
    borderWidth: edited ? "2px" : BORDER_WIDTHS[status],
    borderStyle:
      edited || status === "NeedHover" || status === "NeedTradability"
        ? "dashed"
        : "solid",
  };
}

export function draw_icon(
  canvas: HTMLCanvasElement | null,
  data: ArrayLike<number>,
  width: number,
  height: number,
) {
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  // `icon.data` may arrive as a plain number array or a typed array
  // depending on serde-wasm-bindgen's serialization; both work here.
  const clamped = new Uint8ClampedArray(data);
  const image_data = new ImageData(clamped, width, height);
  ctx.putImageData(image_data, 0, 0);
}
