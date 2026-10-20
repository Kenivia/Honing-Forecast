// Shared with the worker but free of side effects, so the main thread can import it without
// pulling in the wasm glue.
import type { Upgrade } from "@/Utils/KeyedUpgrades";
import type { OcrJob } from "@/Components/Character/InventoryScanner/LoadStorage";

export enum WasmOp {
  OptimizeAverage,
  Histogram,
  Cropper,
  Reserve,
  OcrInit,
  Ocr,
  Record,
}

// THESE BELOW DIRECTLY CORRESPOND TO A RUST STRUCT
export type HistogramPair = [number, number];
// Rust's label keyed maps, flattened to records by to_records below.
export type ByLabel<T> = Record<string, T>;
export interface HistogramOutputs {
  cum_percentiles: ByLabel<HistogramPair[]>;

  // chance the first k ownership bands cover every upgrade, as [bound, +roster, +tradable]
  chance_within: ByLabel<[number, number, number]>;
  avg_used: ByLabel<number>;

  // one entry per plan in the payload, in payload order
  gold_per_plan: ByLabel<number>[];
  metric_per_plan: number[];

  juice_info: any;
  state_bundle: StateBundle;
}

export interface StateBundle {
  upgrade_arr: Upgrade[];
  special_state: number[];
  chest_states: Map<string, number[]>;
  special_invalid_index?: number;
  latest_special_probs?: number[];
  min_resolution: number;
  prep_output: any;
  adv_cache: any;
  metric?: number;
}

// what an OCR job can hand over instead of copying
export function ocr_job_buffers(job: OcrJob) {
  const kind = job.line.kind;
  return [
    job.line.crop.data.buffer,
    ...(typeof kind === "object" ? [kind.Number.icon.data.buffer] : []),
  ];
}
