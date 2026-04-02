import { ALL_LABELS, TIER_LABELS } from "./Constants"
import { InputColumn, InputType } from "./Interfaces"

export function create_input_column(type: InputType, keys: string[], data?: string[], upper_bound?: number[], enabled?: boolean[]): InputColumn {
    return {
        type,
        keys,
        data: data ?? keys.map((_) => "0"),
        upper_bound: upper_bound ?? keys.map((_) => 999999999),
        enabled: enabled ?? keys.map((_) => true),
    }
}

const parts = new Intl.NumberFormat().formatToParts(1234567.89)
const group = parts.find((p) => p.type === "group")?.value ?? ","
const decimal = parts.find((p) => p.type === "decimal")?.value ?? "."

export function parse_locale_int(str: string): number {
    const normalized = str
        .replaceAll(group, "") // remove thousands separators
        .replace(decimal, ".") // normalize decimal separator to '.'

    return parseInt(normalized)
}

export function parse_locale_float(str: string, locale?: string): number {
    const normalized = str.replaceAll(group, "").replace(decimal, ".")

    return parseFloat(normalized)
}
export function parse_input(input_column: InputColumn, index: number, input: string, pretend_enabled?: boolean): number {
    if (!input_column.enabled[index] && !pretend_enabled) {
        return 999999999
    }

    let out = input_column.type === InputType.Int ? parse_locale_int(input) : parse_locale_float(input)
    // console.log(input_column.upper_bound)
    return isFinite(out) ? Math.min(input_column.upper_bound[index], out) : 0
}
export function input_column_to_num(input_column: InputColumn, pretend_enabled?: boolean): number[] {
    return input_column.data.map((x: string, index: number) => parse_input(input_column, index, x, pretend_enabled))
}

export function get_modified_cell(input_column: InputColumn, index: number, event: Event) {
    if (!input_column.enabled[index]) {
        return input_column.data[index]
    }
    return parse_input(input_column, index, (event.target as HTMLInputElement).value.replace(/[^\d,.]/g, ""), true).toLocaleString()
}

export function validate_input_column(old: InputColumn, default_example: InputColumn) {
    for (let row = 0; row < old.data.length; row++) {
        let correct_len = default_example.data.length
        if (
            old.data.length !== correct_len ||
            old.keys.length !== correct_len ||
            old.upper_bound.length !== correct_len ||
            old.enabled.length !== correct_len
        ) {
            old = structuredClone(default_example)
        } else {
            old.data[row] = parse_input(old, row, old.data[row], true).toLocaleString()
        }
    }
}
export function validate_input_column_array(old: InputColumn[], example: InputColumn[]) {
    for (let index = 0; index < old.length; index++) {
        validate_input_column(old[index], example[index])
    }
    while (old.length < TIER_LABELS.length) {
        old.push(create_input_column(InputType.Int, ALL_LABELS[old.length]))
    }
}
