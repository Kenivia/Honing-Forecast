// Minimal assertion helper. A suite exits non-zero if anything failed.
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
