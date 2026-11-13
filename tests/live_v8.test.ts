// Runs the migrations from V8 against the real save in tests/v8 config, whose Serca book
// prices the V7 migration had zeroed.
import fs from "node:fs";
import { check, differences, done } from "./helpers";
import LZString from "lz-string";
import { bundle_key, FALLBACK_PRICES } from "@/Utils/Constants";
import {
  load_roster_config,
  migrate_to_current,
  STORAGE_KEY,
} from "@/Stores/ConfigStorage";
import { from_saved, to_saved } from "@/Stores/SavedConfig";
import { input_column_to_num } from "@/Utils/InputColumn";

const FIXTURE = "tests/v8 config";
if (!fs.existsSync(FIXTURE)) {
  console.log(`  skipped: ${FIXTURE} is not present`);
  process.exit(0);
}
const compressed = fs.readFileSync(FIXTURE, "utf8");
const v8 = JSON.parse(LZString.decompressFromUTF16(compressed));

// the chain up to the current version leaves nothing to a load fallback
const migrated = migrate_to_current(structuredClone(v8), 8);
const { version: _m, ...expected } = migrated;
const fell_back = differences(
  to_saved(from_saved(structuredClone(migrated))),
  expected,
);
check(
  "nothing from a fallback",
  fell_back.length === 0,
  fell_back.slice(0, 10).join("; "),
);

localStorage.clear();
localStorage.setItem("HF_CONFIG_V8_COMPRESSED", compressed);
const loaded = load_roster_config();

check(
  "V8 key consumed",
  localStorage.getItem("HF_CONFIG_V8_COMPRESSED") === null,
);
check("profiles kept", loaded.profiles.length === v8.profiles.length);

const books = [
  "Enhanced 19-20 Armor",
  "Enhanced 19-20 Weapon",
  "12-15 Armor",
  "12-15 Weapon",
  "16-19 Armor",
  "16-19 Weapon",
].map((l) => bundle_key(l, 1));

for (const region of Object.keys(v8.mats_prices)) {
  const old = v8.mats_prices[region].values;
  const got = input_column_to_num(loaded.mats_prices[region], true);
  const bad = books.filter(
    (k) => got[k] !== (old[k] === 0 ? FALLBACK_PRICES[k] : old[k]),
  );
  check(
    `  ${region}: zeroed books reset, others kept`,
    bad.length === 0,
    bad.map((k) => `${k}: ${got[k]}`).join("; "),
  );
  const others = Object.keys(old).filter((k) => !books.includes(k));
  check(
    `  ${region}: every other price untouched`,
    others.every((k) => got[k] === old[k]),
  );
}

// apart from the book prices and V11's added fields, the save comes through unchanged
const again = to_saved(loaded);
for (const region of Object.keys(v8.mats_prices)) {
  for (const k of books) delete again.mats_prices[region].values[k];
  for (const k of books) delete v8.mats_prices[region].values[k];
}
const { version: _v, ...rest } = v8;
// roster ids are uuids since V10; the old ones put back by position, which this save keeps
const profiles = again.profiles.map((p, i) => ({
  ...p,
  roster_id: rest.profiles[i].roster_id,
}));
const want = rest.profiles.map((p) => ({
  ...p,
  income_auto: false,
  income_sources: {},
}));
const changed = [
  ...differences(profiles, want),
  ...differences(again.mats_prices, rest.mats_prices),
];
check(
  "the rest of the save unchanged",
  changed.length === 0,
  changed.slice(0, 10).join("; "),
);

localStorage.removeItem(STORAGE_KEY);
done();
