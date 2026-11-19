// Minimal assertion helper. A suite exits non-zero if anything failed.
import { ALT_BANDS, MAIN_BANDS } from "@/Utils/Constants";
import { ilevel } from "@/Utils/HoningUtil";

let failures = 0;
let total = 0;

export function check(name: string, cond: boolean, extra = "") {
  total += 1;
  if (!cond) {
    failures += 1;
    console.log("  FAIL  " + name + (extra ? " :: " + extra : ""));
  } else {
    console.log("  ok    " + name);
  }
}

export function equal(name: string, got: unknown, want: unknown) {
  const a = JSON.stringify(got);
  const b = JSON.stringify(want);
  check(name, a === b, a === b ? "" : `${a} != ${b}`);
}

// Paths where `got` and `want` differ, at any depth, keys in any order.
export function differences(got: any, want: any, path = ""): string[] {
  if (
    typeof got !== "object" ||
    got === null ||
    typeof want !== "object" ||
    want === null
  ) {
    return JSON.stringify(got) === JSON.stringify(want) ? [] : [path];
  }
  const keys = new Set([...Object.keys(got), ...Object.keys(want)]);
  return [...keys].flatMap((k) => differences(got[k], want[k], `${path}.${k}`));
}

// The V12 migration's main / alt split, checked against the profiles before and after it: per
// roster the highest achieved ilevel (first on a tie) keeps its values, every other profile
// still on the main's values (or none) is on the alt's, the rest are untouched and marked manual.
export function main_alt_ok(before: any[], after: any[]): boolean {
  const same = (a: unknown, b: unknown) =>
    JSON.stringify(a) === JSON.stringify(b);
  const level = (p: any) =>
    Number(ilevel(p, "achieved").replace(/,/g, "")) || -Infinity;
  return before.every((p, i) => {
    const roster = before.filter((q) => q.roster_id === p.roster_id);
    const top = Math.max(...roster.map(level));
    const main = roster.find((q) => level(q) === top);
    const was = p.band_values ?? MAIN_BANDS;
    if (after[i].bands_manual !== !same(was, MAIN_BANDS)) return false;
    if (main === p || !same(was, MAIN_BANDS))
      return same(after[i].band_values ?? MAIN_BANDS, was);
    return same(after[i].band_values, ALT_BANDS);
  });
}

export function section(name: string) {
  console.log(`\n=== ${name} ===`);
}

export function done(note = "") {
  console.log(
    failures === 0
      ? `\n${total} checks passed${note ? " — " + note : ""}`
      : `\n${failures} of ${total} checks FAILED`,
  );
  process.exit(failures === 0 ? 0 : 1);
}
