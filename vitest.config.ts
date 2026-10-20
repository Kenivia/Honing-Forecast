import { defineConfig, mergeConfig } from "vitest/config";
import path from "path";
import viteConfig from "./vite.config";

// Unit and component tests under tests/unit. The older node suites (tests/*.test.ts) run through tests/run.mjs.
export default mergeConfig(
  viteConfig,
  defineConfig({
    // Outside the dev server plugin-vue turns <img src="/x.png"> into an import; point those at public/.
    resolve: {
      alias: [
        {
          find: /^\/(.*\.(png|jpe?g|webp|svg|gif))$/,
          replacement: path.resolve(__dirname, "public/$1"),
        },
      ],
    },
    test: {
      include: ["tests/unit/**/*.test.ts"],
      environment: "jsdom",
      setupFiles: ["tests/unit/setup.ts"],
    },
  }),
);
