import { beforeEach } from "vitest";

// Worker bundles are started by store actions; the runs themselves are not under test.
globalThis.Worker = class {
  postMessage() {}
  terminate() {}
  addEventListener() {}
  removeEventListener() {}
} as unknown as typeof Worker;

// jsdom has no layout; HoverTip measures its lines with this
Range.prototype.getClientRects = () => [] as unknown as DOMRectList;

beforeEach(() => localStorage.clear());
