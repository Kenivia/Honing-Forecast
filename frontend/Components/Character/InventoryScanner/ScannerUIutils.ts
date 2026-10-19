// what a slot's border, or a currency, is coloured by its status
export const STATUS_COLORS = {
  Pending: "var(--text-muted)",
  Good: "var(--achieved)",
  NeedHover: "var(--series-fusion)",
  NeedTradability: "var(--series-fusion)",
  Error: "var(--warning)",
  Irrelevant: "var(--border-very-muted)",
};

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
