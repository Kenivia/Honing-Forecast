import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { subset_numbers } from "@/Components/Character/InventoryScanner/FontSubset";

// the real HarfBuzz build, read from disk in place of the app's fetch
const load = async () =>
  (
    await WebAssembly.instantiate(
      readFileSync("node_modules/harfbuzzjs/dist/harfbuzz-subset.wasm"),
    )
  ).instance.exports;

// table tags of an sfnt's table directory
const tables = (font: Uint8Array) => {
  const view = new DataView(font.buffer, font.byteOffset);
  return Array.from({ length: view.getUint16(4) }, (_, i) =>
    String.fromCharCode(...font.subarray(12 + 16 * i, 16 + 16 * i)),
  );
};

describe("subset_numbers", () => {
  it("keeps a font the reader can use: cmap, outlines and advances", async () => {
    const font = new Uint8Array(readFileSync("public/fonts/Korean.ttf"));
    const out = await subset_numbers(font, load);
    expect(out).not.toBeNull();
    expect(out.byteLength).toBeLessThanOrEqual(font.byteLength);
    expect(tables(out)).toEqual(
      expect.arrayContaining(["cmap", "glyf", "loca", "hmtx", "head", "hhea"]),
    );
  });

  it("gives null for something that is not a font", async () => {
    expect(await subset_numbers(new Uint8Array(100), load)).toBeNull();
  });
});
