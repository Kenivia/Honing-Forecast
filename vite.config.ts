import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { execSync } from "child_process";
import fs from "fs";
import { promisify } from "util";
import zlib from "zlib";
import type { Plugin } from "vite";

// the scanner's files go out as .br in place of the raw ones, which public/ keeps for the tools
const BROTLI_FILES = ["ScannerConfig.msgpack", "text-recognition.rten"];
function brotli_files(): Plugin {
  let out_dir = "";
  return {
    name: "brotli-files",
    apply: "build",
    configResolved(config) {
      out_dir = config.build.outDir;
    },
    async closeBundle() {
      const { BROTLI_PARAM_QUALITY, BROTLI_PARAM_LGWIN } = zlib.constants;
      await Promise.all(
        BROTLI_FILES.map(async (name) => {
          const file = path.join(out_dir, name);
          const compressed = await promisify(zlib.brotliCompress)(
            fs.readFileSync(file),
            {
              params: { [BROTLI_PARAM_QUALITY]: 11, [BROTLI_PARAM_LGWIN]: 24 },
            },
          );
          fs.writeFileSync(file + ".br", compressed);
          fs.unlinkSync(file);
        }),
      );
    },
  };
}

// stamped on debug captures; a build without git (or outside a repo) says so
function commit() {
  try {
    return execSync("git describe --always --dirty").toString().trim();
  } catch {
    return "unknown";
  }
}

export default defineConfig({
  base: "/",
  define: { __BUILD_COMMIT__: JSON.stringify(commit()) },
  plugins: [vue(), tailwindcss(), brotli_files()],
  resolve: {
    alias: {
      // eslint-disable-next-line no-undef
      "@": path.resolve(__dirname, "frontend"),
    },
  },
  build: {
    sourcemap: true,
    // migrate/: the domain move's receiver
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
        migrate: path.resolve(__dirname, "migrate/index.html"),
      },
    },
  },
  assetsInclude: ["**/*.md"],
  // the dep scan would otherwise crawl every .html, stale copies under .tmp included
  optimizeDeps: { entries: ["index.html"] },
  server: {
    watch: {
      ignored: [
        "**/cloudflare/.wrangler/**",
        "**/crates/**",
        "**/ignored/**",
        "**/target/**",
        "**/scanner_setup/game_files/**",
        "**/scanner_setup/test_stills/**",
        "**/scanner_setup/codex/**",
        "**/.tmp/**",
      ],
    },
  },
});
