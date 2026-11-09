import base from "../../vitest.config";

// the unit config with only iteration 3's exporter of real characters included
export default {
  ...base,
  test: { ...(base as any).test, include: ["scripts/arena/export_real3.test.ts"] },
};
