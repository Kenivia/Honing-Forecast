# Driving the screen scanner

Page: `/chars/<region>/<character>/scanner` (capture card, scan loop and slot grid). The capture card shows no preview of the stream.

## Feeding it frames

Screen share cannot be driven headless, so upload a file instead. The capture card has a file input named `Upload image or video`, behind the sidebar's "Show debug options" (`show_debug` in `ScannerUIutils.ts`, on by default on a dev build, off on a release one); it is hidden, so set files on it directly rather than clicking it.

```js
export default async ({ page, hf }) => {
  await hf.open(page, "/chars/NA/Newchar/scanner");
  await page.getByLabel("Upload image or video").setInputFiles(file);
  await page.getByLabel("Frames scanned per second").waitFor();
  await page.locator(".grid canvas").nth(20).waitFor({ timeout: 30_000 });
};
```

- An image becomes a stream repeating that frame. A video plays once, drawn frame by frame, then repeats its last frame twice a second.
- **Feed it debug captures (`.hfcap`), not videos**: a replay scans every recorded frame and repeats exactly, where headless Chromium drops and doubles frames of a video playing at full speed. A still works too. Works headless and headed, in Chromium and Firefox.
- Any file size works. The `Aspect` and `Brightness` selects are under the capture card's "Troubleshooting" toggle (hidden during a replay), both on auto; manual brightness shows an input labelled `In-game brightness`. A change to those two shows `Apply` and `Cancel` buttons, and nothing happens until `Apply` (idle, `Apply` only keeps them for the next capture). Applied settings are saved in localStorage, so within one MCP session a later page load starts with them, and the toggle cannot close the section while one is off its default.
- **Test inputs** (captures in `ignored/scanner_tests/recordings`, stills in `scanner_setup/test_stills/` and `ignored/scanner_tests/stills/`) are listed in `Screen scanner/Developing and testing.md`. Useful stills here: `test_stills/brightness sweep/1080p inventory` (lone inventory) and `1080p storage` (storage layout with a chest tooltip); `test_stills/resolutions/1440p` has `inventory top left.png`, `hover tooltip.png`, the `storage ...` files and `21 by 9 storage.png` (forced 21:9 at 2560x1440).
- The file must be exactly the game's resolution: there is no resolution input. A native 3440x1440 capture is taken as 21:9 by auto.
- The grid keeps its slots after `Stop` and stays editable; starting the next capture wipes it, and the capture card says so while idle.
- The first scan is slow (wasm init, the anchor searches). Slots get a canvas once their page has been looked at, which takes a while for a full page; until then they show `?`. A slot's status is its border colour (`style.borderColor` holds the CSS variable, see `Slot grid.md`).
- Chests read from tooltips show in the dashboard under the grid when a slot with that chest's icon is hovered or clicked. The dashboard's inputs are labelled `Item`, `Amount` and `Tradability` (the last two only while an item is chosen; match `Tradability` exactly, the manifest has inputs ending in it), with buttons `Save edit` (disabled until amount and tradability are filled), `Retry` and `Okay`. Every frame of an uploaded recording is scanned. Texts arrive later than the frame they were cut from, so wait for the OCR queue to empty (`window.__ocr_timings` stops growing) before reading the grid.
- The currencies are buttons in the group `Currencies` (silver, gold, shards in order). A currency's dashboard has inputs labelled `Char bound`, `Roster bound`, `Tradable` (exact; only its bands), and `Save edit` / `Remove edit`.
- `Stop` ends capture. `Share screen` is the real screen-share path.
- The manifest sits under the dashboard. While slots are missing it is a warning with a `Show the manifest without these` button and a list labelled `Missing slots`. Its table is the group `Manifest materials`, with a group per material and read-only text labelled `<label> Char bound` / `Roster bound` / `Tradable`; the chest panels are the groups `Selection chests` and `Obtain-all chests`; the box `Some information from the previous scan was used` sits above the export button while it uses any; the tier tabs are buttons named after `TIER_LABELS`; `Export to calculator` exports.
- Each grid window is a group named after it (`Roster storage`, `Character storage`, `Inventory`). A page not open in game that an earlier export saved is covered while its tab is selected: the group `<window> covered` (`Roster storage covered`, `Character storage covered`, `Inventory covered`), with no buttons, gone once the page is open in game. Injected scans do not turn the tabs, so click the tab of the page to check. The sidebar link is named `Scannerbeta` (its beta tag), so match it with `/^Scanner/`.
- Chest editing is on the calculator page: the `Edit chests` checkbox shows the stored chests with counts `<title> count`, buttons `Delete <title>`, and the card for adding a chest: `New chest count`, `New chest name`, `New chest ownership`, `Option <n> material` and `Option <n> amount`, with buttons `Add option`, `Add chest` (disabled until one option has a material and an amount) and `Remove`.
- The Income page (`/<char>/income`) has the group `Snapshots` (a button `<time> <kind>` per snapshot; overlapping ones stack in DOM order, so `.last()` is the newest), `Take snapshot` (disabled while nothing changed since the latest), the group `Snapshot dashboard` (`Delete <time> <kind>`, and text labelled `<label> <column>` for each change since the snapshot before), and the group `Income table` (`<label> owned`, `<label> income`, the checkbox `Manual income`). Opening it starts optimizer runs of the snapshots.
- `tests/e2e/scanner.spec.ts` covers exports across two characters (ending on "Take snapshot" being disabled when nothing changed), the cover (a saved page not opened stays covered and counts as saved, an opened one counts as read), the snapshots, the character switch dialog (cancel, move, wipe, which goes to the target's scanner page, and a move to another roster; a character of the same roster is a link in the `Characters` navigation, another roster's is reached by its `Roster N` link in the `banner`; a second roster needs its `roster_mats_owned`, `tradable_mats_owned` and `all_regions` entries, as the roster page adds them, or a reload puts its characters on the first roster) and chest editing with injected scans, Reset scan on a stopped scan, a reset mid-replay and a scan moved mid-replay against a straight replay (chests compared by key and count: their order and joined titles follow reading order), and replays four local debug captures in Chromium (skipped where they are absent): `10-18` of one character, `10-20`, `10-21` and `10-23` of a second one (the same bound gold, shards and storage page), each showing other pages (each export checked against the pages read over this character's and the roster's saved ones, plus the other character's saves); and the `CHECKED` captures, each of which must read its listed pages and every slot set by hand in its end record (`wrong_edits`): `16-29`, a native 3440x1440 game (roster storage hovered, character storage and the inventory open; eight gold chests, set with `capture_edit` after checking their tooltips in the frames), and `2026-10-20T11-43`, 1080p, roster storage page 2, character storage and the inventory, all 260 slots set with `capture_edit accept` after the author checked the reading by hand. And `2026-11-18T15-58`, 1080p on the November 2026 client, all five pages (420 slots): the page's reading accepted after checking it against the frames, and the two Vestiges of the End clear chests (CharInventory 0 3,0 Hard, 3,1 Normal) set by hand from their tooltips. The Hard one has a single chest read, whose "20,000 Gold Bars" came back as "220.000", so it stays NeedHover natively (419 of 420); the entry's `misread` lists that slot, which the test lets read wrong (another hover fixes it, so it is left as is). The replays are re-runs, which wait for OCR on every frame that sent lines. `export_now` exports past missing slots (`Show the manifest without these`, which sets them to irrelevant), so the merge test also covers that path; whether a re-run reads right is checked against hand edits instead.
- Also in `scanner.spec.ts`: an injected export shows on the calc page (each Chance column band) and the market page, not only in the store; and uploads in sequence on one page (capture, still, the newest capture `2026-11-19T00-42`, a capture cut off mid-replay by another upload) each read like the same file on a fresh page (Chromium, local captures). `2026-11-19T00-42` has no hand edits yet (its `Roster p1 (9,3)` reads `Error`), so it is a sequence input, not a `CHECKED` one. A capture's start settings overwrite `scan_settings` and are not put back, but a later still reads the same (no auto setting differs).
- To test the manifest without a capture, set `slots`, `chests`, `edits` (through `set_edit`) and `seen_pages` in `ScanStore.ts` from `page.evaluate`. Import the module by the URL the app loaded it from (look it up in `performance.getEntriesByType("resource")`); a hot-reloaded file has a `?t=` query, and the bare path gives a second, unused instance.
- **A Vue warning on every slot of every grid update kills the run.** Playwright keeps logged objects alive, so the headless browser grew past 4 GB and the page stopped answering within seconds (an `active` prop given null instead of false did this). If the page hangs right after capture starts, count console warnings first.

## State you can read

- The card shows no live/idle status. While capturing, with "Troubleshooting" open (a button, `/Troubleshooting/`), it shows the element labelled `Frames scanned per second` (`N fps scanned`); open it and wait on that.
- While capturing, the element labelled `Frames scanned per second` reads the frames handed to the scanner in the last second, as `22 fps scanned`.
- Next to it, also under "Troubleshooting", the element labelled `Lines waiting for OCR` reads the lines not read yet, waiting or with a worker, as `27 in OCR queue` (`ocr_pending` in `OcrRelay.ts`). It is hidden during a replay, which has its texts.
- The button under the card reads `stop cropper` while the scan loop runs.
- Debug captures (`Pipeline.md`, "Debug captures"), behind "Show debug options": the buttons `Record debug capture` / `Stop recording` (enabled while capturing) and `Download capture` (enabled once recording has stopped with something recorded; it raises a download event), and the element labelled `Capture size`, as `122 MB ready`. Setting a `.hfcap` on the upload input re-runs it; when it is through the card shows `Replay finished, re-run on this build.` While recording, `window.__record_timings` has one record per frame recorded (`copy`, `record`, `bytes`) and the scan timings have `forward`.
- Leaving the scanner page pauses the loop but keeps the stream; returning shows `Frames scanned per second` again without a new upload.

## Measuring and checking a change

`pnpm scanner-profile` replays debug captures on this page and dumps the three timing arrays below; how to run it, and the native suite that says whether a change alters what is read, are in `Screen scanner/Developing and testing.md`.

The dumps are the three arrays the page keeps, all in ms:

`window.__scan_timings` holds one record per scan (last 5000): `read` (waiting for a frame), `to_worker`, `copy`, `wasm_call`, `to_main`, `process_result` (merging the result, not rendering), `total`, and `rust`, a list of `[name, total, calls]` for the stages inside the wasm call. A scan saw a tooltip if `rust` has `tooltip/layout`, a chest if it has `tooltip/chest`; a frame skipped as still has no `anchors`. Skip the first record, which carries startup. `window.__scan_renders` has the duration of each grid update. `window.__ocr_timings` has one record per OCR batch: `jobs`, `width` (px of the lines as captured), `took`, `waited` (how long its first reads took from being queued; 0 for a batch with none, so drop those before a median) and `left` (lines still queued).

## Real screen share, headed only

For checks that need the real capture path, launch headed and skip the picker:

- Chrome: `--auto-select-desktop-capture-source=Entire screen`, and `viewport: null`. With an emulated viewport the capture is scaled to the viewport size, so the capture is no longer the game's resolution.
- Firefox: the user pref `media.navigator.permission.disabled: true`. The auto-accept is flaky; check for `Frames scanned per second` and retry.

The game has to be full screen for anything to be recognised.

## Measuring memory

- Sample the browser's processes from outside (private bytes per process, plus system available memory). The page's own heap figure misses worker and frame memory.
- An attached automation client keeps every logged object alive, so any per-frame log of a large object shows up as a leak under Playwright even if a normal tab is fine. To measure what a user sees, start capture and then disconnect: Chrome via a remote debugging port and `connectOverCDP`, installed Firefox via `-remote-debugging-port` and a WebDriver BiDi session that is ended after the click.
- Playwright's Firefox is not the installed Firefox. A leak that reproduced only on the installed build was missed for a long time by testing the patched one.
- A full-screen static image produces no new capture frames, so the loop stops. Put something moving on screen (a blinking corner square is enough).
- Results only carry images once an inventory is recognised, and operations are also much faster then, so always measure with an inventory on screen.
- Chrome warns in the console when a video frame is garbage collected without being closed. Counting that warning is a direct test for unclosed frames.
- Keep the browser window unobscured. The Firefox loop was seen stalling for tens of seconds in headed runs, most likely when its window was covered.
- Memory has not been measured since the scanner state stopped crossing the worker boundary every frame (`Screen scanner/Pipeline tried.md`).
