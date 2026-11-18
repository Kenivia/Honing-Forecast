// Cuts a picked font down to what a slot's number uses, as scanner_setup/game_files/fonts.py does
// for the game's fonts, with HarfBuzz's subsetter (loaded only when a font is picked).
import wasm_url from "harfbuzzjs/dist/harfbuzz-subset.wasm?url";

export const NUMBER_CHARS = "0123456789+";

let hb: Promise<any> | null = null;
const load = () =>
  (hb ??= fetch(wasm_url)
    .then((response) => response.arrayBuffer())
    .then((bytes) => WebAssembly.instantiate(bytes))
    .then(({ instance }) => instance.exports));

// null if HarfBuzz could not subset it
export async function subset_numbers(
  font: Uint8Array,
  load_wasm: () => Promise<any> = load,
): Promise<Uint8Array | null> {
  const hb = await load_wasm();
  const ptr = hb.malloc(font.byteLength);
  new Uint8Array(hb.memory.buffer).set(font, ptr);
  const blob = hb.hb_blob_create(ptr, font.byteLength, 2, 0, 0);
  const face = hb.hb_face_create(blob, 0);
  hb.hb_blob_destroy(blob);
  const input = hb.hb_subset_input_create_or_fail();
  const unicodes = hb.hb_subset_input_unicode_set(input);
  for (const c of NUMBER_CHARS) hb.hb_set_add(unicodes, c.codePointAt(0));
  // no hinting and no layout features, which would pull in other glyphs (fonts.py does the same)
  hb.hb_subset_input_set_flags(input, 1); // HB_SUBSET_FLAGS_NO_HINTING
  hb.hb_set_clear(hb.hb_subset_input_set(input, 6)); // HB_SUBSET_SETS_LAYOUT_FEATURE_TAG
  const subset = hb.hb_subset_or_fail(face, input);
  hb.hb_subset_input_destroy(input);
  let out: Uint8Array | null = null;
  if (subset) {
    const result = hb.hb_face_reference_blob(subset);
    const at = hb.hb_blob_get_data(result, 0);
    const length = hb.hb_blob_get_length(result);
    // copied out: the view dies with the memory
    if (length > 0)
      out = new Uint8Array(hb.memory.buffer).slice(at, at + length);
    hb.hb_blob_destroy(result);
    hb.hb_face_destroy(subset);
  }
  hb.hb_face_destroy(face);
  hb.free(ptr);
  return out;
}
