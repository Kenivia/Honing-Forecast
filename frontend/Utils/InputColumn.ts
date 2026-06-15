import { ALL_LABELS, TIER_LABELS } from "./Constants";

export interface InputColumn {
  data: string[];
  keys: string[];
  type: InputType;
  upper_bound: number[];
  enabled: boolean[];
}
export enum InputType {
  Int,
  Float,
}
export function create_input_column(
  type: InputType,
  keys: string[],
  data?: string[],
  upper_bound?: number[],
  enabled?: boolean[],
): InputColumn {
  return {
    type,
    keys,
    data: data ?? keys.map((_) => "0"),
    upper_bound: upper_bound ?? keys.map((_) => 999999999),
    enabled: enabled ?? keys.map((_) => true),
  };
}

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
  input_column: InputColumn,
  index: number,
  input: string,
  pretend_enabled?: boolean,
): number {
  if (!input_column.enabled[index] && !pretend_enabled) {
    return 999999999;
  }
  let out =
    input_column.type === InputType.Int
      ? parse_locale_int(input)
      : parse_locale_float(input);
  return isFinite(out) ? Math.min(input_column.upper_bound[index], out) : 0;
}
export function input_column_to_num(
  input_column: InputColumn,
  pretend_enabled?: boolean,
): number[] {
  return input_column.data.map((x: string, index: number) =>
    parse_input(input_column, index, x, pretend_enabled),
  );
}

export function get_modified_cell(
  input_column: InputColumn,
  index: number,
  event: Event,
) {
  if (!input_column.enabled[index]) {
    return input_column.data[index];
  }
  return parse_input(
    input_column,
    index,
    (event.target as HTMLInputElement).value,
    true,
  ).toLocaleString();
}

export function validate_input_column(
  old: InputColumn,
  default_example: InputColumn,
) {
  for (let row = 0; row < old.data.length; row++) {
    let correct_len = default_example.data.length;
    if (
      old.data.length !== correct_len ||
      old.keys.length !== correct_len ||
      old.upper_bound.length !== correct_len ||
      old.enabled.length !== correct_len
    ) {
      old = structuredClone(default_example);
    } else {
      old.data[row] = parse_input(
        old,
        row,
        old.data[row],
        true,
      ).toLocaleString();
    }
  }
}
export function validate_input_column_array(
  old: InputColumn[],
  example: InputColumn[],
) {
  for (let index = 0; index < old.length; index++) {
    validate_input_column(old[index], example[index]);
  }
  while (old.length < TIER_LABELS.length) {
    old.push(create_input_column(InputType.Int, ALL_LABELS[old.length]));
  }
}
