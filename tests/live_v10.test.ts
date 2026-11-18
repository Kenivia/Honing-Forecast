// Runs the V10 -> current migrations on the real V10 saves in tests/V10 config*: they add only
// the auto income fields (V11) and empty the scan pages for slot-by-slot ones (V12), nothing comes
// from a load fallback, and the result survives load, save, load again.
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

const V10_KEY = "HF_CONFIG_V10_COMPRESSED";
const fixtures = ["tests/V10 config", "tests/V10 config 2"].filter((f) =>
  fs.existsSync(f),
);
if (fixtures.length === 0) {
  console.log("  skipped: no tests/V10 config present");
  process.exit(0);
}
const show = (d: string[]) => d.slice(0, 10).join("; ");

for (const fixture of fixtures) {
  section(fixture);
  const compressed = fs.readFileSync(fixture, "utf8");
  const v10 = JSON.parse(LZString.decompressFromUTF16(compressed));

  const migrated = migrate_to_current(structuredClone(v10), 10);
  const added = differences(migrated, v10).filter((p) => p !== ".version");
  check(
    "adds only income_auto, income_sources, forbidden_chests, class_name, fetched_at, alt leftover values, new scan pages and show_suggestions",
    added.every((p) =>
      /^\.profiles\.\d+\.(income_auto|income_sources|forbidden_chests|class_name|fetched_at|band_values\.\d)$|^\.scan_(pages|currencies)(\.|$)|^\.show_suggestions$/.test(
        p,
      ),
    ),
    show(added),
  );
  check(
    "  each roster's highest ilevel stays main, the rest on main's values become alts",
    main_alt_ok(v10.profiles, migrated.profiles),
  );
  check(
    "  scan pages emptied",
    migrated.scan_pages.length === 0 && migrated.scan_currencies.length === 0,
  );
  check(
    "  to every profile, at their defaults",
    migrated.version === STORAGE_VERSION &&
      migrated.profiles.every(
        (p: any) =>
          p.income_auto === false &&
          JSON.stringify(p.income_sources) === "{}" &&
          p.forbidden_chests.length === 0 &&
          p.class_name === "",
      ),
  );

  localStorage.clear();
  localStorage.setItem(V10_KEY, compressed);
  const loaded = load_roster_config();
  const once = to_saved(loaded);
  const { version: _v, ...expected } = migrated;

  check("V10 key consumed", localStorage.getItem(V10_KEY) === null);
  check("profiles kept", loaded.profiles.length === v10.profiles.length);
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
}

done();
