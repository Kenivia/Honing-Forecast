import { ref } from "vue";
import { decode } from "@msgpack/msgpack";
import {
  assume_char_bound,
  assume_roster_tradable,
  chests,
  edits,
  game_pages,
  seen_pages,
  slots,
} from "./ScanStore";
import { convert, manifest } from "./Manifest";
import { Detected, scan_settings } from "./ScanSettings";

// A debug capture: every scan as Rust saw it, to be sent in and run again. The scanner worker
// hands back one record per scan and they are kept here until downloaded. The format is in
// crates/scanner/src/capture.rs.

const MAGIC = "HFCAP003";
export const CAPTURE_EXTENSION = ".hfcap";
// recording stops by itself here
const LIMIT = 5e9;

let chunks: Uint8Array[] = [];
export const recording = ref(false);
export const captured_bytes = ref(0);

export function clear_capture() {
  chunks = [];
  captured_bytes.value = 0;
}

export function add_capture(chunk: Uint8Array) {
  chunks.push(chunk);
  captured_bytes.value += chunk.length;
  if (captured_bytes.value > LIMIT) recording.value = false;
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
  const head = new Uint8Array(5);
  head[0] = "E".charCodeAt(0);
  new DataView(head.buffer).setUint32(1, end.length, true);
  const url = URL.createObjectURL(
    new Blob([...chunks, head, end] as BlobPart[]),
  );
  const link = document.createElement("a");
  link.href = url;
  link.download = `honing-forecast-${new Date().toISOString().replace(/[:.]/g, "-")}${CAPTURE_EXTENSION}`;
  link.click();
  URL.revokeObjectURL(url);
}

export interface CaptureStart {
  width: number;
  height: number;
  forced_21_9: Detected<boolean>;
  brightness: Detected<number>;
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

// the start record, and where the scans begin
export async function open_capture(file: File) {
  const magic = await file.slice(0, MAGIC.length).text();
  if (magic !== MAGIC) throw new Error("Not a capture file.");
  const start = await read_record(file, MAGIC.length);
  return { start: decode(start.payload) as CaptureStart, first: start.next };
}
