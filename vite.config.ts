/// <reference types="vitest/config" />
import { defineConfig } from "vite"
import vue from "@vitejs/plugin-vue"
import { configDefaults } from "vitest/config"
import path from "path"

export default defineConfig({
    base: "/",
    plugins: [vue()],
    resolve: {
        alias: {
            // eslint-disable-next-line no-undef
            "@": path.resolve(__dirname, "frontend"),
        },
    },
    test: {
        globals: false,
        environment: "jsdom",
        setupFiles: ["vitest.setup.ts"],
        coverage: { provider: "v8" },
        exclude: [...configDefaults.exclude, "junkyard/*"],
    },
    build: {
        sourcemap: true,
    },
})
