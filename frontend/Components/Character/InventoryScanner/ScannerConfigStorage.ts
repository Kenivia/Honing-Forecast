import { encode, decode } from "@msgpack/msgpack";

export interface ScaledPosition {
  top_left: [number, number];
  width: number;
  height: number;
}

export interface OneIconConfig {
  name: string;
  offset: ScaledPosition;
  data: number[];
  tag: string;
}

export interface Buffer {
  pointer: number;
  size: number;
}
export interface OneSlotInfo {
  icon_name_score: [string, number] | null;
  observed_number: OneIconConfig;
  observed_icon: OneIconConfig;
  observed_id: String;
  progress: any;
  amount: number | null;
  tradability: any;
}

export interface ScannerState {
  slot_infos: Map<any, OneSlotInfo>;
  anchors: any;
  screen_info: any;
  pending_jobs: any;

  buffer: Buffer;
  debugging: boolean;
  debug_info: Map<string, [ScaledPosition, number, number, OneIconConfig]>;
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
