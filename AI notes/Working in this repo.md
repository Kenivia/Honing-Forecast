# Working in this repo

Environment traps and the checks that catch things. Read this before a large edit.

## Line endings: LF everywhere

`.gitattributes` has `* text=auto eol=lf`, the repo-local `core.autocrlf` is `false`
(overriding Git for Windows' system-wide `true`), and VS Code's `files.eol` is `\n`.
Index and working tree are both LF; `git ls-files --eol` should show no `w/crlf` or
`w/mixed`.

CRLF can still be introduced by Windows tools that write the platform newline:

- Python text-mode writes on Windows emit `\r\n`. Pass `newline="\n"` to `open()`, or
  use `Path.write_text(..., newline="\n")`.
- PowerShell `Set-Content` / `Out-File` / `Add-Content` join lines with `\r\n`.
- If an edit script half-matches a file, suspect a mixed-ending file first. Check raw
  bytes (`python -c "print(open(f,'rb').read()[:200])"`); `sed` and `cat -A` in Git Bash
  can hide `\r`.
- Fix stray files with `git ls-files --eol | grep -E 'w/(crlf|mixed)' | cut -f2 | xargs dos2unix`.
  Git sees no diff from that, since the blobs are already LF.

## `.tmp/`

`.tmp/` is gitignored scratch space for throwaway scripts, bundles and dumps. Put
temporary work there and nowhere else, and clear it when done using `pnpm run remove-tmp` instead of a direct command (it may be blocked). The test
runner also builds into `.tmp/tests/` and wipes it on each run.

Do **not** invent another scratch directory in the repo root. A previous `.wip/` folder
was picked up by a commit because it was not ignored.

## Paths from the repo root in Rust

`.cargo/config.toml` sets `HF_ROOT` (`[env]`, `relative = true`) to the repo root, absolute and
with a trailing separator, for every crate Cargo builds here, wasm-pack included. Write paths from
the root as `concat!(env!("HF_ROOT"), "/scanner_setup/...")` for `include_str!`, or through the
arena's and scanner's `ROOT` (`env!("HF_ROOT")`) at runtime, never with chains of `../`. A build
outside the repo (an archived copy) needs `.cargo/config.toml` with it.

## Checks

| Command              | What it covers                                                           |
| -------------------- | ------------------------------------------------------------------------ |
| `pnpm check`         | `vue-tsc --noEmit`: the `.ts` layer **and** `.vue` template expressions. |
| `pnpm test`          | Every suite below in turn: Rust, node, Vitest, browser.                  |
| `pnpm test:node`     | The node-side suites in `tests/`. See `tests/` below.                    |
| `pnpm test:unit`     | Vitest unit and component tests in `tests/unit/`. See `tests/` below.   |
| `pnpm test:e2e`      | Playwright browser tests.                                                |
| `pnpm test:rust`     | Rust unit tests for the core, and the arena's short checks.              |
| `pnpm arena`         | The core's test harness. See `Calculator core/Arena.md`.                 |
| `pnpm arena stress`  | The evaluator's probabilities against an exact convolution.              |
| `pnpm scanner-suite` | The scanner over every recording and still, natively; compares two runs. |

TypeScript is pinned to 5.x because `vue-tsc` needs the `./lib/tsc` entry point that
TypeScript 6 removed. Do not upgrade TypeScript without checking `pnpm check` still runs.

### What the type checker cannot see

`strict` and `noImplicitAny` are both off. The consequence worth remembering:
**`obj[expr]` on a type with no index signature is allowed and yields `any`.** So
`some_input_column[tier]` and `record_keyed_by_label[0]` both type-check and both return
`undefined` at runtime. `vue-tsc` will not help with either, and a payload field that is
`undefined` surfaces as a Rust panic (`invalid type: unit value, expected i64`), not a
JS error. When changing an indexing convention, grep for the old shape rather than
trusting a clean type check, and load the page.

Template expressions in `.vue` files _are_ checked, but only for names and property
access — extra props passed to a component are not rejected, because the generated props
type is intersected with `Record<string, unknown>`.

## tests/

`pnpm test:node` bundles each `tests/*.test.ts` with esbuild and runs it under node
with `tests/shim.mjs` (localStorage, `self`, a stub `Worker`). `tests/helpers.ts` has
`check` / `equal` / `done`; a suite exits non-zero if any check failed. `node tests/run.mjs
<substring>` runs one suite.

- `migration.test.ts`: a hand-built V7 payload through the real migration and save path.
- `live.test.ts`: the same against `tests/V7 config`, a real 27-character save. Skips
  itself if that file is absent.
- `relations.test.ts`: cross-tier conversion, bundle pricing, material deduction, a Serca character counting and spending its T4 stock.

`pnpm test:unit` runs Vitest (`vitest.config.ts`, merged over `vite.config.ts`) with jsdom
and `@vue/test-utils`, over `tests/unit/**/*.test.ts`, one folder per frontend module. Unlike
the node suites it can mount `.vue` files. It runs in the `vmThreads` pool, one jsdom per worker rather than
per file, which halves the run; each file still gets a fresh module graph. `tests/unit/setup.ts` stubs `Worker` and the
`v-tooltip` directive and clears localStorage before each test; `tests/unit/mount.ts`
mounts with a fresh pinia and a memory router carrying the app's route names. Outside the dev
server plugin-vue turns `<img src="/x.png">` into an import; an alias in `vitest.config.ts` points
those at `public/`, so components with absolute image paths mount without mocks. A test
written as `it.fails` pins a suspected bug: it passes while the bug is there. `npx vitest
run tests/unit/<folder>` runs one folder.

Two properties the node suites exist to protect, both of which were real bugs:

- **Values must land on the right label, not the right index.** Real pre-V8 saves carry a
  `keys` array that drifted out of order while `data` stayed positional, so the migration
  reads positionally and ignores the stored keys.
- **Load, save, load again.** A save that cannot be read back identically by the same
  version only shows up on the _second_ load. Both suites write through `write_state` and
  reload through `load_roster_config` twice, rather than calling `to_saved`/`from_saved`
  in memory.

## Build output

`dist/` is committed. A local `vite build` rewrites it and leaves new hashed asset files
behind. If you build to check something, restore it afterwards: `git checkout -- dist/`
and delete the untracked assets the build added.
