/// <reference types="vitest/config" />
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import { configDefaults } from "vitest/config"
import path from "path"
// import { visualizer } from "rollup-plugin-visualizer"

// https://vite.dev/config/
export default defineConfig({
    base: "/Honing-Forecast/",
    plugins: [
        react(),
        // visualizer({
        //     filename: "dist/stats.html", // generates stats file in dist
        //     open: true, // open the report after build
        //     gzipSize: true,
        //     brotliSize: true,
        // }),
    ],
    resolve: {
        alias: {
            // eslint-disable-next-line no-undef
            "@": path.resolve(__dirname, "src"),
            // or add others:
            // '@components': path.resolve(__dirname, 'src/Frontend/Components')
        },
    },

    test: {
        globals: false, // we'll import describe/it/expect from 'vitest' explicitly
        environment: "jsdom",
        setupFiles: ["vitest.setup.ts"],
        coverage: { provider: "v8" },
        exclude: [...configDefaults.exclude, "junkyard/*"],
    },
    build: {
        sourcemap: true, // helpful for source-map-explorer also
        //     rollupOptions: {
        //         output: {
        //             manualChunks: {
        //                 // Vendor libraries chunk
        //                 "vendor-react": ["react", "react-dom"],
        //                 "vendor-mui": ["@mui/material"],
        //                 "vendor-visx": ["@visx/xychart", "@visx/event"],
        //                 "vendor-emotion": ["@emotion/react", "@emotion/styled"],

        //                 // Core components chunk
        //                 components: [
        //                     "./src/Frontend/components/Graph.tsx",
        //                     "./src/Frontend/components/SpreadsheetGrid.tsx",
        //                     "./src/Frontend/components/Icon.tsx",
        //                     "./src/Frontend/components/CheckboxGrid.tsx",
        //                 ],

        //                 // Feature sections chunks
        //                 "chance-to-cost": ["./src/Frontend/features/honing_forecast/ChanceToCostSection.tsx"],
        //                 "cost-to-chance": ["./src/Frontend/features/honing_forecast/CostToChanceSection.tsx"],
        //                 gamba: [
        //                     "./src/Frontend/features/honing_forecast/GambaSection.tsx",
        //                     "./src/Frontend/features/honing_forecast/GambaLogic.tsx",
        //                     "./src/Frontend/features/honing_forecast/GambaInfoBox.tsx",
        //                     "./src/Frontend/features/honing_forecast/GambaSelection.tsx",
        //                 ],

        //                 // Control and utility chunks
        //                 "control-panel": [
        //                     "./src/Frontend/features/honing_forecast/ControlPanel.tsx",
        //                     "./src/Frontend/features/honing_forecast/ControlPanelFunctions.ts",
        //                     "./src/Frontend/features/honing_forecast/NormalHoningPanel.tsx",
        //                     "./src/Frontend/features/honing_forecast/AdvancedHoningPanel.tsx",
        //                 ],

        //                 // WASM and workers chunk
        //                 "wasm-workers": ["./src/Frontend/js_to_wasm.ts", "./src/Frontend/worker_setup.ts"],

        //                 // Utilities chunk
        //                 utils: [
        //                     "./src/Frontend/features/honing_forecast/utils.ts",
        //                     "./src/Frontend/features/honing_forecast/Debounce.ts",
        //                     "./src/Frontend/features/honing_forecast/Marquee.ts",
        //                     "./src/Frontend/features/honing_forecast/HistogramUtils.ts",
        //                     "./src/Frontend/features/honing_forecast/Settings.ts",
        //                     "./src/Frontend/features/honing_forecast/Tooltip.tsx",
        //                     "./src/Frontend/features/honing_forecast/Separator.tsx",
        //                 ],
        //             },
        //         },
        //     },
    },
})
