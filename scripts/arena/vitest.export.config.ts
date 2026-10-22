import base from "../../vitest.config";

// the unit config with only the exporter included
export default {
  ...base,
  test: { ...(base as any).test, include: ["scripts/arena/export_pages.test.ts"] },
};
