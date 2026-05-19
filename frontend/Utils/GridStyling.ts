// this is where the syncing & stuff happens
export interface GridToInputColumnMap {
  label: string;
  col: number | undefined; // which input column to use, which is 0 for t4 and 1 for serca, except for the synced ones which always use 0
  row: number;
}

export interface GridConfig {
  tier?: number | undefined; // only used as key
  grid_template_columns: string;
  rows?: GridToInputColumnMap[];
}
