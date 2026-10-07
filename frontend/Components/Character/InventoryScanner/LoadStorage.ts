import { encode, decode } from "@msgpack/msgpack";

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
  normalized?: boolean;
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
  bound: boolean;
}

export interface ChestRow {
  name_read: string;
  count_read: string;
  item: string | null;
  crop: OneIconConfig | null;
}

// read off the tooltip alone, scanner only (not part of the calculator's types)
export interface Chest {
  kind: ChestKind;
  contents: ChestContent[];
  amount: string | null;
  tradability: string | null;
  last_read_title: string;
  title: string | null;
  icon: string | null;
  column: [string, number, number] | null;
  slot: SlotAddress | null;
  rows: ChestRow[];
}

export type SlotStatus =
  | "Pending"
  | "Good"
  | "NeedHover"
  | "Error"
  | "Irrelevant";

export interface SlotResult {
  address: SlotAddress;
  // null when the slot matches no icon
  icon_name_score: [string, number] | null;
  amount: string | null;
  tooltip_amount: string | null;
  tradability: string | null;
  status: SlotStatus;
  reason: string;
  // the tooltip's amount, else the number on the icon
  value: number | null;
  // the whole slot; only when it changed
  image?: OneIconConfig;
}

// What a scan returns. The scanner state stays in the worker; images, debug entries and
// chests only come when they changed, or all of them when `full`.
export interface ScanResult {
  full: boolean;
  buffer: Buffer;
  slots: SlotResult[];
  hover: Hover | null;
  // the page each located window shows in game
  pages: [string, number][];
  chests?: Chest[];
  debug: [string, ScaledPosition, number, number, OneIconConfig[]][];
  ocr_jobs: OcrJob[];
}

// one line of text for the OCR worker, a strip 64px tall
export interface OcrJob {
  id: number;
  priority: number;
  width: number;
  height: number;
  data: Uint8Array;
}

let configPromise: Promise<OneIconConfig[]> | null = null;
export function getScannerConfig() {
  if (!configPromise) {
    configPromise = (
      load_file("/ScannerConfig.msgpack", true) as Promise<OneIconConfig[]>
    ).then((configs) => configs.map((item) => ({ ...item, normalized: true })));
  }
  return configPromise;
}

let modelPromise: Promise<Uint8Array> | null = null;
export function getModel() {
  if (!modelPromise) {
    modelPromise = load_file(
      "/text-recognition.rten",
      false,
    ) as Promise<Uint8Array>;
  }
  return modelPromise;
}

export function download_as_msg_pack(
  data: OneIconConfig[],
  filename: string,
): void {
  const encoded = encode(data);
  // Blob wants a BufferSource; encode() returns a Uint8Array, which works directly
  const blob = new Blob([encoded], { type: "application/x-msgpack" });
  const url = URL.createObjectURL(blob);

  const link = document.createElement("a");
  link.href = url;
  link.download = filename.endsWith(".msgpack")
    ? filename
    : `${filename}.msgpack`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // Defer revoke slightly so the download actually starts in all browsers
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

// function renameKeyDeep(obj, oldKey, newKey) {
//   if (Array.isArray(obj)) {
//     return obj.map((item) => renameKeyDeep(item, oldKey, newKey));
//   }
//   if (obj !== null && typeof obj === "object") {
//     return Object.fromEntries(
//       Object.entries(obj).map(([key, value]) => [
//         key === oldKey ? newKey : key,
//         renameKeyDeep(value, oldKey, newKey),
//       ]),
//     );
//   }
//   return obj;
// }

export async function load_file(url: string, msg_pack: boolean) {
  try {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch ${url}: ${response.status} ${response.statusText}`,
      );
    }
    const buffer = await response.arrayBuffer();
    // console.log(buffer, url);
    return msg_pack ? decode(new Uint8Array(buffer)) : new Uint8Array(buffer); // renameKeyDeep(
  } catch (e) {
    console.log("loading msgpack failed with error ", e);

    return [];
  }
}
