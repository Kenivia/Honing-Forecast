import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
    base: "/",
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
});
