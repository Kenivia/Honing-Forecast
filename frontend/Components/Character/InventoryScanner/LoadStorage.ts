import { decode } from "@msgpack/msgpack";
import { computed, reactive } from "vue";

export interface ScaledPosition {
  top_left: [number, number];
  width: number;
  height: number;
}

export interface OneIconConfig {
  name: string;
  offset: ScaledPosition;
  data: number[] | Uint8Array;
  tag: string;
  required_confidence?: number | null;
}

export interface Buffer {
  pointer: number;
  size: number;
}
export interface Hover {
  last_read_title: string;
  title: string | null;
  amount: string | null;
  tradability: string | null;
}

export interface SlotAddress {
  inventory_type: string;
  page_num: number;
  pos_in_inv: [number, number];
}

export type ChestKind = "SelectOne" | "Random" | "ObtainAll";

export interface ChestContent {
  item: string;
  amount: number;
  // the chest of scanner_setup/templates/chests.json it is, when it is one
  chest: number | null;
}

export interface ChestRow {
  name_read: string;
  count_read: string;
}

// One of scanner_setup/templates/chests.json, found by what its tooltip lists. Scanner only (not part of the
// calculator's types).
export interface Chest {
  // the chests its rows read as, by id; kind and contents are the first one's
  variants: number[];
  title: string;
  kind: ChestKind;
  contents: ChestContent[];
  // the slot icons those chests are drawn with
  icons: string[];
  // every chest it can be opens to nothing that is counted
  irrelevant: boolean;
  amount: string | null;
  tradability: string | null;
  column: [string, number, number] | null;
  slot: SlotAddress | null;
  rows: ChestRow[];
}

export type SlotStatus =
  | "Pending"
  | "Good"
  | "NeedHover"
  | "NeedTradability"
  | "Error"
  | "Irrelevant";

export type Currency = "Silver" | "Gold" | "Shards";

// read off the inventory, the currency window and the bar at the top of the screen
export interface CurrencyResult {
  currency: Currency;
  status: SlotStatus;
  reason: string;
  value: number | null;
  // char bound, roster bound, tradable
  bands: [number, number, number] | null;
}

export interface SlotResult {
  address: SlotAddress;
  // null when the slot matches no icon
  icon_name_score: [string, number] | null;
  // other icons it may be drawn with: plain chests look alike
  alternatives: string[];
  amount: string | null;
  tooltip_amount: string | null;
  tradability: string | null;
  // the material the tooltip said it is, where the icon alone does not say
  label: string | null;
  status: SlotStatus;
  // its number is with OCR, a first read or a re-read
  reading: boolean;
  reason: string;
  // the tooltip's amount, else the number on the icon
  value: number | null;
  // for a chest: the chests of scanner_setup/templates/chests.json its icon can be, by id
  variants: number[];
  // the whole slot; only when it changed
  image?: OneIconConfig;
}

// What a scan returns. The scanner state stays in the worker; images and chests only come
// when they changed, or all of them when `full`.
export interface ScanResult {
  full: boolean;
  buffer: Buffer;
  slots: SlotResult[];
  hover: Hover | null;
  // the page each located window shows in game
  pages: [string, number][];
  chests?: Chest[];
  currencies: CurrencyResult[];
  // as detected or given, null while not known yet
  forced_21_9: boolean | null;
  brightness: number | null;
  ocr_jobs: OcrJob[];
  // the frame scanned, RGBA, for the recorder worker while recording
  pixels?: Uint8Array;
}

export interface Pixels {
  width: number;
  height: number;
  data: Uint8Array;
}

// One line of text for an OCR worker, as captured. The page only passes it on: the worker
// makes it into what the recogniser takes.
export interface OcrJob {
  id: number;
  priority: number;
  // the read it is a line of, 0 for a slot's count
  group: number;
  line: {
    crop: Pixels;
    // a slot's count comes with its icon crop and the name of the template it matched
    kind:
      | "Text"
      | "Yellow"
      | "Digits"
      | { Number: { icon: Pixels; template: string } };
    brightness: number;
  };
}

// downloads in progress by url, in bytes; total is 0 when not known
export const downloads = reactive<
  Record<string, { loaded: number; total: number }>
>({});

export const download_note = computed(() => {
  const all = Object.values(downloads);
  if (!all.length) return null;
  const mb = (bytes: number) => (bytes / 1e6).toFixed(1);
  const loaded = mb(all.reduce((sum, d) => sum + d.loaded, 0));
  return all.every((d) => d.total)
    ? `${loaded} / ${mb(all.reduce((sum, d) => sum + d.total, 0))} MB`
    : `${loaded} MB`;
});

// fetched once per session, but a failed download is tried again on the next call
function once<T>(load: () => Promise<T>) {
  let promise: Promise<T> | null = null;
  return () =>
    (promise ??= load().catch((e) => {
      promise = null;
      throw e;
    }));
}

export const getScannerConfig = once(
  async () =>
    decode(await load_file("/ScannerConfig.msgpack")) as OneIconConfig[],
);
export const getModel = once(() => load_file("/text-recognition.rten"));

// A build serves these brotli compressed as <url>.br (vite.config.ts), the dev server raw. A
// server that sends the .br with Content-Encoding: br has the browser decompress it instead.
export async function load_file(url: string) {
  if (import.meta.env.DEV) return (await download(url)).bytes;
  const [{ decompress }, { bytes, encoded }] = await Promise.all([
    import("brotli-dec-wasm").then((module) => module.default),
    download(url + ".br"),
  ]);
  return encoded ? bytes : decompress(bytes);
}

async function download(url: string) {
  try {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`${url}: ${response.status}`);
    // a missing file comes back as the app's index.html
    if (response.headers.get("Content-Type")?.startsWith("text/html"))
      throw new Error(`${url}: not found`);
    // a compressed response's length is not that of the bytes read
    const encoded = !!response.headers.get("Content-Encoding");
    const total = encoded
      ? 0
      : Number(response.headers.get("Content-Length") ?? 0);
    downloads[url] = { loaded: 0, total };
    const chunks: Uint8Array[] = [];
    const reader = response.body!.getReader();
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
      downloads[url].loaded += value.length;
    }
    const bytes = new Uint8Array(
      await new Blob(chunks as BlobPart[]).arrayBuffer(),
    );
    return { bytes, encoded };
  } finally {
    delete downloads[url];
  }
}
