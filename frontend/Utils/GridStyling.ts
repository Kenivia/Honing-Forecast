// this is where the syncing & stuff happens
export interface GridToInputColumnMap {
  label: string;
  col: number;
  row: number;
}

export interface GridConfig {
  tier: number;
  cols: string;
  grid_row_span: string | undefined;
  rows: GridToInputColumnMap[];
}
