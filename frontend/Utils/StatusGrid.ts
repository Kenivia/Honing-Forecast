import { ADV_COLS, NUM_ADV_PIECES } from "./Constants";
import { StatusGrid, UpgradeStatus } from "./KeyedUpgrades";

export function create_status_grid(
  rows: number,
  cols: number,
  tier: number,
  adv: boolean,
): StatusGrid {
  // console.log(
  //   "create",
  //   Array.from({ length: rows }).map((_, row) =>
  //     Array(cols)
  //       .fill(UpgradeStatus.NotYet)
  //       .map((_, col) =>
  //         row === NUM_ADV_PIECES
  //           ? UpgradeStatus.NotYet
  //           : col < (tier === 0 ? (adv ? 0 : 10) : adv ? ADV_COLS : 11)
  //             ? UpgradeStatus.FetchedDone
  //             : UpgradeStatus.NotYet,
  //       ),
  //   ),
  // );
  return Array.from({ length: rows }).map((_, row) =>
    Array(cols)
      .fill(UpgradeStatus.NotYet)
      .map((_, col) =>
        row === NUM_ADV_PIECES
          ? UpgradeStatus.NotYet
          : col < (tier === 0 ? (adv ? 0 : 10) : adv ? ADV_COLS : 11)
            ? UpgradeStatus.FetchedDone
            : UpgradeStatus.NotYet,
      ),
  );
}

export function is_enum<T extends object>(
  targetEnum: T,
  inp: unknown,
): inp is T[keyof T] {
  return Object.values(targetEnum)
    .filter((v) => typeof v === "number")
    .includes(inp as number);
}

// returning instead of mutating in place because javascript
export function get_valid_status_grid(
  status_grid: StatusGrid,
  example: StatusGrid,
) {
  const isValid =
    status_grid.length === example.length &&
    example.every(
      (row, i) =>
        row.length === status_grid[i].length &&
        status_grid[i].every((cell) => is_enum(UpgradeStatus, cell)),
    );
  // console.log(isValid, status_grid, example)
  return isValid ? status_grid : example.slice();
}
