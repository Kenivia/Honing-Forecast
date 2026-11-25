// Runs the V11 -> current migrations on the real V11 save in tests/V11 config: they only empty the
// scan pages for slot-by-slot ones and add forbidden chests (V12), nothing comes from a load
// fallback, and the result survives load, save, load again.
import fs from "node:fs";
import { check, differences, done, main_alt_ok, section } from "./helpers";
import LZString from "lz-string";
import {
  export_config,
  import_config,
  load_roster_config,
  migrate_to_current,
  STORAGE_KEY,
  STORAGE_VERSION,
  write_state,
} from "@/Stores/ConfigStorage";
import { to_saved } from "@/Stores/SavedConfig";

const V11_KEY = "HF_CONFIG_V11_COMPRESSED";
const fixture = "tests/V11 config";
if (!fs.existsSync(fixture)) {
  console.log("  skipped: no tests/V11 config present");
  process.exit(0);
}
const show = (d: string[]) => d.slice(0, 10).join("; ");

section(fixture);
const compressed = fs.readFileSync(fixture, "utf8");
const v11 = JSON.parse(LZString.decompressFromUTF16(compressed));
check("is a V11 save", v11.version === 11);

const migrated = migrate_to_current(structuredClone(v11), 11);
const added = differences(migrated, v11).filter((p) => p !== ".version");
check(
  "adds only forbidden_chests, class_name, fetched_at, sidereal_level, bands_manual, alt leftover values, new scan pages, show_suggestions, hide_free_tap_hint and roster_names",
  added.every((p) =>
    /^\.profiles\.\d+\.(forbidden_chests|class_name|fetched_at|sidereal_level|bands_manual|band_values\.\d)$|^\.scan_(pages|currencies)(\.|$)|^\.(show_suggestions|hide_free_tap_hint|roster_names)$/.test(
      p,
    ),
  ),
  show(added),
);
check(
  "  each roster's highest ilevel stays main, the rest on main's values become alts",
  main_alt_ok(v11.profiles, migrated.profiles),
);
check(
  "  scan pages emptied",
  migrated.scan_pages.length === 0 && migrated.scan_currencies.length === 0,
);
check(
  "  forbidden chests empty and no class on every profile",
  migrated.version === STORAGE_VERSION &&
    migrated.profiles.every(
      (p: any) => p.forbidden_chests.length === 0 && p.class_name === "",
    ),
);

localStorage.clear();
localStorage.setItem(V11_KEY, compressed);
const loaded = load_roster_config();
const once = to_saved(loaded);
const { version: _v, ...expected } = migrated;

check("V11 key consumed", localStorage.getItem(V11_KEY) === null);
check("profiles kept", loaded.profiles.length === v11.profiles.length);
check("snapshots kept", loaded.snapshots.length === v11.snapshots.length);
// an old save's choice_seed is dropped on load
const fell_back = differences(once, expected).filter(
  (p) => !p.endsWith(".choice_seed"),
);
check(
  "loads as migrated, nothing from a fallback",
  fell_back.length === 0,
  show(fell_back),
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
