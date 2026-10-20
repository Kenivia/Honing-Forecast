import { ADV_COLS, NORMAL_FLOOR, NUM_ADV_PIECES } from "./Constants";
import { StatusGrid, UpgradeStatus } from "./KeyedUpgrades";

export function create_status_grid(
  rows: number,
  cols: number,
  tier: number,
  adv: boolean,
): StatusGrid {
  return Array.from({ length: rows }).map((_, row) =>
    Array(cols)
      .fill(UpgradeStatus.NotYet)
      .map((_, col) =>
        row === NUM_ADV_PIECES
          ? UpgradeStatus.NotYet
          : col < (adv ? (tier === 0 ? 0 : ADV_COLS) : NORMAL_FLOOR[tier])
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
  status_grid: StatusGrid | undefined,
  example: StatusGrid,
): StatusGrid {
  const isValid =
    Array.isArray(status_grid) &&
    status_grid.length === example.length &&
    example.every(
      (row, i) =>
        Array.isArray(status_grid[i]) &&
        row.length === status_grid[i].length &&
        status_grid[i].every((cell) => is_enum(UpgradeStatus, cell)),
    );
  // cloned, not sliced: a shallow copy would share row arrays with the default
  return isValid ? status_grid : structuredClone(example);
}

const { Done, Want, NotYet, FetchedDone } = UpgradeStatus;
const level = (status: UpgradeStatus) =>
  status === NotYet ? 0 : status === Want ? 1 : 2;

// What clicking a cell does to its row. A locked fetched cell never changes.
export function click_cell(row: UpgradeStatus[], col: number, locked: boolean) {
  const set = (index: number, status: UpgradeStatus) => {
    if (!(locked && row[index] === FetchedDone)) row[index] = status;
  };
  const current = row[col];
  if (current === FetchedDone && locked) return;
  if (current === NotYet) {
    // wants this level, what is not done below it, everything after a want below it, and what was done above it
    let seen_want = false;
    for (let index = 0; index <= col; index++) {
      if (row[index] === Want) seen_want = true;
      if (seen_want || row[index] === NotYet) set(index, Want);
    }
    for (let index = col + 1; index < row.length; index++)
      if (level(row[index]) === 2) set(index, Want);
    set(col, Want);
  } else if (current === Want) {
    for (let index = 0; index <= col; index++) set(index, Done);
  } else {
    for (let index = col; index < row.length; index++) set(index, NotYet);
  }
}

// The status a whole column shows: the least advanced of its rows, the vambrace row ignored.
export function column_status(grid: StatusGrid, col: number): UpgradeStatus {
  const cells = grid
    .filter((_, row) => row !== NUM_ADV_PIECES)
    .map((row) => row[col]);
  if (cells.every((status) => status === FetchedDone)) return FetchedDone;
  return [NotYet, Want, Done].find((status) =>
    cells.some((cell) => level(cell) === level(status)),
  );
}

// A header click: each row at the column's status is clicked, the rows past it are left alone.
export function click_column(grid: StatusGrid, col: number, locked: boolean) {
  const status = column_status(grid, col);
  grid.forEach((row, index) => {
    if (index !== NUM_ADV_PIECES && level(row[col]) === level(status))
      click_cell(row, col, locked);
  });
}
