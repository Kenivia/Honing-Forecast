import { Material } from "./Constants";

export interface GridConfig {
  tier?: number | undefined; // only used as key
  grid_template_columns: string;
  materials?: Material[];
}
