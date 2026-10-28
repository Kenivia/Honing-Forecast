import base from "../../vitest.config";

// the unit config with only the exporter of real characters included
export default {
  ...base,
  test: { ...(base as any).test, include: ["scripts/arena/export_real.test.ts"] },
};
