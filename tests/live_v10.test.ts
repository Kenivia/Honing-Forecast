// Loads the real current-version save in tests/V10 config: no migration runs, so it must
// come back as stored, and survive load, save, load again.
import fs from "node:fs";
import { check, done } from "./helpers";
import LZString from "lz-string";
import {
  export_config,
  import_config,
  load_roster_config,
  STORAGE_KEY,
  write_state,
} from "@/Stores/ConfigStorage";
import { to_saved } from "@/Stores/SavedConfig";

const FIXTURE = "tests/V10 config";
if (!fs.existsSync(FIXTURE)) {
  console.log(`  skipped: ${FIXTURE} is not present`);
  process.exit(0);
}
const compressed = fs.readFileSync(FIXTURE, "utf8");
const v10 = JSON.parse(LZString.decompressFromUTF16(compressed));
const { version: _v, ...stored } = v10;

localStorage.clear();
localStorage.setItem(STORAGE_KEY, compressed);
const loaded = load_roster_config();
const once = to_saved(loaded);

check("profiles kept", loaded.profiles.length === v10.profiles.length);
check("loads as stored", JSON.stringify(once) === JSON.stringify(stored));
check(
  "export -> import is lossless",
  JSON.stringify(to_saved(import_config(export_config(loaded)))) ===
    JSON.stringify(once),
);

write_state({ roster_config: loaded });
const second = load_roster_config();
check(
  "second load is identical",
  JSON.stringify(to_saved(second)) === JSON.stringify(once),
);
write_state({ roster_config: second });
check(
  "third load is identical",
  JSON.stringify(to_saved(load_roster_config())) === JSON.stringify(once),
);

localStorage.removeItem(STORAGE_KEY);
done();
