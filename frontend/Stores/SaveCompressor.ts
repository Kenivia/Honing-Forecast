import LZString from "lz-string";

// Compresses saves off the main thread. One instance lives for the whole session.
self.onmessage = ({ data: [at, json] }: MessageEvent<[number, string]>) =>
  self.postMessage([at, LZString.compressToUTF16(json)]);
