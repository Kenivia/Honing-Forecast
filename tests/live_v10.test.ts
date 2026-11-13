// Runs the V10 -> V11 migration on the real V10 saves in tests/V10 config*: it adds only the
// auto income fields, nothing comes from a load fallback, and the result survives load, save,
// load again.
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
    "V11 adds only income_auto and income_sources",
    added.every((p) => /^\.profiles\.\d+\.income_(auto|sources)$/.test(p)),
    show(added),
  );
  check(
    "  to every profile, at their defaults",
    migrated.version === 11 &&
      migrated.profiles.every(
        (p: any) =>
          p.income_auto === false && JSON.stringify(p.income_sources) === "{}",
      ),
  );

  localStorage.clear();
  localStorage.setItem(V10_KEY, compressed);
  const loaded = load_roster_config();
  const once = to_saved(loaded);
  const { version: _v, ...expected } = migrated;

  check("V10 key consumed", localStorage.getItem(V10_KEY) === null);
  check("profiles kept", loaded.profiles.length === v10.profiles.length);
  const fell_back = differences(once, expected);
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
