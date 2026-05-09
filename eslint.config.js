import js from "@eslint/js";
import globals from "globals";
import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import pluginVue from "eslint-plugin-vue";

const unusedVarsRule = [
    "error",
    {
        varsIgnorePattern: "^[A-Z_]",
        argsIgnorePattern: "^_",
        caughtErrors: "none",
    },
];

export default [
    // Global ignores
    { ignores: ["dist/**", "junkyard/**", "crates/wasm/pkg/**"] },

    // Plain JS files
    {
        files: ["**/*.{js,mjs,cjs}"],
        languageOptions: {
            ecmaVersion: "latest",
            sourceType: "module",
            globals: globals.browser,
        },
        rules: {
            ...js.configs.recommended.rules,
            "no-unused-vars": unusedVarsRule,
        },
    },

    // TypeScript files
    {
        files: ["**/*.{ts,mts,cts}"],
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
            ...js.configs.recommended.rules,
            "no-unused-vars": "off",
            "@typescript-eslint/no-unused-vars": unusedVarsRule,
        },
    },

    // Vue files
    ...pluginVue.configs["flat/essential"],
    {
        files: ["**/*.vue"],
        plugins: { "@typescript-eslint": tsPlugin },
        languageOptions: {
            globals: globals.browser,
            parserOptions: {
                parser: tsParser,
            },
        },
        rules: {
            "no-unused-vars": "off",
            "@typescript-eslint/no-unused-vars": unusedVarsRule,
            "vue/multi-word-component-names": "off",
        },
    },
];
