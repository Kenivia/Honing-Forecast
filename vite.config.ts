/// <reference types="vitest/config" />
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import { configDefaults } from "vitest/config"

// https://vite.dev/config/
export default defineConfig({
    base: "/Honing-Forecast/",
    plugins: [react()],
    test: {
        globals: false, // we'll import describe/it/expect from 'vitest' explicitly
        environment: "node", // 'jsdom' if you need DOM
        coverage: { provider: "v8" },
        exclude: [...configDefaults.exclude, "junkyard/*"],
    },
})
