import { ref } from "vue";
import { WasmOp } from "@/WasmInterface/WasmWorker";
import { create_worker_bundle } from "@/WasmInterface/WorkerBundle";
import { OcrJob, OneIconConfig } from "./LoadStorage";

// The scanner worker never runs OCR itself. Each scan result lists text lines to read, which
// are passed to the OCR workers here, and their texts go back in with a later scan.
// Module state, like the scanner worker it lives and dies with.

const BATCH = 4; // lines per op, so the first texts are not held back by a long batch
// Half the hardware threads less one, which leaves room for the scanner worker and the page.
// Each worker loads the model and the templates again, so no more than eight.
const WORKERS = Math.min(
  8,
  Math.max(1, Math.floor((navigator.hardwareConcurrency ?? 4) / 2) - 1),
);

interface OcrWorker {
  bundle: ReturnType<typeof create_worker_bundle>;
  ready: boolean;
  busy: boolean;
}

let workers: OcrWorker[] = [];
let waiting: (OcrJob & { queued_at: number })[] = [];
let done: [number, string][] = [];
// lines not read yet, waiting or with a worker
export const ocr_pending = ref(0);

// per-batch timings in ms, kept on window for profiling
const ocr_timings: any[] = ((globalThis as any).__ocr_timings ??= []);

export function start_ocr(model: Uint8Array, config: OneIconConfig[]) {
  stop_ocr();
  workers = Array.from({ length: WORKERS }, () => ({
    bundle: create_worker_bundle(),
    ready: false,
    busy: false,
  }));
  for (const worker of workers) {
    worker.bundle.debounced_start(
      WasmOp.OcrInit,
      { model, config },
      () => {
        worker.ready = true;
        pump();
      },
      0,
      false,
    );
  }
}

export function stop_ocr() {
  for (const worker of workers) worker.bundle.cancel();
  workers = [];
  waiting = [];
  done = [];
  ocr_pending.value = 0;
}

export function queue_ocr(jobs: OcrJob[]) {
  const queued_at = performance.now();
  for (const job of jobs) waiting.push({ ...job, queued_at });
  ocr_pending.value += jobs.length;
  pump();
}

// [id, text] of everything read since the last call
export function take_ocr_results() {
  const out = done;
  done = [];
  return out;
}

function pump() {
  // a hover's first read goes ahead of other hovers' second and third
  waiting.sort((a, b) => a.priority - b.priority || a.id - b.id);
  for (const worker of workers) {
    if (!worker.ready || worker.busy || waiting.length === 0) continue;
    const batch = waiting.splice(0, BATCH);
    worker.busy = true;
    const start = performance.now();
    worker.bundle.debounced_start(
      WasmOp.Ocr,
      batch,
      (texts: [number, string][]) => {
        // a stopped worker never answers, so this one is still current
        worker.busy = false;
        done.push(...texts);
        ocr_pending.value -= texts.length;
        const now = performance.now();
        ocr_timings.push({
          at: now,
          jobs: batch.length,
          width: batch.reduce((sum, job) => sum + job.line.crop.width, 0),
          took: now - start,
          // how long its first reads waited, queue and all
          waited: Math.max(
            0,
            ...batch
              .filter((job) => job.priority === 0)
              .map((job) => now - job.queued_at),
          ),
          left: waiting.length,
        });
        if (ocr_timings.length > 5000) ocr_timings.shift();
        pump();
      },
      0,
      false,
    );
  }
}
