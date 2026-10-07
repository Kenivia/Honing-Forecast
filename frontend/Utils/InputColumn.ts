// A column of numeric inputs, keyed by material label. Display order is not stored here:
// callers iterate the labels they want to show, which comes from Constants.
export interface InputColumn {
  type: InputType;
  values: Record<string, string>; // locale-formatted, as typed
  upper_bound: Record<string, number>;
  enabled: Record<string, boolean>;
}
export enum InputType {
  Int,
  Float,
}

export interface ColumnInit {
  value?: (label: string) => string;
  upper_bound?: (label: string) => number;
  enabled?: (label: string) => boolean;
}

export function create_input_column(
  type: InputType,
  labels: string[],
  init: ColumnInit = {},
): InputColumn {
  const out: InputColumn = {
    type,
    values: {},
    upper_bound: {},
    enabled: {},
  };
  for (const label of labels) {
    out.values[label] = init.value?.(label) ?? "0";
    out.upper_bound[label] = init.upper_bound?.(label) ?? 999999999;
    out.enabled[label] = init.enabled?.(label) ?? true;
  }
  return out;
}

export const column_labels = (column: InputColumn): string[] =>
  Object.keys(column.values);

const parts = new Intl.NumberFormat().formatToParts(1234567.89);
export const LOCALE_GROUP = parts.find((p) => p.type === "group")?.value ?? ",";
export const LOCALE_DECIMAL =
  parts.find((p) => p.type === "decimal")?.value ?? ".";

function normalize_locale(str: string): string {
  return str
    .replace(/[^\d,.]/g, "")
    .replaceAll(LOCALE_GROUP, "")
    .replace(LOCALE_DECIMAL, ".");
}

function has_arithmetic(str: string): boolean {
  return /[+\-*/()]/.test(str);
}

function parse_arithmetic(
  expr: string,
  parseNum: (s: string) => number,
): number {
  const tokens = expr.replace(/\s+/g, "").match(/[\d.,]+|[+\-*/()]/g) ?? [];
  let pos = 0;

  const consume = () => tokens[pos++];
  const peek = () => tokens[pos];

  function parseExpr(): number {
    let left = parseTerm();
    while (peek() === "+" || peek() === "-") {
      const op = consume();
      const right = parseTerm();
      left = op === "+" ? left + right : left - right;
    }
    return left;
  }

  function parseTerm(): number {
    let left = parseAtom();
    while (peek() === "*" || peek() === "/") {
      const op = consume();
      const right = parseAtom();
      left = op === "*" ? left * right : left / right;
    }
    return left;
  }

  function parseAtom(): number {
    const tok = consume();
    if (tok === "(") {
      const val = parseExpr();
      consume(); // ')'
      return val;
    }
    return parseNum(tok);
  }

  return parseExpr();
}

export function parse_locale_int(str: string): number {
  if (has_arithmetic(str)) {
    return Math.trunc(
      parse_arithmetic(str, (s) => parseFloat(normalize_locale(s))),
    );
  }
  return parseInt(normalize_locale(str));
}

export function parse_locale_float(str: string): number {
  if (has_arithmetic(str)) {
    return parse_arithmetic(str, (s) => parseFloat(normalize_locale(s)));
  }
  return parseFloat(normalize_locale(str));
}

export function parse_input(
  column: InputColumn,
  label: string,
  input: string,
  pretend_enabled?: boolean,
): number {
  if (!column.enabled[label] && !pretend_enabled) {
    return 999999999;
  }
  const out =
    column.type === InputType.Int
      ? parse_locale_int(input)
      : parse_locale_float(input);
  return isFinite(out) ? Math.min(column.upper_bound[label], out) : 0;
}

export function input_column_to_num(
  column: InputColumn,
  pretend_enabled?: boolean,
): Record<string, number> {
  const out: Record<string, number> = {};
  for (const [label, value] of Object.entries(column.values)) {
    out[label] = parse_input(column, label, value, pretend_enabled);
  }
  return out;
}

// Rust indexes material arrays by row, so ordered arrays are built only at that boundary.
export function column_to_array(
  column: InputColumn,
  labels: string[],
  pretend_enabled?: boolean,
): number[] {
  return labels.map((label) =>
    parse_input(column, label, column.values[label], pretend_enabled),
  );
}

export function set_cell(column: InputColumn, label: string, value: string) {
  column.values[label] = parse_input(column, label, value, true).toLocaleString();
}

export function get_modified_cell(
  column: InputColumn,
  label: string,
  event: Event,
) {
  if (!column.enabled[label]) {
    return column.values[label];
  }
  return parse_input(
    column,
    label,
    (event.target as HTMLInputElement).value,
    true,
  ).toLocaleString();
}
