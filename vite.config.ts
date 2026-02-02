/// <reference types="vitest/config" />
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import { configDefaults } from "vitest/config"
import path from "path"
// import { visualizer } from "rollup-plugin-visualizer"

// https://vite.dev/config/
export default defineConfig({
    base: "/",
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
            "@": path.resolve(__dirname, "frontend"),
            // or add others:
            // '@components': path.resolve(__dirname, 'frontend/Components')
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
        //                     "./frontend/components/Graph.tsx",
        //                     "./frontend/components/SpreadsheetGrid.tsx",
        //                     "./frontend/components/Icon.tsx",
        //                     "./frontend/components/CheckboxGrid.tsx",
        //                 ],

        //                 // Feature sections chunks
        //                 "chance-to-cost": ["./frontend/features/honing_forecast/ChanceToCostSection.tsx"],
        //                 "cost-to-chance": ["./frontend/features/honing_forecast/DistributionSection.tsx"],
        //                 gamba: [
        //                     "./frontend/features/honing_forecast/GambaSection.tsx",
        //                     "./frontend/features/honing_forecast/GambaLogic.tsx",
        //                     "./frontend/features/honing_forecast/GambaInfoBox.tsx",
        //                     "./frontend/features/honing_forecast/GambaSelection.tsx",
        //                 ],

        //                 // Control and utility chunks
        //                 "control-panel": [
        //                     "./frontend/features/honing_forecast/ControlPanel.tsx",
        //                     "./frontend/features/honing_forecast/ControlPanelFunctions.ts",
        //                     "./frontend/features/honing_forecast/NormalHoningPanel.tsx",
        //                     "./frontend/features/honing_forecast/AdvancedHoningPanel.tsx",
        //                 ],

        //                 // WASM and workers chunk
        //                 "wasm-workers": ["./frontend/js_to_wasm.ts", "./frontend/worker_setup.ts"],

        //                 // Utilities chunk
        //                 utils: [
        //                     "./frontend/features/honing_forecast/utils.ts",
        //                     "./frontend/features/honing_forecast/Debounce.ts",
        //                     "./frontend/features/honing_forecast/Marquee.ts",
        //                     "./frontend/features/honing_forecast/HistogramUtils.ts",
        //                     "./frontend/features/honing_forecast/Settings.ts",
        //                     "./frontend/features/honing_forecast/Tooltip.tsx",
        //                     "./frontend/features/honing_forecast/Separator.tsx",
        //                 ],
        //             },
        //         },
        //     },
    },
})
