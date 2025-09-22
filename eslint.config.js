
import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tsParser from '@typescript-eslint/parser'
import tsPlugin from '@typescript-eslint/eslint-plugin'
import { defineConfig, globalIgnores } from 'eslint/config'


export default defineConfig([
    globalIgnores(['dist', 'junkyard', "vitest.setup.ts"]),
    // Base config for JS/JSX files
    {
        files: ['**/*.{js,jsx}'],
        extends: [
            js.configs.recommended,
            reactHooks.configs['recommended-latest'],
            reactRefresh.configs.vite,
        ],
        plugins: { '@typescript-eslint': tsPlugin },
        languageOptions: {
            ecmaVersion: 2020,
            globals: globals.browser,
            parserOptions: {
                ecmaVersion: 'latest',
                ecmaFeatures: { jsx: true },
                sourceType: 'module',
            },
        },
        rules: {
            // disable base rule and use TypeScript-aware rule
            'no-unused-vars': 'off',
            '@typescript-eslint/no-unused-vars': ['error', { varsIgnorePattern: '^[A-Z_]', "argsIgnorePattern": "^_" }],
        },
    },
    // TS/TSX-specific overrides
    {
        files: ['**/*.{ts,tsx}'],
        extends: [
            js.configs.recommended,
            reactHooks.configs['recommended-latest'],
            reactRefresh.configs.vite,
        ],
        languageOptions: {
            ecmaVersion: 2020,
            globals: { ...globals.browser, React: 'readonly' },
            parser: tsParser,
            parserOptions: {
                ecmaVersion: 'latest',
                ecmaFeatures: { jsx: true },
                sourceType: 'module',
                project: './tsconfig.json',
            },
        },
        rules: {
            'no-unused-vars': ['error', { varsIgnorePattern: '^[A-Z_]', "argsIgnorePattern": "^_", "caughtErrors": "none", }]
        },
    },

])
