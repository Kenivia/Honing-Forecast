import { encode, decode } from "@msgpack/msgpack";
import { OneIconConfig } from "./Setup.vue";

let configPromise: Promise<OneIconConfig[]> | null = null;

export function getScannerConfig() {
  if (!configPromise) {
    configPromise = load_from_msg_pack("/ScannerConfig.msgpack") as Promise<
      OneIconConfig[]
    >;
  }
  return configPromise;
}

export function download_as_msg_pack(data: unknown, filename: string): void {
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

export async function load_from_msg_pack(url: string) {
  try {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch ${url}: ${response.status} ${response.statusText}`,
      );
    }
    const buffer = await response.arrayBuffer();
    console.log(buffer, url);
    return decode(new Uint8Array(buffer));
  } catch (e) {
    console.log("loading msgpack failed with error ", e);

    return [];
  }
}
