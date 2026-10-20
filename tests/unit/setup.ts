import { beforeEach } from "vitest";
import { config } from "@vue/test-utils";

// Worker bundles are started by store actions; the runs themselves are not under test.
globalThis.Worker = class {
  postMessage() {}
  terminate() {}
  addEventListener() {}
  removeEventListener() {}
} as unknown as typeof Worker;

// primevue's tooltip is registered in main.ts, not per component
config.global.directives = { tooltip: {} };

beforeEach(() => localStorage.clear());
