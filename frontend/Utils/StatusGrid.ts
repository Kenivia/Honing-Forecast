import { BoolGrid, StatusGrid, UpgradeStatus } from "./Interfaces"

export function status_to_bool_grid(status_grid: StatusGrid): BoolGrid {
    return status_grid.map((row: UpgradeStatus[]) => row.map((cell) => cell == UpgradeStatus.Want))
}
export function createStatusGrid(
    rows: number,
    cols: number,
    data: UpgradeStatus[][] = Array.from({ length: rows }, () => Array(cols).fill(UpgradeStatus.NotYet)),
): StatusGrid {
    if (!data) {
        data = Array.from({ length: rows }, () => Array(cols).fill(UpgradeStatus.NotYet))
    }
    return data
}

export function is_enum<T extends object>(targetEnum: T, inp: unknown): inp is T[keyof T] {
    return Object.values(targetEnum)
        .filter((v) => typeof v === "number")
        .includes(inp as number)
}

// returning instead of mutating in place because javascript
export function get_valid_status_grid(status_grid: StatusGrid, example: StatusGrid) {
    const isValid =
        status_grid.length === example.length &&
        example.every((row, i) => row.length === status_grid[i].length && status_grid[i].every((cell) => is_enum(UpgradeStatus, cell)))
    // console.log(isValid, status_grid, example)
    return isValid ? status_grid : example.slice()
}
