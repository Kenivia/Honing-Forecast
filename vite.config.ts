/// <reference types="vitest/config" />
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import { configDefaults } from "vitest/config"
import { visualizer } from "rollup-plugin-visualizer"

// https://vite.dev/config/
export default defineConfig({
    base: "/Honing-Forecast/",
    plugins: [
        react(),
        visualizer({
            filename: "dist/stats.html", // generates stats file in dist
            open: true, // open the report after build
            gzipSize: true,
            brotliSize: true,
        }),
    ],
    test: {
        globals: false, // we'll import describe/it/expect from 'vitest' explicitly
        environment: "jsdom",
        setupFiles: ["vitest.setup.ts"],
        coverage: { provider: "v8" },
        exclude: [...configDefaults.exclude, "junkyard/*"],
    },
    build: {
        sourcemap: true, // helpful for source-map-explorer also
    },
})
