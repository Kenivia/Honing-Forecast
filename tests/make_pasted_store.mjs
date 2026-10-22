// Turns a raw HF_CONFIG_V8_COMPRESSED string into a devtools-console snippet that loads it.
// Usage: node tests/make_pasted_store.mjs ["tests/v8 config"] ["ignored/pasted store"]
import fs from "fs";
import LZString from "lz-string";

const [input = "tests/v8 config", output = "ignored/pasted store"] = process.argv.slice(2);
const raw = fs.readFileSync(input, "utf8").replace(/\r?\n$/, "");
if (!LZString.decompressFromUTF16(raw)) throw new Error("input does not decompress");

fs.writeFileSync(
  output,
  `localStorage.clear();
localStorage.setItem("HF_CONFIG_V8_COMPRESSED", ${JSON.stringify(raw)});
window.addEventListener("pagehide", e => e.stopImmediatePropagation(), true);
location.reload();
`,
);
console.log(`wrote ${output}`);
