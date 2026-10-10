# Working in this repo

Environment traps and the checks that catch things. Read this before a large edit.

## Line endings: the repo is LF, the working tree is CRLF

`core.autocrlf=true` and there is no `.gitattributes`. So:

- Blobs in git are **LF**. Files on disk are **CRLF**.
- Prettier's `endOfLine` default is `lf`, so `prettier --check` reports _every_ file as
  failing in a fresh clone. That noise is environmental. It is not something a change
  introduced, and "62 files failing" means nothing on its own.
- Converting a file's line endings is a no-op as far as git is concerned: the clean filter
  normalises CRLF to LF before comparing, so a working-tree conversion produces **no
  diff**. `git status` may still list the file until the stat cache refreshes. Trust
  `git diff HEAD --numstat`, or compare `git hash-object --path <f> -- <f>` against
  `git rev-parse HEAD:<f>`; do not trust `git status` alone after a bulk rewrite.

What this means in practice:

- **Multi-line string matching fails silently.** A script that reads a file and searches
  for a two-line pattern written with `\n` will not match, because the file has `\r\n`.
  Normalise on read (`read().replace("\r\n", "\n")`) or build the pattern from the file's
  own line ending. This has cost real time more than once.
- `sed` in Git Bash strips `\r` from its output, so piping through `sed` or `cat -A`
  can hide CRLF. Check the raw bytes instead: `python -c "print(open(f,'rb').read()[:200])"`.
- Mixing endings inside one file is the worst case: a tool rewrites some lines as LF and
  later searches then match only part of the file. If an edit script half-worked, suspect
  this first.
- Prettier rewrites to LF. Running it on a file is fine; it just makes that file LF on
  disk like everything in the index already is.

## `.tmp/`

`.tmp/` is gitignored scratch space for throwaway scripts, bundles and dumps. Put
temporary work there and nowhere else, and clear it when done using `pnpm run remove-tmp` instead of a direct command (it may be blocked). The test
runner also builds into `.tmp/tests/` and wipes it on each run.

Do **not** invent another scratch directory in the repo root. A previous `.wip/` folder
was picked up by a commit because it was not ignored.

## Checks

| Command              | What it covers                                                           |
| -------------------- | ------------------------------------------------------------------------ |
| `pnpm check`         | `vue-tsc --noEmit`: the `.ts` layer **and** `.vue` template expressions. |
| `pnpm test:frontend` | The node-side suites in `tests/`. See `tests/` below.                    |
| `pnpm e2e`           | Playwright browser tests.                                                |
| `pnpm test`          | Rust unit tests. Works again now that `verification` is commented out.   |
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

`pnpm test:frontend` bundles each `tests/*.test.ts` with esbuild and runs it under node
with `tests/shim.mjs` (localStorage, `self`, a stub `Worker`). `tests/helpers.ts` has
`check` / `equal` / `done`; a suite exits non-zero if any check failed. `node tests/run.mjs
<substring>` runs one suite.

- `migration.test.ts`: a hand-built V7 payload through the real migration and save path.
- `live.test.ts`: the same against `scripts/V7 config`, a real 27-character save. Skips
  itself if that file is absent.
- `relations.test.ts`: cross-tier conversion, bundle pricing, material deduction.

Two properties these exist to protect, both of which were real bugs:

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
