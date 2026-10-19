import { ref } from "vue";
import { decode, encode } from "@msgpack/msgpack";
import wasm_url from "@/../crates/wasm/pkg/hf_wasm_bg.wasm?url";
import {
  assume_char_bound,
  assume_roster_tradable,
  chests,
  edits,
  game_pages,
  seen_pages,
  slots,
  SlotEdit,
} from "./ScanStore";
import { convert, manifest } from "./Manifest";
import { Detected, scan_settings } from "./ScanSettings";
import {
  create_worker_bundle,
  WorkerBundle,
} from "@/WasmInterface/WorkerBundle";
import { WasmOp } from "@/WasmInterface/WasmWorker";

// A debug capture, to be sent in and run again. Per frame the scanner was sent: the input (the
// frame, its time and the edits), which any build can run, written by a recorder worker of its
// own; and the trace (texts, time budget cut, digest), which only this build can follow, written
// by the scanner. The page writes the start and the end. The format is in
// crates/scanner/src/dev_utils/capture.rs.

const MAGIC = "HFCAP004";
export const CAPTURE_EXTENSION = ".hfcap";
// recording stops by itself here
const LIMIT = 5e9;

let start: Uint8Array | null = null;
let inputs: Uint8Array[] = [];
let traces: Uint8Array[] = [];
export const recording = ref(false);
export const captured_bytes = ref(0);

function record(tag: string, payload: Uint8Array) {
  const out = new Uint8Array(5 + payload.length);
  out[0] = tag.charCodeAt(0);
  new DataView(out.buffer).setUint32(1, payload.length, true);
  out.set(payload, 5);
  return out;
}

function add(bytes: number) {
  captured_bytes.value += bytes;
  if (captured_bytes.value > LIMIT) recording.value = false;
}

export interface Build {
  commit: string;
  // the start of the wasm binary's SHA-256: a trace only replays exactly on the same one
  wasm: string;
}

let build: Promise<Build> | null = null;
export function build_stamp() {
  build ??= fetch(wasm_url)
    .then((response) => response.arrayBuffer())
    .then((bytes) => crypto.subtle.digest("SHA-256", bytes))
    .then((hash) => ({
      commit: __BUILD_COMMIT__,
      wasm: [...new Uint8Array(hash).slice(0, 8)]
        .map((x) => x.toString(16).padStart(2, "0"))
        .join(""),
    }));
  return build;
}

export interface CaptureStart {
  width: number;
  height: number;
  forced_21_9: Detected<boolean>;
  brightness: Detected<number>;
  build: Build;
}

// the head of a capture, from a scanner state just reserved and cleared
export async function begin_capture_file(info: Omit<CaptureStart, "build">) {
  const payload = encode({ ...info, build: await build_stamp() });
  start = record("S", payload);
  add(start.length);
}

export function clear_capture() {
  stop_recorder();
  recorder_fell_behind.value = false;
  start = null;
  traces = [];
  captured_bytes.value = 0;
}

// The scanner worker forwards the frames it was sent to the recorder, so a frame is only taken
// from the browser once. Frames waiting there are counted.
let recorder: WorkerBundle | null = null;
export const in_recorder = ref(0);
// past this many frames waiting the recorder is not keeping up, and recording stops
const RECORDER_BACKLOG = 8;
export const recorder_fell_behind = ref(false);
// per frame recorded, in ms, kept on window for profiling
const record_timings: any[] = ((globalThis as any).__record_timings ??= []);

// a scan's trace, from the scanner
export function add_trace(trace: Uint8Array) {
  traces.push(trace);
  add(trace.length);
}

// a frame's pixels as the scanner forwarded them, its timestamp in microseconds and the edits
// its scan took in
export function record_input(
  pixels: Uint8Array,
  time: number,
  frame_edits: SlotEdit[],
) {
  if (in_recorder.value >= RECORDER_BACKLOG) {
    recording.value = false;
    recorder_fell_behind.value = true;
    return;
  }
  recorder ??= create_worker_bundle();
  in_recorder.value++;
  recorder.debounced_start(
    WasmOp.Record,
    { pixels, time, edits: frame_edits },
    (input: Uint8Array, timings) => {
      in_recorder.value--;
      inputs.push(input);
      add(input.length);
      record_timings.push({ ...timings, bytes: input.length });
    },
    0,
    false,
  );
}

function stop_recorder() {
  recorder?.cancel();
  recorder = null;
  in_recorder.value = 0;
  inputs = [];
}

// How the session ended on the page. The user corrects the grid by hand before downloading, so
// this is the right answer the replay is held against.
function reference() {
  return {
    saved_at: new Date().toISOString(),
    user_agent: navigator.userAgent,
    scan_settings,
    slots: [...slots.value.values()],
    edits: edits.value,
    chests: chests.value,
    game_pages: game_pages.value,
    assume_roster_tradable: assume_roster_tradable.value,
    assume_char_bound: assume_char_bound.value,
    seen_pages: seen_pages.value,
    convert: convert.value,
    manifest: manifest.value,
  };
}

export function download_capture() {
  const end = new TextEncoder().encode(
    // slot images are in the frames already
    JSON.stringify(reference(), (key, value) =>
      key === "image" ? undefined : value,
    ),
  );
  // a frame whose input or trace is missing (recording stopped while it was out) is left out
  const frames = Math.min(inputs.length, traces.length);
  const parts = [new TextEncoder().encode(MAGIC), start];
  for (let i = 0; i < frames; i++) parts.push(inputs[i], traces[i]);
  parts.push(record("E", end));
  const url = URL.createObjectURL(new Blob(parts as BlobPart[]));
  const link = document.createElement("a");
  link.href = url;
  link.download = `honing-forecast-${new Date().toISOString().replace(/[:.]/g, "-")}${CAPTURE_EXTENSION}`;
  link.click();
  URL.revokeObjectURL(url);
}

// null at the end of the file
export async function read_record(file: File, offset: number) {
  if (offset + 5 > file.size) return null;
  const head = new DataView(await file.slice(offset, offset + 5).arrayBuffer());
  const next = offset + 5 + head.getUint32(1, true);
  return {
    tag: String.fromCharCode(head.getUint8(0)),
    payload: new Uint8Array(await file.slice(offset + 5, next).arrayBuffer()),
    next,
  };
}

// the start record, and where the frames begin
export async function open_capture(file: File) {
  const magic = await file.slice(0, MAGIC.length).text();
  if (magic !== MAGIC) throw new Error("Not a capture file of this version.");
  const start = await read_record(file, MAGIC.length);
  return { start: decode(start.payload) as CaptureStart, first: start.next };
}

// a frame the same as the one before, sent while a re-run waits for its last texts
export const SAME_FRAME = encode({ delta: null, time: 0, edits: [] });
