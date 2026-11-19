// Runs the V12 -> current migrations on the real V12 export in tests/v12 config.json (plain JSON,
// as the site's export writes it): it only adds hide_free_tap_hint (V13), imports and loads with
// nothing from a fallback, and survives load, save, load again.
import fs from "node:fs";
import { check, differences, done, section } from "./helpers";
import LZString from "lz-string";
import {
  export_config,
  import_config,
  load_roster_config,
  migrate_to_current,
  STORAGE_KEY,
  write_state,
} from "@/Stores/ConfigStorage";
import { to_saved } from "@/Stores/SavedConfig";

const V12_KEY = "HF_CONFIG_V12_COMPRESSED";
const fixture = "tests/v12 config.json";
if (!fs.existsSync(fixture)) {
  console.log("  skipped: no tests/v12 config.json present");
  process.exit(0);
}
const show = (d: string[]) => d.slice(0, 10).join("; ");

section(fixture);
const text = fs.readFileSync(fixture, "utf8");
const v12 = JSON.parse(text);
check("is a V12 save", v12.version === 12);

const migrated = migrate_to_current(structuredClone(v12), 12);
const added = differences(migrated, v12).filter((p) => p !== ".version");
check(
  "adds only hide_free_tap_hint",
  added.every((p) => p === ".hide_free_tap_hint") &&
    migrated.hide_free_tap_hint === false,
  show(added),
);

// an old save's choice_seed is dropped on load
const { version: _v, ...expected } = migrated;
const imported = to_saved(import_config(text));
const import_fell_back = differences(imported, expected).filter(
  (p) => !p.endsWith(".choice_seed"),
);
check(
  "imports as migrated, nothing from a fallback",
  import_fell_back.length === 0,
  show(import_fell_back),
);

localStorage.clear();
localStorage.setItem(V12_KEY, LZString.compressToUTF16(text));
const loaded = load_roster_config();
const once = to_saved(loaded);

check("V12 key consumed", localStorage.getItem(V12_KEY) === null);
check("profiles kept", loaded.profiles.length === v12.profiles.length);
check("snapshots kept", loaded.snapshots.length === v12.snapshots.length);
check(
  "loads the same as the import",
  JSON.stringify(once) === JSON.stringify(imported),
);
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
