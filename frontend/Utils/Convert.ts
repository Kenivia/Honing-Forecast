import { CONVERTIBLE_MATERIALS } from "./Constants";
import { input_column_to_num, InputColumn, set_cell } from "./InputColumn";

// Converts a roster's bound and tradable stock up a tier, pooled, in whole conversions.
// What comes out is roster-bound; roster-bound stock is spent first, the leftover stays.
export function convert_pooled(roster: InputColumn, tradable: InputColumn) {
  const r = input_column_to_num(roster, true);
  const t = input_column_to_num(tradable, true);
  for (const mat of CONVERTIBLE_MATERIALS) {
    const { label: source, ratio } = mat.from;
    if (!(source in roster.values)) continue;
    const gained = Math.floor((r[source] + t[source]) / ratio);
    const from_roster = Math.min(r[source], gained * ratio);
    set_cell(roster, mat.label, String(r[mat.label] + gained));
    set_cell(roster, source, String(r[source] - from_roster));
    set_cell(
      tradable,
      source,
      String(t[source] - (gained * ratio - from_roster)),
    );
  }
}

// e.g. "5 Red to 1 Serca Red"
export const CONVERT_SUMMARY = CONVERTIBLE_MATERIALS.map(
  (mat) => `${mat.from.ratio} ${mat.from.label} to 1 ${mat.label}`,
).join(", ");
