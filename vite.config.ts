import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { execSync } from "child_process";

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
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      // eslint-disable-next-line no-undef
      "@": path.resolve(__dirname, "frontend"),
    },
  },
  build: {
    sourcemap: true,
  },
  assetsInclude: ["**/*.md"],
  server: {
    watch: {
      ignored: [
        "**/cloudflare/.wrangler/**",
        "**/crates/**",
        "**/ignored/**",
        "**/target/**",
        "**/scanner_setup/game_files/**",
        "**/scanner_setup/brightness/**",
        "**/scanner_setup/codex/**",
        "**/.tmp/**",
      ],
    },
  },
});
