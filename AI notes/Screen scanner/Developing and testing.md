# Developing and testing the inventory scanner

How to change the scanner and prove a change: the test inputs, which check answers which question, reproducing a case in the live game, and finding out why something was misread. Driving the game itself is in `Lost Ark client.md`; driving the scanner page in a browser is in `Browser harness/Screen scanner.md`.

Everything runs natively. The native bins run the same Rust the page does (`crates/scanner/src/dev_utils/native.rs` holds what they share), and a re-run repeats exactly: no time budget, every line read before the next frame, nothing by hash-map order. So two runs of the same tree print the same bytes. Native and wasm may still differ on a score sitting at a limit (other SIMD paths in the FFT and the resizer).

## Test inputs

| Where | What | Tracked |
| --- | --- | --- |
| `scanner_setup/test_stills/resolutions/<height>/` | native stills at 720p to 2160p, some forced 21:9; `1440p/` is what the older anchors were cut from | yes |
| `scanner_setup/test_stills/lone inventory 1440p/` | the lone inventory at its default place and dragged to each corner, 16:9 and forced 21:9 | yes |
| `scanner_setup/test_stills/currency 1440p/` | the inventory, the currency window and gold's tooltip (setting 50), with `expected.json` | yes |
| `scanner_setup/test_stills/brightness sweep/1080p inventory`, `1080p storage` | one scene at every brightness setting (file name = setting); the brightness curves were fitted on these | yes |
| `scanner_setup/test_stills/tooltips/` | item and chest tooltips | yes |
| `ignored/scanner_tests/stills/<folder>/` | stills of the live game, a folder per subject (`hud_size/`, every HUD size at 16:9 and forced 21:9, with `expected.json`) | this PC |
| `ignored/scanner_tests/recordings/` | debug captures (`.hfcap`): users' (`honing-forecast-*`) and the live game's (`live-*`), `expected.json` | this PC |
| `ignored/live/` | work in progress, not a test | this PC |

- **A still must be exactly a screen's size**: the size is taken as the game resolution, and one a pixel short is not a screen.
- **`expected.json`** in any of these folders names files of its folder and what they must read: currencies as `[char bound, roster bound, tradable]` (`{"x.png": {"Gold": [280000, 0, 3960769]}}`). A capture listed with `{}` is checked for its hand edits only. Format and rules are in the header of `crates/scanner/src/bin/scanner_check.rs`.
- **Only list what was checked by eye** against the frames or the game. A reading is never an answer, and only a capture's hand edits are right answers (`capture_edit`, below). A capture is sent in because something was read wrong.
- **The e2e tests name their captures** (`tests/e2e/scanner.spec.ts`: the merge set, the `CHECKED` set with its `misread` list, the sequence set); they skip a capture that is not on this PC.
- Old-format captures (before `HFCAP005`) are not read. They sit in `ignored/junkyard/`.

## Which check answers what

| Question | Command | Notes |
| --- | --- | --- |
| Is what is known still read right? | `pnpm scanner-check [-- <part of a name>...]` | every `expected.json`; exits 1 on a miss; about a minute |
| Did a change alter anything read? | `pnpm scanner-suite run <name>`, then `cmp <a> <b>` | every still folder and capture under both roots; a run named `reference` is kept from the last change |
| Does the page read the same in a browser? | `pnpm test:e2e scanner.spec` | captures replayed on the page, `CHECKED` ones held to their hand edits |
| Do the priors behave? | `cargo run --release --bin prior_test -- <captures>` | plus the status rules' unit tests in `pnpm test:rust` |
| What does it cost in the browser? | `pnpm scanner-profile`, `pnpm scanner-compare` | below |

**The usual order for a change**: `pnpm scanner-suite run before` (or reuse `reference` if it is of the current tree), make the change, `run after`, `cmp before after`, `pnpm scanner-check`, then `pnpm check` and the scanner e2e tests if the page or the wasm boundary moved. When the change is meant to alter reads, look at every `DIFF` with `capture_replay` before calling it right. Then run `run reference` again.

- `suite.py` (docstring) runs `tooltip_test` per still folder and `capture_replay --timings` per capture, at most four at once (`--seq`: one at a time), into a fresh `target/scanner-suite/<name>/`. `cmp` compares `.txt` byte for byte and `.err` by stage names and call counts.
- **Before/after against HEAD**: build a copy of HEAD (`git archive`, harness memory) and run the suite there. The bins load `public/` from the tree they were built in.

## Timing

- **Natively**: `pnpm scanner-suite run <name> --seq`, then `timings <a> [<b>]` (ms per scanned frame per stage, from `capture_replay --timings`). A stage with no timer gets a temporary `timing::timed("m/...", ...)`; a timed call costs about 60 ns.
- **In the browser**: `pnpm scanner-profile [--name=<run>] [--no-build] [--keep] [captures...]` (`scanner_setup/browser/profile_scanner.mjs`) builds the wasm, starts vite if nothing answers on :5173, replays each capture on the scanner page and dumps `window.__scan_timings` and the OCR and render timings to `target/scan-profiles/<run>/`. `pnpm scanner-compare <a> <b>` prints two runs side by side. A replay scans each frame once its texts are back, so compare scan stages, not OCR waits.
- **Only compare runs made back to back** with the game closed: the same build gave 13.1 and 16.4 ms wasm calls an hour apart. Don't edit project files during a browser run (vite reloads the page).
- With no dev server up, the first browser run usually times out while vite compiles; run it again with `--keep`.

## Reproducing a case in the live game

1. Ask whether the game may be used, then set it up with `lostark-market` (`Lost Ark client.md`, "Getting around").
2. `pnpm game-capture still <file.png>`, or `pnpm game-capture record <file.hfcap> [--seconds N]` in the background while driving the game with `run`. It stops on F10, after `--seconds`, or when `<file>.stop` appears (`touch` it). Frames come every 33 ms or as fast as the screenshot allows (about 17 fps and 7 MB/s with the world moving), with no cursor. Code: `crates/market/src/bin/game_capture.rs`.
3. Look at it: `capture_replay <file>` (or `tooltip_test <still>`).
4. Set the right answers: in `expected.json` for currencies, and with `capture_edit` for slots (below).
5. Move it into `scanner_tests` (or `test_stills/` if it should be tracked) and run `scanner-check`. Leave the game as found.

## Why something was misread

Start from the slot and work down to the line that was read:

1. `capture_replay <file.hfcap> --slot "Roster 0 0,2"` (a slot named as the end record's `edits` name it) prints every change to that slot and each hover beside it, with the scans to look at.
2. `--ocr --from N --to M` prints each line sent to be read, what it was for and its text. `--dump <folder>` writes those scans' frames and lines as PNG, each line also as the strip the recogniser reads.
3. `--check` holds the run to the capture's hand edits. `--reference <file>` writes the end record.
4. **Correct the capture**, after checking each value against its tooltip in the dumped frames: `capture_edit <file> set "<slot>" --amount 13 --tradability RosterBound [--item <icon>|none]`, plus `remove`, `list`, and `accept` (every slot as the page showed it, for a capture whose whole reading was checked by hand). An edit says icon, amount and tradability, not which of two chests drawn alike.

For one stage:

- **Icon matching**: `slot_scores <still> [--dump <folder>]` (the closest icons and decoys of every slot, whatever the limits). `scanner_setup/game_files/find_icon.py <crop>` says which game icon a slot is drawn with; it cannot be trusted on a slot with an item level across it.
- **Slot numbers**: `number_probe <out> <inventory> <page> <row> <col> <still or capture>` writes every stage of the clean-up per read; `scanner_setup/scanner/number_stages.py <out>` stacks them.
- **Chest rows**: `chest_rows "<name> | <count>" ...` prints the chests some rows read as.
- **Placement**: `anchor_locate` and `icon_locate` (sub-pixel positions of anchors and icons on stills, per height).
- `cap_dump` (in `hf-market`) writes every Nth frame of a capture as PNG.

Every bin takes `--help`. The scanner bins share three switches (`native::Options`): `--no-shapes` (every slot number to the recogniser), `--text-strip "<TextParams>"` (strips made another way), `--slot-budget` (keep the browser's time budget, so results vary with load).

## Benches

Each one tries variants against the same inputs. Add a variant to the bin's `variants()` list and score the output with its script.

- **Slot numbers**: `number_bench <stills or captures>... > reads.tsv`, then `scanner_setup/scanner/number_bench.py reads.tsv`. Stills of one scene are scored against the read most agree on, captures against their tooltip amounts and hand edits, per read and per slot (last read and the vote). It also has the shape reader alone and as the scanner reads. `--glyphs <folder>` keeps each crop with its expected background. The usual set is the two brightness sweeps and every capture.
- **Text strips**: `text_bench <captures>... > text.tsv`, then `scanner_setup/scanner/text_bench.py text.tsv [--show N]`. Titles and chest names are scored against the game's English titles, the rest against what most variants read. It takes about 8 minutes for the captures.
- **Icon matching**: `slot_dump <capture> target/slot-bench/<name>` writes every distinct look of every slot (normalised icon `_i`, with margin `_m`, raw `_r`, and `index.jsonl`). `scanner_setup/scanner/icon_data.py` labels them from the end record and picks each slot's clean look (the one other tooltip-free looks are closest to); its `LABELS` hold hand corrections. `icon_bench.py [--brief] [--lossy] [method...]` scores methods with and without decoys (docstring). It scores the 60 nearest templates and clean looks only, nothing under the hover highlight, with Python-made templates that score true matches about 0.003 higher than the scanner's.
- `target/` is not watched by vite: bench dumps are hundreds of thousands of files. Findings so far are in `Further learnings/`.

## Changing what the scanner knows

- **A new item, chest or game patch**: `Game files.md`, "After a patch", then `Config and calibration.md`. Rebuild the wasm (`pnpm run wasm`): the tables are compiled in.
- **A new or moved piece of UI**: cut its anchor or tab at 1440p and whole pixels with `anchors.py`, and put it in `ANCHORS` (`Config and calibration.md`). A template's name is a contract with the Rust constants.
- **A new kind of input** (setting, layout, resolution): reproduce it in game, keep the still or capture as a test with checked answers, then change the code until `scanner-check` passes and the suite `cmp` shows only the intended changes.
