import js from "@eslint/js"
import globals from "globals"
import tsParser from "@typescript-eslint/parser"
import tsPlugin from "@typescript-eslint/eslint-plugin"
import pluginVue from "eslint-plugin-vue"
import { defineConfig, globalIgnores } from "eslint/config"

export default defineConfig([
    globalIgnores(["dist", "junkyard", "vitest.setup.ts", "/src/types/vitest.d.ts"]),
    {
        files: ["**/*.{js,mjs,cjs}"],
        extends: [js.configs.recommended],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            globals: globals.browser,
        },
        rules: {
            "no-unused-vars": ["error", { varsIgnorePattern: "^[A-Z_]", argsIgnorePattern: "^_", caughtErrors: "none" }],
        },
    },
    {
        files: ["**/*.{ts,mts,cts}"],
        extends: [js.configs.recommended],
        plugins: { "@typescript-eslint": tsPlugin },
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            globals: globals.browser,
            parser: tsParser,
            parserOptions: {
                project: "./tsconfig.json",
            },
        },
        rules: {
            "no-unused-vars": "off",
            "@typescript-eslint/no-unused-vars": ["error", { varsIgnorePattern: "^[A-Z_]", argsIgnorePattern: "^_", caughtErrors: "none" }],
        },
    },
    // vue-eslint-parser handles .vue files; tsParser is passed as the inner
    // parser so <script setup lang="ts"> blocks get full TS analysis.
    ...pluginVue.configs["flat/essential"],
    {
        files: ["**/*.vue"],
        plugins: { "@typescript-eslint": tsPlugin },
        languageOptions: {
            globals: globals.browser,
            parserOptions: {
                // Hand off <script> blocks to the TS parser.
                // No `project` here — tsconfig doesn't include .vue files and
                // TS can't process them natively anyway. The rules we care about
                // (no-parsing-error, no-unused-vars) don't need type information.
                parser: tsParser,
            },
        },
        rules: {
            // Let @typescript-eslint handle this instead of the base rule
            "no-unused-vars": "off",

            "@typescript-eslint/no-unused-vars": ["error", { varsIgnorePattern: "^[A-Z_]", argsIgnorePattern: "^_", caughtErrors: "none" }],
            // Explicitly ensure the parse-error rule is at error level.
            // flat/essential already sets this, but calling it out makes the
            // intent clear and lets you tune individual sub-errors if needed.
            "vue/no-parsing-error": [
                "error",
                {
                    "unexpected-character-in-attribute-name": true,
                    "missing-whitespace-between-attributes": true,
                    "multi-word-component-names": false,
                },
            ],
        },
    },
])
