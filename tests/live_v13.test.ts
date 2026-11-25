// Runs the V13 -> current migration on the real V13 export in tests/v13 config.json (plain JSON,
// as the site's export writes it): it adds roster_names and unnames default Newchar names (V14),
// imports and loads with nothing from a fallback, and survives load, save, load again.
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

const V13_KEY = "HF_CONFIG_V13_COMPRESSED";
const fixture = "tests/v13 config.json";
if (!fs.existsSync(fixture)) {
  console.log("  skipped: no tests/v13 config.json present");
  process.exit(0);
}
const show = (d: string[]) => d.slice(0, 10).join("; ");

section(fixture);
const text = fs.readFileSync(fixture, "utf8");
const v13 = JSON.parse(text);
check("is a V13 save", v13.version === 13);

const migrated = migrate_to_current(structuredClone(v13), 13);
const added = differences(migrated, v13).filter((p) => p !== ".version");
check(
  "adds only roster_names and the sidereal weapon fields",
  added.every(
    (p) =>
      p === ".roster_names" ||
      /^\.profiles\.\d+\.(sidereal_level|show_weapon_honing)$/.test(p),
  ) && JSON.stringify(migrated.roster_names) === "{}",
  show(added),
);

// default names become unnamed; anything else is kept
const planted = structuredClone(v13);
planted.profiles[0].char_name = "Newchar";
planted.profiles[1].char_name = "Newchar12";
planted.profiles[2].char_name = "Newcharlie";
const unnamed = migrate_to_current(planted, 13).profiles.map(
  (p: any) => p.char_name,
);
check(
  "Newchar names become unnamed, only those",
  JSON.stringify(unnamed.slice(0, 4)) ===
    JSON.stringify(["", "", "Newcharlie", v13.profiles[3].char_name]),
  show(unnamed.slice(0, 4)),
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
localStorage.setItem(V13_KEY, LZString.compressToUTF16(text));
const loaded = load_roster_config();
const once = to_saved(loaded);

check("V13 key consumed", localStorage.getItem(V13_KEY) === null);
check("profiles kept", loaded.profiles.length === v13.profiles.length);
check("snapshots kept", loaded.snapshots.length === v13.snapshots.length);
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
