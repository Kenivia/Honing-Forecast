import { decode } from "@msgpack/msgpack";

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
  // the chest of templates/chests.json it is, when it is one
  chest: number | null;
}

export interface ChestRow {
  name_read: string;
  count_read: string;
}

// One of templates/chests.json, found by what its tooltip lists. Scanner only (not part of the
// calculator's types).
export interface Chest {
  // the chests its rows read as, by id; kind and contents are the first one's
  variants: number[];
  title: string;
  kind: ChestKind;
  contents: ChestContent[];
  // the slot icons those chests are drawn with
  icons: string[];
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
  reason: string;
  // the tooltip's amount, else the number on the icon
  value: number | null;
  // for a chest: the chests of templates/chests.json its icon can be, by id
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
  ocr_jobs: OcrJob[];
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
  line: {
    crop: Pixels;
    // a slot's count comes with its icon crop and the name of the template it matched
    kind: "Text" | "Yellow" | { Number: { icon: Pixels; template: string } };
    brightness: number;
  };
}

let configPromise: Promise<OneIconConfig[]> | null = null;
export function getScannerConfig() {
  if (!configPromise) {
    configPromise = load_file("/ScannerConfig.msgpack", true) as Promise<
      OneIconConfig[]
    >;
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
